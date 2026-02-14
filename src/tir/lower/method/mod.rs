use anyhow::Result;

use crate::tir::{
    builtin::BuiltinFn, type_rules, CallResult, CallTarget, TirExpr, TirExprKind, TirStmt,
    ValueType,
};

use super::Lowering;

pub mod bytearray;
pub mod bytes;
pub mod dict;
pub mod list;
pub mod set;
pub mod str;

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

// ── Dispatcher ───────────────────────────────────────────────────────

impl Lowering {
    /// Dispatch method calls for builtin types to type-specific handlers.
    ///
    /// This function routes method calls to per-type lowering functions based on
    /// the object's type. Each type has its own dedicated module that generates
    /// TIR directly without using type rules.
    ///
    /// Supported types:
    /// - `list[T]` → `list::lower_list_method_call`
    /// - `dict[K, V]` → `dict::lower_dict_method_call`
    /// - `set[T]` → `set::lower_set_method_call`
    /// - `str` → `str::lower_str_method_call`
    /// - `bytes` → `bytes::lower_bytes_method_call`
    /// - `bytearray` → `bytearray::lower_bytearray_method_call`
    ///
    /// Other types fall back to the old centralized `lower_builtin_method_call`.
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
            ValueType::Str => str::lower_str_method_call(self, line, obj_expr, method_name, args),
            ValueType::Bytes => {
                bytes::lower_bytes_method_call(self, line, obj_expr, method_name, args)
            }
            ValueType::ByteArray => {
                bytearray::lower_bytearray_method_call(self, line, obj_expr, method_name, args)
            }
            // For now, other types (e.g., Tuple, Class) fall back to the old implementation
            ref ty => {
                // Get type name for error messages
                let type_name = type_rules::builtin_type_display_name(ty);

                // Look up method in type rules
                let lookup = type_rules::lookup_builtin_method(ty, method_name);

                // Call old implementation
                self.lower_builtin_method_call(
                    line,
                    obj_expr,
                    args,
                    method_name,
                    &type_name,
                    lookup,
                )
            }
        }
    }
}
