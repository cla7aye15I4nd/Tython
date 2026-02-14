use anyhow::Result;

use crate::tir::{
    builtin::BuiltinFn, CallResult, CallTarget, IntrinsicOp, TirExpr, TirExprKind, TirStmt,
    ValueType,
};

use super::super::Lowering;

fn set_eq_tag(ctx: &mut Lowering, line: usize, inner_type: &ValueType) -> Result<i64> {
    ctx.require_intrinsic_eq_support(line, inner_type)?;
    Ok(ctx.register_intrinsic_instance(IntrinsicOp::Eq, inner_type))
}

/// Lower a method call on a set to TIR.
///
/// Handles all set methods:
/// - Regular methods: add, clear, copy, discard, pop, remove
/// - Algebra methods: difference/intersection/union/update variants
/// - Relation methods: isdisjoint/issubset/issuperset
/// - Magic methods: set operators + comparisons + contains/eq/len
///
/// Directly generates TIR without using type rules - all logic is self-contained here.
pub fn lower_set_method_call(
    ctx: &mut Lowering,
    line: usize,
    obj: TirExpr,
    method_name: &str,
    args: Vec<TirExpr>,
    inner_type: &ValueType,
) -> Result<CallResult> {
    let set_ty = ValueType::Set(Box::new(inner_type.clone()));
    let type_name = format!("set[{}]", inner_type);

    match method_name {
        // ── Regular Methods ──────────────────────────────────────────────
        "add" => {
            super::check_arity(ctx, line, &type_name, method_name, 1, args.len())?;
            super::check_type(ctx, line, &type_name, method_name, &args[0], inner_type)?;
            let eq_tag = set_eq_tag(ctx, line, inner_type)?;
            Ok(CallResult::VoidStmt(Box::new(TirStmt::VoidCall {
                target: CallTarget::Builtin(BuiltinFn::SetAddByTag),
                args: vec![
                    obj.clone(),
                    args[0].clone(),
                    TirExpr {
                        kind: TirExprKind::IntLiteral(eq_tag),
                        ty: ValueType::Int,
                    },
                ],
            })))
        }

        "clear" => {
            super::check_arity(ctx, line, &type_name, method_name, 0, args.len())?;
            Ok(super::void_call(BuiltinFn::SetClear, obj.clone(), args))
        }

        "copy" => {
            super::check_arity(ctx, line, &type_name, method_name, 0, args.len())?;
            Ok(super::expr_call(
                BuiltinFn::SetCopy,
                set_ty,
                obj.clone(),
                args,
            ))
        }

        "discard" => {
            super::check_arity(ctx, line, &type_name, method_name, 1, args.len())?;
            super::check_type(ctx, line, &type_name, method_name, &args[0], inner_type)?;
            let eq_tag = set_eq_tag(ctx, line, inner_type)?;
            Ok(CallResult::VoidStmt(Box::new(TirStmt::VoidCall {
                target: CallTarget::Builtin(BuiltinFn::SetDiscardByTag),
                args: vec![
                    obj.clone(),
                    args[0].clone(),
                    TirExpr {
                        kind: TirExprKind::IntLiteral(eq_tag),
                        ty: ValueType::Int,
                    },
                ],
            })))
        }

        "pop" => {
            super::check_arity(ctx, line, &type_name, method_name, 0, args.len())?;
            Ok(super::expr_call(
                BuiltinFn::SetPop,
                inner_type.clone(),
                obj.clone(),
                args,
            ))
        }

        "remove" => {
            super::check_arity(ctx, line, &type_name, method_name, 1, args.len())?;
            super::check_type(ctx, line, &type_name, method_name, &args[0], inner_type)?;
            let eq_tag = set_eq_tag(ctx, line, inner_type)?;
            Ok(CallResult::VoidStmt(Box::new(TirStmt::VoidCall {
                target: CallTarget::Builtin(BuiltinFn::SetRemoveByTag),
                args: vec![
                    obj.clone(),
                    args[0].clone(),
                    TirExpr {
                        kind: TirExprKind::IntLiteral(eq_tag),
                        ty: ValueType::Int,
                    },
                ],
            })))
        }

        "difference" | "intersection" | "symmetric_difference" | "union" => {
            super::check_arity(ctx, line, &type_name, method_name, 1, args.len())?;
            super::check_type(ctx, line, &type_name, method_name, &args[0], &set_ty)?;
            let eq_tag = set_eq_tag(ctx, line, inner_type)?;
            let func = if method_name == "difference" {
                BuiltinFn::SetDifferenceByTag
            } else if method_name == "intersection" {
                BuiltinFn::SetIntersectionByTag
            } else if method_name == "symmetric_difference" {
                BuiltinFn::SetSymmetricDifferenceByTag
            } else {
                BuiltinFn::SetUnionByTag
            };
            Ok(CallResult::Expr(TirExpr {
                kind: TirExprKind::ExternalCall {
                    func,
                    args: vec![
                        obj.clone(),
                        args[0].clone(),
                        TirExpr {
                            kind: TirExprKind::IntLiteral(eq_tag),
                            ty: ValueType::Int,
                        },
                    ],
                },
                ty: set_ty.clone(),
            }))
        }

        "difference_update" | "intersection_update" | "symmetric_difference_update" | "update" => {
            super::check_arity(ctx, line, &type_name, method_name, 1, args.len())?;
            super::check_type(ctx, line, &type_name, method_name, &args[0], &set_ty)?;
            let eq_tag = set_eq_tag(ctx, line, inner_type)?;
            let func = if method_name == "difference_update" {
                BuiltinFn::SetDifferenceUpdateByTag
            } else if method_name == "intersection_update" {
                BuiltinFn::SetIntersectionUpdateByTag
            } else if method_name == "symmetric_difference_update" {
                BuiltinFn::SetSymmetricDifferenceUpdateByTag
            } else {
                BuiltinFn::SetUpdateByTag
            };
            Ok(CallResult::VoidStmt(Box::new(TirStmt::VoidCall {
                target: CallTarget::Builtin(func),
                args: vec![
                    obj.clone(),
                    args[0].clone(),
                    TirExpr {
                        kind: TirExprKind::IntLiteral(eq_tag),
                        ty: ValueType::Int,
                    },
                ],
            })))
        }

        "isdisjoint" | "issubset" | "issuperset" => {
            super::check_arity(ctx, line, &type_name, method_name, 1, args.len())?;
            super::check_type(ctx, line, &type_name, method_name, &args[0], &set_ty)?;
            let eq_tag = set_eq_tag(ctx, line, inner_type)?;
            let func = if method_name == "isdisjoint" {
                BuiltinFn::SetIsDisjointByTag
            } else if method_name == "issubset" {
                BuiltinFn::SetIsSubsetByTag
            } else {
                BuiltinFn::SetIsSupersetByTag
            };
            Ok(CallResult::Expr(TirExpr {
                kind: TirExprKind::ExternalCall {
                    func,
                    args: vec![
                        obj.clone(),
                        args[0].clone(),
                        TirExpr {
                            kind: TirExprKind::IntLiteral(eq_tag),
                            ty: ValueType::Int,
                        },
                    ],
                },
                ty: ValueType::Bool,
            }))
        }

        // ── Magic Methods ────────────────────────────────────────────────
        "__contains__" => {
            super::check_arity(ctx, line, &type_name, method_name, 1, args.len())?;
            super::check_type(ctx, line, &type_name, method_name, &args[0], inner_type)?;
            let eq_tag = set_eq_tag(ctx, line, inner_type)?;
            Ok(CallResult::Expr(TirExpr {
                kind: TirExprKind::ExternalCall {
                    func: BuiltinFn::SetContainsByTag,
                    args: vec![
                        obj.clone(),
                        args[0].clone(),
                        TirExpr {
                            kind: TirExprKind::IntLiteral(eq_tag),
                            ty: ValueType::Int,
                        },
                    ],
                },
                ty: ValueType::Bool,
            }))
        }

        "__eq__" => {
            super::check_arity(ctx, line, &type_name, method_name, 1, args.len())?;
            super::check_type(ctx, line, &type_name, method_name, &args[0], &set_ty)?;
            let eq_tag = set_eq_tag(ctx, line, inner_type)?;
            Ok(CallResult::Expr(TirExpr {
                kind: TirExprKind::ExternalCall {
                    func: BuiltinFn::SetEqByTag,
                    args: vec![
                        obj.clone(),
                        args[0].clone(),
                        TirExpr {
                            kind: TirExprKind::IntLiteral(eq_tag),
                            ty: ValueType::Int,
                        },
                    ],
                },
                ty: ValueType::Bool,
            }))
        }

        "__ne__" => {
            super::check_arity(ctx, line, &type_name, method_name, 1, args.len())?;
            super::check_type(ctx, line, &type_name, method_name, &args[0], &set_ty)?;
            let eq_tag = set_eq_tag(ctx, line, inner_type)?;
            let eq_expr = TirExpr {
                kind: TirExprKind::ExternalCall {
                    func: BuiltinFn::SetEqByTag,
                    args: vec![
                        obj.clone(),
                        args[0].clone(),
                        TirExpr {
                            kind: TirExprKind::IntLiteral(eq_tag),
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

        "__and__" | "__or__" | "__sub__" | "__xor__" | "__rand__" | "__ror__" | "__rsub__"
        | "__rxor__" => {
            super::check_arity(ctx, line, &type_name, method_name, 1, args.len())?;
            super::check_type(ctx, line, &type_name, method_name, &args[0], &set_ty)?;
            let eq_tag = set_eq_tag(ctx, line, inner_type)?;
            let (func, reversed) = if method_name == "__and__" {
                (BuiltinFn::SetIntersectionByTag, false)
            } else if method_name == "__or__" {
                (BuiltinFn::SetUnionByTag, false)
            } else if method_name == "__sub__" {
                (BuiltinFn::SetDifferenceByTag, false)
            } else if method_name == "__xor__" {
                (BuiltinFn::SetSymmetricDifferenceByTag, false)
            } else if method_name == "__rand__" {
                (BuiltinFn::SetIntersectionByTag, true)
            } else if method_name == "__ror__" {
                (BuiltinFn::SetUnionByTag, true)
            } else if method_name == "__rsub__" {
                (BuiltinFn::SetDifferenceByTag, true)
            } else {
                (BuiltinFn::SetSymmetricDifferenceByTag, true)
            };
            let (lhs, rhs) = if reversed {
                (args[0].clone(), obj.clone())
            } else {
                (obj.clone(), args[0].clone())
            };
            Ok(CallResult::Expr(TirExpr {
                kind: TirExprKind::ExternalCall {
                    func,
                    args: vec![
                        lhs,
                        rhs,
                        TirExpr {
                            kind: TirExprKind::IntLiteral(eq_tag),
                            ty: ValueType::Int,
                        },
                    ],
                },
                ty: set_ty.clone(),
            }))
        }

        "__iand__" | "__ior__" | "__isub__" | "__ixor__" => {
            super::check_arity(ctx, line, &type_name, method_name, 1, args.len())?;
            super::check_type(ctx, line, &type_name, method_name, &args[0], &set_ty)?;
            let eq_tag = set_eq_tag(ctx, line, inner_type)?;
            let func = if method_name == "__iand__" {
                BuiltinFn::SetIAndByTag
            } else if method_name == "__ior__" {
                BuiltinFn::SetIOrByTag
            } else if method_name == "__isub__" {
                BuiltinFn::SetISubByTag
            } else {
                BuiltinFn::SetIXorByTag
            };
            Ok(CallResult::Expr(TirExpr {
                kind: TirExprKind::ExternalCall {
                    func,
                    args: vec![
                        obj.clone(),
                        args[0].clone(),
                        TirExpr {
                            kind: TirExprKind::IntLiteral(eq_tag),
                            ty: ValueType::Int,
                        },
                    ],
                },
                ty: set_ty.clone(),
            }))
        }

        "__lt__" | "__le__" | "__gt__" | "__ge__" => {
            super::check_arity(ctx, line, &type_name, method_name, 1, args.len())?;
            super::check_type(ctx, line, &type_name, method_name, &args[0], &set_ty)?;
            let eq_tag = set_eq_tag(ctx, line, inner_type)?;
            let func = if method_name == "__lt__" {
                BuiltinFn::SetLtByTag
            } else if method_name == "__le__" {
                BuiltinFn::SetLeByTag
            } else if method_name == "__gt__" {
                BuiltinFn::SetGtByTag
            } else {
                BuiltinFn::SetGeByTag
            };
            Ok(CallResult::Expr(TirExpr {
                kind: TirExprKind::ExternalCall {
                    func,
                    args: vec![
                        obj.clone(),
                        args[0].clone(),
                        TirExpr {
                            kind: TirExprKind::IntLiteral(eq_tag),
                            ty: ValueType::Int,
                        },
                    ],
                },
                ty: ValueType::Bool,
            }))
        }

        "__iter__" => {
            super::check_arity(ctx, line, &type_name, method_name, 0, args.len())?;
            Ok(super::expr_call(
                BuiltinFn::SetCopy,
                set_ty.clone(),
                obj.clone(),
                args,
            ))
        }

        "__hash__" => {
            super::check_arity(ctx, line, &type_name, method_name, 0, args.len())?;
            Err(ctx.type_error(line, "unhashable type: 'set'"))
        }

        "__len__" => {
            super::check_arity(ctx, line, &type_name, method_name, 0, args.len())?;
            Ok(super::expr_call(
                BuiltinFn::SetLen,
                ValueType::Int,
                obj.clone(),
                args,
            ))
        }

        // ── Unknown Method ───────────────────────────────────────────────
        _ => Err(ctx.attribute_error(
            line,
            format!("set[{}] has no method `{}`", inner_type, method_name),
        )),
    }
}
