// Re-export binop type rules from the lowering module
pub use crate::tir::lower::expr::binops::{
    binop_type_error_message, lookup_binop, lookup_class_binop_magic, resolve_typed_binop,
    BinOpRule, ClassBinOpMagicRule, Coercion,
};

use crate::tir::{OrderedCmpOp, TypedCompare, ValueType};

/// Convert an ordered comparison operator + operand type into a fully-typed `TypedCompare`.
/// Must only be called for primitive types (Int, Float, Bool) that have direct LLVM compare instructions.
/// String/bytes comparisons should remain as ExternalCall.
pub fn resolve_typed_compare(op: OrderedCmpOp, operand_ty: &ValueType) -> TypedCompare {
    use OrderedCmpOp::*;
    use ValueType::*;

    match operand_ty {
        Int => match op {
            Eq => TypedCompare::IntEq,
            NotEq => TypedCompare::IntNotEq,
            Lt => TypedCompare::IntLt,
            LtEq => TypedCompare::IntLtEq,
            Gt => TypedCompare::IntGt,
            GtEq => TypedCompare::IntGtEq,
        },
        Float => match op {
            Eq => TypedCompare::FloatEq,
            NotEq => TypedCompare::FloatNotEq,
            Lt => TypedCompare::FloatLt,
            LtEq => TypedCompare::FloatLtEq,
            Gt => TypedCompare::FloatGt,
            GtEq => TypedCompare::FloatGtEq,
        },
        Bool => match op {
            Eq => TypedCompare::BoolEq,
            NotEq => TypedCompare::BoolNotEq,
            _ => panic!("ICE: bool only supports Eq/NotEq comparisons, got {:?}", op),
        },
        _ => panic!(
            "ICE: resolve_typed_compare called for non-primitive type: {:?}",
            operand_ty
        ),
    }
}
