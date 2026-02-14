use anyhow::Result;

use crate::tir::{builtin::BuiltinFn, CallResult, TirExpr, ValueType};

use super::super::Lowering;

/// Lower a method call on a str to TIR.
///
/// Handles all str methods:
/// - Regular methods: join, read, split, strip
/// - Magic methods: __add__, __contains__, __eq__, __getitem__, __len__, __mul__
///
/// Directly generates TIR without using type rules - all logic is self-contained here.
pub fn lower_str_method_call(
    ctx: &Lowering,
    line: usize,
    obj: TirExpr,
    method_name: &str,
    args: Vec<TirExpr>,
) -> Result<CallResult> {
    // Generate common helper functions using macro
    let type_name = "str";

    match method_name {
        // ── Regular Methods ──────────────────────────────────────────────
        "join" => {
            super::check_arity(ctx, line, type_name, method_name, 1, args.len())?;
            super::check_type(
                ctx,
                line,
                type_name,
                method_name,
                &args[0],
                &ValueType::List(Box::new(ValueType::Str)),
            )?;
            Ok(super::expr_call(
                BuiltinFn::StrJoin,
                ValueType::Str,
                obj.clone(),
                args,
            ))
        }

        "read" => {
            // Compatibility shim for open(...).read()
            super::check_arity(ctx, line, type_name, method_name, 0, args.len())?;
            Ok(super::expr_call(
                BuiltinFn::StrRead,
                ValueType::Str,
                obj.clone(),
                args,
            ))
        }

        "split" => {
            super::check_arity(ctx, line, type_name, method_name, 1, args.len())?;
            super::check_type(ctx, line, type_name, method_name, &args[0], &ValueType::Str)?;
            Ok(super::expr_call(
                BuiltinFn::StrSplit,
                ValueType::List(Box::new(ValueType::Str)),
                obj.clone(),
                args,
            ))
        }

        "strip" => {
            super::check_arity(ctx, line, type_name, method_name, 0, args.len())?;
            Ok(super::expr_call(
                BuiltinFn::StrStrip,
                ValueType::Str,
                obj.clone(),
                args,
            ))
        }

        // ── Magic Methods ────────────────────────────────────────────────
        "__add__" => {
            super::check_arity(ctx, line, type_name, method_name, 1, args.len())?;
            super::check_type(ctx, line, type_name, method_name, &args[0], &ValueType::Str)?;
            Ok(super::expr_call(
                BuiltinFn::StrConcat,
                ValueType::Str,
                obj.clone(),
                args,
            ))
        }

        "__contains__" => {
            super::check_arity(ctx, line, type_name, method_name, 1, args.len())?;
            super::check_type(ctx, line, type_name, method_name, &args[0], &ValueType::Str)?;
            Ok(super::expr_call(
                BuiltinFn::StrContains,
                ValueType::Bool,
                obj.clone(),
                args,
            ))
        }

        "__eq__" => {
            super::check_arity(ctx, line, type_name, method_name, 1, args.len())?;
            super::check_type(ctx, line, type_name, method_name, &args[0], &ValueType::Str)?;
            Ok(super::expr_call(
                BuiltinFn::StrEq,
                ValueType::Bool,
                obj.clone(),
                args,
            ))
        }

        "__getitem__" => {
            super::check_arity(ctx, line, type_name, method_name, 1, args.len())?;
            super::check_type(ctx, line, type_name, method_name, &args[0], &ValueType::Int)?;
            Ok(super::expr_call(
                BuiltinFn::StrGetChar,
                ValueType::Str,
                obj.clone(),
                args,
            ))
        }

        "__len__" => {
            super::check_arity(ctx, line, type_name, method_name, 0, args.len())?;
            Ok(super::expr_call(
                BuiltinFn::StrLen,
                ValueType::Int,
                obj.clone(),
                args,
            ))
        }

        "__mul__" => {
            super::check_arity(ctx, line, type_name, method_name, 1, args.len())?;
            super::check_type(ctx, line, type_name, method_name, &args[0], &ValueType::Int)?;
            Ok(super::expr_call(
                BuiltinFn::StrRepeat,
                ValueType::Str,
                obj.clone(),
                args,
            ))
        }

        // ── Unknown Method ───────────────────────────────────────────────
        _ => Err(ctx.attribute_error(line, format!("str has no method `{}`", method_name))),
    }
}
