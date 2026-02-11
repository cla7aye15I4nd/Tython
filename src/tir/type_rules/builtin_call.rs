use crate::tir::builtin::BuiltinFn;
use crate::tir::ValueType;

/// Result of resolving a built-in call to its type-checked form.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuiltinCallRule {
    /// Identity conversion: reuse argument directly.
    Identity,
    /// Resolves to a runtime function call.
    ExternalCall {
        func: BuiltinFn,
        return_type: ValueType,
    },
    /// Resolves to a left-fold of a binary runtime function call.
    FoldExternalCall {
        func: BuiltinFn,
        return_type: ValueType,
    },
    /// Primitive cast lowered to `TirExprKind::Cast`.
    PrimitiveCast { target_type: ValueType },
    /// Compile-time constant i64 result.
    ConstInt(i64),
    /// `pow(float, float)` lowers to a `BinOp(**)` instead of a runtime call.
    PowFloat,
}

pub fn is_builtin_call(name: &str) -> bool {
    matches!(
        name,
        "str"
            | "repr"
            | "bytes"
            | "bytearray"
            | "int"
            | "float"
            | "bool"
            | "len"
            | "abs"
            | "round"
            | "pow"
            | "min"
            | "max"
    )
}

/// Resolve a unary numeric builtin (like abs) that has int and float variants.
fn numeric_unary_builtin(
    arg_types: &[&ValueType],
    int_fn: BuiltinFn,
    float_fn: BuiltinFn,
) -> Option<BuiltinCallRule> {
    match arg_types {
        [ValueType::Int] => Some(BuiltinCallRule::ExternalCall {
            func: int_fn,
            return_type: ValueType::Int,
        }),
        [ValueType::Float] => Some(BuiltinCallRule::ExternalCall {
            func: float_fn,
            return_type: ValueType::Float,
        }),
        _ => None,
    }
}

/// Resolve a variadic numeric builtin (like min/max) that has int and float variants.
fn numeric_variadic_builtin(
    arg_types: &[&ValueType],
    int_fn: BuiltinFn,
    float_fn: BuiltinFn,
) -> Option<BuiltinCallRule> {
    if arg_types.len() < 2 {
        return None;
    }

    if arg_types.iter().all(|ty| **ty == ValueType::Int) {
        return Some(BuiltinCallRule::FoldExternalCall {
            func: int_fn,
            return_type: ValueType::Int,
        });
    }

    if arg_types.iter().all(|ty| **ty == ValueType::Float) {
        return Some(BuiltinCallRule::FoldExternalCall {
            func: float_fn,
            return_type: ValueType::Float,
        });
    }

    None
}

/// Look up the type rule for a built-in call.
/// Returns `None` if the name is not built-in or argument types are invalid.
pub fn lookup_builtin_call(name: &str, arg_types: &[&ValueType]) -> Option<BuiltinCallRule> {
    match (name, arg_types) {
        // Conversions/constructors
        ("str", [ValueType::Str]) => Some(BuiltinCallRule::Identity),
        ("str", [ValueType::Int]) => Some(BuiltinCallRule::ExternalCall {
            func: BuiltinFn::StrFromInt,
            return_type: ValueType::Str,
        }),
        ("str", [ValueType::Float]) => Some(BuiltinCallRule::ExternalCall {
            func: BuiltinFn::StrFromFloat,
            return_type: ValueType::Str,
        }),
        ("str", [ValueType::Bool]) => Some(BuiltinCallRule::ExternalCall {
            func: BuiltinFn::StrFromBool,
            return_type: ValueType::Str,
        }),
        ("repr", [ValueType::Int]) => Some(BuiltinCallRule::ExternalCall {
            func: BuiltinFn::StrFromInt,
            return_type: ValueType::Str,
        }),
        ("repr", [ValueType::Float]) => Some(BuiltinCallRule::ExternalCall {
            func: BuiltinFn::StrFromFloat,
            return_type: ValueType::Str,
        }),
        ("repr", [ValueType::Bool]) => Some(BuiltinCallRule::ExternalCall {
            func: BuiltinFn::StrFromBool,
            return_type: ValueType::Str,
        }),

        ("bytes", [ValueType::Bytes]) => Some(BuiltinCallRule::Identity),
        ("bytes", [ValueType::Int]) => Some(BuiltinCallRule::ExternalCall {
            func: BuiltinFn::BytesFromInt,
            return_type: ValueType::Bytes,
        }),
        ("bytes", [ValueType::Str]) => Some(BuiltinCallRule::ExternalCall {
            func: BuiltinFn::BytesFromStr,
            return_type: ValueType::Bytes,
        }),

        ("bytearray", []) => Some(BuiltinCallRule::ExternalCall {
            func: BuiltinFn::ByteArrayEmpty,
            return_type: ValueType::ByteArray,
        }),
        ("bytearray", [ValueType::ByteArray]) => Some(BuiltinCallRule::Identity),
        ("bytearray", [ValueType::Int]) => Some(BuiltinCallRule::ExternalCall {
            func: BuiltinFn::ByteArrayFromInt,
            return_type: ValueType::ByteArray,
        }),
        ("bytearray", [ValueType::Bytes]) => Some(BuiltinCallRule::ExternalCall {
            func: BuiltinFn::ByteArrayFromBytes,
            return_type: ValueType::ByteArray,
        }),

        ("int", [ValueType::Int]) => Some(BuiltinCallRule::Identity),
        ("int", [ValueType::Float | ValueType::Bool]) => Some(BuiltinCallRule::PrimitiveCast {
            target_type: ValueType::Int,
        }),
        ("float", [ValueType::Float]) => Some(BuiltinCallRule::Identity),
        ("float", [ValueType::Int | ValueType::Bool]) => Some(BuiltinCallRule::PrimitiveCast {
            target_type: ValueType::Float,
        }),
        ("bool", [ValueType::Bool]) => Some(BuiltinCallRule::Identity),
        ("bool", [ValueType::Int | ValueType::Float]) => Some(BuiltinCallRule::PrimitiveCast {
            target_type: ValueType::Bool,
        }),

        // Built-in functions
        ("len", [ValueType::Str]) => Some(BuiltinCallRule::ExternalCall {
            func: BuiltinFn::StrLen,
            return_type: ValueType::Int,
        }),
        ("len", [ValueType::Bytes]) => Some(BuiltinCallRule::ExternalCall {
            func: BuiltinFn::BytesLen,
            return_type: ValueType::Int,
        }),
        ("len", [ValueType::ByteArray]) => Some(BuiltinCallRule::ExternalCall {
            func: BuiltinFn::ByteArrayLen,
            return_type: ValueType::Int,
        }),
        ("len", [ValueType::List(_)]) => Some(BuiltinCallRule::ExternalCall {
            func: BuiltinFn::ListLen,
            return_type: ValueType::Int,
        }),
        ("len", [ValueType::Tuple(elements)]) => {
            Some(BuiltinCallRule::ConstInt(elements.len() as i64))
        }

        ("abs", _) => numeric_unary_builtin(arg_types, BuiltinFn::AbsInt, BuiltinFn::AbsFloat),
        ("min", _) => numeric_variadic_builtin(arg_types, BuiltinFn::MinInt, BuiltinFn::MinFloat),
        ("max", _) => numeric_variadic_builtin(arg_types, BuiltinFn::MaxInt, BuiltinFn::MaxFloat),

        ("pow", [ValueType::Int, ValueType::Int]) => Some(BuiltinCallRule::ExternalCall {
            func: BuiltinFn::PowInt,
            return_type: ValueType::Int,
        }),
        ("pow", [ValueType::Float, ValueType::Float]) => Some(BuiltinCallRule::PowFloat),

        ("round", [ValueType::Float]) => Some(BuiltinCallRule::ExternalCall {
            func: BuiltinFn::RoundFloat,
            return_type: ValueType::Int,
        }),
        _ => None,
    }
}

