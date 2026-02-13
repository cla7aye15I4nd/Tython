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
        "append" => Some(Ok(MethodCallRule {
            params: vec![inner.clone()],
            result: MethodCallResult::Void(BuiltinFn::ListAppend),
        })),
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
            params: vec![ValueType::List(Box::new(inner.clone()))],
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

pub fn lookup_dict_method(
    key: &ValueType,
    value: &ValueType,
    name: &str,
) -> Option<Result<MethodCallRule, String>> {
    match name {
        "clear" => Some(Ok(MethodCallRule {
            params: vec![],
            result: MethodCallResult::Void(BuiltinFn::DictClear),
        })),
        "get" => Some(Ok(MethodCallRule {
            params: vec![key.clone()],
            result: MethodCallResult::Expr {
                func: BuiltinFn::DictGet,
                return_type: value.clone(),
            },
        })),
        "pop" => Some(Ok(MethodCallRule {
            params: vec![key.clone()],
            result: MethodCallResult::Expr {
                func: BuiltinFn::DictPop,
                return_type: value.clone(),
            },
        })),
        "copy" => Some(Ok(MethodCallRule {
            params: vec![],
            result: MethodCallResult::Expr {
                func: BuiltinFn::DictCopy,
                return_type: ValueType::Dict(Box::new(key.clone()), Box::new(value.clone())),
            },
        })),
        "values" => Some(Ok(MethodCallRule {
            params: vec![],
            result: MethodCallResult::Expr {
                func: BuiltinFn::DictValues,
                return_type: ValueType::List(Box::new(value.clone())),
            },
        })),
        _ => None,
    }
}

