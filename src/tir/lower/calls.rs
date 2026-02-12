use anyhow::Result;
use pyo3::prelude::*;

use crate::ast::{ClassInfo, Type};
use crate::tir::{
    type_rules, CallResult, CallTarget, FloatArithOp, TirExpr, TirExprKind, TirStmt, TypedBinOp,
    ValueType,
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

                if type_rules::is_builtin_call(&func_name) {
                    if !keyword_args.is_empty() {
                        return Err(self.syntax_error(
                            line,
                            format!("builtin `{}` does not support keyword arguments", func_name),
                        ));
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
                        let return_type_resolved =
                            self.check_call_args(line, &func_name, &scope_type, &bound_args)?;

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
                        let return_type =
                            self.check_call_args(line, &func_name, &func_type, &bound_args)?;
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
                        let return_type = {
                            let label = attr.to_string();
                            self.check_call_args(line, &label, &func_type, &bound_args)?
                        };

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

        if tir_args.len() != init_method.params.len() {
            return Err(self.type_error(
                line,
                format!(
                    "{}() expects {} argument{}, got {}",
                    qualified_name,
                    init_method.params.len(),
                    if init_method.params.len() == 1 {
                        ""
                    } else {
                        "s"
                    },
                    tir_args.len()
                ),
            ));
        }
        for (i, (arg, expected)) in tir_args.iter().zip(init_method.params.iter()).enumerate() {
            if arg.ty.to_type() != *expected {
                return Err(self.type_error(
                    line,
                    format!(
                        "argument {} type mismatch in {}(): expected `{}`, got `{}`",
                        i, qualified_name, expected, arg.ty
                    ),
                ));
            }
        }

        Ok(CallResult::Expr(TirExpr {
            kind: TirExprKind::Construct {
                class_name: qualified_name.to_string(),
                init_mangled_name: init_method.mangled_name.clone(),
                args: tir_args,
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
