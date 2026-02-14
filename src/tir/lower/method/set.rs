use anyhow::Result;

use crate::tir::{builtin::BuiltinFn, CallResult, TirExpr, ValueType};

use super::super::Lowering;

/// Lower a method call on a set to TIR.
///
/// Handles all set methods:
/// - Regular methods: add, clear, copy, discard, pop, remove
/// - Magic methods: __contains__, __eq__, __len__
///
/// Directly generates TIR without using type rules - all logic is self-contained here.
pub fn lower_set_method_call(
    ctx: &Lowering,
    line: usize,
    obj: TirExpr,
    method_name: &str,
    args: Vec<TirExpr>,
    inner_type: &ValueType,
) -> Result<CallResult> {
    let set_ty = ValueType::Set(Box::new(inner_type.clone()));

    // Generate common helper functions using macro
    let type_name = format!("set[{}]", inner_type);

    match method_name {
        // ── Regular Methods ──────────────────────────────────────────────
        "add" => {
            super::check_arity(ctx, line, &type_name, method_name, 1, args.len())?;
            super::check_type(ctx, line, &type_name, method_name, &args[0], inner_type)?;
            Ok(super::void_call(BuiltinFn::SetAdd, obj.clone(), args))
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
            Ok(super::void_call(BuiltinFn::SetDiscard, obj.clone(), args))
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
            Ok(super::void_call(BuiltinFn::SetRemove, obj.clone(), args))
        }

        // ── Magic Methods ────────────────────────────────────────────────
        "__contains__" => {
            super::check_arity(ctx, line, &type_name, method_name, 1, args.len())?;
            super::check_type(ctx, line, &type_name, method_name, &args[0], inner_type)?;
            Ok(super::expr_call(
                BuiltinFn::SetContains,
                ValueType::Bool,
                obj.clone(),
                args,
            ))
        }

        "__eq__" => {
            super::check_arity(ctx, line, &type_name, method_name, 1, args.len())?;
            super::check_type(ctx, line, &type_name, method_name, &args[0], &set_ty)?;
            Ok(super::expr_call(
                BuiltinFn::SetEq,
                ValueType::Bool,
                obj.clone(),
                args,
            ))
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