pub fn lookup_set_method(inner: &ValueType, name: &str) -> Option<Result<MethodCallRule, String>> {
    match name {
        "add" => Some(Ok(MethodCallRule {
            params: vec![inner.clone()],
            result: MethodCallResult::Void(BuiltinFn::SetAdd),
        })),
        "remove" => Some(Ok(MethodCallRule {
            params: vec![inner.clone()],
            result: MethodCallResult::Void(BuiltinFn::SetRemove),
        })),
        "discard" => Some(Ok(MethodCallRule {
            params: vec![inner.clone()],
            result: MethodCallResult::Void(BuiltinFn::SetDiscard),
        })),
        "pop" => Some(Ok(MethodCallRule {
            params: vec![],
            result: MethodCallResult::Expr {
                func: BuiltinFn::SetPop,
                return_type: inner.clone(),
            },
        })),
        "clear" => Some(Ok(MethodCallRule {
            params: vec![],
            result: MethodCallResult::Void(BuiltinFn::SetClear),
        })),
        "copy" => Some(Ok(MethodCallRule {
            params: vec![],
            result: MethodCallResult::Expr {
                func: BuiltinFn::SetCopy,
                return_type: ValueType::Set(Box::new(inner.clone())),
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
        // Compatibility shim for open(...).read() lowering to plain str.
        "read" => Some(Ok(MethodCallRule {
            params: vec![],
            result: MethodCallResult::Expr {
                func: BuiltinFn::StrRead,
                return_type: ValueType::Str,
            },
        })),
        "strip" => Some(Ok(MethodCallRule {
            params: vec![],
            result: MethodCallResult::Expr {
                func: BuiltinFn::StrStrip,
                return_type: ValueType::Str,
            },
        })),
        "split" => Some(Ok(MethodCallRule {
            params: vec![ValueType::Str],
            result: MethodCallResult::Expr {
                func: BuiltinFn::StrSplit,
                return_type: ValueType::List(Box::new(ValueType::Str)),
            },
        })),
        "join" => Some(Ok(MethodCallRule {
            params: vec![ValueType::List(Box::new(ValueType::Str))],
            result: MethodCallResult::Expr {
                func: BuiltinFn::StrJoin,
                return_type: ValueType::Str,
            },
        })),
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
        "capitalize" => Some(Ok(MethodCallRule {
            params: vec![],
            result: MethodCallResult::Expr {
                func: BuiltinFn::BytesCapitalize,
                return_type: ValueType::Bytes,
            },
        })),
        "center" => Some(Ok(MethodCallRule {
            params: vec![ValueType::Int, ValueType::Bytes],
            result: MethodCallResult::Expr {
                func: BuiltinFn::BytesCenter,
                return_type: ValueType::Bytes,
            },
        })),
        "count" => Some(Ok(MethodCallRule {
            params: vec![ValueType::Bytes],
            result: MethodCallResult::Expr {
                func: BuiltinFn::BytesCount,
                return_type: ValueType::Int,
            },
        })),
        "decode" => Some(Ok(MethodCallRule {
            params: vec![],
            result: MethodCallResult::Expr {
                func: BuiltinFn::BytesDecode,
                return_type: ValueType::Str,
            },
        })),
        "endswith" => Some(Ok(MethodCallRule {
            params: vec![ValueType::Bytes],
            result: MethodCallResult::Expr {
                func: BuiltinFn::BytesEndsWith,
                return_type: ValueType::Bool,
            },
        })),
        "expandtabs" => Some(Ok(MethodCallRule {
            params: vec![ValueType::Int],
            result: MethodCallResult::Expr {
                func: BuiltinFn::BytesExpandTabs,
                return_type: ValueType::Bytes,
            },
        })),
        "find" => Some(Ok(MethodCallRule {
            params: vec![ValueType::Bytes],
            result: MethodCallResult::Expr {
                func: BuiltinFn::BytesFind,
                return_type: ValueType::Int,
            },
        })),
        "fromhex" => Some(Ok(MethodCallRule {
            params: vec![ValueType::Str],
            result: MethodCallResult::Expr {
                func: BuiltinFn::BytesFromHex,
                return_type: ValueType::Bytes,
            },
        })),
        "hex" => Some(Ok(MethodCallRule {
            params: vec![],
            result: MethodCallResult::Expr {
                func: BuiltinFn::BytesHex,
                return_type: ValueType::Str,
            },
        })),
        "index" => Some(Ok(MethodCallRule {
            params: vec![ValueType::Bytes],
            result: MethodCallResult::Expr {
                func: BuiltinFn::BytesIndex,
                return_type: ValueType::Int,
            },
        })),
        "isalnum" => Some(Ok(MethodCallRule {
            params: vec![],
            result: MethodCallResult::Expr {
                func: BuiltinFn::BytesIsAlnum,
                return_type: ValueType::Bool,
            },
        })),
        "isalpha" => Some(Ok(MethodCallRule {
            params: vec![],
            result: MethodCallResult::Expr {
                func: BuiltinFn::BytesIsAlpha,
                return_type: ValueType::Bool,
            },
        })),
        "isascii" => Some(Ok(MethodCallRule {
            params: vec![],
            result: MethodCallResult::Expr {
                func: BuiltinFn::BytesIsAscii,
                return_type: ValueType::Bool,
            },
        })),
        "isdigit" => Some(Ok(MethodCallRule {
            params: vec![],
            result: MethodCallResult::Expr {
                func: BuiltinFn::BytesIsDigit,
                return_type: ValueType::Bool,
            },
        })),
        "islower" => Some(Ok(MethodCallRule {
            params: vec![],
            result: MethodCallResult::Expr {
                func: BuiltinFn::BytesIsLower,
                return_type: ValueType::Bool,
            },
        })),
        "isspace" => Some(Ok(MethodCallRule {
            params: vec![],
            result: MethodCallResult::Expr {
                func: BuiltinFn::BytesIsSpace,
                return_type: ValueType::Bool,
            },
        })),
        "istitle" => Some(Ok(MethodCallRule {
            params: vec![],
            result: MethodCallResult::Expr {
                func: BuiltinFn::BytesIsTitle,
                return_type: ValueType::Bool,
            },
        })),
        "isupper" => Some(Ok(MethodCallRule {
            params: vec![],
            result: MethodCallResult::Expr {
                func: BuiltinFn::BytesIsUpper,
                return_type: ValueType::Bool,
            },
        })),
        "join" => Some(Ok(MethodCallRule {
            params: vec![ValueType::List(Box::new(ValueType::Bytes))],
            result: MethodCallResult::Expr {
                func: BuiltinFn::BytesJoin,
                return_type: ValueType::Bytes,
            },
        })),
        "ljust" => Some(Ok(MethodCallRule {
            params: vec![ValueType::Int, ValueType::Bytes],
            result: MethodCallResult::Expr {
                func: BuiltinFn::BytesLJust,
                return_type: ValueType::Bytes,
            },
        })),
        "lower" => Some(Ok(MethodCallRule {
            params: vec![],
            result: MethodCallResult::Expr {
                func: BuiltinFn::BytesLower,
                return_type: ValueType::Bytes,
            },
        })),
        "lstrip" => Some(Ok(MethodCallRule {
            params: vec![ValueType::Bytes],
            result: MethodCallResult::Expr {
                func: BuiltinFn::BytesLStrip,
                return_type: ValueType::Bytes,
            },
        })),
        "maketrans" => Some(Ok(MethodCallRule {
            params: vec![ValueType::Bytes, ValueType::Bytes],
            result: MethodCallResult::Expr {
                func: BuiltinFn::BytesMakeTrans,
                return_type: ValueType::Bytes,
            },
        })),
        "partition" => Some(Ok(MethodCallRule {
            params: vec![ValueType::Bytes],
            result: MethodCallResult::Expr {
                func: BuiltinFn::BytesPartition,
                return_type: ValueType::Tuple(vec![
                    ValueType::Bytes,
                    ValueType::Bytes,
                    ValueType::Bytes,
                ]),
            },
        })),
        "removeprefix" => Some(Ok(MethodCallRule {
            params: vec![ValueType::Bytes],
            result: MethodCallResult::Expr {
                func: BuiltinFn::BytesRemovePrefix,
                return_type: ValueType::Bytes,
            },
        })),
        "removesuffix" => Some(Ok(MethodCallRule {
            params: vec![ValueType::Bytes],
            result: MethodCallResult::Expr {
                func: BuiltinFn::BytesRemoveSuffix,
                return_type: ValueType::Bytes,
            },
        })),
        "replace" => Some(Ok(MethodCallRule {
            params: vec![ValueType::Bytes, ValueType::Bytes],
            result: MethodCallResult::Expr {
                func: BuiltinFn::BytesReplace,
                return_type: ValueType::Bytes,
            },
        })),
        "rfind" => Some(Ok(MethodCallRule {
            params: vec![ValueType::Bytes],
            result: MethodCallResult::Expr {
                func: BuiltinFn::BytesRFind,
                return_type: ValueType::Int,
            },
        })),
        "rindex" => Some(Ok(MethodCallRule {
            params: vec![ValueType::Bytes],
            result: MethodCallResult::Expr {
                func: BuiltinFn::BytesRIndex,
                return_type: ValueType::Int,
            },
        })),
        "rjust" => Some(Ok(MethodCallRule {
            params: vec![ValueType::Int, ValueType::Bytes],
            result: MethodCallResult::Expr {
                func: BuiltinFn::BytesRJust,
                return_type: ValueType::Bytes,
            },
        })),
        "rpartition" => Some(Ok(MethodCallRule {
            params: vec![ValueType::Bytes],
            result: MethodCallResult::Expr {
                func: BuiltinFn::BytesRPartition,
                return_type: ValueType::Tuple(vec![
                    ValueType::Bytes,
                    ValueType::Bytes,
                    ValueType::Bytes,
                ]),
            },
        })),
        "rsplit" => Some(Ok(MethodCallRule {
            params: vec![ValueType::Bytes],
            result: MethodCallResult::Expr {
                func: BuiltinFn::BytesRSplit,
                return_type: ValueType::List(Box::new(ValueType::Bytes)),
            },
        })),
        "rstrip" => Some(Ok(MethodCallRule {
            params: vec![ValueType::Bytes],
            result: MethodCallResult::Expr {
                func: BuiltinFn::BytesRStrip,
                return_type: ValueType::Bytes,
            },
        })),
        "split" => Some(Ok(MethodCallRule {
            params: vec![ValueType::Bytes],
            result: MethodCallResult::Expr {
                func: BuiltinFn::BytesSplit,
                return_type: ValueType::List(Box::new(ValueType::Bytes)),
            },
        })),
        "splitlines" => Some(Ok(MethodCallRule {
            params: vec![],
            result: MethodCallResult::Expr {
                func: BuiltinFn::BytesSplitLines,
                return_type: ValueType::List(Box::new(ValueType::Bytes)),
            },
        })),
        "startswith" => Some(Ok(MethodCallRule {
            params: vec![ValueType::Bytes],
            result: MethodCallResult::Expr {
                func: BuiltinFn::BytesStartsWith,
                return_type: ValueType::Bool,
            },
        })),
        "strip" => Some(Ok(MethodCallRule {
            params: vec![ValueType::Bytes],
            result: MethodCallResult::Expr {
                func: BuiltinFn::BytesStrip,
                return_type: ValueType::Bytes,
            },
        })),
        "swapcase" => Some(Ok(MethodCallRule {
            params: vec![],
            result: MethodCallResult::Expr {
                func: BuiltinFn::BytesSwapCase,
                return_type: ValueType::Bytes,
            },
        })),
        "title" => Some(Ok(MethodCallRule {
            params: vec![],
            result: MethodCallResult::Expr {
                func: BuiltinFn::BytesTitle,
                return_type: ValueType::Bytes,
            },
        })),
        "translate" => Some(Ok(MethodCallRule {
            params: vec![ValueType::Bytes],
            result: MethodCallResult::Expr {
                func: BuiltinFn::BytesTranslate,
                return_type: ValueType::Bytes,
            },
        })),
        "upper" => Some(Ok(MethodCallRule {
            params: vec![],
            result: MethodCallResult::Expr {
                func: BuiltinFn::BytesUpper,
                return_type: ValueType::Bytes,
            },
        })),
        "zfill" => Some(Ok(MethodCallRule {
            params: vec![ValueType::Int],
            result: MethodCallResult::Expr {
                func: BuiltinFn::BytesZFill,
                return_type: ValueType::Bytes,
            },
        })),
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
