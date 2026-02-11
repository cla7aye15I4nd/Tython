use super::builtin::BuiltinFn;
use super::{ArithBinOp, TypedBinOp, UnaryOpKind, ValueType};
use crate::ast::Type;

/// Describes what coercion to apply to an operand before the operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Coercion {
    /// No coercion needed; use the operand as-is.
    None,
    /// Cast the operand to Float.
    ToFloat,
}

/// Result of looking up a valid (TypedBinOp, left_type, right_type) combination.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BinOpRule {
    pub left_coercion: Coercion,
    pub right_coercion: Coercion,
    pub result_type: Type,
}

/// Result of looking up a valid (UnaryOpKind, operand_type) combination.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnaryOpRule {
    pub result_type: Type,
}

impl BinOpRule {
    fn same(ty: Type) -> Self {
        Self {
            left_coercion: Coercion::None,
            right_coercion: Coercion::None,
            result_type: ty,
        }
    }

    fn promote_both_to_float() -> Self {
        Self {
            left_coercion: Coercion::ToFloat,
            right_coercion: Coercion::ToFloat,
            result_type: Type::Float,
        }
    }

    fn promote_left_to_float() -> Self {
        Self {
            left_coercion: Coercion::ToFloat,
            right_coercion: Coercion::None,
            result_type: Type::Float,
        }
    }

    fn promote_right_to_float() -> Self {
        Self {
            left_coercion: Coercion::None,
            right_coercion: Coercion::ToFloat,
            result_type: Type::Float,
        }
    }
}

/// Standard numeric type rule: same type preserves type, mixed int/float promotes to float.
fn standard_numeric_rule(left: &Type, right: &Type) -> Option<BinOpRule> {
    use Type::*;
    match (left, right) {
        (Int, Int) => Some(BinOpRule::same(Int)),
        (Float, Float) => Some(BinOpRule::same(Float)),
        (Int, Float) => Some(BinOpRule::promote_left_to_float()),
        (Float, Int) => Some(BinOpRule::promote_right_to_float()),
        _ => None,
    }
}

/// Check for sequence-type binary operations (concat, repeat).
fn sequence_binop_rule(op: TypedBinOp, left: &Type, right: &Type) -> Option<BinOpRule> {
    use Type::*;
    match op {
        TypedBinOp::Arith(ArithBinOp::Add) => match (left, right) {
            (Str, Str) => Some(BinOpRule::same(Str)),
            (Bytes, Bytes) => Some(BinOpRule::same(Bytes)),
            (ByteArray, ByteArray) => Some(BinOpRule::same(ByteArray)),
            _ => None,
        },
        TypedBinOp::Arith(ArithBinOp::Mul) => match (left, right) {
            (Str, Int) | (Int, Str) => Some(BinOpRule::same(Str)),
            (Bytes, Int) | (Int, Bytes) => Some(BinOpRule::same(Bytes)),
            (ByteArray, Int) | (Int, ByteArray) => Some(BinOpRule::same(ByteArray)),
            _ => None,
        },
        _ => None,
    }
}

/// Look up the type rule for a binary operation.
/// Returns `None` if the (op, left, right) combination is invalid.
pub fn lookup_binop(op: TypedBinOp, left: &Type, right: &Type) -> Option<BinOpRule> {
    use Type::*;

    // Sequence operations (concat, repeat)
    if let Some(rule) = sequence_binop_rule(op, left, right) {
        return Some(rule);
    }

    match op {
        TypedBinOp::Bitwise(_) => match (left, right) {
            (Int, Int) => Some(BinOpRule::same(Int)),
            _ => None,
        },
        // True division: always produces Float (even Int / Int)
        TypedBinOp::Arith(ArithBinOp::Div) => match (left, right) {
            (Int, Int) => Some(BinOpRule::promote_both_to_float()),
            _ => standard_numeric_rule(left, right),
        },
        // All other arithmetic: standard numeric rules
        TypedBinOp::Arith(_) => standard_numeric_rule(left, right),
    }
}

/// Look up the type rule for a unary operation.
/// Returns `None` if the (op, operand) combination is invalid.
pub fn lookup_unaryop(op: UnaryOpKind, operand: &Type) -> Option<UnaryOpRule> {
    use Type::*;
    use UnaryOpKind::*;

    match (op, operand) {
        // ── Negation / Positive: numeric types, preserves type ───────
        (Neg | Pos, Int) => Some(UnaryOpRule { result_type: Int }),
        (Neg | Pos, Float) => Some(UnaryOpRule { result_type: Float }),

        // ── Logical not: any value type → Bool ───────────────────────
        (Not, Int) => Some(UnaryOpRule { result_type: Bool }),
        (Not, Float) => Some(UnaryOpRule { result_type: Bool }),
        (Not, Bool) => Some(UnaryOpRule { result_type: Bool }),

        // ── Bitwise not: Int only → Int ──────────────────────────────
        (BitNot, Int) => Some(UnaryOpRule { result_type: Int }),

        // ── Everything else: invalid ─────────────────────────────────
        _ => None,
    }
}

