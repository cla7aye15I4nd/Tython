use anyhow::Result;
use pyo3::prelude::*;
use std::path::Path;

use crate::ast::{ClassInfo, Type};
use crate::tir::{
    type_rules, ArithBinOp, CallResult, CallTarget, FloatArithOp, RawBinOp, TirExpr, TirExprKind,
    TirStmt, TypedBinOp, ValueType,
};
use crate::{ast_get_list, ast_get_string, ast_getattr, ast_type_name};

use super::Lowering;

impl Lowering {
    fn bind_user_function_args(
        &self,
        line: usize,
        func_display_name: &str,
        signature_key: &str,
        func_type: &Type,
        positional_args: Vec<TirExpr>,
        keyword_args: Vec<(String, TirExpr)>,
    ) -> Result<Vec<TirExpr>> {
        let (params, _) = match func_type {
            Type::Function {
                params,
                return_type,
            } => (params, return_type),
            _ => return Err(self.type_error(line, "cannot call non-function type")),
        };

        let param_count = params.len();
        if positional_args.len() > param_count {
            return Err(self.type_error(
                line,
                format!(
                    "function `{}` expects at most {} argument{}, got {}",
                    func_display_name,
                    param_count,
                    if param_count == 1 { "" } else { "s" },
                    positional_args.len()
                ),
            ));
        }

        let sig = self.function_signatures.get(signature_key);
        if sig.is_none() {
            if keyword_args.is_empty() && positional_args.len() == param_count {
                return Ok(positional_args);
            }
            return Err(self.syntax_error(
                line,
                format!(
                    "function `{}` is missing signature metadata for keyword/default argument binding",
                    func_display_name
                ),
            ));
        }
        let sig = sig.expect("checked is_some above");

        if sig.param_names.len() != param_count || sig.default_values.len() != param_count {
            return Err(self.syntax_error(
                line,
                format!(
                    "function `{}` has inconsistent signature metadata",
                    func_display_name
                ),
            ));
        }

        let mut bound: Vec<Option<TirExpr>> = vec![None; param_count];
        let positional_count = positional_args.len();
        for (i, arg) in positional_args.into_iter().enumerate() {
            bound[i] = Some(arg);
        }

        for (kw_name, kw_value) in keyword_args {
            let Some(idx) = sig.param_names.iter().position(|p| p == &kw_name) else {
                return Err(self.type_error(
                    line,
                    format!(
                        "function `{}` got an unexpected keyword argument `{}`",
                        func_display_name, kw_name
                    ),
                ));
            };
            if idx < positional_count || bound[idx].is_some() {
                return Err(self.type_error(
                    line,
                    format!(
                        "function `{}` got multiple values for argument `{}`",
                        func_display_name, kw_name
                    ),
                ));
            }
            bound[idx] = Some(kw_value);
        }

        for (i, slot) in bound.iter_mut().enumerate() {
            if slot.is_none() {
                *slot = sig.default_values[i].clone();
            }
        }

        let missing: Vec<String> = bound
            .iter()
            .enumerate()
            .filter_map(|(i, arg)| {
                if arg.is_none() {
                    Some(format!("`{}`", sig.param_names[i]))
                } else {
                    None
                }
            })
            .collect();
        if !missing.is_empty() {
            return Err(self.type_error(
                line,
                format!(
                    "function `{}` missing required argument{}: {}",
                    func_display_name,
                    if missing.len() == 1 { "" } else { "s" },
                    missing.join(", ")
                ),
            ));
        }

        Ok(bound
            .into_iter()
            .map(|arg| arg.expect("checked missing arguments above"))
            .collect())
    }

    fn coerce_args_to_param_types(&self, mut args: Vec<TirExpr>, params: &[Type]) -> Vec<TirExpr> {
        for (arg, expected) in args.iter_mut().zip(params.iter()) {
            if arg.ty.to_type() == *expected {
                continue;
            }
            let target = match expected {
                Type::Float => Some(ValueType::Float),
                Type::Int => Some(ValueType::Int),
                Type::Bool => Some(ValueType::Bool),
                _ => None,
            };
            let Some(target_ty) = target else {
                continue;
            };
            let from = arg.ty.clone();
            if matches!(
                (&from, &target_ty),
                (ValueType::Int, ValueType::Float)
                    | (ValueType::Bool, ValueType::Float)
                    | (ValueType::Bool, ValueType::Int)
                    | (ValueType::Int, ValueType::Bool)
                    | (ValueType::Float, ValueType::Bool)
            ) {
                let old = std::mem::replace(
                    arg,
                    TirExpr {
                        kind: TirExprKind::IntLiteral(0),
                        ty: ValueType::Int,
                    },
                );
                *arg = TirExpr {
                    kind: TirExprKind::Cast {
                        kind: Self::compute_cast_kind(&from, &target_ty),
                        arg: Box::new(old),
                    },
                    ty: target_ty,
                };
            }
        }
        args
    }

