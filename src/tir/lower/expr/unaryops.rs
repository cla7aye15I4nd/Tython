use crate::ast::Type;
use crate::tir::{UnaryOpKind, ValueType};

/// Look up the class magic method for a unary operation.
/// Returns `(method_name, expected_return_type, negate_result)`.
pub fn class_unary_magic(op: UnaryOpKind) -> (&'static str, Option<ValueType>, bool) {
    use UnaryOpKind::*;
    match op {
        Neg => ("__neg__", None, false),
        Pos => ("__pos__", None, false),
        BitNot => ("__invert__", None, false),
        Not => ("__bool__", Some(ValueType::Bool), true),
    }
}

/// Check whether a unary operation is valid for the given operand type.
/// Returns `true` if the (op, operand) combination is valid.
pub fn is_valid_unaryop(op: UnaryOpKind, operand: &Type) -> bool {
    use Type::*;
    use UnaryOpKind::*;

    matches!(
        (op, operand),
        (Neg | Pos, Int)
            | (Neg | Pos, Float)
            | (Not, Int)
            | (Not, Float)
            | (Not, Bool)
            | (Not, Str)
            | (Not, Bytes)
            | (Not, ByteArray)
            | (Not, List(_))
            | (Not, Dict(_, _))
            | (Not, Set(_))
            | (Not, Tuple(_))
            | (BitNot, Int)
    )
}
