use super::{ArithBinOp, TypedBinOp, UnaryOpKind};
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

/// Look up the type rule for a binary operation.
/// Returns `None` if the (op, left, right) combination is invalid.
pub fn lookup_binop(op: TypedBinOp, left: &Type, right: &Type) -> Option<BinOpRule> {
    use Type::*;

    match op {
        // ── Bitwise: Int × Int → Int only ────────────────────────────
        TypedBinOp::Bitwise(_) => match (left, right) {
            (Int, Int) => Some(BinOpRule::same(Int)),
            _ => None,
        },

        // ── Arithmetic ───────────────────────────────────────────────
        TypedBinOp::Arith(arith) => match arith {
            // True division: always produces Float
            ArithBinOp::Div => match (left, right) {
                (Int, Int) => Some(BinOpRule::promote_both_to_float()),
                (Float, Float) => Some(BinOpRule::same(Float)),
                (Int, Float) => Some(BinOpRule::promote_left_to_float()),
                (Float, Int) => Some(BinOpRule::promote_right_to_float()),
                _ => None,
            },
            // Other arithmetic: same type → same type, mixed → Float
            ArithBinOp::Add
            | ArithBinOp::Sub
            | ArithBinOp::Mul
            | ArithBinOp::FloorDiv
            | ArithBinOp::Mod
            | ArithBinOp::Pow => match (left, right) {
                (Int, Int) => Some(BinOpRule::same(Int)),
                (Float, Float) => Some(BinOpRule::same(Float)),
                (Int, Float) => Some(BinOpRule::promote_left_to_float()),
                (Float, Int) => Some(BinOpRule::promote_right_to_float()),
                _ => None,
            },
        },
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
