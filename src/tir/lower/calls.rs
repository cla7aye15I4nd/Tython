use anyhow::Result;
use pyo3::prelude::*;

use crate::ast::{ClassInfo, Type};
use crate::tir::{
    builtin, type_rules, CallResult, CallTarget, FloatArithOp, TirExpr, TirExprKind, TirStmt,
    TypedBinOp, ValueType,
};
use crate::{ast_get_list, ast_get_string, ast_getattr, ast_type_name};

use super::Lowering;

impl Lowering {
    pub(super) fn lower_call(&mut self, node: &Bound<PyAny>, line: usize) -> Result<CallResult> {
        let func_node = ast_getattr!(node, "func");
        let args_list = ast_get_list!(node, "args");

        let mut tir_args = Vec::new();
        for arg in args_list.iter() {
            tir_args.push(self.lower_expr(&arg)?);
        }

        let func_node_type = ast_type_name!(func_node);
        match func_node_type.as_str() {
            "Name" => {
                let func_name = ast_get_string!(func_node, "id");

                if func_name == "print" {
                    return Err(self.syntax_error(line, "print() can only be used as a statement"));
                }

                if tir_args.len() == 1 && matches!(tir_args[0].ty, ValueType::Class(_)) {
                    if let Some(magic_rule) = type_rules::lookup_builtin_class_magic(&func_name) {
                        let arg = tir_args.remove(0);
                        return Ok(CallResult::Expr(self.lower_class_magic_method(
                            line,
                            arg,
                            magic_rule.method_names,
                            magic_rule.return_type,
                            &func_name,
                        )?));
                    }
                }

                if type_rules::is_builtin_call(&func_name) {
                    let arg_types: Vec<&ValueType> = tir_args.iter().map(|a| &a.ty).collect();
                    let rule = type_rules::lookup_builtin_call(&func_name, &arg_types).ok_or_else(
                        || {
                            self.type_error(
                                line,
                                type_rules::builtin_call_error_message(
                                    &func_name,
                                    &arg_types,
                                    tir_args.len(),
                                ),
                            )
                        },
                    )?;
                    return Ok(Self::lower_builtin_rule(rule, tir_args));
                }

                let scope_type = self.lookup(&func_name).cloned().ok_or_else(|| {
                    self.name_error(line, format!("undefined function `{}`", func_name))
                })?;

                match &scope_type {
                    Type::Function { .. } => {
                        let return_type =
                            self.check_call_args(line, &func_name, &scope_type, &tir_args)?;
                        let mangled = self.mangle_name(&func_name);
                        if return_type == Type::Unit {
                            Ok(CallResult::VoidStmt(Box::new(TirStmt::VoidCall {
                                target: CallTarget::Named(mangled),
                                args: tir_args,
                            })))
                        } else {
                            Ok(CallResult::Expr(TirExpr {
                                kind: TirExprKind::Call {
                                    func: mangled,
                                    args: tir_args,
                                },
                                ty: Self::to_value_type(&return_type),
                            }))
                        }
                    }
                    Type::Module(mangled) => {
                        // Check if this is an imported class constructor
                        if let Some(class_info) = self.class_registry.get(mangled).cloned() {
                            return self.lower_constructor_call(
                                line,
                                mangled,
                                &class_info,
                                tir_args,
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
                        let return_type =
                            self.check_call_args(line, &func_name, &func_type, &tir_args)?;
                        if return_type == Type::Unit {
                            Ok(CallResult::VoidStmt(Box::new(TirStmt::VoidCall {
                                target: CallTarget::Named(mangled.clone()),
                                args: tir_args,
                            })))
                        } else {
                            Ok(CallResult::Expr(TirExpr {
                                kind: TirExprKind::Call {
                                    func: mangled.clone(),
                                    args: tir_args,
                                },
                                ty: Self::to_value_type(&return_type),
                            }))
                        }
                    }
                    Type::Class(name) => {
                        // Constructor call
                        let class_info = self
                            .class_registry
                            .get(name)
                            .ok_or_else(|| {
                                self.name_error(line, format!("unknown class `{}`", name))
                            })?
                            .clone();
                        self.lower_constructor_call(line, name, &class_info, tir_args)
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
                            return self.lower_constructor_call(
                                line,
                                &resolved,
                                &class_info,
                                tir_args,
                            );
                        }

                        let func_type = self
                            .symbol_table
                            .get(&resolved)
                            .ok_or_else(|| {
                                self.name_error(line, format!("undefined function `{}`", attr))
                            })?
                            .clone();

                        let return_type = {
                            let label = attr.to_string();
                            self.check_call_args(line, &label, &func_type, &tir_args)?
                        };

                        return if return_type == Type::Unit {
                            Ok(CallResult::VoidStmt(Box::new(TirStmt::VoidCall {
                                target: CallTarget::Named(resolved),
                                args: tir_args,
                            })))
                        } else {
                            Ok(CallResult::Expr(TirExpr {
                                kind: TirExprKind::Call {
                                    func: resolved,
                                    args: tir_args,
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
                        return self.lower_constructor_call(
                            line,
                            &qualified,
                            &class_info,
                            tir_args,
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

                        if tir_args.len() != method.params.len() {
                            return Err(self.type_error(
                                line,
                                format!(
                                    "{}.{}() expects {} argument{}, got {}",
                                    class_name,
                                    attr,
                                    method.params.len(),
                                    if method.params.len() == 1 { "" } else { "s" },
                                    tir_args.len()
                                ),
                            ));
                        }
                        for (i, (arg, expected)) in
                            tir_args.iter().zip(method.params.iter()).enumerate()
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

                        if *return_type == Type::Unit {
                            Ok(CallResult::VoidStmt(Box::new(TirStmt::VoidCall {
                                target: CallTarget::MethodCall {
                                    mangled_name: mangled,
                                    object: obj_expr,
                                },
                                args: tir_args,
                            })))
                        } else {
                            Ok(CallResult::Expr(TirExpr {
                                kind: TirExprKind::MethodCall {
                                    object: Box::new(obj_expr),
                                    method_mangled_name: mangled,
                                    args: tir_args,
                                },
                                ty: Self::to_value_type(return_type),
                            }))
                        }
                    }
                    ValueType::ByteArray => match attr.as_str() {
                        "append" => {
                            if tir_args.len() != 1 {
                                return Err(self.type_error(
                                    line,
                                    format!(
                                        "bytearray.append() expects 1 argument, got {}",
                                        tir_args.len()
                                    ),
                                ));
                            }
                            if tir_args[0].ty != ValueType::Int {
                                return Err(self.type_error(
                                    line,
                                    format!(
                                        "bytearray.append() expects `int`, got `{}`",
                                        tir_args[0].ty
                                    ),
                                ));
                            }
                            Ok(CallResult::VoidStmt(Box::new(TirStmt::VoidCall {
                                target: CallTarget::Builtin(builtin::BuiltinFn::ByteArrayAppend),
                                args: vec![obj_expr, tir_args.remove(0)],
                            })))
                        }
                        "extend" => {
                            if tir_args.len() != 1 {
                                return Err(self.type_error(
                                    line,
                                    format!(
                                        "bytearray.extend() expects 1 argument, got {}",
                                        tir_args.len()
                                    ),
                                ));
                            }
                            if tir_args[0].ty != ValueType::Bytes {
                                return Err(self.type_error(
                                    line,
                                    format!(
                                        "bytearray.extend() expects `bytes`, got `{}`",
                                        tir_args[0].ty
                                    ),
                                ));
                            }
                            Ok(CallResult::VoidStmt(Box::new(TirStmt::VoidCall {
                                target: CallTarget::Builtin(builtin::BuiltinFn::ByteArrayExtend),
                                args: vec![obj_expr, tir_args.remove(0)],
                            })))
                        }
                        "clear" => {
                            if !tir_args.is_empty() {
                                return Err(self.type_error(
                                    line,
                                    "bytearray.clear() takes no arguments".to_string(),
                                ));
                            }
                            Ok(CallResult::VoidStmt(Box::new(TirStmt::VoidCall {
                                target: CallTarget::Builtin(builtin::BuiltinFn::ByteArrayClear),
                                args: vec![obj_expr],
                            })))
                        }
                        _ => Err(self
                            .attribute_error(line, format!("bytearray has no method `{}`", attr))),
                    },
                    ValueType::List(inner) => {
                        match attr.as_str() {
                            "append" => {
                                if !inner.is_primitive() {
                                    return Err(self.type_error(
                                    line,
                                    format!(
                                        "list[{}].append() is not supported; only list[int], list[float], list[bool] support append",
                                        inner
                                    ),
                                ));
                                }
                                if tir_args.len() != 1 {
                                    return Err(self.type_error(
                                        line,
                                        format!(
                                            "list.append() expects 1 argument, got {}",
                                            tir_args.len()
                                        ),
                                    ));
                                }
                                if tir_args[0].ty != *inner.as_ref() {
                                    return Err(self.type_error(
                                        line,
                                        format!(
                                            "list[{}].append() expects `{}`, got `{}`",
                                            inner, inner, tir_args[0].ty
                                        ),
                                    ));
                                }
                                Ok(CallResult::VoidStmt(Box::new(TirStmt::VoidCall {
                                    target: CallTarget::Builtin(builtin::BuiltinFn::ListAppend),
                                    args: vec![obj_expr, tir_args.remove(0)],
                                })))
                            }
                            "clear" => {
                                if !tir_args.is_empty() {
                                    return Err(self.type_error(
                                        line,
                                        "list.clear() takes no arguments".to_string(),
                                    ));
                                }
                                Ok(CallResult::VoidStmt(Box::new(TirStmt::VoidCall {
                                    target: CallTarget::Builtin(builtin::BuiltinFn::ListClear),
                                    args: vec![obj_expr],
                                })))
                            }
                            "pop" => {
                                if !tir_args.is_empty() {
                                    return Err(self.type_error(
                                        line,
                                        "list.pop() takes no arguments".to_string(),
                                    ));
                                }
                                Ok(CallResult::Expr(TirExpr {
                                    kind: TirExprKind::ExternalCall {
                                        func: builtin::BuiltinFn::ListPop,
                                        args: vec![obj_expr],
                                    },
                                    ty: (*inner).clone(),
                                }))
                            }
                            _ => Err(self
                                .attribute_error(line, format!("list has no method `{}`", attr))),
                        }
                    }
                    other => {
                        Err(self.type_error(line, format!("`{}` is not a class instance", other)))
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

    fn lower_builtin_rule(
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
        expected_return_type: ValueType,
        caller_name: &str,
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

        if !method.params.is_empty() {
            return Err(self.type_error(
                line,
                format!(
                    "{}.{}() must take no arguments besides `self`, found {}",
                    class_name,
                    method.name,
                    method.params.len()
                ),
            ));
        }

        if method.return_type != expected_return_type.to_type() {
            return Err(self.type_error(
                line,
                format!(
                    "{}.{}() must return `{}`, got `{}`",
                    class_name, method.name, expected_return_type, method.return_type
                ),
            ));
        }

        Ok(TirExpr {
            kind: TirExprKind::MethodCall {
                object: Box::new(object),
                method_mangled_name: method.mangled_name.clone(),
                args: vec![],
            },
            ty: expected_return_type,
        })
    }
}