    fn append_nested_captures_if_needed(
        &self,
        line: usize,
        mangled: &str,
        args: &mut Vec<TirExpr>,
    ) -> Result<()> {
        let Some(captures) = self.nested_function_captures.get(mangled) else {
            return Ok(());
        };
        for (name, ty) in captures {
            let resolved = self.lookup(name).cloned().ok_or_else(|| {
                self.name_error(
                    line,
                    format!(
                        "captured variable `{}` not found at call to `{}`",
                        name, mangled
                    ),
                )
            })?;
            if &resolved != ty {
                return Err(self.type_error(
                    line,
                    format!(
                        "captured variable `{}` type mismatch at call to `{}`: expected `{}`, got `{}`",
                        name, mangled, ty, resolved
                    ),
                ));
            }
            args.push(TirExpr {
                kind: TirExprKind::Var(name.clone()),
                ty: Self::to_value_type(ty),
            });
        }
        Ok(())
    }

    pub(super) fn lower_call(&mut self, node: &Bound<PyAny>, line: usize) -> Result<CallResult> {
        let func_node = ast_getattr!(node, "func");
        let args_list = ast_get_list!(node, "args");
        let keywords_list = ast_get_list!(node, "keywords");

        let mut positional_args = Vec::new();
        for arg in args_list.iter() {
            positional_args.push(self.lower_expr(&arg)?);
        }

        let mut keyword_args: Vec<(String, TirExpr)> = Vec::new();
        for kw in keywords_list.iter() {
            let kw_name_node = ast_getattr!(kw, "arg");
            if kw_name_node.is_none() {
                return Err(self.syntax_error(
                    line,
                    "dictionary unpacking in calls (`**kwargs`) is not supported",
                ));
            }
            let kw_name = kw_name_node.extract::<String>()?;
            let kw_value = self.lower_expr(&ast_getattr!(kw, "value"))?;
            keyword_args.push((kw_name, kw_value));
        }

        let func_node_type = ast_type_name!(func_node);
        match func_node_type.as_str() {
            "Name" => {
                let func_name = ast_get_string!(func_node, "id");

                if func_name == "print" {
                    return Err(self.syntax_error(line, "print() can only be used as a statement"));
                }
                if func_name == "open" {
                    if !keyword_args.is_empty() {
                        return Err(self.syntax_error(line, "open() does not accept keywords"));
                    }
                    if positional_args.len() != 1 {
                        return Err(self.type_error(
                            line,
                            format!("open() expects 1 argument, got {}", positional_args.len()),
                        ));
                    }
                    let mut path_arg = positional_args.remove(0);
                    if path_arg.ty != ValueType::Str {
                        return Err(self.type_error(
                            line,
                            format!("open() path must be `str`, got `{}`", path_arg.ty),
                        ));
                    }

                    if let TirExprKind::StrLiteral(path) = &path_arg.kind {
                        let p = Path::new(path);
                        if p.is_relative() {
                            let base = Path::new(&self.current_file)
                                .parent()
                                .expect("source file should have a parent directory");
                            let abs = base.join(p);
                            path_arg = TirExpr {
                                kind: TirExprKind::StrLiteral(abs.to_string_lossy().into_owned()),
                                ty: ValueType::Str,
                            };
                        }
                    }

                    return Ok(CallResult::Expr(TirExpr {
                        kind: TirExprKind::ExternalCall {
                            func: crate::tir::builtin::BuiltinFn::OpenReadAll,
                            args: vec![path_arg],
                        },
                        ty: ValueType::Str,
                    }));
                }

                if type_rules::is_builtin_call(&func_name) {
                    if !keyword_args.is_empty() {
                        return Err(self.syntax_error(
                            line,
                            format!("builtin `{}` does not support keyword arguments", func_name),
                        ));
                    }
                    if (func_name == "min" || func_name == "max")
                        && positional_args
                            .iter()
                            .all(|a| matches!(a.ty, ValueType::Int | ValueType::Float))
                        && positional_args.iter().any(|a| a.ty == ValueType::Float)
                    {
                        positional_args = positional_args
                            .into_iter()
                            .map(|arg| self.cast_to_float_if_needed(arg))
                            .collect();
                    }
                    if func_name == "sum" && positional_args.len() == 2 {
                        let list_expr = positional_args[0].clone();
                        let start_expr = positional_args[1].clone();
                        if let ValueType::List(inner) = &list_expr.ty {
                            let elem_ty = (**inner).clone();
                            if elem_ty == start_expr.ty && matches!(elem_ty, ValueType::Class(_)) {
                                return Ok(CallResult::Expr(self.lower_sum_class_list(
                                    line, list_expr, start_expr, elem_ty,
                                )?));
                            }
                        }
                    }
                    let arg_types: Vec<&ValueType> =
                        positional_args.iter().map(|a| &a.ty).collect();
                    let rule = type_rules::lookup_builtin_call(&func_name, &arg_types).ok_or_else(
                        || {
                            self.type_error(
                                line,
                                type_rules::builtin_call_error_message(
                                    &func_name,
                                    &arg_types,
                                    positional_args.len(),
                                ),
                            )
                        },
                    )?;
                    if let type_rules::BuiltinCallRule::ClassMagic {
                        method_names,
                        return_type,
                    } = rule
                    {
                        let arg = positional_args.remove(0);
                        return Ok(CallResult::Expr(self.lower_class_magic_method(
                            line,
                            arg,
                            method_names,
                            return_type,
                            &func_name,
                        )?));
                    }
                    if matches!(rule, type_rules::BuiltinCallRule::StrAuto) {
                        let arg = positional_args.remove(0);
                        return Ok(CallResult::Expr(self.lower_str_auto(arg)));
                    }
                    if matches!(rule, type_rules::BuiltinCallRule::ReprAuto) {
                        let arg = positional_args.remove(0);
                        return Ok(CallResult::Expr(self.lower_repr_str_expr(arg)));
                    }
                    return Ok(Self::lower_builtin_rule(rule, positional_args));
                }

                let scope_type = self.lookup(&func_name).cloned().ok_or_else(|| {
                    self.name_error(line, format!("undefined function `{}`", func_name))
                })?;

                match &scope_type {
                    Type::Function {
                        params: _,
                        return_type: _,
                    } => {
                        let mangled = self.function_mangled_names.get(&func_name).cloned().ok_or_else(|| {
                            self.type_error(
                                line,
                                format!(
                                    "`{}` is not callable (indirect calls through function pointers are not supported)",
                                    func_name
                                ),
                            )
                        })?;
                        let bound_args = self.bind_user_function_args(
                            line,
                            &func_name,
                            &mangled,
                            &scope_type,
                            positional_args,
                            keyword_args,
                        )?;
                        let mut bound_args = match &scope_type {
                            Type::Function { params, .. } => {
                                self.coerce_args_to_param_types(bound_args, params)
                            }
                            _ => bound_args,
                        };
                        let return_type_resolved =
                            self.check_call_args(line, &func_name, &scope_type, &bound_args)?;
                        self.append_nested_captures_if_needed(line, &mangled, &mut bound_args)?;

                        // Direct call to a known function definition
                        if return_type_resolved == Type::Unit {
                            Ok(CallResult::VoidStmt(Box::new(TirStmt::VoidCall {
                                target: CallTarget::Named(mangled),
                                args: bound_args,
                            })))
                        } else {
                            Ok(CallResult::Expr(TirExpr {
                                kind: TirExprKind::Call {
                                    func: mangled,
                                    args: bound_args,
                                },
                                ty: Self::to_value_type(&return_type_resolved),
                            }))
                        }
                    }
                    Type::Module(mangled) => {
                        // Check if this is an imported class constructor
                        if let Some(class_info) = self.class_registry.get(mangled).cloned() {
                            if !keyword_args.is_empty() {
                                return Err(self.syntax_error(
                                    line,
                                    "constructor keyword arguments are not supported",
                                ));
                            }
                            return self.lower_constructor_call(
                                line,
                                mangled,
                                &class_info,
                                positional_args,
                            );
                        }

                        let func_type = self
                            .symbol_table
                            .get(mangled)
                            .ok_or_else(|| {
                                self.name_error(
                                    line,
                                    format!(
                                        "imported symbol `{}` not found in symbol table",
                                        func_name
                                    ),
                                )
                            })?
                            .clone();
                        let bound_args = self.bind_user_function_args(
                            line,
                            &func_name,
                            mangled,
                            &func_type,
                            positional_args,
                            keyword_args,
                        )?;
                        let mut bound_args = match &func_type {
                            Type::Function { params, .. } => {
                                self.coerce_args_to_param_types(bound_args, params)
                            }
                            _ => bound_args,
                        };
                        let return_type =
                            self.check_call_args(line, &func_name, &func_type, &bound_args)?;
                        self.append_nested_captures_if_needed(line, mangled, &mut bound_args)?;
                        if return_type == Type::Unit {
                            Ok(CallResult::VoidStmt(Box::new(TirStmt::VoidCall {
                                target: CallTarget::Named(mangled.clone()),
                                args: bound_args,
                            })))
                        } else {
                            Ok(CallResult::Expr(TirExpr {
                                kind: TirExprKind::Call {
                                    func: mangled.clone(),
                                    args: bound_args,
                                },
                                ty: Self::to_value_type(&return_type),
                            }))
                        }
                    }
                    Type::Class(name) => {
                        // Constructor call
                        if !keyword_args.is_empty() {
                            return Err(self.syntax_error(
                                line,
                                "constructor keyword arguments are not supported",
                            ));
                        }
                        let class_info = self
                            .class_registry
                            .get(name)
                            .ok_or_else(|| {
                                self.name_error(line, format!("unknown class `{}`", name))
                            })?
                            .clone();
                        self.lower_constructor_call(line, name, &class_info, positional_args)
                    }
                    _ => Err(self.type_error(line, format!("`{}` is not callable", func_name))),
                }
            }

            "Attribute" => {
                let value_node = ast_getattr!(func_node, "value");
                let attr = ast_get_string!(func_node, "attr");

                // Check if value_node is a Name that resolves to a module
                // (modules are not value types, so we must handle them before lower_expr)
                if ast_type_name!(value_node) == "Name" {
                    let name = ast_get_string!(value_node, "id");
                    if let Some(Type::Module(mod_path)) = self.lookup(&name).cloned() {
                        if mod_path == "math" || mod_path == "random" {
                            return self.lower_native_module_call(
                                line,
                                &mod_path,
                                &attr,
                                positional_args,
                                keyword_args,
                            );
                        }
                        let resolved = format!("{}${}", mod_path, attr);

                        // Check for class constructor first
                        if let Some(class_info) = self.class_registry.get(&resolved).cloned() {
                            if !keyword_args.is_empty() {
                                return Err(self.syntax_error(
                                    line,
                                    "constructor keyword arguments are not supported",
                                ));
                            }
                            return self.lower_constructor_call(
                                line,
                                &resolved,
                                &class_info,
                                positional_args,
                            );
                        }

                        let func_type = self
                            .symbol_table
                            .get(&resolved)
                            .ok_or_else(|| {
                                self.name_error(line, format!("undefined function `{}`", attr))
                            })?
                            .clone();

                        let bound_args = self.bind_user_function_args(
                            line,
                            &attr,
                            &resolved,
                            &func_type,
                            positional_args,
                            keyword_args,
                        )?;
                        let mut bound_args = match &func_type {
                            Type::Function { params, .. } => {
                                self.coerce_args_to_param_types(bound_args, params)
                            }
                            _ => bound_args,
                        };
                        let return_type = {
                            let label = attr.to_string();
                            self.check_call_args(line, &label, &func_type, &bound_args)?
                        };
                        self.append_nested_captures_if_needed(line, &resolved, &mut bound_args)?;

                        return if return_type == Type::Unit {
                            Ok(CallResult::VoidStmt(Box::new(TirStmt::VoidCall {
                                target: CallTarget::Named(resolved),
                                args: bound_args,
                            })))
                        } else {
                            Ok(CallResult::Expr(TirExpr {
                                kind: TirExprKind::Call {
                                    func: resolved,
                                    args: bound_args,
                                },
                                ty: Self::to_value_type(&return_type),
                            }))
                        };
                    }
                }

                // Check if the full dotted path resolves to a class
                // (e.g., Outer.Inner(...), Deep.Mid.Leaf(...))
                if let Some(qualified) = self.try_resolve_class_path(&func_node) {
                    if let Some(class_info) = self.class_registry.get(&qualified).cloned() {
                        if !keyword_args.is_empty() {
                            return Err(self.syntax_error(
                                line,
                                "constructor keyword arguments are not supported",
                            ));
                        }
                        return self.lower_constructor_call(
                            line,
                            &qualified,
                            &class_info,
                            positional_args,
                        );
                    }
                }

                // Not a class path — lower value as an expression (must be a class instance)
                let obj_expr = self.lower_expr(&value_node)?;

                let obj_ty = obj_expr.ty.clone();
                match obj_ty {
                    ValueType::Class(class_name) => {
                        // Method call on a class instance
                        let class_info = self
                            .class_registry
                            .get(&class_name)
                            .ok_or_else(|| {
                                self.name_error(line, format!("unknown class `{}`", class_name))
                            })?
                            .clone();

                        let method = class_info.methods.get(&attr).ok_or_else(|| {
                            self.attribute_error(
                                line,
                                format!("class `{}` has no method `{}`", class_name, attr),
                            )
                        })?;

                        if !keyword_args.is_empty() {
                            return Err(self
                                .syntax_error(line, "method keyword arguments are not supported"));
                        }
                        if positional_args.len() != method.params.len() {
                            return Err(self.type_error(
                                line,
                                format!(
                                    "{}.{}() expects {} argument{}, got {}",
                                    class_name,
                                    attr,
                                    method.params.len(),
                                    if method.params.len() == 1 { "" } else { "s" },
                                    positional_args.len()
                                ),
                            ));
                        }
                        for (i, (arg, expected)) in
                            positional_args.iter().zip(method.params.iter()).enumerate()
                        {
                            if arg.ty.to_type() != *expected {
                                return Err(self.type_error(
                                    line,
                                    format!(
                                        "argument {} type mismatch in {}.{}(): expected `{}`, got `{}`",
                                        i, class_name, attr, expected, arg.ty
                                    ),
                                ));
                            }
                        }

                        let return_type = &method.return_type;
                        let mangled = method.mangled_name.clone();

                        // Prepend self (obj_expr) to args — method is just a Call
                        let mut all_args = vec![obj_expr];
                        all_args.extend(positional_args);

                        if *return_type == Type::Unit {
                            Ok(CallResult::VoidStmt(Box::new(TirStmt::VoidCall {
                                target: CallTarget::Named(mangled),
                                args: all_args,
                            })))
                        } else {
                            Ok(CallResult::Expr(TirExpr {
                                kind: TirExprKind::Call {
                                    func: mangled,
                                    args: all_args,
                                },
                                ty: Self::to_value_type(return_type),
                            }))
                        }
                    }
                    ref ty => {
                        let type_name = type_rules::builtin_type_display_name(ty);
                        let lookup = type_rules::lookup_builtin_method(ty, &attr);
                        self.lower_builtin_method_call(
                            line,
                            obj_expr,
                            positional_args,
                            &attr,
                            &type_name,
                            lookup,
                        )
                    }
                }
            }

            _ => Err(self.syntax_error(
                line,
                "only direct function calls and module.function calls are supported",
            )),
        }
    }

