use anyhow::Result;

use crate::tir::{builtin::BuiltinFn, CallResult, TirExpr, ValueType};

use super::super::Lowering;

/// Lower a method call on a list to TIR.
///
/// Handles all list methods including:
/// - Regular methods: append, clear, copy, count, extend, index, insert, pop, remove, reverse, sort
/// - Magic methods: __add__, __iadd__, __mul__, __imul__, __contains__, __eq__, __getitem__, __delitem__, __len__, __reversed__
///
/// Directly generates TIR without using type rules - all logic is self-contained here.
pub fn lower_list_method_call(
    ctx: &Lowering,
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
            Ok(super::expr_call(
                BuiltinFn::ListCount,
                ValueType::Int,
                obj.clone(),
                args,
            ))
        }

        "extend" => {
            super::check_arity(ctx, line, &type_name, method_name, 1, args.len())?;
            super::check_type(ctx, line, &type_name, method_name, &args[0], &list_ty)?;
            Ok(super::void_call(BuiltinFn::ListExtend, obj.clone(), args))
        }

        "index" => {
            super::check_arity(ctx, line, &type_name, method_name, 1, args.len())?;
            super::check_type(ctx, line, &type_name, method_name, &args[0], inner_type)?;
            Ok(super::expr_call(
                BuiltinFn::ListIndex,
                ValueType::Int,
                obj.clone(),
                args,
            ))
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
            Ok(super::void_call(BuiltinFn::ListRemove, obj.clone(), args))
        }

        "reverse" => {
            super::check_arity(ctx, line, &type_name, method_name, 0, args.len())?;
            Ok(super::void_call(BuiltinFn::ListReverse, obj.clone(), args))
        }

        "sort" => {
            super::check_arity(ctx, line, &type_name, method_name, 0, args.len())?;
            let sort_fn = match inner_type {
                ValueType::Int | ValueType::Bool => BuiltinFn::ListSortInt,
                ValueType::Float => BuiltinFn::ListSortFloat,
                ValueType::Str => BuiltinFn::ListSortStr,
                ValueType::Bytes => BuiltinFn::ListSortBytes,
                ValueType::ByteArray => BuiltinFn::ListSortByteArray,
                _ => {
                    return Err(ctx.type_error(
                        line,
                        format!(
                            "list[{}].sort() is not supported; element type has no `__lt__`",
                            inner_type
                        ),
                    ))
                }
            };
            Ok(super::void_call(sort_fn, obj.clone(), args))
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
            Ok(super::expr_call(
                BuiltinFn::ListContains,
                ValueType::Bool,
                obj.clone(),
                args,
            ))
        }

        "__eq__" => {
            super::check_arity(ctx, line, &type_name, method_name, 1, args.len())?;
            super::check_type(ctx, line, &type_name, method_name, &args[0], &list_ty)?;
            // Choose shallow or deep equality based on inner type
            let eq_fn = match inner_type {
                ValueType::Int
                | ValueType::Float
                | ValueType::Bool
                | ValueType::Str
                | ValueType::Bytes
                | ValueType::ByteArray => BuiltinFn::ListEqShallow,
                _ => BuiltinFn::ListEqDeep,
            };
            Ok(super::expr_call(eq_fn, ValueType::Bool, obj.clone(), args))
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

        "__reversed__" => {
            super::check_arity(ctx, line, &type_name, method_name, 0, args.len())?;
            Ok(super::expr_call(
                BuiltinFn::ReversedList,
                list_ty,
                obj.clone(),
                args,
            ))
        }

        // ── Unknown Method ───────────────────────────────────────────────
        _ => Err(ctx.attribute_error(
            line,
            format!("list[{}] has no method `{}`", inner_type, method_name),
        )),
    }
}
