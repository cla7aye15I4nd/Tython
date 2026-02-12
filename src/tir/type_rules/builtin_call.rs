use super::builtin_class::lookup_builtin_dunder;
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

#[inline]
fn external_call(func: BuiltinFn, return_type: ValueType) -> BuiltinCallRule {
    BuiltinCallRule::ExternalCall { func, return_type }
}

#[inline]
fn fold_external_call(func: BuiltinFn, return_type: ValueType) -> BuiltinCallRule {
    BuiltinCallRule::FoldExternalCall { func, return_type }
}

#[inline]
fn class_magic(
    method_names: &'static [&'static str],
    return_type: Option<ValueType>,
) -> BuiltinCallRule {
    BuiltinCallRule::ClassMagic {
        method_names,
        return_type,
    }
}

fn sum_builtin(arg_types: &[&ValueType]) -> Option<BuiltinCallRule> {
    match arg_types {
        [ValueType::List(inner)] => match inner.as_ref() {
            ValueType::Int | ValueType::Bool => {
                Some(external_call(BuiltinFn::SumInt, ValueType::Int))
            }
            ValueType::Float => Some(external_call(BuiltinFn::SumFloat, ValueType::Float)),
            _ => None,
        },
        [ValueType::List(inner), ValueType::Int] => match inner.as_ref() {
            ValueType::Int | ValueType::Bool => {
                Some(external_call(BuiltinFn::SumIntStart, ValueType::Int))
            }
            _ => None,
        },
        [ValueType::List(inner), ValueType::Float] => match inner.as_ref() {
            ValueType::Float => Some(external_call(BuiltinFn::SumFloatStart, ValueType::Float)),
            _ => None,
        },
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
        return Some(fold_external_call(int_fn, ValueType::Int));
    }

    if arg_types.iter().all(|ty| **ty == ValueType::Float) {
        return Some(fold_external_call(float_fn, ValueType::Float));
    }

    None
}

/// Look up the type rule for a built-in call.
/// Returns `None` if the name is not built-in or argument types are invalid.
pub fn lookup_builtin_call(name: &str, arg_types: &[&ValueType]) -> Option<BuiltinCallRule> {
    match (name, arg_types) {
        // Conversions/constructors — str()/repr() via unified __str__ dunder
        ("str" | "repr", [ValueType::Class(_)]) => {
            if name == "repr" {
                Some(class_magic(&["__repr__"], Some(ValueType::Str)))
            } else {
                Some(class_magic(&["__str__", "__repr__"], Some(ValueType::Str)))
            }
        }
        ("str" | "repr", [ty]) => lookup_builtin_dunder(ty, "__str__"),

        ("bytes", [ValueType::Bytes]) => Some(BuiltinCallRule::Identity),
        ("bytes", [ValueType::Int]) => {
            Some(external_call(BuiltinFn::BytesFromInt, ValueType::Bytes))
        }

        ("bytearray", []) => Some(external_call(
            BuiltinFn::ByteArrayEmpty,
            ValueType::ByteArray,
        )),
        ("bytearray", [ValueType::ByteArray]) => Some(BuiltinCallRule::Identity),
        ("bytearray", [ValueType::Int]) => Some(external_call(
            BuiltinFn::ByteArrayFromInt,
            ValueType::ByteArray,
        )),
        ("bytearray", [ValueType::Bytes]) => Some(external_call(
            BuiltinFn::ByteArrayFromBytes,
            ValueType::ByteArray,
        )),

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

        // Built-in functions — len() via unified __len__ dunder
        ("len", [ValueType::Tuple(elements)]) => {
            Some(BuiltinCallRule::ConstInt(elements.len() as i64))
        }
        ("len", [ValueType::Class(_)]) => Some(class_magic(&["__len__"], Some(ValueType::Int))),
        ("len", [ty]) => lookup_builtin_dunder(ty, "__len__"),

        ("abs", _) => numeric_unary_builtin(arg_types, BuiltinFn::AbsInt, BuiltinFn::AbsFloat),
        ("min", _) => numeric_variadic_builtin(arg_types, BuiltinFn::MinInt, BuiltinFn::MinFloat),
        ("max", _) => numeric_variadic_builtin(arg_types, BuiltinFn::MaxInt, BuiltinFn::MaxFloat),

        ("pow", [ValueType::Int, ValueType::Int]) => {
            Some(external_call(BuiltinFn::PowInt, ValueType::Int))
        }
        ("pow", [ValueType::Float, ValueType::Float]) => Some(BuiltinCallRule::PowFloat),

        ("round", [ValueType::Float]) => Some(external_call(BuiltinFn::RoundFloat, ValueType::Int)),

        ("sum", _) => sum_builtin(arg_types),

        ("all", [ValueType::List(_)]) => Some(external_call(BuiltinFn::AllList, ValueType::Bool)),

        ("any", [ValueType::List(_)]) => Some(external_call(BuiltinFn::AnyList, ValueType::Bool)),

        ("sorted", [ValueType::List(inner)]) => {
            let sorted_fn = match inner.as_ref() {
                ValueType::Int | ValueType::Bool => BuiltinFn::SortedInt,
                ValueType::Float => BuiltinFn::SortedFloat,
                ValueType::Str => BuiltinFn::SortedStr,
                ValueType::Bytes => BuiltinFn::SortedBytes,
                ValueType::ByteArray => BuiltinFn::SortedByteArray,
                _ => return None,
            };
            Some(external_call(sorted_fn, ValueType::List(inner.clone())))
        }

        // Class dunder-method dispatch for iter/next
        ("iter", [ValueType::Class(_)]) => Some(class_magic(&["__iter__"], None)),
        ("next", [ValueType::Class(_)]) => Some(class_magic(&["__next__"], None)),

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
                format!(
                    "sorted() requires a list whose elements support ordering (`__lt__`), got `{}`",
                    arg_types[0]
                )
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
