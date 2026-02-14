use anyhow::Result;

use crate::tir::{builtin::BuiltinFn, CallResult, TirExpr, ValueType};

use super::super::Lowering;

pub mod bytearray;
pub mod bytes;
pub mod r#str;

pub use bytearray::lower_bytearray_method_call;
pub use bytes::lower_bytes_method_call;
pub use r#str::lower_str_method_call;

fn lower_fixed_void_method(
    ctx: &Lowering,
    line: usize,
    type_name: &str,
    obj: TirExpr,
    method_name: &str,
    args: Vec<TirExpr>,
    expected: &[ValueType],
    func: BuiltinFn,
) -> Result<CallResult> {
    super::check_arity(
        ctx,
        line,
        type_name,
        method_name,
        expected.len(),
        args.len(),
    )?;
    for (arg, ty) in args.iter().zip(expected.iter()) {
        super::check_type(ctx, line, type_name, method_name, arg, ty)?;
    }
    Ok(super::void_call(func, obj, args))
}

fn lower_fixed_expr_method(
    ctx: &Lowering,
    line: usize,
    type_name: &str,
    obj: TirExpr,
    method_name: &str,
    args: Vec<TirExpr>,
    expected: &[ValueType],
    func: BuiltinFn,
    return_type: ValueType,
) -> Result<CallResult> {
    super::check_arity(
        ctx,
        line,
        type_name,
        method_name,
        expected.len(),
        args.len(),
    )?;
    for (arg, ty) in args.iter().zip(expected.iter()) {
        super::check_type(ctx, line, type_name, method_name, arg, ty)?;
    }
    Ok(super::expr_call(func, return_type, obj, args))
}
