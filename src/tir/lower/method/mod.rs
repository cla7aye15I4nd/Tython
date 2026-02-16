use anyhow::Result;

use crate::tir::{
    builtin::BuiltinFn, CallResult, CallTarget, CastKind, TirExpr, TirExprKind, TirStmt, ValueType,
};

use super::Lowering;

pub mod bytearray;
pub mod bytes;
pub mod dict;
pub mod list;
pub mod set;
pub mod r#str;

// ── Helper Functions ─────────────────────────────────────────────────

/// Check that the method call has the expected number of arguments.
#[inline]
pub fn check_arity(
    ctx: &Lowering,
    line: usize,
    type_name: &str,
    method_name: &str,
    expected: usize,
    actual: usize,
) -> Result<()> {
    if actual != expected {
        return Err(ctx.type_error(
            line,
            format!(
                "{}.{}() takes {} argument{}, got {}",
                type_name,
                method_name,
                expected,
                if expected == 1 { "" } else { "s" },
                actual
            ),
        ));
    }
    Ok(())
}

/// Check that an argument has the expected type.
#[inline]
pub fn check_type(
    ctx: &Lowering,
    line: usize,
    type_name: &str,
    method_name: &str,
    arg: &TirExpr,
    expected: &ValueType,
) -> Result<()> {
    if &arg.ty != expected {
        return Err(ctx.type_error(
            line,
            format!(
                "{}.{}() expected argument of type {}, got {}",
                type_name, method_name, expected, arg.ty
            ),
        ));
    }
    Ok(())
}

/// Build a void call statement (method that returns nothing).
#[inline]
pub fn void_call(func: BuiltinFn, obj: TirExpr, mut args: Vec<TirExpr>) -> CallResult {
    args.insert(0, obj);
    CallResult::VoidStmt(Box::new(TirStmt::VoidCall {
        target: CallTarget::Builtin(func),
        args,
    }))
}

/// Build an expression call (method that returns a value).
#[inline]
pub fn expr_call(
    func: BuiltinFn,
    return_type: ValueType,
    obj: TirExpr,
    mut args: Vec<TirExpr>,
) -> CallResult {
    args.insert(0, obj);
    CallResult::Expr(TirExpr {
        kind: TirExprKind::ExternalCall { func, args },
        ty: return_type,
    })
}

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
    check_arity(
        ctx,
        line,
        type_name,
        method_name,
        expected.len(),
        args.len(),
    )?;
    for (arg, ty) in args.iter().zip(expected.iter()) {
        check_type(ctx, line, type_name, method_name, arg, ty)?;
    }
    Ok(void_call(func, obj, args))
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
    check_arity(
        ctx,
        line,
        type_name,
        method_name,
        expected.len(),
        args.len(),
    )?;
    for (arg, ty) in args.iter().zip(expected.iter()) {
        check_type(ctx, line, type_name, method_name, arg, ty)?;
    }
    Ok(expr_call(func, return_type, obj, args))
}

// ── Dispatcher ───────────────────────────────────────────────────────

impl Lowering {
    /// Dispatch method calls for builtin types.
    pub(in crate::tir::lower) fn lower_method_call(
        &mut self,
        line: usize,
        obj_expr: TirExpr,
        method_name: &str,
        args: Vec<TirExpr>,
    ) -> Result<CallResult> {
        // Clone what we need from obj_expr before matching to avoid borrow issues
        let obj_ty = obj_expr.ty.clone();

        match obj_ty {
            ValueType::List(inner) => {
                list::lower_list_method_call(self, line, obj_expr, method_name, args, &inner)
            }
            ValueType::Dict(key, value) => {
                dict::lower_dict_method_call(self, line, obj_expr, method_name, args, &key, &value)
            }
            ValueType::Set(inner) => {
                set::lower_set_method_call(self, line, obj_expr, method_name, args, &inner)
            }
            ValueType::Str => r#str::lower_str_method_call(self, line, obj_expr, method_name, args),
            ValueType::Bytes => {
                bytes::lower_bytes_method_call(self, line, obj_expr, method_name, args)
            }
            ValueType::ByteArray => {
                bytearray::lower_bytearray_method_call(self, line, obj_expr, method_name, args)
            }
            ty => {
                Err(self.attribute_error(line, format!("{} has no method `{}`", ty, method_name)))
            }
        }
    }

