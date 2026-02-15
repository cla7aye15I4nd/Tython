use anyhow::Result;

use crate::tir::lower::Lowering;
use crate::tir::{builtin::BuiltinFn, CallResult, TirExpr, ValueType};

use super::lower_fixed_expr_method;

pub fn lower_str_method_call(
    ctx: &Lowering,
    line: usize,
    obj: TirExpr,
    method_name: &str,
    args: Vec<TirExpr>,
) -> Result<CallResult> {
    match method_name {
        "read" => lower_fixed_expr_method(
            ctx,
            line,
            "str",
            obj,
            method_name,
            args,
            &[],
            BuiltinFn::StrRead,
            ValueType::Str,
        ),
        "strip" => lower_fixed_expr_method(
            ctx,
            line,
            "str",
            obj,
            method_name,
            args,
            &[],
            BuiltinFn::StrStrip,
            ValueType::Str,
        ),
        "split" => lower_fixed_expr_method(
            ctx,
            line,
            "str",
            obj,
            method_name,
            args,
            &[ValueType::Str],
            BuiltinFn::StrSplit,
            ValueType::List(Box::new(ValueType::Str)),
        ),
        "join" => lower_fixed_expr_method(
            ctx,
            line,
            "str",
            obj,
            method_name,
            args,
            &[ValueType::List(Box::new(ValueType::Str))],
            BuiltinFn::StrJoin,
            ValueType::Str,
        ),
        "__add__" => lower_fixed_expr_method(
            ctx,
            line,
            "str",
            obj,
            method_name,
            args,
            &[ValueType::Str],
            BuiltinFn::StrConcat,
            ValueType::Str,
        ),
        "__mul__" | "__rmul__" => lower_fixed_expr_method(
            ctx,
            line,
            "str",
            obj,
            method_name,
            args,
            &[ValueType::Int],
            BuiltinFn::StrRepeat,
            ValueType::Str,
        ),
        _ => Err(ctx.attribute_error(line, format!("{} has no method `{}`", "str", method_name))),
    }
}