    // ── call argument checking ─────────────────────────────────────────

    pub(super) fn lower_builtin_rule(
        rule: type_rules::BuiltinCallRule,
        mut tir_args: Vec<TirExpr>,
    ) -> CallResult {
        match rule {
            type_rules::BuiltinCallRule::Identity => {
                let arg = tir_args
                    .into_iter()
                    .next()
                    .expect("ICE: identity conversion expects one arg");
                CallResult::Expr(arg)
            }
            type_rules::BuiltinCallRule::ExternalCall { func, return_type } => {
                CallResult::Expr(TirExpr {
                    kind: TirExprKind::ExternalCall {
                        func,
                        args: tir_args,
                    },
                    ty: return_type,
                })
            }
            type_rules::BuiltinCallRule::FoldExternalCall { func, return_type } => {
                let mut iter = tir_args.into_iter();
                let mut acc = iter
                    .next()
                    .expect("ICE: FoldExternalCall expects at least two args");
                for arg in iter {
                    acc = TirExpr {
                        kind: TirExprKind::ExternalCall {
                            func,
                            args: vec![acc, arg],
                        },
                        ty: return_type.clone(),
                    };
                }
                CallResult::Expr(acc)
            }
            type_rules::BuiltinCallRule::PrimitiveCast { target_type } => {
                let arg = tir_args
                    .into_iter()
                    .next()
                    .expect("ICE: primitive cast expects one arg");
                let cast_kind = Self::compute_cast_kind(&arg.ty, &target_type);
                CallResult::Expr(TirExpr {
                    kind: TirExprKind::Cast {
                        kind: cast_kind,
                        arg: Box::new(arg),
                    },
                    ty: target_type,
                })
            }
            type_rules::BuiltinCallRule::ConstInt(value) => CallResult::Expr(TirExpr {
                kind: TirExprKind::IntLiteral(value),
                ty: ValueType::Int,
            }),
            type_rules::BuiltinCallRule::PowFloat => {
                let right = tir_args.remove(1);
                let left = tir_args.remove(0);
                CallResult::Expr(TirExpr {
                    kind: TirExprKind::BinOp {
                        op: TypedBinOp::FloatArith(FloatArithOp::Pow),
                        left: Box::new(left),
                        right: Box::new(right),
                    },
                    ty: ValueType::Float,
                })
            }
            type_rules::BuiltinCallRule::ClassMagic { .. } => {
                unreachable!("ICE: ClassMagic should be handled before lower_builtin_rule")
            }
            type_rules::BuiltinCallRule::StrAuto => {
                unreachable!("ICE: StrAuto should be handled before lower_builtin_rule")
            }
            type_rules::BuiltinCallRule::ReprAuto => {
                unreachable!("ICE: ReprAuto should be handled before lower_builtin_rule")
            }
        }
    }

