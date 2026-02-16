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