/// Generate a descriptive error message for an invalid BinOp type combination.
pub fn binop_type_error_message(op: TypedBinOp, left: &Type, right: &Type) -> String {
    match op {
        TypedBinOp::Bitwise(_) => {
            format!(
                "bitwise operator `{}` requires `int` operands, got `{}` and `{}`",
                op, left, right
            )
        }
        TypedBinOp::Arith(_) => {
            format!(
                "operator `{}` requires numeric operands, got `{}` and `{}`",
                op, left, right
            )
        }
    }
}

/// Generate a descriptive error message for an invalid UnaryOp type combination.
pub fn unaryop_type_error_message(op: UnaryOpKind, operand: &Type) -> String {
    use UnaryOpKind::*;

    match op {
        Neg => format!("unary `-` requires a numeric operand, got `{}`", operand),
        Pos => format!("unary `+` requires a numeric operand, got `{}`", operand),
        Not => format!("unary `not` is not supported for `{}`", operand),
        BitNot => format!("bitwise `~` requires an `int` operand, got `{}`", operand),
    }
}

// ── Built-in function type rules ────────────────────────────────────

/// Result of resolving a built-in function call to its type-checked form.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuiltinCallRule {
    /// Resolves to a runtime function call.
    ExternalCall {
        func: BuiltinFn,
        return_type: ValueType,
    },
    /// `pow(float, float)` lowers to a `BinOp(**)` instead of a runtime call.
    PowFloat,
}

/// Return the expected arity of a built-in numeric function,
/// or `None` if the name is not a built-in.
pub fn builtin_fn_arity(name: &str) -> Option<usize> {
    match name {
        "abs" | "round" | "len" => Some(1),
        "pow" | "min" | "max" => Some(2),
        _ => None,
    }
}

/// Look up the type rule for a built-in function call.
/// Returns `None` if the argument types are invalid for the function.
/// Caller should check arity with [`builtin_fn_arity`] first.
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

/// Resolve a binary numeric builtin (like min/max) that has int and float variants.
fn numeric_binary_builtin(
    arg_types: &[&ValueType],
    int_fn: BuiltinFn,
    float_fn: BuiltinFn,
) -> Option<BuiltinCallRule> {
    match arg_types {
        [ValueType::Int, ValueType::Int] => Some(BuiltinCallRule::ExternalCall {
            func: int_fn,
            return_type: ValueType::Int,
        }),
        [ValueType::Float, ValueType::Float] => Some(BuiltinCallRule::ExternalCall {
            func: float_fn,
            return_type: ValueType::Float,
        }),
        _ => None,
    }
}

pub fn lookup_builtin_fn(name: &str, arg_types: &[&ValueType]) -> Option<BuiltinCallRule> {
    match name {
        "len" => match arg_types {
            [ValueType::Str] => Some(BuiltinCallRule::ExternalCall {
                func: BuiltinFn::StrLen,
                return_type: ValueType::Int,
            }),
            [ValueType::Bytes] => Some(BuiltinCallRule::ExternalCall {
                func: BuiltinFn::BytesLen,
                return_type: ValueType::Int,
            }),
            [ValueType::ByteArray] => Some(BuiltinCallRule::ExternalCall {
                func: BuiltinFn::ByteArrayLen,
                return_type: ValueType::Int,
            }),
            _ => None,
        },
        "abs" => numeric_unary_builtin(arg_types, BuiltinFn::AbsInt, BuiltinFn::AbsFloat),
        "min" => numeric_binary_builtin(arg_types, BuiltinFn::MinInt, BuiltinFn::MinFloat),
        "max" => numeric_binary_builtin(arg_types, BuiltinFn::MaxInt, BuiltinFn::MaxFloat),
        "pow" => match arg_types {
            [ValueType::Int, ValueType::Int] => Some(BuiltinCallRule::ExternalCall {
                func: BuiltinFn::PowInt,
                return_type: ValueType::Int,
            }),
            [ValueType::Float, ValueType::Float] => Some(BuiltinCallRule::PowFloat),
            _ => None,
        },
        "round" => match arg_types {
            [ValueType::Float] => Some(BuiltinCallRule::ExternalCall {
                func: BuiltinFn::RoundFloat,
                return_type: ValueType::Int,
            }),
            _ => None,
        },
        _ => None,
    }
}

/// Generate a descriptive error message for a built-in function call
/// that has correct arity but invalid argument types.
pub fn builtin_fn_type_error_message(name: &str, arg_types: &[&ValueType]) -> String {
    match name {
        "len" => format!(
            "len() requires a `str`, `bytes`, or `bytearray` argument, got `{}`",
            arg_types[0]
        ),
        "abs" => format!("abs() requires a numeric argument, got `{}`", arg_types[0]),
        "round" => format!(
            "round() requires a `float` argument, got `{}`",
            arg_types[0]
        ),
        "pow" | "min" | "max" => {
            if arg_types[0] != arg_types[1] {
                format!(
                    "{}() arguments must have the same type: got `{}` and `{}`",
                    name, arg_types[0], arg_types[1]
                )
            } else {
                format!(
                    "{}() requires numeric arguments, got `{}`",
                    name, arg_types[0]
                )
            }
        }
        _ => unreachable!("not a built-in function: {}", name),
    }
}
