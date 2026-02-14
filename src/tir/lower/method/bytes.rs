use anyhow::Result;

use crate::tir::{builtin::BuiltinFn, CallResult, TirExpr, ValueType};

use super::super::Lowering;

/// Lower a method call on bytes to TIR.
///
/// Handles all bytes methods (40+ string-like methods).
///
/// Directly generates TIR without using type rules - all logic is self-contained here.
pub fn lower_bytes_method_call(
    ctx: &Lowering,
    line: usize,
    obj: TirExpr,
    method_name: &str,
    args: Vec<TirExpr>,
) -> Result<CallResult> {
    // Generate common helper functions using macro
    let type_name = "bytes";

    match method_name {
        // ── Transformation Methods ───────────────────────────────────────
        "capitalize" => {
            super::check_arity(ctx, line, type_name, method_name, 0, args.len())?;
            Ok(super::expr_call(
                BuiltinFn::BytesCapitalize,
                ValueType::Bytes,
                obj.clone(),
                args,
            ))
        }

        "center" => {
            super::check_arity(ctx, line, type_name, method_name, 2, args.len())?;
            super::check_type(ctx, line, type_name, method_name, &args[0], &ValueType::Int)?;
            super::check_type(
                ctx,
                line,
                type_name,
                method_name,
                &args[1],
                &ValueType::Bytes,
            )?;
            Ok(super::expr_call(
                BuiltinFn::BytesCenter,
                ValueType::Bytes,
                obj.clone(),
                args,
            ))
        }

        "decode" => {
            super::check_arity(ctx, line, type_name, method_name, 0, args.len())?;
            Ok(super::expr_call(
                BuiltinFn::BytesDecode,
                ValueType::Str,
                obj.clone(),
                args,
            ))
        }

        "expandtabs" => {
            super::check_arity(ctx, line, type_name, method_name, 1, args.len())?;
            super::check_type(ctx, line, type_name, method_name, &args[0], &ValueType::Int)?;
            Ok(super::expr_call(
                BuiltinFn::BytesExpandTabs,
                ValueType::Bytes,
                obj.clone(),
                args,
            ))
        }

        "ljust" => {
            super::check_arity(ctx, line, type_name, method_name, 2, args.len())?;
            super::check_type(ctx, line, type_name, method_name, &args[0], &ValueType::Int)?;
            super::check_type(
                ctx,
                line,
                type_name,
                method_name,
                &args[1],
                &ValueType::Bytes,
            )?;
            Ok(super::expr_call(
                BuiltinFn::BytesLJust,
                ValueType::Bytes,
                obj.clone(),
                args,
            ))
        }

        "lower" => {
            super::check_arity(ctx, line, type_name, method_name, 0, args.len())?;
            Ok(super::expr_call(
                BuiltinFn::BytesLower,
                ValueType::Bytes,
                obj.clone(),
                args,
            ))
        }

        "lstrip" => {
            super::check_arity(ctx, line, type_name, method_name, 1, args.len())?;
            super::check_type(
                ctx,
                line,
                type_name,
                method_name,
                &args[0],
                &ValueType::Bytes,
            )?;
            Ok(super::expr_call(
                BuiltinFn::BytesLStrip,
                ValueType::Bytes,
                obj.clone(),
                args,
            ))
        }

        "removeprefix" => {
            super::check_arity(ctx, line, type_name, method_name, 1, args.len())?;
            super::check_type(
                ctx,
                line,
                type_name,
                method_name,
                &args[0],
                &ValueType::Bytes,
            )?;
            Ok(super::expr_call(
                BuiltinFn::BytesRemovePrefix,
                ValueType::Bytes,
                obj.clone(),
                args,
            ))
        }

        "removesuffix" => {
            super::check_arity(ctx, line, type_name, method_name, 1, args.len())?;
            super::check_type(
                ctx,
                line,
                type_name,
                method_name,
                &args[0],
                &ValueType::Bytes,
            )?;
            Ok(super::expr_call(
                BuiltinFn::BytesRemoveSuffix,
                ValueType::Bytes,
                obj.clone(),
                args,
            ))
        }

        "replace" => {
            super::check_arity(ctx, line, type_name, method_name, 2, args.len())?;
            super::check_type(
                ctx,
                line,
                type_name,
                method_name,
                &args[0],
                &ValueType::Bytes,
            )?;
            super::check_type(
                ctx,
                line,
                type_name,
                method_name,
                &args[1],
                &ValueType::Bytes,
            )?;
            Ok(super::expr_call(
                BuiltinFn::BytesReplace,
                ValueType::Bytes,
                obj.clone(),
                args,
            ))
        }

        "rjust" => {
            super::check_arity(ctx, line, type_name, method_name, 2, args.len())?;
            super::check_type(ctx, line, type_name, method_name, &args[0], &ValueType::Int)?;
            super::check_type(
                ctx,
                line,
                type_name,
                method_name,
                &args[1],
                &ValueType::Bytes,
            )?;
            Ok(super::expr_call(
                BuiltinFn::BytesRJust,
                ValueType::Bytes,
                obj.clone(),
                args,
            ))
        }

        "rstrip" => {
            super::check_arity(ctx, line, type_name, method_name, 1, args.len())?;
            super::check_type(
                ctx,
                line,
                type_name,
                method_name,
                &args[0],
                &ValueType::Bytes,
            )?;
            Ok(super::expr_call(
                BuiltinFn::BytesRStrip,
                ValueType::Bytes,
                obj.clone(),
                args,
            ))
        }

        "strip" => {
            super::check_arity(ctx, line, type_name, method_name, 1, args.len())?;
            super::check_type(
                ctx,
                line,
                type_name,
                method_name,
                &args[0],
                &ValueType::Bytes,
            )?;
            Ok(super::expr_call(
                BuiltinFn::BytesStrip,
                ValueType::Bytes,
                obj.clone(),
                args,
            ))
        }

        "swapcase" => {
            super::check_arity(ctx, line, type_name, method_name, 0, args.len())?;
            Ok(super::expr_call(
                BuiltinFn::BytesSwapCase,
                ValueType::Bytes,
                obj.clone(),
                args,
            ))
        }

        "title" => {
            super::check_arity(ctx, line, type_name, method_name, 0, args.len())?;
            Ok(super::expr_call(
                BuiltinFn::BytesTitle,
                ValueType::Bytes,
                obj.clone(),
                args,
            ))
        }

        "translate" => {
            super::check_arity(ctx, line, type_name, method_name, 1, args.len())?;
            super::check_type(
                ctx,
                line,
                type_name,
                method_name,
                &args[0],
                &ValueType::Bytes,
            )?;
            Ok(super::expr_call(
                BuiltinFn::BytesTranslate,
                ValueType::Bytes,
                obj.clone(),
                args,
            ))
        }

        "upper" => {
            super::check_arity(ctx, line, type_name, method_name, 0, args.len())?;
            Ok(super::expr_call(
                BuiltinFn::BytesUpper,
                ValueType::Bytes,
                obj.clone(),
                args,
            ))
        }

        "zfill" => {
            super::check_arity(ctx, line, type_name, method_name, 1, args.len())?;
            super::check_type(ctx, line, type_name, method_name, &args[0], &ValueType::Int)?;
            Ok(super::expr_call(
                BuiltinFn::BytesZFill,
                ValueType::Bytes,
                obj.clone(),
                args,
            ))
        }

        // ── Search/Query Methods ─────────────────────────────────────────
        "count" => {
            super::check_arity(ctx, line, type_name, method_name, 1, args.len())?;
            super::check_type(
                ctx,
                line,
                type_name,
                method_name,
                &args[0],
                &ValueType::Bytes,
            )?;
            Ok(super::expr_call(
                BuiltinFn::BytesCount,
                ValueType::Int,
                obj.clone(),
                args,
            ))
        }

        "endswith" => {
            super::check_arity(ctx, line, type_name, method_name, 1, args.len())?;
            super::check_type(
                ctx,
                line,
                type_name,
                method_name,
                &args[0],
                &ValueType::Bytes,
            )?;
            Ok(super::expr_call(
                BuiltinFn::BytesEndsWith,
                ValueType::Bool,
                obj.clone(),
                args,
            ))
        }

        "find" => {
            super::check_arity(ctx, line, type_name, method_name, 1, args.len())?;
            super::check_type(
                ctx,
                line,
                type_name,
                method_name,
                &args[0],
                &ValueType::Bytes,
            )?;
            Ok(super::expr_call(
                BuiltinFn::BytesFind,
                ValueType::Int,
                obj.clone(),
                args,
            ))
        }

        "index" => {
            super::check_arity(ctx, line, type_name, method_name, 1, args.len())?;
            super::check_type(
                ctx,
                line,
                type_name,
                method_name,
                &args[0],
                &ValueType::Bytes,
            )?;
            Ok(super::expr_call(
                BuiltinFn::BytesIndex,
                ValueType::Int,
                obj.clone(),
                args,
            ))
        }

        "rfind" => {
            super::check_arity(ctx, line, type_name, method_name, 1, args.len())?;
            super::check_type(
                ctx,
                line,
                type_name,
                method_name,
                &args[0],
                &ValueType::Bytes,
            )?;
            Ok(super::expr_call(
                BuiltinFn::BytesRFind,
                ValueType::Int,
                obj.clone(),
                args,
            ))
        }

        "rindex" => {
            super::check_arity(ctx, line, type_name, method_name, 1, args.len())?;
            super::check_type(
                ctx,
                line,
                type_name,
                method_name,
                &args[0],
                &ValueType::Bytes,
            )?;
            Ok(super::expr_call(
                BuiltinFn::BytesRIndex,
                ValueType::Int,
                obj.clone(),
                args,
            ))
        }

        "startswith" => {
            super::check_arity(ctx, line, type_name, method_name, 1, args.len())?;
            super::check_type(
                ctx,
                line,
                type_name,
                method_name,
                &args[0],
                &ValueType::Bytes,
            )?;
            Ok(super::expr_call(
                BuiltinFn::BytesStartsWith,
                ValueType::Bool,
                obj.clone(),
                args,
            ))
        }

        // ── Predicate Methods ────────────────────────────────────────────
        "isalnum" => {
            super::check_arity(ctx, line, type_name, method_name, 0, args.len())?;
            Ok(super::expr_call(
                BuiltinFn::BytesIsAlnum,
                ValueType::Bool,
                obj.clone(),
                args,
            ))
        }

        "isalpha" => {
            super::check_arity(ctx, line, type_name, method_name, 0, args.len())?;
            Ok(super::expr_call(
                BuiltinFn::BytesIsAlpha,
                ValueType::Bool,
                obj.clone(),
                args,
            ))
        }

        "isascii" => {
            super::check_arity(ctx, line, type_name, method_name, 0, args.len())?;
            Ok(super::expr_call(
                BuiltinFn::BytesIsAscii,
                ValueType::Bool,
                obj.clone(),
                args,
            ))
        }

        "isdigit" => {
            super::check_arity(ctx, line, type_name, method_name, 0, args.len())?;
            Ok(super::expr_call(
                BuiltinFn::BytesIsDigit,
                ValueType::Bool,
                obj.clone(),
                args,
            ))
        }

        "islower" => {
            super::check_arity(ctx, line, type_name, method_name, 0, args.len())?;
            Ok(super::expr_call(
                BuiltinFn::BytesIsLower,
                ValueType::Bool,
                obj.clone(),
                args,
            ))
        }

        "isspace" => {
            super::check_arity(ctx, line, type_name, method_name, 0, args.len())?;
            Ok(super::expr_call(
                BuiltinFn::BytesIsSpace,
                ValueType::Bool,
                obj.clone(),
                args,
            ))
        }

        "istitle" => {
            super::check_arity(ctx, line, type_name, method_name, 0, args.len())?;
            Ok(super::expr_call(
                BuiltinFn::BytesIsTitle,
                ValueType::Bool,
                obj.clone(),
                args,
            ))
        }

        "isupper" => {
            super::check_arity(ctx, line, type_name, method_name, 0, args.len())?;
            Ok(super::expr_call(
                BuiltinFn::BytesIsUpper,
                ValueType::Bool,
                obj.clone(),
                args,
            ))
        }

        // ── Split/Join Methods ───────────────────────────────────────────
        "join" => {
            super::check_arity(ctx, line, type_name, method_name, 1, args.len())?;
            super::check_type(
                ctx,
                line,
                type_name,
                method_name,
                &args[0],
                &ValueType::List(Box::new(ValueType::Bytes)),
            )?;
            Ok(super::expr_call(
                BuiltinFn::BytesJoin,
                ValueType::Bytes,
                obj.clone(),
                args,
            ))
        }

        "partition" => {
            super::check_arity(ctx, line, type_name, method_name, 1, args.len())?;
            super::check_type(
                ctx,
                line,
                type_name,
                method_name,
                &args[0],
                &ValueType::Bytes,
            )?;
            Ok(super::expr_call(
                BuiltinFn::BytesPartition,
                ValueType::Tuple(vec![ValueType::Bytes, ValueType::Bytes, ValueType::Bytes]),
                obj.clone(),
                args,
            ))
        }

        "rpartition" => {
            super::check_arity(ctx, line, type_name, method_name, 1, args.len())?;
            super::check_type(
                ctx,
                line,
                type_name,
                method_name,
                &args[0],
                &ValueType::Bytes,
            )?;
            Ok(super::expr_call(
                BuiltinFn::BytesRPartition,
                ValueType::Tuple(vec![ValueType::Bytes, ValueType::Bytes, ValueType::Bytes]),
                obj.clone(),
                args,
            ))
        }

        "rsplit" => {
            super::check_arity(ctx, line, type_name, method_name, 1, args.len())?;
            super::check_type(
                ctx,
                line,
                type_name,
                method_name,
                &args[0],
                &ValueType::Bytes,
            )?;
            Ok(super::expr_call(
                BuiltinFn::BytesRSplit,
                ValueType::List(Box::new(ValueType::Bytes)),
                obj.clone(),
                args,
            ))
        }

        "split" => {
            super::check_arity(ctx, line, type_name, method_name, 1, args.len())?;
            super::check_type(
                ctx,
                line,
                type_name,
                method_name,
                &args[0],
                &ValueType::Bytes,
            )?;
            Ok(super::expr_call(
                BuiltinFn::BytesSplit,
                ValueType::List(Box::new(ValueType::Bytes)),
                obj.clone(),
                args,
            ))
        }

        "splitlines" => {
            super::check_arity(ctx, line, type_name, method_name, 0, args.len())?;
            Ok(super::expr_call(
                BuiltinFn::BytesSplitLines,
                ValueType::List(Box::new(ValueType::Bytes)),
                obj.clone(),
                args,
            ))
        }

        // ── Utility Methods ──────────────────────────────────────────────
        "fromhex" => {
            super::check_arity(ctx, line, type_name, method_name, 1, args.len())?;
            super::check_type(ctx, line, type_name, method_name, &args[0], &ValueType::Str)?;
            Ok(super::expr_call(
                BuiltinFn::BytesFromHex,
                ValueType::Bytes,
                obj.clone(),
                args,
            ))
        }

        "hex" => {
            super::check_arity(ctx, line, type_name, method_name, 0, args.len())?;
            Ok(super::expr_call(
                BuiltinFn::BytesHex,
                ValueType::Str,
                obj.clone(),
                args,
            ))
        }

        "maketrans" => {
            super::check_arity(ctx, line, type_name, method_name, 2, args.len())?;
            super::check_type(
                ctx,
                line,
                type_name,
                method_name,
                &args[0],
                &ValueType::Bytes,
            )?;
            super::check_type(
                ctx,
                line,
                type_name,
                method_name,
                &args[1],
                &ValueType::Bytes,
            )?;
            Ok(super::expr_call(
                BuiltinFn::BytesMakeTrans,
                ValueType::Bytes,
                obj.clone(),
                args,
            ))
        }

        // ── Magic Methods ────────────────────────────────────────────────
        "__add__" => {
            super::check_arity(ctx, line, type_name, method_name, 1, args.len())?;
            super::check_type(
                ctx,
                line,
                type_name,
                method_name,
                &args[0],
                &ValueType::Bytes,
            )?;
            Ok(super::expr_call(
                BuiltinFn::BytesConcat,
                ValueType::Bytes,
                obj.clone(),
                args,
            ))
        }

        "__eq__" => {
            super::check_arity(ctx, line, type_name, method_name, 1, args.len())?;
            super::check_type(
                ctx,
                line,
                type_name,
                method_name,
                &args[0],
                &ValueType::Bytes,
            )?;
            Ok(super::expr_call(
                BuiltinFn::BytesEq,
                ValueType::Bool,
                obj.clone(),
                args,
            ))
        }

        "__getitem__" => {
            super::check_arity(ctx, line, type_name, method_name, 1, args.len())?;
            super::check_type(ctx, line, type_name, method_name, &args[0], &ValueType::Int)?;
            Ok(super::expr_call(
                BuiltinFn::BytesGet,
                ValueType::Int,
                obj.clone(),
                args,
            ))
        }

        "__len__" => {
            super::check_arity(ctx, line, type_name, method_name, 0, args.len())?;
            Ok(super::expr_call(
                BuiltinFn::BytesLen,
                ValueType::Int,
                obj.clone(),
                args,
            ))
        }

        "__mul__" => {
            super::check_arity(ctx, line, type_name, method_name, 1, args.len())?;
            super::check_type(ctx, line, type_name, method_name, &args[0], &ValueType::Int)?;
            Ok(super::expr_call(
                BuiltinFn::BytesRepeat,
                ValueType::Bytes,
                obj.clone(),
                args,
            ))
        }

        // ── Unknown Method ───────────────────────────────────────────────
        _ => Err(ctx.attribute_error(line, format!("bytes has no method `{}`", method_name))),
    }
}