/// Generate a descriptive error message for a built-in call
/// with invalid arity or argument types.
pub fn builtin_call_error_message(name: &str, arg_types: &[&ValueType], provided: usize) -> String {
    match name {
        "str" | "bytes" | "int" | "float" | "bool" => {
            if provided != 1 {
                format!("{}() expects exactly 1 argument, got {}", name, provided)
            } else {
                format!("{}() cannot convert `{}`", name, arg_types[0])
            }
        }
        "repr" => {
            if provided != 1 {
                format!("repr() expects exactly 1 argument, got {}", provided)
            } else {
                format!(
                    "repr() requires a class with `__repr__() -> str` or a numeric/bool value, got `{}`",
                    arg_types[0]
                )
            }
        }
        "bytearray" => {
            if provided > 1 {
                format!("bytearray() expects 0 or 1 arguments, got {}", provided)
            } else {
                format!("bytearray() cannot convert `{}`", arg_types[0])
            }
        }
        "len" => {
            if provided != 1 {
                format!("len() expects exactly 1 argument, got {}", provided)
            } else {
                format!(
                    "len() requires a `str`, `bytes`, `bytearray`, `list`, `tuple`, or a class with `__len__() -> int`, got `{}`",
                    arg_types[0]
                )
            }
        }
        "abs" => {
            if provided != 1 {
                format!("abs() expects exactly 1 argument, got {}", provided)
            } else {
                format!("abs() requires a numeric argument, got `{}`", arg_types[0])
            }
        }
        "round" => {
            if provided != 1 {
                format!("round() expects exactly 1 argument, got {}", provided)
            } else {
                format!(
                    "round() requires a `float` argument, got `{}`",
                    arg_types[0]
                )
            }
        }
        "pow" => {
            if provided != 2 {
                format!("pow() expects 2 arguments, got {}", provided)
            } else if arg_types[0] != arg_types[1] {
                format!(
                    "pow() arguments must have the same type: got `{}` and `{}`",
                    arg_types[0], arg_types[1]
                )
            } else {
                format!("pow() requires numeric arguments, got `{}`", arg_types[0])
            }
        }
        "min" | "max" => {
            if provided < 2 {
                format!("{}() expects at least 2 arguments, got {}", name, provided)
            } else if arg_types.iter().any(|ty| *ty != arg_types[0]) {
                format!(
                    "{}() arguments must have the same type: got `{}` and `{}`",
                    name,
                    arg_types[0],
                    arg_types
                        .iter()
                        .copied()
                        .find(|ty| *ty != arg_types[0])
                        .unwrap_or(arg_types[0])
                )
            } else {
                format!(
                    "{}() requires numeric arguments, got `{}`",
                    name, arg_types[0]
                )
            }
        }
        _ => unreachable!("not a built-in call: {}", name),
    }
}