    fn cast_to_float_if_needed(&self, arg: TirExpr) -> TirExpr {
        if arg.ty == ValueType::Float {
            return arg;
        }
        let kind = Self::compute_cast_kind(&arg.ty, &ValueType::Float);
        TirExpr {
            kind: TirExprKind::Cast {
                kind,
                arg: Box::new(arg),
            },
            ty: ValueType::Float,
        }
    }

    fn lower_sum_class_list(
        &mut self,
        line: usize,
        list_expr: TirExpr,
        start_expr: TirExpr,
        elem_ty: ValueType,
    ) -> Result<TirExpr> {
        let list_var = self.fresh_internal("sum_list");
        let acc_var = self.fresh_internal("sum_acc");
        let loop_var = self.fresh_internal("sum_item");
        let idx_var = self.fresh_internal("sum_idx");
        let len_var = self.fresh_internal("sum_len");

        self.declare(list_var.clone(), list_expr.ty.to_type());
        self.declare(acc_var.clone(), elem_ty.to_type());
        self.declare(loop_var.clone(), elem_ty.to_type());
        self.declare(idx_var.clone(), Type::Int);
        self.declare(len_var.clone(), Type::Int);

        let lhs = TirExpr {
            kind: TirExprKind::Var(acc_var.clone()),
            ty: elem_ty.clone(),
        };
        let rhs = TirExpr {
            kind: TirExprKind::Var(loop_var.clone()),
            ty: elem_ty.clone(),
        };
        let add_expr = self.resolve_binop(line, RawBinOp::Arith(ArithBinOp::Add), lhs, rhs)?;

        self.pre_stmts.push(TirStmt::Let {
            name: list_var.clone(),
            ty: list_expr.ty.clone(),
            value: list_expr,
        });
        self.pre_stmts.push(TirStmt::Let {
            name: acc_var.clone(),
            ty: elem_ty.clone(),
            value: start_expr,
        });
        self.pre_stmts.push(TirStmt::ForList {
            loop_var: loop_var.clone(),
            loop_var_ty: elem_ty.clone(),
            list_var,
            index_var: idx_var,
            len_var,
            body: vec![TirStmt::Let {
                name: acc_var.clone(),
                ty: elem_ty.clone(),
                value: add_expr,
            }],
            else_body: vec![],
        });

        Ok(TirExpr {
            kind: TirExprKind::Var(acc_var),
            ty: elem_ty,
        })
    }

