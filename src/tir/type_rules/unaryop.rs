use crate::ast::Type;
use crate::tir::UnaryOpKind;

/// Result of looking up a valid (UnaryOpKind, operand_type) combination.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnaryOpRule {
    pub result_type: Type,
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
