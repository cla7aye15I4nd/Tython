use anyhow::Result;

use crate::tir::{
    builtin::BuiltinFn, CallResult, CallTarget, IntrinsicOp, TirExpr, TirExprKind, TirStmt,
    ValueType,
};

use super::super::Lowering;

fn set_eq_tag(ctx: &mut Lowering, line: usize, inner_type: &ValueType) -> i64 {
    ctx.require_intrinsic_eq_support(line, inner_type);
    ctx.register_intrinsic_instance(IntrinsicOp::Eq, inner_type)
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
            let eq_tag = set_eq_tag(ctx, line, inner_type);
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
            let eq_tag = set_eq_tag(ctx, line, inner_type);
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
            let eq_tag = set_eq_tag(ctx, line, inner_type);
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

        "difference" => {
            super::check_arity(ctx, line, &type_name, method_name, 1, args.len())?;
            super::check_type(ctx, line, &type_name, method_name, &args[0], &set_ty)?;
            let eq_tag = set_eq_tag(ctx, line, inner_type);
            Ok(CallResult::Expr(TirExpr {
                kind: TirExprKind::ExternalCall {
                    func: BuiltinFn::SetDifferenceByTag,
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

        "intersection" => {
            super::check_arity(ctx, line, &type_name, method_name, 1, args.len())?;
            super::check_type(ctx, line, &type_name, method_name, &args[0], &set_ty)?;
            let eq_tag = set_eq_tag(ctx, line, inner_type);
            Ok(CallResult::Expr(TirExpr {
                kind: TirExprKind::ExternalCall {
                    func: BuiltinFn::SetIntersectionByTag,
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

        "symmetric_difference" => {
            super::check_arity(ctx, line, &type_name, method_name, 1, args.len())?;
            super::check_type(ctx, line, &type_name, method_name, &args[0], &set_ty)?;
            let eq_tag = set_eq_tag(ctx, line, inner_type);
            Ok(CallResult::Expr(TirExpr {
                kind: TirExprKind::ExternalCall {
                    func: BuiltinFn::SetSymmetricDifferenceByTag,
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

        "union" => {
            super::check_arity(ctx, line, &type_name, method_name, 1, args.len())?;
            super::check_type(ctx, line, &type_name, method_name, &args[0], &set_ty)?;
            let eq_tag = set_eq_tag(ctx, line, inner_type);
            Ok(CallResult::Expr(TirExpr {
                kind: TirExprKind::ExternalCall {
                    func: BuiltinFn::SetUnionByTag,
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

        "difference_update" => {
            super::check_arity(ctx, line, &type_name, method_name, 1, args.len())?;
            super::check_type(ctx, line, &type_name, method_name, &args[0], &set_ty)?;
            let eq_tag = set_eq_tag(ctx, line, inner_type);
            Ok(CallResult::VoidStmt(Box::new(TirStmt::VoidCall {
                target: CallTarget::Builtin(BuiltinFn::SetDifferenceUpdateByTag),
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

        "intersection_update" => {
            super::check_arity(ctx, line, &type_name, method_name, 1, args.len())?;
            super::check_type(ctx, line, &type_name, method_name, &args[0], &set_ty)?;
            let eq_tag = set_eq_tag(ctx, line, inner_type);
            Ok(CallResult::VoidStmt(Box::new(TirStmt::VoidCall {
                target: CallTarget::Builtin(BuiltinFn::SetIntersectionUpdateByTag),
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

        "symmetric_difference_update" => {
            super::check_arity(ctx, line, &type_name, method_name, 1, args.len())?;
            super::check_type(ctx, line, &type_name, method_name, &args[0], &set_ty)?;
            let eq_tag = set_eq_tag(ctx, line, inner_type);
            Ok(CallResult::VoidStmt(Box::new(TirStmt::VoidCall {
                target: CallTarget::Builtin(BuiltinFn::SetSymmetricDifferenceUpdateByTag),
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

        "update" => {
            super::check_arity(ctx, line, &type_name, method_name, 1, args.len())?;
            super::check_type(ctx, line, &type_name, method_name, &args[0], &set_ty)?;
            let eq_tag = set_eq_tag(ctx, line, inner_type);
            Ok(CallResult::VoidStmt(Box::new(TirStmt::VoidCall {
                target: CallTarget::Builtin(BuiltinFn::SetUpdateByTag),
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

        "isdisjoint" => {
            super::check_arity(ctx, line, &type_name, method_name, 1, args.len())?;
            super::check_type(ctx, line, &type_name, method_name, &args[0], &set_ty)?;
            let eq_tag = set_eq_tag(ctx, line, inner_type);
            Ok(CallResult::Expr(TirExpr {
                kind: TirExprKind::ExternalCall {
                    func: BuiltinFn::SetIsDisjointByTag,
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

        "issubset" => {
            super::check_arity(ctx, line, &type_name, method_name, 1, args.len())?;
            super::check_type(ctx, line, &type_name, method_name, &args[0], &set_ty)?;
            let eq_tag = set_eq_tag(ctx, line, inner_type);
            Ok(CallResult::Expr(TirExpr {
                kind: TirExprKind::ExternalCall {
                    func: BuiltinFn::SetIsSubsetByTag,
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

        "issuperset" => {
            super::check_arity(ctx, line, &type_name, method_name, 1, args.len())?;
            super::check_type(ctx, line, &type_name, method_name, &args[0], &set_ty)?;
            let eq_tag = set_eq_tag(ctx, line, inner_type);
            Ok(CallResult::Expr(TirExpr {
                kind: TirExprKind::ExternalCall {
                    func: BuiltinFn::SetIsSupersetByTag,
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
            let eq_tag = set_eq_tag(ctx, line, inner_type);
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
            let eq_tag = set_eq_tag(ctx, line, inner_type);
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
            let eq_tag = set_eq_tag(ctx, line, inner_type);
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

        "__and__" => {
            super::check_arity(ctx, line, &type_name, method_name, 1, args.len())?;
            super::check_type(ctx, line, &type_name, method_name, &args[0], &set_ty)?;
            let eq_tag = set_eq_tag(ctx, line, inner_type);
            Ok(CallResult::Expr(TirExpr {
                kind: TirExprKind::ExternalCall {
                    func: BuiltinFn::SetIntersectionByTag,
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

        "__or__" => {
            super::check_arity(ctx, line, &type_name, method_name, 1, args.len())?;
            super::check_type(ctx, line, &type_name, method_name, &args[0], &set_ty)?;
            let eq_tag = set_eq_tag(ctx, line, inner_type);
            Ok(CallResult::Expr(TirExpr {
                kind: TirExprKind::ExternalCall {
                    func: BuiltinFn::SetUnionByTag,
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

        "__sub__" => {
            super::check_arity(ctx, line, &type_name, method_name, 1, args.len())?;
            super::check_type(ctx, line, &type_name, method_name, &args[0], &set_ty)?;
            let eq_tag = set_eq_tag(ctx, line, inner_type);
            Ok(CallResult::Expr(TirExpr {
                kind: TirExprKind::ExternalCall {
                    func: BuiltinFn::SetDifferenceByTag,
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

        "__xor__" => {
            super::check_arity(ctx, line, &type_name, method_name, 1, args.len())?;
            super::check_type(ctx, line, &type_name, method_name, &args[0], &set_ty)?;
            let eq_tag = set_eq_tag(ctx, line, inner_type);
            Ok(CallResult::Expr(TirExpr {
                kind: TirExprKind::ExternalCall {
                    func: BuiltinFn::SetSymmetricDifferenceByTag,
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

        "__rand__" => {
            super::check_arity(ctx, line, &type_name, method_name, 1, args.len())?;
            super::check_type(ctx, line, &type_name, method_name, &args[0], &set_ty)?;
            let eq_tag = set_eq_tag(ctx, line, inner_type);
            Ok(CallResult::Expr(TirExpr {
                kind: TirExprKind::ExternalCall {
                    func: BuiltinFn::SetIntersectionByTag,
                    args: vec![
                        args[0].clone(),
                        obj.clone(),
                        TirExpr {
                            kind: TirExprKind::IntLiteral(eq_tag),
                            ty: ValueType::Int,
                        },
                    ],
                },
                ty: set_ty.clone(),
            }))
        }

        "__ror__" => {
            super::check_arity(ctx, line, &type_name, method_name, 1, args.len())?;
            super::check_type(ctx, line, &type_name, method_name, &args[0], &set_ty)?;
            let eq_tag = set_eq_tag(ctx, line, inner_type);
            Ok(CallResult::Expr(TirExpr {
                kind: TirExprKind::ExternalCall {
                    func: BuiltinFn::SetUnionByTag,
                    args: vec![
                        args[0].clone(),
                        obj.clone(),
                        TirExpr {
                            kind: TirExprKind::IntLiteral(eq_tag),
                            ty: ValueType::Int,
                        },
                    ],
                },
                ty: set_ty.clone(),
            }))
        }

        "__rsub__" => {
            super::check_arity(ctx, line, &type_name, method_name, 1, args.len())?;
            super::check_type(ctx, line, &type_name, method_name, &args[0], &set_ty)?;
            let eq_tag = set_eq_tag(ctx, line, inner_type);
            Ok(CallResult::Expr(TirExpr {
                kind: TirExprKind::ExternalCall {
                    func: BuiltinFn::SetDifferenceByTag,
                    args: vec![
                        args[0].clone(),
                        obj.clone(),
                        TirExpr {
                            kind: TirExprKind::IntLiteral(eq_tag),
                            ty: ValueType::Int,
                        },
                    ],
                },
                ty: set_ty.clone(),
            }))
        }

        "__rxor__" => {
            super::check_arity(ctx, line, &type_name, method_name, 1, args.len())?;
            super::check_type(ctx, line, &type_name, method_name, &args[0], &set_ty)?;
            let eq_tag = set_eq_tag(ctx, line, inner_type);
            Ok(CallResult::Expr(TirExpr {
                kind: TirExprKind::ExternalCall {
                    func: BuiltinFn::SetSymmetricDifferenceByTag,
                    args: vec![
                        args[0].clone(),
                        obj.clone(),
                        TirExpr {
                            kind: TirExprKind::IntLiteral(eq_tag),
                            ty: ValueType::Int,
                        },
                    ],
                },
                ty: set_ty.clone(),
            }))
        }

        "__iand__" => {
            super::check_arity(ctx, line, &type_name, method_name, 1, args.len())?;
            super::check_type(ctx, line, &type_name, method_name, &args[0], &set_ty)?;
            let eq_tag = set_eq_tag(ctx, line, inner_type);
            Ok(CallResult::Expr(TirExpr {
                kind: TirExprKind::ExternalCall {
                    func: BuiltinFn::SetIAndByTag,
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

        "__ior__" => {
            super::check_arity(ctx, line, &type_name, method_name, 1, args.len())?;
            super::check_type(ctx, line, &type_name, method_name, &args[0], &set_ty)?;
            let eq_tag = set_eq_tag(ctx, line, inner_type);
            Ok(CallResult::Expr(TirExpr {
                kind: TirExprKind::ExternalCall {
                    func: BuiltinFn::SetIOrByTag,
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

        "__isub__" => {
            super::check_arity(ctx, line, &type_name, method_name, 1, args.len())?;
            super::check_type(ctx, line, &type_name, method_name, &args[0], &set_ty)?;
            let eq_tag = set_eq_tag(ctx, line, inner_type);
            Ok(CallResult::Expr(TirExpr {
                kind: TirExprKind::ExternalCall {
                    func: BuiltinFn::SetISubByTag,
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

        "__ixor__" => {
            super::check_arity(ctx, line, &type_name, method_name, 1, args.len())?;
            super::check_type(ctx, line, &type_name, method_name, &args[0], &set_ty)?;
            let eq_tag = set_eq_tag(ctx, line, inner_type);
            Ok(CallResult::Expr(TirExpr {
                kind: TirExprKind::ExternalCall {
                    func: BuiltinFn::SetIXorByTag,
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

        "__lt__" => {
            super::check_arity(ctx, line, &type_name, method_name, 1, args.len())?;
            super::check_type(ctx, line, &type_name, method_name, &args[0], &set_ty)?;
            let eq_tag = set_eq_tag(ctx, line, inner_type);
            Ok(CallResult::Expr(TirExpr {
                kind: TirExprKind::ExternalCall {
                    func: BuiltinFn::SetLtByTag,
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

        "__le__" => {
            super::check_arity(ctx, line, &type_name, method_name, 1, args.len())?;
            super::check_type(ctx, line, &type_name, method_name, &args[0], &set_ty)?;
            let eq_tag = set_eq_tag(ctx, line, inner_type);
            Ok(CallResult::Expr(TirExpr {
                kind: TirExprKind::ExternalCall {
                    func: BuiltinFn::SetLeByTag,
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

        "__gt__" => {
            super::check_arity(ctx, line, &type_name, method_name, 1, args.len())?;
            super::check_type(ctx, line, &type_name, method_name, &args[0], &set_ty)?;
            let eq_tag = set_eq_tag(ctx, line, inner_type);
            Ok(CallResult::Expr(TirExpr {
                kind: TirExprKind::ExternalCall {
                    func: BuiltinFn::SetGtByTag,
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

        "__ge__" => {
            super::check_arity(ctx, line, &type_name, method_name, 1, args.len())?;
            super::check_type(ctx, line, &type_name, method_name, &args[0], &set_ty)?;
            let eq_tag = set_eq_tag(ctx, line, inner_type);
            Ok(CallResult::Expr(TirExpr {
                kind: TirExprKind::ExternalCall {
                    func: BuiltinFn::SetGeByTag,
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

        "__str__" | "__repr__" => {
            super::check_arity(ctx, line, &type_name, method_name, 0, args.len())?;
            let str_tag = ctx.register_intrinsic_instance(IntrinsicOp::Str, inner_type);
            Ok(CallResult::Expr(TirExpr {
                kind: TirExprKind::ExternalCall {
                    func: BuiltinFn::SetStrByTag,
                    args: vec![
                        obj.clone(),
                        TirExpr {
                            kind: TirExprKind::IntLiteral(str_tag),
                            ty: ValueType::Int,
                        },
                    ],
                },
                ty: ValueType::Str,
            }))
        }

        // ── Unknown Method ───────────────────────────────────────────────
        _ => Err(ctx.attribute_error(
            line,
            format!("set[{}] has no method `{}`", inner_type, method_name),
        )),
    }
}