    fn lower_native_module_call(
        &self,
        line: usize,
        module: &str,
        attr: &str,
        mut positional_args: Vec<TirExpr>,
        keyword_args: Vec<(String, TirExpr)>,
    ) -> Result<CallResult> {
        match (module, attr) {
            ("math", "log") | ("math", "exp") => {
                if !keyword_args.is_empty() {
                    return Err(self.syntax_error(
                        line,
                        format!("{}.{}() does not accept keywords", module, attr),
                    ));
                }
                if positional_args.len() != 1 {
                    return Err(self.type_error(
                        line,
                        format!(
                            "{}.{}() expects 1 argument, got {}",
                            module,
                            attr,
                            positional_args.len()
                        ),
                    ));
                }
                let arg = self.cast_to_float_if_needed(positional_args.remove(0));
                let func = if attr == "log" {
                    crate::tir::builtin::BuiltinFn::MathLog
                } else {
                    crate::tir::builtin::BuiltinFn::MathExp
                };
                Ok(CallResult::Expr(TirExpr {
                    kind: TirExprKind::ExternalCall {
                        func,
                        args: vec![arg],
                    },
                    ty: ValueType::Float,
                }))
            }
            ("random", "seed") => {
                if !keyword_args.is_empty() {
                    return Err(self.syntax_error(line, "random.seed() does not accept keywords"));
                }
                if positional_args.len() != 1 || positional_args[0].ty != ValueType::Int {
                    return Err(
                        self.type_error(line, "random.seed() expects exactly one `int` argument")
                    );
                }
                Ok(CallResult::VoidStmt(Box::new(TirStmt::VoidCall {
                    target: CallTarget::Builtin(crate::tir::builtin::BuiltinFn::RandomSeed),
                    args: positional_args,
                })))
            }
            ("random", "gauss") => {
                if !keyword_args.is_empty() {
                    return Err(self.syntax_error(line, "random.gauss() does not accept keywords"));
                }
                if positional_args.len() != 2 {
                    return Err(self.type_error(
                        line,
                        format!(
                            "random.gauss() expects 2 arguments, got {}",
                            positional_args.len()
                        ),
                    ));
                }
                let mu = self.cast_to_float_if_needed(positional_args.remove(0));
                let sigma = self.cast_to_float_if_needed(positional_args.remove(0));
                Ok(CallResult::Expr(TirExpr {
                    kind: TirExprKind::ExternalCall {
                        func: crate::tir::builtin::BuiltinFn::RandomGauss,
                        args: vec![mu, sigma],
                    },
                    ty: ValueType::Float,
                }))
            }
            ("random", "shuffle") => {
                if !keyword_args.is_empty() {
                    return Err(
                        self.syntax_error(line, "random.shuffle() does not accept keywords")
                    );
                }
                if positional_args.len() != 1 {
                    return Err(self.type_error(
                        line,
                        format!(
                            "random.shuffle() expects 1 argument, got {}",
                            positional_args.len()
                        ),
                    ));
                }
                let list_arg = positional_args.remove(0);
                if !matches!(list_arg.ty, ValueType::List(_)) {
                    return Err(self.type_error(
                        line,
                        format!("random.shuffle() expects `list`, got `{}`", list_arg.ty),
                    ));
                }
                Ok(CallResult::VoidStmt(Box::new(TirStmt::VoidCall {
                    target: CallTarget::Builtin(crate::tir::builtin::BuiltinFn::RandomShuffle),
                    args: vec![list_arg],
                })))
            }
            ("random", "choices") => {
                if positional_args.len() != 1 {
                    return Err(self.type_error(
                        line,
                        format!(
                            "random.choices() expects population as 1 positional argument, got {}",
                            positional_args.len()
                        ),
                    ));
                }
                let population = positional_args.remove(0);
                if population.ty != ValueType::List(Box::new(ValueType::Int)) {
                    return Err(self.type_error(
                        line,
                        format!(
                            "random.choices() population must be `list[int]`, got `{}`",
                            population.ty
                        ),
                    ));
                }

                if keyword_args.len() != 1 || keyword_args[0].0 != "weights" {
                    return Err(self.type_error(
                        line,
                        "random.choices() currently requires exactly keyword argument `weights=`",
                    ));
                }
                let weights = keyword_args[0].1.clone();
                if weights.ty != ValueType::List(Box::new(ValueType::Float)) {
                    return Err(self.type_error(
                        line,
                        format!(
                            "random.choices() weights must be `list[float]`, got `{}`",
                            weights.ty
                        ),
                    ));
                }

                Ok(CallResult::Expr(TirExpr {
                    kind: TirExprKind::ExternalCall {
                        func: crate::tir::builtin::BuiltinFn::RandomChoicesInt,
                        args: vec![population, weights],
                    },
                    ty: ValueType::List(Box::new(ValueType::Int)),
                }))
            }
            _ => Err(self.name_error(
                line,
                format!("unsupported native module function {}.{}", module, attr),
            )),
        }
    }