    /// Dispatch a unary operation through a dunder method (__neg__, __pos__, __invert__).
    pub(in crate::tir::lower) fn dispatch_unary_method(
        &mut self,
        line: usize,
        operand: TirExpr,
        method: &str,
        expected_return_type: Option<ValueType>,
    ) -> Result<TirExpr> {
        match &operand.ty {
            ValueType::Class(class_name) => {
                let class_info = self.lookup_class(line, class_name)?;
                if class_info.methods.contains_key(method) {
                    self.lower_class_magic_method_with_args(
                        line,
                        operand,
                        &[method],
                        expected_return_type,
                        "unary operator",
                        vec![],
                    )
                } else {
                    Err(self.type_error(
                        line,
                        format!(
                            "type `{}` does not support this unary operator (no `{}`)",
                            operand.ty, method
                        ),
                    ))
                }
            }
            _ => match self.lower_method_call(line, operand.clone(), method, vec![]) {
                Ok(CallResult::Expr(e)) => Ok(e),
                Ok(CallResult::VoidStmt(_)) => Err(self.type_error(
                    line,
                    format!(
                        "unary `{}` is not supported for type `{}`",
                        method, operand.ty
                    ),
                )),
                Err(e) => Err(e),
            },
        }
    }

    /// Return a TirExpr of type Str representing the repr() of a value.
    /// Primitives are converted directly; ref types forward to __repr__ via method dispatch.
    pub(in crate::tir::lower) fn lower_repr_str_expr(
        &mut self,
        line: usize,
        arg: TirExpr,
    ) -> Result<TirExpr> {
        match &arg.ty {
            ValueType::Int => Ok(TirExpr {
                kind: TirExprKind::ExternalCall {
                    func: BuiltinFn::StrFromInt,
                    args: vec![arg],
                },
                ty: ValueType::Str,
            }),
            ValueType::Float => Ok(TirExpr {
                kind: TirExprKind::ExternalCall {
                    func: BuiltinFn::StrFromFloat,
                    args: vec![arg],
                },
                ty: ValueType::Str,
            }),
            ValueType::Bool => Ok(TirExpr {
                kind: TirExprKind::ExternalCall {
                    func: BuiltinFn::StrFromBool,
                    args: vec![arg],
                },
                ty: ValueType::Str,
            }),
            ValueType::Class(name) => {
                let class_info = self.class_registry.get(name).cloned();
                if let Some(info) = class_info {
                    if let Some(repr_method) = info.methods.get("__repr__") {
                        Ok(TirExpr {
                            kind: TirExprKind::Call {
                                func: repr_method.mangled_name.clone(),
                                args: vec![arg],
                            },
                            ty: ValueType::Str,
                        })
                    } else {
                        Ok(TirExpr {
                            kind: TirExprKind::StrLiteral("<?>".to_string()),
                            ty: ValueType::Str,
                        })
                    }
                } else {
                    Ok(TirExpr {
                        kind: TirExprKind::StrLiteral("<?>".to_string()),
                        ty: ValueType::Str,
                    })
                }
            }
            _ => match self.lower_method_call(line, arg, "__repr__", vec![])? {
                CallResult::Expr(e) => Ok(e),
                CallResult::VoidStmt(_) => unreachable!("__repr__ should return a value"),
            },
        }
    }

    pub(in crate::tir::lower) fn compute_cast_kind(from: &ValueType, to: &ValueType) -> CastKind {
        match (from, to) {
            (ValueType::Int, ValueType::Float) => CastKind::IntToFloat,
            (ValueType::Float, ValueType::Int) => CastKind::FloatToInt,
            (ValueType::Bool, ValueType::Float) => CastKind::BoolToFloat,
            (ValueType::Int, ValueType::Bool) => CastKind::IntToBool,
            (ValueType::Float, ValueType::Bool) => CastKind::FloatToBool,
            (ValueType::Bool, ValueType::Int) => CastKind::BoolToInt,
            _ => unreachable!("identity cast should have been eliminated"),
        }
    }
}
