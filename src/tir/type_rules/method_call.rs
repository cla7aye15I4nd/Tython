use crate::tir::builtin::BuiltinFn;
use crate::tir::ValueType;

/// Describes a resolved method call on a builtin type (list, bytearray, etc.).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MethodCallRule {
    /// Expected argument types (not including the receiver/self object).
    pub params: Vec<ValueType>,
    /// What the method call produces.
    pub result: MethodCallResult,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MethodCallResult {
    /// Method returns nothing; emitted as VoidCall.
    Void(BuiltinFn),
    /// Method returns a value; emitted as ExternalCall.
    Expr {
        func: BuiltinFn,
        return_type: ValueType,
    },
}

/// Look up a method on `list[inner]`.
///
/// Returns:
/// - `Some(Ok(rule))` — method exists and is supported for this inner type
/// - `Some(Err(msg))` — method is recognized but unsupported for this inner type
/// - `None` — method name is not a known list method
pub fn lookup_list_method(inner: &ValueType, name: &str) -> Option<Result<MethodCallRule, String>> {
    match name {
        "append" => {
            if !inner.is_primitive() {
                return Some(Err(format!(
                    "list[{}].append() is not supported; only list[int], list[float], list[bool] support append",
                    inner
                )));
            }
            Some(Ok(MethodCallRule {
                params: vec![inner.clone()],
                result: MethodCallResult::Void(BuiltinFn::ListAppend),
            }))
        }
        "clear" => Some(Ok(MethodCallRule {
            params: vec![],
            result: MethodCallResult::Void(BuiltinFn::ListClear),
        })),
        "pop" => Some(Ok(MethodCallRule {
            params: vec![],
            result: MethodCallResult::Expr {
                func: BuiltinFn::ListPop,
                return_type: inner.clone(),
            },
        })),
        "insert" => Some(Ok(MethodCallRule {
            params: vec![ValueType::Int, inner.clone()],
            result: MethodCallResult::Void(BuiltinFn::ListInsert),
        })),
        "remove" => Some(Ok(MethodCallRule {
            params: vec![inner.clone()],
            result: MethodCallResult::Void(BuiltinFn::ListRemove),
        })),
        "index" => Some(Ok(MethodCallRule {
            params: vec![inner.clone()],
            result: MethodCallResult::Expr {
                func: BuiltinFn::ListIndex,
                return_type: ValueType::Int,
            },
        })),
        "count" => Some(Ok(MethodCallRule {
            params: vec![inner.clone()],
            result: MethodCallResult::Expr {
                func: BuiltinFn::ListCount,
                return_type: ValueType::Int,
            },
        })),
        "reverse" => Some(Ok(MethodCallRule {
            params: vec![],
            result: MethodCallResult::Void(BuiltinFn::ListReverse),
        })),
        "sort" => {
            let sort_fn = match inner {
                ValueType::Int | ValueType::Bool => BuiltinFn::ListSortInt,
                ValueType::Float => BuiltinFn::ListSortFloat,
                ValueType::Str => BuiltinFn::ListSortStr,
                ValueType::Bytes => BuiltinFn::ListSortBytes,
                ValueType::ByteArray => BuiltinFn::ListSortByteArray,
                _ => {
                    return Some(Err(format!(
                        "list[{}].sort() is not supported; element type has no `__lt__`",
                        inner
                    )));
                }
            };
            Some(Ok(MethodCallRule {
                params: vec![],
                result: MethodCallResult::Void(sort_fn),
            }))
        }
        "extend" => Some(Ok(MethodCallRule {
            params: {
                if !inner.is_primitive() {
                    return Some(Err(format!(
                        "list[{}].extend() is not supported; only list[int], list[float], list[bool] support extend",
                        inner
                    )));
                }
                vec![ValueType::List(Box::new(inner.clone()))]
            },
            result: MethodCallResult::Void(BuiltinFn::ListExtend),
        })),
        "copy" => Some(Ok(MethodCallRule {
            params: vec![],
            result: MethodCallResult::Expr {
                func: BuiltinFn::ListCopy,
                return_type: ValueType::List(Box::new(inner.clone())),
            },
        })),
        _ => None,
    }
}

/// Look up a method on `bytearray`.
///
/// Returns:
/// - `Some(Ok(rule))` — method exists and is supported
/// - `Some(Err(msg))` — method is recognized but unsupported
/// - `None` — method name is not a known bytearray method
pub fn lookup_bytearray_method(name: &str) -> Option<Result<MethodCallRule, String>> {
    match name {
        "append" => Some(Ok(MethodCallRule {
            params: vec![ValueType::Int],
            result: MethodCallResult::Void(BuiltinFn::ByteArrayAppend),
        })),
        "extend" => Some(Ok(MethodCallRule {
            params: vec![ValueType::Bytes],
            result: MethodCallResult::Void(BuiltinFn::ByteArrayExtend),
        })),
        "clear" => Some(Ok(MethodCallRule {
            params: vec![],
            result: MethodCallResult::Void(BuiltinFn::ByteArrayClear),
        })),
        "insert" => Some(Ok(MethodCallRule {
            params: vec![ValueType::Int, ValueType::Int],
            result: MethodCallResult::Void(BuiltinFn::ByteArrayInsert),
        })),
        "remove" => Some(Ok(MethodCallRule {
            params: vec![ValueType::Int],
            result: MethodCallResult::Void(BuiltinFn::ByteArrayRemove),
        })),
        "reverse" => Some(Ok(MethodCallRule {
            params: vec![],
            result: MethodCallResult::Void(BuiltinFn::ByteArrayReverse),
        })),
        _ => None,
    }
}

/// Look up a method on `str`.
///
/// Returns:
/// - `Some(Ok(rule))` — method exists and is supported
/// - `Some(Err(msg))` — method is recognized but unsupported
/// - `None` — method name is not a known str method
pub fn lookup_str_method(name: &str) -> Option<Result<MethodCallRule, String>> {
    match name {
        // No str methods exposed yet; stub for unified dispatch.
        _ => None,
    }
}

/// Look up a method on `bytes`.
///
/// Returns:
/// - `Some(Ok(rule))` — method exists and is supported
/// - `Some(Err(msg))` — method is recognized but unsupported
/// - `None` — method name is not a known bytes method
pub fn lookup_bytes_method(name: &str) -> Option<Result<MethodCallRule, String>> {
    match name {
        // No bytes methods exposed yet; stub for unified dispatch.
        _ => None,
    }
}

/// Generate error message for wrong argument count in a method call.
pub fn method_call_arity_error(
    type_name: &str,
    method_name: &str,
    expected: usize,
    got: usize,
) -> String {
    if expected == 0 {
        format!("{}.{}() takes no arguments", type_name, method_name)
    } else {
        format!(
            "{}.{}() expects {} argument{}, got {}",
            type_name,
            method_name,
            expected,
            if expected == 1 { "" } else { "s" },
            got,
        )
    }
}

/// Generate error message for wrong argument type in a method call.
pub fn method_call_type_error(
    type_name: &str,
    method_name: &str,
    expected: &ValueType,
    got: &ValueType,
) -> String {
    format!(
        "{}.{}() expects `{}`, got `{}`",
        type_name, method_name, expected, got
    )
}