    fn check_call_args(
        &self,
        line: usize,
        func_name: &str,
        func_type: &Type,
        args: &[TirExpr],
    ) -> Result<Type> {
        match func_type {
            Type::Function {
                params,
                return_type,
            } => {
                if args.len() != params.len() {
                    return Err(self.type_error(
                        line,
                        format!(
                            "function `{}` expects {} argument{}, got {}",
                            func_name,
                            params.len(),
                            if params.len() == 1 { "" } else { "s" },
                            args.len()
                        ),
                    ));
                }
                for (i, (arg, expected)) in args.iter().zip(params.iter()).enumerate() {
                    if arg.ty.to_type() != *expected {
                        return Err(self.type_error(
                            line,
                            format!(
                                "argument {} type mismatch in call to `{}`: expected `{}`, got `{}`",
                                i, func_name, expected, arg.ty
                            ),
                        ));
                    }
                }
                Ok(*return_type.clone())
            }
            _ => Err(self.type_error(line, "cannot call non-function type")),
        }
    }

    fn lower_constructor_call(
        &self,
        line: usize,
        qualified_name: &str,
        class_info: &ClassInfo,
        tir_args: Vec<TirExpr>,
    ) -> Result<CallResult> {
        let init_method = class_info.methods.get("__init__").ok_or_else(|| {
            self.syntax_error(
                line,
                format!("class `{}` has no __init__ method", qualified_name),
            )
        })?;
        let init_type = Type::Function {
            params: init_method.params.clone(),
            return_type: Box::new(Type::Unit),
        };
        let bound_args = self.bind_user_function_args(
            line,
            qualified_name,
            &init_method.mangled_name,
            &init_type,
            tir_args,
            vec![],
        )?;
        let bound_args = self.coerce_args_to_param_types(bound_args, &init_method.params);
        self.check_call_args(line, qualified_name, &init_type, &bound_args)?;

        Ok(CallResult::Expr(TirExpr {
            kind: TirExprKind::Construct {
                class_name: qualified_name.to_string(),
                init_mangled_name: init_method.mangled_name.clone(),
                args: bound_args,
            },
            ty: ValueType::Class(qualified_name.to_string()),
        }))
    }

