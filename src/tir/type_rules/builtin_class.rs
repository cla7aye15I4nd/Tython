//! Unified builtin type method lookup.
//!
//! Provides a single entry point for looking up methods on builtin types
//! (list, bytearray, str, bytes, etc.), abstracting over the per-type
//! lookup functions in `method_call.rs`.

use super::builtin_call::BuiltinCallRule;
use super::method_call::{
    lookup_bytearray_method, lookup_bytes_method, lookup_list_method, lookup_str_method,
    MethodCallRule,
};
use crate::tir::builtin::BuiltinFn;
use crate::tir::ValueType;

/// Look up a method on a builtin (non-class) value type.
///
/// Returns:
/// - `Some(Ok(rule))` — method exists and is supported
/// - `Some(Err(msg))` — method is recognized but unsupported for this type configuration
/// - `None` — not a builtin type with methods, or method name is unknown
pub fn lookup_builtin_method(
    ty: &ValueType,
    method_name: &str,
) -> Option<Result<MethodCallRule, String>> {
    match ty {
        ValueType::List(inner) => lookup_list_method(inner, method_name),
        ValueType::ByteArray => lookup_bytearray_method(method_name),
        ValueType::Str => lookup_str_method(method_name),
        ValueType::Bytes => lookup_bytes_method(method_name),
        _ => None,
    }
}

/// Look up a dunder method on a builtin type, returning the equivalent `BuiltinCallRule`.
///
/// This allows builtins like `len(x)` and `str(x)` to resolve via a unified
/// dunder lookup (`__len__`, `__str__`) rather than per-type match arms in
/// `lookup_builtin_call()`.
pub fn lookup_builtin_dunder(ty: &ValueType, dunder: &str) -> Option<BuiltinCallRule> {
    match dunder {
        "__len__" => match ty {
            ValueType::Str => Some(BuiltinCallRule::ExternalCall {
                func: BuiltinFn::StrLen,
                return_type: ValueType::Int,
            }),
            ValueType::Bytes => Some(BuiltinCallRule::ExternalCall {
                func: BuiltinFn::BytesLen,
                return_type: ValueType::Int,
            }),
            ValueType::ByteArray => Some(BuiltinCallRule::ExternalCall {
                func: BuiltinFn::ByteArrayLen,
                return_type: ValueType::Int,
            }),
            ValueType::List(_) => Some(BuiltinCallRule::ExternalCall {
                func: BuiltinFn::ListLen,
                return_type: ValueType::Int,
            }),
            _ => None,
        },
        "__str__" => match ty {
            ValueType::Str => Some(BuiltinCallRule::Identity),
            ValueType::Int => Some(BuiltinCallRule::ExternalCall {
                func: BuiltinFn::StrFromInt,
                return_type: ValueType::Str,
            }),
            ValueType::Float => Some(BuiltinCallRule::ExternalCall {
                func: BuiltinFn::StrFromFloat,
                return_type: ValueType::Str,
            }),
            ValueType::Bool => Some(BuiltinCallRule::ExternalCall {
                func: BuiltinFn::StrFromBool,
                return_type: ValueType::Str,
            }),
            ValueType::Bytes => Some(BuiltinCallRule::ExternalCall {
                func: BuiltinFn::StrFromBytes,
                return_type: ValueType::Str,
            }),
            ValueType::ByteArray => Some(BuiltinCallRule::ExternalCall {
                func: BuiltinFn::StrFromByteArray,
                return_type: ValueType::Str,
            }),
            ValueType::List(_) | ValueType::Tuple(_) => Some(BuiltinCallRule::StrAuto),
            _ => None,
        },
        _ => None,
    }
}

/// Return the human-readable type name for error messages.
pub fn builtin_type_display_name(ty: &ValueType) -> String {
    match ty {
        ValueType::List(inner) => format!("list[{}]", inner),
        _ => ty.to_string(),
    }
}
