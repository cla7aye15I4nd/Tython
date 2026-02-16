use anyhow::Result;

use crate::tir::{
    builtin::BuiltinFn, CallResult, CallTarget, IntrinsicOp, TirExpr, TirExprKind, TirStmt,
    ValueType,
};

use super::super::Lowering;

/// Lower a method call on a list to TIR.
///
/// Handles all list methods including:
/// - Regular methods: append, clear, copy, count, extend, index, insert, pop, remove, reverse, sort
/// - Magic methods: __add__, __iadd__, __mul__, __imul__, __contains__, __eq__, __getitem__, __delitem__, __len__, __reversed__
///
/// Directly generates TIR without using type rules - all logic is self-contained here.
pub fn lower_list_method_call(
    ctx: &mut Lowering,
    line: usize,
    obj: TirExpr,
    method_name: &str,
    args: Vec<TirExpr>,
    inner_type: &ValueType,
) -> Result<CallResult> {
    let list_ty = ValueType::List(Box::new(inner_type.clone()));
    let type_name = format!("list[{}]", inner_type);

    match method_name {
        // ── Regular Methods ──────────────────────────────────────────────
        "append" => {
            super::check_arity(ctx, line, &type_name, method_name, 1, args.len())?;
            super::check_type(ctx, line, &type_name, method_name, &args[0], inner_type)?;
            Ok(super::void_call(BuiltinFn::ListAppend, obj.clone(), args))
        }

        "clear" => {
            super::check_arity(ctx, line, &type_name, method_name, 0, args.len())?;
            Ok(super::void_call(BuiltinFn::ListClear, obj.clone(), args))
        }

        "copy" => {
            super::check_arity(ctx, line, &type_name, method_name, 0, args.len())?;
            Ok(super::expr_call(
                BuiltinFn::ListCopy,
                list_ty,
                obj.clone(),
                args,
            ))
        }

        "count" => {
            super::check_arity(ctx, line, &type_name, method_name, 1, args.len())?;
            super::check_type(ctx, line, &type_name, method_name, &args[0], inner_type)?;
            ctx.require_list_leaf_eq_support(line, inner_type);
            let eq_tag = ctx.register_intrinsic_instance(IntrinsicOp::Eq, inner_type);
            let mut call_args = vec![obj.clone(), args[0].clone()];
            call_args.push(TirExpr {
                kind: TirExprKind::IntLiteral(eq_tag),
                ty: ValueType::Int,
            });
            Ok(CallResult::Expr(TirExpr {
                kind: TirExprKind::ExternalCall {
                    func: BuiltinFn::ListCountByTag,
                    args: call_args,
                },
                ty: ValueType::Int,
            }))
        }

        "extend" => {
            super::check_arity(ctx, line, &type_name, method_name, 1, args.len())?;
            super::check_type(ctx, line, &type_name, method_name, &args[0], &list_ty)?;
            Ok(super::void_call(BuiltinFn::ListExtend, obj.clone(), args))
        }

        "index" => {
            super::check_arity(ctx, line, &type_name, method_name, 1, args.len())?;
            super::check_type(ctx, line, &type_name, method_name, &args[0], inner_type)?;
            ctx.require_list_leaf_eq_support(line, inner_type);
            let eq_tag = ctx.register_intrinsic_instance(IntrinsicOp::Eq, inner_type);
            let mut call_args = vec![obj.clone(), args[0].clone()];
            call_args.push(TirExpr {
                kind: TirExprKind::IntLiteral(eq_tag),
                ty: ValueType::Int,
            });
            Ok(CallResult::Expr(TirExpr {
                kind: TirExprKind::ExternalCall {
                    func: BuiltinFn::ListIndexByTag,
                    args: call_args,
                },
                ty: ValueType::Int,
            }))
        }

        "insert" => {
            super::check_arity(ctx, line, &type_name, method_name, 2, args.len())?;
            super::check_type(
                ctx,
                line,
                &type_name,
                method_name,
                &args[0],
                &ValueType::Int,
            )?;
            super::check_type(ctx, line, &type_name, method_name, &args[1], inner_type)?;
            Ok(super::void_call(BuiltinFn::ListInsert, obj.clone(), args))
        }

        "pop" => {
            super::check_arity(ctx, line, &type_name, method_name, 0, args.len())?;
            Ok(super::expr_call(
                BuiltinFn::ListPop,
                inner_type.clone(),
                obj.clone(),
                args,
            ))
        }

        "remove" => {
            super::check_arity(ctx, line, &type_name, method_name, 1, args.len())?;
            super::check_type(ctx, line, &type_name, method_name, &args[0], inner_type)?;
            ctx.require_list_leaf_eq_support(line, inner_type);
            let eq_tag = ctx.register_intrinsic_instance(IntrinsicOp::Eq, inner_type);
            let mut call_args = vec![obj.clone(), args[0].clone()];
            call_args.push(TirExpr {
                kind: TirExprKind::IntLiteral(eq_tag),
                ty: ValueType::Int,
            });
            Ok(CallResult::VoidStmt(Box::new(TirStmt::VoidCall {
                target: CallTarget::Builtin(BuiltinFn::ListRemoveByTag),
                args: call_args,
            })))
        }

        "reverse" => {
            super::check_arity(ctx, line, &type_name, method_name, 0, args.len())?;
            Ok(super::void_call(BuiltinFn::ListReverse, obj.clone(), args))
        }

        "sort" => {
            super::check_arity(ctx, line, &type_name, method_name, 0, args.len())?;
            ctx.require_list_leaf_lt_support(line, inner_type)?;
            let lt_tag = ctx.register_intrinsic_instance(IntrinsicOp::Lt, inner_type);
            Ok(CallResult::VoidStmt(Box::new(TirStmt::VoidCall {
                target: CallTarget::Builtin(BuiltinFn::ListSortByTag),
                args: vec![
                    obj.clone(),
                    TirExpr {
                        kind: TirExprKind::IntLiteral(lt_tag),
                        ty: ValueType::Int,
                    },
                ],
            })))
        }

        // ── Magic Methods ────────────────────────────────────────────────
        "__add__" => {
            super::check_arity(ctx, line, &type_name, method_name, 1, args.len())?;
            super::check_type(ctx, line, &type_name, method_name, &args[0], &list_ty)?;
            Ok(super::expr_call(
                BuiltinFn::ListConcat,
                list_ty,
                obj.clone(),
                args,
            ))
        }

        "__iadd__" => {
            super::check_arity(ctx, line, &type_name, method_name, 1, args.len())?;
            super::check_type(ctx, line, &type_name, method_name, &args[0], &list_ty)?;
            Ok(super::expr_call(
                BuiltinFn::ListIAdd,
                list_ty,
                obj.clone(),
                args,
            ))
        }

        "__mul__" => {
            super::check_arity(ctx, line, &type_name, method_name, 1, args.len())?;
            super::check_type(
                ctx,
                line,
                &type_name,
                method_name,
                &args[0],
                &ValueType::Int,
            )?;
            Ok(super::expr_call(
                BuiltinFn::ListRepeat,
                list_ty,
                obj.clone(),
                args,
            ))
        }

        "__rmul__" => {
            super::check_arity(ctx, line, &type_name, method_name, 1, args.len())?;
            super::check_type(
                ctx,
                line,
                &type_name,
                method_name,
                &args[0],
                &ValueType::Int,
            )?;
            Ok(super::expr_call(
                BuiltinFn::ListRepeat,
                list_ty,
                obj.clone(),
                args,
            ))
        }

        "__imul__" => {
            super::check_arity(ctx, line, &type_name, method_name, 1, args.len())?;
            super::check_type(
                ctx,
                line,
                &type_name,
                method_name,
                &args[0],
                &ValueType::Int,
            )?;
            Ok(super::expr_call(
                BuiltinFn::ListIMul,
                list_ty,
                obj.clone(),
                args,
            ))
        }

        "__contains__" => {
            super::check_arity(ctx, line, &type_name, method_name, 1, args.len())?;
            super::check_type(ctx, line, &type_name, method_name, &args[0], inner_type)?;
            ctx.require_list_leaf_eq_support(line, inner_type);
            let eq_tag = ctx.register_intrinsic_instance(IntrinsicOp::Eq, inner_type);
            Ok(CallResult::Expr(TirExpr {
                kind: TirExprKind::ExternalCall {
                    func: BuiltinFn::ListContainsByTag,
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
            super::check_type(ctx, line, &type_name, method_name, &args[0], &list_ty)?;
            ctx.require_list_leaf_eq_support(line, inner_type);
            let eq_tag = ctx.register_intrinsic_instance(IntrinsicOp::Eq, inner_type);
            Ok(CallResult::Expr(TirExpr {
                kind: TirExprKind::ExternalCall {
                    func: BuiltinFn::ListEqByTag,
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

        "__getitem__" => {
            super::check_arity(ctx, line, &type_name, method_name, 1, args.len())?;
            super::check_type(
                ctx,
                line,
                &type_name,
                method_name,
                &args[0],
                &ValueType::Int,
            )?;
            Ok(super::expr_call(
                BuiltinFn::ListGet,
                inner_type.clone(),
                obj.clone(),
                args,
            ))
        }

        "__delitem__" => {
            super::check_arity(ctx, line, &type_name, method_name, 1, args.len())?;
            super::check_type(
                ctx,
                line,
                &type_name,
                method_name,
                &args[0],
                &ValueType::Int,
            )?;
            Ok(super::void_call(BuiltinFn::ListDel, obj.clone(), args))
        }

        "__len__" => {
            super::check_arity(ctx, line, &type_name, method_name, 0, args.len())?;
            Ok(super::expr_call(
                BuiltinFn::ListLen,
                ValueType::Int,
                obj.clone(),
                args,
            ))
        }

        "__lt__" => {
            super::check_arity(ctx, line, &type_name, method_name, 1, args.len())?;
            super::check_type(ctx, line, &type_name, method_name, &args[0], &list_ty)?;
            ctx.require_list_leaf_lt_support(line, inner_type)?;
            let lt_tag = ctx.register_intrinsic_instance(IntrinsicOp::Lt, inner_type);
            Ok(CallResult::Expr(TirExpr {
                kind: TirExprKind::ExternalCall {
                    func: BuiltinFn::ListLtByTag,
                    args: vec![
                        obj.clone(),
                        args[0].clone(),
                        TirExpr {
                            kind: TirExprKind::IntLiteral(lt_tag),
                            ty: ValueType::Int,
                        },
                    ],
                },
                ty: ValueType::Bool,
            }))
        }

        "__reversed__" => {
            super::check_arity(ctx, line, &type_name, method_name, 0, args.len())?;
            Ok(super::expr_call(
                BuiltinFn::ReversedList,
                list_ty,
                obj.clone(),
                args,
            ))
        }

        "__str__" | "__repr__" => {
            super::check_arity(ctx, line, &type_name, method_name, 0, args.len())?;
            let str_tag = ctx.register_intrinsic_instance(IntrinsicOp::Str, inner_type);
            Ok(CallResult::Expr(TirExpr {
                kind: TirExprKind::ExternalCall {
                    func: BuiltinFn::ListStrByTag,
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
            format!("list[{}] has no method `{}`", inner_type, method_name),
        )),
    }
}
