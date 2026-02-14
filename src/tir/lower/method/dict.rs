use anyhow::Result;

use crate::tir::{
    builtin::BuiltinFn, CallResult, CallTarget, IntrinsicOp, TirExpr, TirExprKind, TirStmt,
    ValueType,
};

use super::super::Lowering;

fn dict_key_eq_tag(ctx: &mut Lowering, line: usize, key_type: &ValueType) -> Result<i64> {
    ctx.require_intrinsic_eq_support(line, key_type)?;
    Ok(ctx.register_intrinsic_instance(IntrinsicOp::Eq, key_type))
}

/// Lower a method call on a dict to TIR.
///
/// Handles all dict methods:
/// - Regular methods: clear, copy, get, keys, pop, setdefault, update, values
/// - Magic methods: __contains__, __eq__, __getitem__, __setitem__, __delitem__, __len__
///
/// Directly generates TIR without using type rules - all logic is self-contained here.
pub fn lower_dict_method_call(
    ctx: &mut Lowering,
    line: usize,
    obj: TirExpr,
    method_name: &str,
    args: Vec<TirExpr>,
    key_type: &ValueType,
    value_type: &ValueType,
) -> Result<CallResult> {
    let dict_ty = ValueType::Dict(Box::new(key_type.clone()), Box::new(value_type.clone()));
    let type_name = format!("dict[{}, {}]", key_type, value_type);

    match method_name {
        // ── Regular Methods ──────────────────────────────────────────────
        "clear" => {
            super::check_arity(ctx, line, &type_name, method_name, 0, args.len())?;
            Ok(super::void_call(BuiltinFn::DictClear, obj.clone(), args))
        }

        "copy" => {
            super::check_arity(ctx, line, &type_name, method_name, 0, args.len())?;
            Ok(super::expr_call(
                BuiltinFn::DictCopy,
                dict_ty,
                obj.clone(),
                args,
            ))
        }

        "get" => {
            if args.len() != 1 && args.len() != 2 {
                return Err(ctx.type_error(
                    line,
                    format!(
                        "{}.{}() takes 1 or 2 arguments, got {}",
                        type_name,
                        method_name,
                        args.len()
                    ),
                ));
            }
            super::check_type(ctx, line, &type_name, method_name, &args[0], key_type)?;
            let key_eq_tag = dict_key_eq_tag(ctx, line, key_type)?;
            let (func, call_args) = if args.len() == 1 {
                (
                    BuiltinFn::DictGetByTag,
                    vec![
                        obj.clone(),
                        args[0].clone(),
                        TirExpr {
                            kind: TirExprKind::IntLiteral(key_eq_tag),
                            ty: ValueType::Int,
                        },
                    ],
                )
            } else {
                super::check_type(ctx, line, &type_name, method_name, &args[1], value_type)?;
                (
                    BuiltinFn::DictGetDefaultByTag,
                    vec![
                        obj.clone(),
                        args[0].clone(),
                        args[1].clone(),
                        TirExpr {
                            kind: TirExprKind::IntLiteral(key_eq_tag),
                            ty: ValueType::Int,
                        },
                    ],
                )
            };
            Ok(CallResult::Expr(TirExpr {
                kind: TirExprKind::ExternalCall {
                    func,
                    args: call_args,
                },
                ty: value_type.clone(),
            }))
        }

        "keys" => {
            super::check_arity(ctx, line, &type_name, method_name, 0, args.len())?;
            Ok(super::expr_call(
                BuiltinFn::DictKeys,
                ValueType::List(Box::new(key_type.clone())),
                obj.clone(),
                args,
            ))
        }

        "items" => {
            super::check_arity(ctx, line, &type_name, method_name, 0, args.len())?;
            if matches!(key_type, ValueType::Bool) && matches!(value_type, ValueType::Bool) {
                return Err(ctx.type_error(
                    line,
                    "dict.items() is not supported for dict[bool, bool] yet",
                ));
            }
            Ok(CallResult::Expr(TirExpr {
                kind: TirExprKind::ExternalCall {
                    func: BuiltinFn::DictItems,
                    args: vec![obj.clone()],
                },
                ty: ValueType::List(Box::new(ValueType::Tuple(vec![
                    key_type.clone(),
                    value_type.clone(),
                ]))),
            }))
        }

        "pop" => {
            if args.len() != 1 && args.len() != 2 {
                return Err(ctx.type_error(
                    line,
                    format!(
                        "{}.{}() takes 1 or 2 arguments, got {}",
                        type_name,
                        method_name,
                        args.len()
                    ),
                ));
            }
            super::check_type(ctx, line, &type_name, method_name, &args[0], key_type)?;
            let key_eq_tag = dict_key_eq_tag(ctx, line, key_type)?;
            let (func, call_args) = if args.len() == 1 {
                (
                    BuiltinFn::DictPopByTag,
                    vec![
                        obj.clone(),
                        args[0].clone(),
                        TirExpr {
                            kind: TirExprKind::IntLiteral(key_eq_tag),
                            ty: ValueType::Int,
                        },
                    ],
                )
            } else {
                super::check_type(ctx, line, &type_name, method_name, &args[1], value_type)?;
                (
                    BuiltinFn::DictPopDefaultByTag,
                    vec![
                        obj.clone(),
                        args[0].clone(),
                        args[1].clone(),
                        TirExpr {
                            kind: TirExprKind::IntLiteral(key_eq_tag),
                            ty: ValueType::Int,
                        },
                    ],
                )
            };
            Ok(CallResult::Expr(TirExpr {
                kind: TirExprKind::ExternalCall {
                    func,
                    args: call_args,
                },
                ty: value_type.clone(),
            }))
        }

        "setdefault" => {
            if args.len() != 2 {
                return Err(ctx.type_error(
                    line,
                    format!(
                        "{}.{}() takes 2 arguments, got {}",
                        type_name,
                        method_name,
                        args.len()
                    ),
                ));
            }
            super::check_type(ctx, line, &type_name, method_name, &args[0], key_type)?;
            super::check_type(ctx, line, &type_name, method_name, &args[1], value_type)?;
            let key_eq_tag = dict_key_eq_tag(ctx, line, key_type)?;
            Ok(CallResult::Expr(TirExpr {
                kind: TirExprKind::ExternalCall {
                    func: BuiltinFn::DictSetDefaultByTag,
                    args: vec![
                        obj.clone(),
                        args[0].clone(),
                        args[1].clone(),
                        TirExpr {
                            kind: TirExprKind::IntLiteral(key_eq_tag),
                            ty: ValueType::Int,
                        },
                    ],
                },
                ty: value_type.clone(),
            }))
        }

        "fromkeys" => {
            if args.len() != 2 {
                return Err(ctx.type_error(
                    line,
                    format!(
                        "{}.{}() takes 2 arguments, got {}",
                        type_name,
                        method_name,
                        args.len()
                    ),
                ));
            }
            super::check_type(
                ctx,
                line,
                &type_name,
                method_name,
                &args[0],
                &ValueType::List(Box::new(key_type.clone())),
            )?;
            super::check_type(ctx, line, &type_name, method_name, &args[1], value_type)?;
            let key_eq_tag = dict_key_eq_tag(ctx, line, key_type)?;
            Ok(CallResult::Expr(TirExpr {
                kind: TirExprKind::ExternalCall {
                    func: BuiltinFn::DictFromKeysByTag,
                    args: vec![
                        args[0].clone(),
                        args[1].clone(),
                        TirExpr {
                            kind: TirExprKind::IntLiteral(key_eq_tag),
                            ty: ValueType::Int,
                        },
                    ],
                },
                ty: dict_ty.clone(),
            }))
        }

        "update" => {
            super::check_arity(ctx, line, &type_name, method_name, 1, args.len())?;
            super::check_type(ctx, line, &type_name, method_name, &args[0], &dict_ty)?;
            let key_eq_tag = dict_key_eq_tag(ctx, line, key_type)?;
            Ok(CallResult::VoidStmt(Box::new(TirStmt::VoidCall {
                target: CallTarget::Builtin(BuiltinFn::DictUpdateByTag),
                args: vec![
                    obj.clone(),
                    args[0].clone(),
                    TirExpr {
                        kind: TirExprKind::IntLiteral(key_eq_tag),
                        ty: ValueType::Int,
                    },
                ],
            })))
        }

        "values" => {
            super::check_arity(ctx, line, &type_name, method_name, 0, args.len())?;
            Ok(super::expr_call(
                BuiltinFn::DictValues,
                ValueType::List(Box::new(value_type.clone())),
                obj.clone(),
                args,
            ))
        }

        "popitem" => {
            super::check_arity(ctx, line, &type_name, method_name, 0, args.len())?;
            if matches!(key_type, ValueType::Bool) && matches!(value_type, ValueType::Bool) {
                return Err(ctx.type_error(
                    line,
                    "dict.popitem() is not supported for dict[bool, bool] yet",
                ));
            }
            Ok(CallResult::Expr(TirExpr {
                kind: TirExprKind::ExternalCall {
                    func: BuiltinFn::DictPopItem,
                    args: vec![obj.clone()],
                },
                ty: ValueType::Tuple(vec![key_type.clone(), value_type.clone()]),
            }))
        }

        // ── Magic Methods ────────────────────────────────────────────────
        "__contains__" => {
            super::check_arity(ctx, line, &type_name, method_name, 1, args.len())?;
            super::check_type(ctx, line, &type_name, method_name, &args[0], key_type)?;
            let key_eq_tag = dict_key_eq_tag(ctx, line, key_type)?;
            Ok(CallResult::Expr(TirExpr {
                kind: TirExprKind::ExternalCall {
                    func: BuiltinFn::DictContainsByTag,
                    args: vec![
                        obj.clone(),
                        args[0].clone(),
                        TirExpr {
                            kind: TirExprKind::IntLiteral(key_eq_tag),
                            ty: ValueType::Int,
                        },
                    ],
                },
                ty: ValueType::Bool,
            }))
        }

        "__eq__" => {
            super::check_arity(ctx, line, &type_name, method_name, 1, args.len())?;
            super::check_type(ctx, line, &type_name, method_name, &args[0], &dict_ty)?;
            ctx.require_intrinsic_eq_support(line, key_type)?;
            ctx.require_intrinsic_eq_support(line, value_type)?;
            let key_eq_tag = ctx.register_intrinsic_instance(IntrinsicOp::Eq, key_type);
            let value_eq_tag = ctx.register_intrinsic_instance(IntrinsicOp::Eq, value_type);
            Ok(CallResult::Expr(TirExpr {
                kind: TirExprKind::ExternalCall {
                    func: BuiltinFn::DictEqByTag,
                    args: vec![
                        obj.clone(),
                        args[0].clone(),
                        TirExpr {
                            kind: TirExprKind::IntLiteral(key_eq_tag),
                            ty: ValueType::Int,
                        },
                        TirExpr {
                            kind: TirExprKind::IntLiteral(value_eq_tag),
                            ty: ValueType::Int,
                        },
                    ],
                },
                ty: ValueType::Bool,
            }))
        }

        "__ne__" => {
            super::check_arity(ctx, line, &type_name, method_name, 1, args.len())?;
            super::check_type(ctx, line, &type_name, method_name, &args[0], &dict_ty)?;
            ctx.require_intrinsic_eq_support(line, key_type)?;
            ctx.require_intrinsic_eq_support(line, value_type)?;
            let key_eq_tag = ctx.register_intrinsic_instance(IntrinsicOp::Eq, key_type);
            let value_eq_tag = ctx.register_intrinsic_instance(IntrinsicOp::Eq, value_type);
            let eq_expr = TirExpr {
                kind: TirExprKind::ExternalCall {
                    func: BuiltinFn::DictEqByTag,
                    args: vec![
                        obj.clone(),
                        args[0].clone(),
                        TirExpr {
                            kind: TirExprKind::IntLiteral(key_eq_tag),
                            ty: ValueType::Int,
                        },
                        TirExpr {
                            kind: TirExprKind::IntLiteral(value_eq_tag),
                            ty: ValueType::Int,
                        },
                    ],
                },
                ty: ValueType::Bool,
            };
            Ok(CallResult::Expr(TirExpr {
                kind: TirExprKind::Not(Box::new(eq_expr)),
                ty: ValueType::Bool,
            }))
        }

        "__getitem__" => {
            super::check_arity(ctx, line, &type_name, method_name, 1, args.len())?;
            super::check_type(ctx, line, &type_name, method_name, &args[0], key_type)?;
            let key_eq_tag = dict_key_eq_tag(ctx, line, key_type)?;
            Ok(CallResult::Expr(TirExpr {
                kind: TirExprKind::ExternalCall {
                    func: BuiltinFn::DictGetByTag,
                    args: vec![
                        obj.clone(),
                        args[0].clone(),
                        TirExpr {
                            kind: TirExprKind::IntLiteral(key_eq_tag),
                            ty: ValueType::Int,
                        },
                    ],
                },
                ty: value_type.clone(),
            }))
        }

        "__setitem__" => {
            super::check_arity(ctx, line, &type_name, method_name, 2, args.len())?;
            super::check_type(ctx, line, &type_name, method_name, &args[0], key_type)?;
            super::check_type(ctx, line, &type_name, method_name, &args[1], value_type)?;
            let key_eq_tag = dict_key_eq_tag(ctx, line, key_type)?;
            Ok(CallResult::VoidStmt(Box::new(TirStmt::VoidCall {
                target: CallTarget::Builtin(BuiltinFn::DictSetByTag),
                args: vec![
                    obj.clone(),
                    args[0].clone(),
                    args[1].clone(),
                    TirExpr {
                        kind: TirExprKind::IntLiteral(key_eq_tag),
                        ty: ValueType::Int,
                    },
                ],
            })))
        }

        "__delitem__" => {
            super::check_arity(ctx, line, &type_name, method_name, 1, args.len())?;
            super::check_type(ctx, line, &type_name, method_name, &args[0], key_type)?;
            let key_eq_tag = dict_key_eq_tag(ctx, line, key_type)?;
            Ok(CallResult::VoidStmt(Box::new(TirStmt::VoidCall {
                target: CallTarget::Builtin(BuiltinFn::DictDelByTag),
                args: vec![
                    obj.clone(),
                    args[0].clone(),
                    TirExpr {
                        kind: TirExprKind::IntLiteral(key_eq_tag),
                        ty: ValueType::Int,
                    },
                ],
            })))
        }

        "__or__" | "__ror__" | "__ior__" => {
            super::check_arity(ctx, line, &type_name, method_name, 1, args.len())?;
            super::check_type(ctx, line, &type_name, method_name, &args[0], &dict_ty)?;
            let key_eq_tag = dict_key_eq_tag(ctx, line, key_type)?;
            let (func, lhs, rhs) = match method_name {
                "__or__" => (BuiltinFn::DictOrByTag, obj.clone(), args[0].clone()),
                "__ror__" => (BuiltinFn::DictOrByTag, args[0].clone(), obj.clone()),
                "__ior__" => (BuiltinFn::DictIOrByTag, obj.clone(), args[0].clone()),
                _ => unreachable!(),
            };
            Ok(CallResult::Expr(TirExpr {
                kind: TirExprKind::ExternalCall {
                    func,
                    args: vec![
                        lhs,
                        rhs,
                        TirExpr {
                            kind: TirExprKind::IntLiteral(key_eq_tag),
                            ty: ValueType::Int,
                        },
                    ],
                },
                ty: dict_ty.clone(),
            }))
        }

        "__iter__" => {
            super::check_arity(ctx, line, &type_name, method_name, 0, args.len())?;
            Ok(super::expr_call(
                BuiltinFn::DictKeys,
                ValueType::List(Box::new(key_type.clone())),
                obj.clone(),
                args,
            ))
        }

        "__reversed__" => {
            super::check_arity(ctx, line, &type_name, method_name, 0, args.len())?;
            let keys_expr = TirExpr {
                kind: TirExprKind::ExternalCall {
                    func: BuiltinFn::DictKeys,
                    args: vec![obj.clone()],
                },
                ty: ValueType::List(Box::new(key_type.clone())),
            };
            Ok(CallResult::Expr(TirExpr {
                kind: TirExprKind::ExternalCall {
                    func: BuiltinFn::ReversedList,
                    args: vec![keys_expr],
                },
                ty: ValueType::List(Box::new(key_type.clone())),
            }))
        }

        "__lt__" | "__le__" | "__gt__" | "__ge__" => {
            super::check_arity(ctx, line, &type_name, method_name, 1, args.len())?;
            super::check_type(ctx, line, &type_name, method_name, &args[0], &dict_ty)?;
            Err(ctx.type_error(line, "dict ordering comparison is not supported"))
        }

        "__len__" => {
            super::check_arity(ctx, line, &type_name, method_name, 0, args.len())?;
            Ok(super::expr_call(
                BuiltinFn::DictLen,
                ValueType::Int,
                obj.clone(),
                args,
            ))
        }

        // ── Unknown Method ───────────────────────────────────────────────
        _ => Err(ctx.attribute_error(
            line,
            format!(
                "dict[{}, {}] has no method `{}`",
                key_type, value_type, method_name
            ),
        )),
    }
}