    pub(super) fn lower_class_magic_method(
        &self,
        line: usize,
        object: TirExpr,
        method_names: &[&str],
        expected_return_type: Option<ValueType>,
        caller_name: &str,
    ) -> Result<TirExpr> {
        self.lower_class_magic_method_with_args(
            line,
            object,
            method_names,
            expected_return_type,
            caller_name,
            vec![],
        )
    }

    pub(super) fn lower_class_magic_method_with_args(
        &self,
        line: usize,
        object: TirExpr,
        method_names: &[&str],
        expected_return_type: Option<ValueType>,
        caller_name: &str,
        args: Vec<TirExpr>,
    ) -> Result<TirExpr> {
        if method_names.is_empty() {
            return Err(self.syntax_error(
                line,
                format!(
                    "internal error: {}() magic method list is empty",
                    caller_name
                ),
            ));
        }

        let class_name = match &object.ty {
            ValueType::Class(name) => name.clone(),
            _ => {
                return Err(self.type_error(
                    line,
                    format!("{}() cannot convert `{}`", caller_name, object.ty),
                ))
            }
        };

        let class_info = self.lookup_class(line, &class_name)?;

        let method = method_names
            .iter()
            .find_map(|name| class_info.methods.get(*name))
            .ok_or_else(|| {
                if method_names.len() == 1 {
                    self.attribute_error(
                        line,
                        format!("class `{}` has no method `{}`", class_name, method_names[0]),
                    )
                } else {
                    let choices = method_names
                        .iter()
                        .map(|name| format!("`{}`", name))
                        .collect::<Vec<_>>()
                        .join(" or ");
                    self.attribute_error(
                        line,
                        format!("class `{}` has no method {}", class_name, choices),
                    )
                }
            })?;

        if method.params.len() != args.len() {
            return Err(self.type_error(
                line,
                format!(
                    "{}.{}() expects {} argument{} besides `self`, got {}",
                    class_name,
                    method.name,
                    method.params.len(),
                    if method.params.len() == 1 { "" } else { "s" },
                    args.len()
                ),
            ));
        }
        for (i, (arg, expected)) in args.iter().zip(method.params.iter()).enumerate() {
            if arg.ty.to_type() != *expected {
                return Err(self.type_error(
                    line,
                    format!(
                        "argument {} type mismatch in {}.{}(): expected `{}`, got `{}`",
                        i, class_name, method.name, expected, arg.ty
                    ),
                ));
            }
        }

        let return_type = if let Some(ref expected) = expected_return_type {
            if method.return_type != expected.to_type() {
                return Err(self.type_error(
                    line,
                    format!(
                        "{}.{}() must return `{}`, got `{}`",
                        class_name, method.name, expected, method.return_type
                    ),
                ));
            }
            expected.clone()
        } else {
            if method.return_type == Type::Unit {
                return Err(self.type_error(
                    line,
                    format!(
                        "{}.{}() must return a value, got `None`",
                        class_name, method.name
                    ),
                ));
            }
            Self::to_value_type(&method.return_type)
        };

        let mut call_args = Vec::with_capacity(1 + args.len());
        call_args.push(object);
        call_args.extend(args);

        Ok(TirExpr {
            kind: TirExprKind::Call {
                func: method.mangled_name.clone(),
                args: call_args,
            },
            ty: return_type,
        })
    }

