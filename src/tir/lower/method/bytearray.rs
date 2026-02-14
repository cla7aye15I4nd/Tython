use anyhow::Result;

use crate::tir::{builtin::BuiltinFn, CallResult, TirExpr, ValueType};

use super::super::Lowering;

/// Lower a method call on bytearray to TIR.
///
/// Handles all bytearray methods (50+ string-like and mutable methods).
///
/// Directly generates TIR without using type rules - all logic is self-contained here.
pub fn lower_bytearray_method_call(
    ctx: &Lowering,
    line: usize,
    obj: TirExpr,
    method_name: &str,
    args: Vec<TirExpr>,
) -> Result<CallResult> {
    // Generate common helper functions using macro
    let type_name = "bytearray";

    match method_name {
        // ── Mutable Methods ──────────────────────────────────────────────
        "append" => {
            super::check_arity(ctx, line, type_name, method_name, 1, args.len())?;
            super::check_type(ctx, line, type_name, method_name, &args[0], &ValueType::Int)?;
            Ok(super::void_call(
                BuiltinFn::ByteArrayAppend,
                obj.clone(),
                args,
            ))
        }

        "clear" => {
            super::check_arity(ctx, line, type_name, method_name, 0, args.len())?;
            Ok(super::void_call(
                BuiltinFn::ByteArrayClear,
                obj.clone(),
                args,
            ))
        }

        "extend" => {
            super::check_arity(ctx, line, type_name, method_name, 1, args.len())?;
            super::check_type(
                ctx,
                line,
                type_name,
                method_name,
                &args[0],
                &ValueType::Bytes,
            )?;
            Ok(super::void_call(
                BuiltinFn::ByteArrayExtend,
                obj.clone(),
                args,
            ))
        }

        "insert" => {
            super::check_arity(ctx, line, type_name, method_name, 2, args.len())?;
            super::check_type(ctx, line, type_name, method_name, &args[0], &ValueType::Int)?;
            super::check_type(ctx, line, type_name, method_name, &args[1], &ValueType::Int)?;
            Ok(super::void_call(
                BuiltinFn::ByteArrayInsert,
                obj.clone(),
                args,
            ))
        }

        "remove" => {
            super::check_arity(ctx, line, type_name, method_name, 1, args.len())?;
            super::check_type(ctx, line, type_name, method_name, &args[0], &ValueType::Int)?;
            Ok(super::void_call(
                BuiltinFn::ByteArrayRemove,
                obj.clone(),
                args,
            ))
        }

        "reverse" => {
            super::check_arity(ctx, line, type_name, method_name, 0, args.len())?;
            Ok(super::void_call(
                BuiltinFn::ByteArrayReverse,
                obj.clone(),
                args,
            ))
        }

        "pop" => {
            super::check_arity(ctx, line, type_name, method_name, 0, args.len())?;
            Ok(super::expr_call(
                BuiltinFn::ByteArrayPop,
                ValueType::Int,
                obj.clone(),
                args,
            ))
        }

        "copy" => {
            super::check_arity(ctx, line, type_name, method_name, 0, args.len())?;
            Ok(super::expr_call(
                BuiltinFn::ByteArrayCopy,
                ValueType::ByteArray,
                obj.clone(),
                args,
            ))
        }

        // ── Transformation Methods ───────────────────────────────────────
        "capitalize" => {
            super::check_arity(ctx, line, type_name, method_name, 0, args.len())?;
            Ok(super::expr_call(
                BuiltinFn::ByteArrayCapitalize,
                ValueType::ByteArray,
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
                BuiltinFn::ByteArrayCenter,
                ValueType::ByteArray,
                obj.clone(),
                args,
            ))
        }

        "decode" => {
            super::check_arity(ctx, line, type_name, method_name, 0, args.len())?;
            Ok(super::expr_call(
                BuiltinFn::ByteArrayDecode,
                ValueType::Str,
                obj.clone(),
                args,
            ))
        }

        "expandtabs" => {
            super::check_arity(ctx, line, type_name, method_name, 1, args.len())?;
            super::check_type(ctx, line, type_name, method_name, &args[0], &ValueType::Int)?;
            Ok(super::expr_call(
                BuiltinFn::ByteArrayExpandTabs,
                ValueType::ByteArray,
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
                BuiltinFn::ByteArrayLJust,
                ValueType::ByteArray,
                obj.clone(),
                args,
            ))
        }

        "lower" => {
            super::check_arity(ctx, line, type_name, method_name, 0, args.len())?;
            Ok(super::expr_call(
                BuiltinFn::ByteArrayLower,
                ValueType::ByteArray,
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
                BuiltinFn::ByteArrayLStrip,
                ValueType::ByteArray,
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
                BuiltinFn::ByteArrayRemovePrefix,
                ValueType::ByteArray,
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
                BuiltinFn::ByteArrayRemoveSuffix,
                ValueType::ByteArray,
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
                BuiltinFn::ByteArrayReplace,
                ValueType::ByteArray,
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
                BuiltinFn::ByteArrayRJust,
                ValueType::ByteArray,
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
                BuiltinFn::ByteArrayRStrip,
                ValueType::ByteArray,
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
                BuiltinFn::ByteArrayStrip,
                ValueType::ByteArray,
                obj.clone(),
                args,
            ))
        }

        "swapcase" => {
            super::check_arity(ctx, line, type_name, method_name, 0, args.len())?;
            Ok(super::expr_call(
                BuiltinFn::ByteArraySwapCase,
                ValueType::ByteArray,
                obj.clone(),
                args,
            ))
        }

        "title" => {
            super::check_arity(ctx, line, type_name, method_name, 0, args.len())?;
            Ok(super::expr_call(
                BuiltinFn::ByteArrayTitle,
                ValueType::ByteArray,
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
                BuiltinFn::ByteArrayTranslate,
                ValueType::ByteArray,
                obj.clone(),
                args,
            ))
        }

        "upper" => {
            super::check_arity(ctx, line, type_name, method_name, 0, args.len())?;
            Ok(super::expr_call(
                BuiltinFn::ByteArrayUpper,
                ValueType::ByteArray,
                obj.clone(),
                args,
            ))
        }

        "zfill" => {
            super::check_arity(ctx, line, type_name, method_name, 1, args.len())?;
            super::check_type(ctx, line, type_name, method_name, &args[0], &ValueType::Int)?;
            Ok(super::expr_call(
                BuiltinFn::ByteArrayZFill,
                ValueType::ByteArray,
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
                BuiltinFn::ByteArrayCount,
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
                BuiltinFn::ByteArrayEndsWith,
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
                BuiltinFn::ByteArrayFind,
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
                BuiltinFn::ByteArrayIndex,
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
                BuiltinFn::ByteArrayRFind,
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
                BuiltinFn::ByteArrayRIndex,
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
                BuiltinFn::ByteArrayStartsWith,
                ValueType::Bool,
                obj.clone(),
                args,
            ))
        }

        // ── Predicate Methods ────────────────────────────────────────────
        "isalnum" => {
            super::check_arity(ctx, line, type_name, method_name, 0, args.len())?;
            Ok(super::expr_call(
                BuiltinFn::ByteArrayIsAlnum,
                ValueType::Bool,
                obj.clone(),
                args,
            ))
        }

        "isalpha" => {
            super::check_arity(ctx, line, type_name, method_name, 0, args.len())?;
            Ok(super::expr_call(
                BuiltinFn::ByteArrayIsAlpha,
                ValueType::Bool,
                obj.clone(),
                args,
            ))
        }

        "isascii" => {
            super::check_arity(ctx, line, type_name, method_name, 0, args.len())?;
            Ok(super::expr_call(
                BuiltinFn::ByteArrayIsAscii,
                ValueType::Bool,
                obj.clone(),
                args,
            ))
        }

        "isdigit" => {
            super::check_arity(ctx, line, type_name, method_name, 0, args.len())?;
            Ok(super::expr_call(
                BuiltinFn::ByteArrayIsDigit,
                ValueType::Bool,
                obj.clone(),
                args,
            ))
        }

        "islower" => {
            super::check_arity(ctx, line, type_name, method_name, 0, args.len())?;
            Ok(super::expr_call(
                BuiltinFn::ByteArrayIsLower,
                ValueType::Bool,
                obj.clone(),
                args,
            ))
        }

        "isspace" => {
            super::check_arity(ctx, line, type_name, method_name, 0, args.len())?;
            Ok(super::expr_call(
                BuiltinFn::ByteArrayIsSpace,
                ValueType::Bool,
                obj.clone(),
                args,
            ))
        }

        "istitle" => {
            super::check_arity(ctx, line, type_name, method_name, 0, args.len())?;
            Ok(super::expr_call(
                BuiltinFn::ByteArrayIsTitle,
                ValueType::Bool,
                obj.clone(),
                args,
            ))
        }

        "isupper" => {
            super::check_arity(ctx, line, type_name, method_name, 0, args.len())?;
            Ok(super::expr_call(
                BuiltinFn::ByteArrayIsUpper,
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
                &ValueType::List(Box::new(ValueType::ByteArray)),
            )?;
            Ok(super::expr_call(
                BuiltinFn::ByteArrayJoin,
                ValueType::ByteArray,
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
                BuiltinFn::ByteArrayPartition,
                ValueType::Tuple(vec![
                    ValueType::ByteArray,
                    ValueType::ByteArray,
                    ValueType::ByteArray,
                ]),
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
                BuiltinFn::ByteArrayRPartition,
                ValueType::Tuple(vec![
                    ValueType::ByteArray,
                    ValueType::ByteArray,
                    ValueType::ByteArray,
                ]),
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
                BuiltinFn::ByteArrayRSplit,
                ValueType::List(Box::new(ValueType::ByteArray)),
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
                BuiltinFn::ByteArraySplit,
                ValueType::List(Box::new(ValueType::ByteArray)),
                obj.clone(),
                args,
            ))
        }

        "splitlines" => {
            super::check_arity(ctx, line, type_name, method_name, 0, args.len())?;
            Ok(super::expr_call(
                BuiltinFn::ByteArraySplitLines,
                ValueType::List(Box::new(ValueType::ByteArray)),
                obj.clone(),
                args,
            ))
        }

        // ── Utility Methods ──────────────────────────────────────────────
        "fromhex" => {
            super::check_arity(ctx, line, type_name, method_name, 1, args.len())?;
            super::check_type(ctx, line, type_name, method_name, &args[0], &ValueType::Str)?;
            Ok(super::expr_call(
                BuiltinFn::ByteArrayFromHex,
                ValueType::ByteArray,
                obj.clone(),
                args,
            ))
        }

        "hex" => {
            super::check_arity(ctx, line, type_name, method_name, 0, args.len())?;
            Ok(super::expr_call(
                BuiltinFn::ByteArrayHex,
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
                BuiltinFn::ByteArrayMakeTrans,
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
                &ValueType::ByteArray,
            )?;
            Ok(super::expr_call(
                BuiltinFn::ByteArrayConcat,
                ValueType::ByteArray,
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
                &ValueType::ByteArray,
            )?;
            Ok(super::expr_call(
                BuiltinFn::ByteArrayEq,
                ValueType::Bool,
                obj.clone(),
                args,
            ))
        }

        "__getitem__" => {
            super::check_arity(ctx, line, type_name, method_name, 1, args.len())?;
            super::check_type(ctx, line, type_name, method_name, &args[0], &ValueType::Int)?;
            Ok(super::expr_call(
                BuiltinFn::ByteArrayGet,
                ValueType::Int,
                obj.clone(),
                args,
            ))
        }

        "__len__" => {
            super::check_arity(ctx, line, type_name, method_name, 0, args.len())?;
            Ok(super::expr_call(
                BuiltinFn::ByteArrayLen,
                ValueType::Int,
                obj.clone(),
                args,
            ))
        }

        "__mul__" => {
            super::check_arity(ctx, line, type_name, method_name, 1, args.len())?;
            super::check_type(ctx, line, type_name, method_name, &args[0], &ValueType::Int)?;
            Ok(super::expr_call(
                BuiltinFn::ByteArrayRepeat,
                ValueType::ByteArray,
                obj.clone(),
                args,
            ))
        }

        // ── Unknown Method ───────────────────────────────────────────────
        _ => Err(ctx.attribute_error(line, format!("bytearray has no method `{}`", method_name))),
    }
}
