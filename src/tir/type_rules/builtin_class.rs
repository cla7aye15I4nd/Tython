//! Unified builtin dunder lookup for builtin value types.

use super::builtin_call::BuiltinCallRule;
use crate::tir::builtin::BuiltinFn;
use crate::tir::ValueType;

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
            ValueType::Dict(_, _) => Some(BuiltinCallRule::ExternalCall {
                func: BuiltinFn::DictLen,
                return_type: ValueType::Int,
            }),
            ValueType::Set(_) => Some(BuiltinCallRule::ExternalCall {
                func: BuiltinFn::SetLen,
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
            ValueType::List(_) => Some(BuiltinCallRule::StrAuto),
            _ => None,
        },
        _ => None,
    }
}