    /// Lower a method call on a builtin type using a resolved `MethodCallRule`.
    fn lower_builtin_method_call(
        &self,
        line: usize,
        obj_expr: TirExpr,
        tir_args: Vec<TirExpr>,
        method_name: &str,
        type_name: &str,
        lookup: Option<Result<type_rules::MethodCallRule, String>>,
    ) -> Result<CallResult> {
        let rule = match lookup {
            Some(Ok(rule)) => rule,
            Some(Err(msg)) => return Err(self.type_error(line, msg)),
            None => {
                return Err(self.attribute_error(
                    line,
                    format!("{} has no method `{}`", type_name, method_name),
                ))
            }
        };

        if tir_args.len() != rule.params.len() {
            return Err(self.type_error(
                line,
                type_rules::method_call_arity_error(
                    type_name,
                    method_name,
                    rule.params.len(),
                    tir_args.len(),
                ),
            ));
        }
        for (arg, expected) in tir_args.iter().zip(rule.params.iter()) {
            if arg.ty != *expected {
                return Err(self.type_error(
                    line,
                    type_rules::method_call_type_error(type_name, method_name, expected, &arg.ty),
                ));
            }
        }

        let mut full_args = Vec::with_capacity(1 + tir_args.len());
        full_args.push(obj_expr);
        full_args.extend(tir_args);

        Ok(match rule.result {
            type_rules::MethodCallResult::Void(func) => {
                CallResult::VoidStmt(Box::new(TirStmt::VoidCall {
                    target: CallTarget::Builtin(func),
                    args: full_args,
                }))
            }
            type_rules::MethodCallResult::Expr { func, return_type } => CallResult::Expr(TirExpr {
                kind: TirExprKind::ExternalCall {
                    func,
                    args: full_args,
                },
                ty: return_type,
            }),
        })
    }
}
