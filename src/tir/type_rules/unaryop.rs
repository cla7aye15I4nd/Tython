use crate::ast::Type;
use crate::tir::{TypedUnaryOp, UnaryOpKind, ValueType};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassUnaryOpMagicRule {
    pub method_name: &'static str,
    pub expected_return_type: Option<ValueType>,
    /// If true, apply logical-not to the method result.
    pub negate_result: bool,
}

pub fn lookup_class_unary_magic(op: UnaryOpKind) -> Option<ClassUnaryOpMagicRule> {
    use UnaryOpKind::*;
    Some(match op {
        Neg => ClassUnaryOpMagicRule {
            method_name: "__neg__",
            expected_return_type: None,
            negate_result: false,
        },
        Pos => ClassUnaryOpMagicRule {
            method_name: "__pos__",
            expected_return_type: None,
            negate_result: false,
        },
        BitNot => ClassUnaryOpMagicRule {
            method_name: "__invert__",
            expected_return_type: None,
            negate_result: false,
        },
        Not => ClassUnaryOpMagicRule {
            method_name: "__bool__",
            expected_return_type: Some(ValueType::Bool),
            negate_result: true,
        },
    })
}

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
        (Not, Str) => Some(UnaryOpRule { result_type: Bool }),
        (Not, Bytes) => Some(UnaryOpRule { result_type: Bool }),
        (Not, ByteArray) => Some(UnaryOpRule { result_type: Bool }),
        (Not, List(_)) => Some(UnaryOpRule { result_type: Bool }),
        (Not, Dict(_, _)) => Some(UnaryOpRule { result_type: Bool }),
        (Not, Set(_)) => Some(UnaryOpRule { result_type: Bool }),
        (Not, Tuple(_)) => Some(UnaryOpRule { result_type: Bool }),

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

/// Resolve a raw unary operation to a fully-typed operation.
/// This must be called after `lookup_unaryop()` succeeds.
/// Returns `None` for `Pos` (unary +), which is a no-op.
pub fn resolve_typed_unaryop(op: UnaryOpKind, operand: &Type) -> Option<TypedUnaryOp> {
    use Type::*;
    use UnaryOpKind::*;

    match (op, operand) {
        (Neg, Int) => Some(TypedUnaryOp::IntNeg),
        (Neg, Float) => Some(TypedUnaryOp::FloatNeg),
        (Pos, _) => None, // Unary + is a no-op, handled during lowering
        (Not, _) => Some(TypedUnaryOp::Not),
        (BitNot, Int) => Some(TypedUnaryOp::BitNot),
        _ => panic!(
            "ICE: resolve_typed_unaryop called with invalid op/type combination: {:?}/{:?}",
            op, operand
        ),
    }
}
