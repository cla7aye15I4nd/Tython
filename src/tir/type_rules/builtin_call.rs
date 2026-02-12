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
    /// Dispatch to a dunder method on a user-defined class.
    ClassMagic {
        method_names: &'static [&'static str],
        /// `Some(ty)` = validate the method returns exactly `ty`.
        /// `None` = infer the return type from the method declaration.
        return_type: Option<ValueType>,
    },
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
            | "sum"
            | "all"
            | "any"
            | "sorted"
            | "iter"
            | "next"
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
        ("repr", [ValueType::Str]) => Some(BuiltinCallRule::Identity),

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

        ("sum", [ValueType::List(inner)]) => match inner.as_ref() {
            ValueType::Int | ValueType::Bool => Some(BuiltinCallRule::ExternalCall {
                func: BuiltinFn::SumInt,
                return_type: ValueType::Int,
            }),
            ValueType::Float => Some(BuiltinCallRule::ExternalCall {
                func: BuiltinFn::SumFloat,
                return_type: ValueType::Float,
            }),
            _ => None,
        },
        ("sum", [ValueType::List(inner), ValueType::Int]) => match inner.as_ref() {
            ValueType::Int | ValueType::Bool => Some(BuiltinCallRule::ExternalCall {
                func: BuiltinFn::SumIntStart,
                return_type: ValueType::Int,
            }),
            _ => None,
        },
        ("sum", [ValueType::List(inner), ValueType::Float]) => match inner.as_ref() {
            ValueType::Float => Some(BuiltinCallRule::ExternalCall {
                func: BuiltinFn::SumFloatStart,
                return_type: ValueType::Float,
            }),
            _ => None,
        },

        ("all", [ValueType::List(_)]) => Some(BuiltinCallRule::ExternalCall {
            func: BuiltinFn::AllList,
            return_type: ValueType::Bool,
        }),

        ("any", [ValueType::List(_)]) => Some(BuiltinCallRule::ExternalCall {
            func: BuiltinFn::AnyList,
            return_type: ValueType::Bool,
        }),

        // Class dunder-method dispatch: when a builtin is called on a user-defined class,
        // resolve to the corresponding magic method(s).
        ("str", [ValueType::Class(_)]) => Some(BuiltinCallRule::ClassMagic {
            method_names: &["__str__", "__repr__"],
            return_type: Some(ValueType::Str),
        }),
        ("repr", [ValueType::Class(_)]) => Some(BuiltinCallRule::ClassMagic {
            method_names: &["__repr__"],
            return_type: Some(ValueType::Str),
        }),
        ("len", [ValueType::Class(_)]) => Some(BuiltinCallRule::ClassMagic {
            method_names: &["__len__"],
            return_type: Some(ValueType::Int),
        }),
        ("iter", [ValueType::Class(_)]) => Some(BuiltinCallRule::ClassMagic {
            method_names: &["__iter__"],
            return_type: None,
        }),
        ("next", [ValueType::Class(_)]) => Some(BuiltinCallRule::ClassMagic {
            method_names: &["__next__"],
            return_type: None,
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
                    "repr() requires a class with `__repr__() -> str` or a str/numeric/bool value, got `{}`",
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
        "sum" => {
            if provided == 0 || provided > 2 {
                format!("sum() expects 1 or 2 arguments, got {}", provided)
            } else {
                format!(
                    "sum() requires a list of numbers and optional start value, got `{}`",
                    arg_types[0]
                )
            }
        }
        "all" | "any" => {
            if provided != 1 {
                format!("{}() expects exactly 1 argument, got {}", name, provided)
            } else {
                format!("{}() requires a list, got `{}`", name, arg_types[0])
            }
        }
        "sorted" => {
            if provided != 1 {
                format!("sorted() expects exactly 1 argument, got {}", provided)
            } else {
                format!("sorted() requires a list, got `{}`", arg_types[0])
            }
        }
        "iter" => {
            if provided != 1 {
                format!("iter() expects 1 argument, got {}", provided)
            } else {
                format!(
                    "iter() argument must be a class with `__iter__`, got `{}`",
                    arg_types[0]
                )
            }
        }
        "next" => {
            if provided != 1 {
                format!("next() expects 1 argument, got {}", provided)
            } else {
                format!(
                    "next() argument must be a class with `__next__`, got `{}`",
                    arg_types[0]
                )
            }
        }
        _ => unreachable!("not a built-in call: {}", name),
    }
}
