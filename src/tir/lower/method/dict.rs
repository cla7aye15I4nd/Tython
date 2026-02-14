use anyhow::Result;

use crate::tir::{builtin::BuiltinFn, CallResult, TirExpr, ValueType};

use super::super::Lowering;

/// Lower a method call on a dict to TIR.
///
/// Handles all dict methods:
/// - Regular methods: clear, copy, get, pop, values
/// - Magic methods: __contains__, __eq__, __getitem__, __setitem__, __delitem__, __len__
///
/// Directly generates TIR without using type rules - all logic is self-contained here.
pub fn lower_dict_method_call(
    ctx: &Lowering,
    line: usize,
    obj: TirExpr,
    method_name: &str,
    args: Vec<TirExpr>,
    key_type: &ValueType,
    value_type: &ValueType,
) -> Result<CallResult> {
    let dict_ty = ValueType::Dict(Box::new(key_type.clone()), Box::new(value_type.clone()));

    // Generate common helper functions using macro
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
            super::check_arity(ctx, line, &type_name, method_name, 1, args.len())?;
            super::check_type(ctx, line, &type_name, method_name, &args[0], key_type)?;
            Ok(super::expr_call(
                BuiltinFn::DictGet,
                value_type.clone(),
                obj.clone(),
                args,
            ))
        }

        "pop" => {
            super::check_arity(ctx, line, &type_name, method_name, 1, args.len())?;
            super::check_type(ctx, line, &type_name, method_name, &args[0], key_type)?;
            Ok(super::expr_call(
                BuiltinFn::DictPop,
                value_type.clone(),
                obj.clone(),
                args,
            ))
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

        // ── Magic Methods ────────────────────────────────────────────────
        "__contains__" => {
            super::check_arity(ctx, line, &type_name, method_name, 1, args.len())?;
            super::check_type(ctx, line, &type_name, method_name, &args[0], key_type)?;
            Ok(super::expr_call(
                BuiltinFn::DictContains,
                ValueType::Bool,
                obj.clone(),
                args,
            ))
        }

        "__eq__" => {
            super::check_arity(ctx, line, &type_name, method_name, 1, args.len())?;
            super::check_type(ctx, line, &type_name, method_name, &args[0], &dict_ty)?;
            Ok(super::expr_call(
                BuiltinFn::DictEq,
                ValueType::Bool,
                obj.clone(),
                args,
            ))
        }

        "__getitem__" => {
            super::check_arity(ctx, line, &type_name, method_name, 1, args.len())?;
            super::check_type(ctx, line, &type_name, method_name, &args[0], key_type)?;
            Ok(super::expr_call(
                BuiltinFn::DictGet,
                value_type.clone(),
                obj.clone(),
                args,
            ))
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

        // Note: __setitem__ and __delitem__ are handled as statements, not method calls

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
