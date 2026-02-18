use anyhow::Result;

use crate::tir::lower::Lowering;
use crate::tir::{builtin::BuiltinFn, CallResult, TirExpr, ValueType};

use super::{lower_fixed_expr_method, lower_fixed_void_method};

pub fn lower_file_method_call(
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
            "file",
            obj,
            method_name,
            args,
            &[],
            BuiltinFn::FileRead,
            ValueType::Str,
        ),
        "write" => lower_fixed_expr_method(
            ctx,
            line,
            "file",
            obj,
            method_name,
            args,
            &[ValueType::Str],
            BuiltinFn::FileWrite,
            ValueType::Int,
        ),
        "close" => lower_fixed_void_method(
            ctx,
            line,
            "file",
            obj,
            method_name,
            args,
            &[],
            BuiltinFn::FileClose,
        ),
        _ => Err(ctx.attribute_error(line, format!("{} has no method `{}`", "file", method_name))),
    }
}
