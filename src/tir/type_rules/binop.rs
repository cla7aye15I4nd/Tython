use crate::ast::Type;
use crate::tir::{ArithBinOp, FloatArithOp, IntArithOp, RawBinOp, TypedBinOp};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClassBinOpMagicRule {
    pub left_method: &'static str,
    pub right_method: &'static str,
}

pub fn lookup_class_binop_magic(op: RawBinOp) -> Option<ClassBinOpMagicRule> {
    use crate::tir::BitwiseBinOp::*;
    use ArithBinOp::*;
    use RawBinOp::*;

    Some(match op {
        Arith(Add) => ClassBinOpMagicRule {
            left_method: "__add__",
            right_method: "__radd__",
        },
        Arith(Sub) => ClassBinOpMagicRule {
            left_method: "__sub__",
            right_method: "__rsub__",
        },
        Arith(Mul) => ClassBinOpMagicRule {
            left_method: "__mul__",
            right_method: "__rmul__",
        },
        Arith(Div) => ClassBinOpMagicRule {
            left_method: "__truediv__",
            right_method: "__rtruediv__",
        },
        Arith(FloorDiv) => ClassBinOpMagicRule {
            left_method: "__floordiv__",
            right_method: "__rfloordiv__",
        },
        Arith(Mod) => ClassBinOpMagicRule {
            left_method: "__mod__",
            right_method: "__rmod__",
        },
        Arith(Pow) => ClassBinOpMagicRule {
            left_method: "__pow__",
            right_method: "__rpow__",
        },
        Bitwise(BitAnd) => ClassBinOpMagicRule {
            left_method: "__and__",
            right_method: "__rand__",
        },
        Bitwise(BitOr) => ClassBinOpMagicRule {
            left_method: "__or__",
            right_method: "__ror__",
        },
        Bitwise(BitXor) => ClassBinOpMagicRule {
            left_method: "__xor__",
            right_method: "__rxor__",
        },
        Bitwise(LShift) => ClassBinOpMagicRule {
            left_method: "__lshift__",
            right_method: "__rlshift__",
        },
        Bitwise(RShift) => ClassBinOpMagicRule {
            left_method: "__rshift__",
            right_method: "__rrshift__",
        },
    })
}

/// Describes what coercion to apply to an operand before the operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Coercion {
    /// No coercion needed; use the operand as-is.
    None,
    /// Cast the operand to Float.
    ToFloat,
}

/// Result of looking up a valid (RawBinOp, left_type, right_type) combination.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BinOpRule {
    pub left_coercion: Coercion,
    pub right_coercion: Coercion,
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
fn sequence_binop_rule(op: RawBinOp, left: &Type, right: &Type) -> Option<BinOpRule> {
    use Type::*;
    match op {
        RawBinOp::Arith(ArithBinOp::Add) => match (left, right) {
            (Str, Str) => Some(BinOpRule::same(Str)),
            (Bytes, Bytes) => Some(BinOpRule::same(Bytes)),
            (ByteArray, ByteArray) => Some(BinOpRule::same(ByteArray)),
            _ => None,
        },
        RawBinOp::Arith(ArithBinOp::Mul) => match (left, right) {
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
pub fn lookup_binop(op: RawBinOp, left: &Type, right: &Type) -> Option<BinOpRule> {
    use Type::*;

    // Sequence operations (concat, repeat)
    if let Some(rule) = sequence_binop_rule(op, left, right) {
        return Some(rule);
    }

    match op {
        RawBinOp::Bitwise(_) => match (left, right) {
            (Int, Int) => Some(BinOpRule::same(Int)),
            _ => None,
        },
        // True division: always produces Float (even Int / Int)
        RawBinOp::Arith(ArithBinOp::Div) => match (left, right) {
            (Int, Int) => Some(BinOpRule::promote_both_to_float()),
            _ => standard_numeric_rule(left, right),
        },
        // All other arithmetic: standard numeric rules
        RawBinOp::Arith(_) => standard_numeric_rule(left, right),
    }
}

/// Generate a descriptive error message for an invalid BinOp type combination.
pub fn binop_type_error_message(op: RawBinOp, left: &Type, right: &Type) -> String {
    match op {
        RawBinOp::Bitwise(_) => {
            format!(
                "bitwise operator `{}` requires `int` operands, got `{}` and `{}`",
                op, left, right
            )
        }
        RawBinOp::Arith(_) => {
            format!(
                "operator `{}` requires numeric operands, got `{}` and `{}`",
                op, left, right
            )
        }
    }
}

// ── Raw → Typed resolution ──────────────────────────────────────────

/// Convert a raw binary operator + result type into a fully-typed `TypedBinOp`.
/// Must only be called after `lookup_binop` has validated the combination.
pub fn resolve_typed_binop(raw_op: RawBinOp, result_ty: &Type) -> TypedBinOp {
    use Type::*;

    // Sequence operations are handled by the lowering layer (emitted as ExternalCall).
    // This function must only be called for arithmetic/bitwise operations.

    match raw_op {
        RawBinOp::Bitwise(bw) => TypedBinOp::Bitwise(bw),
        RawBinOp::Arith(arith) => {
            if *result_ty == Float {
                TypedBinOp::FloatArith(match arith {
                    ArithBinOp::Add => FloatArithOp::Add,
                    ArithBinOp::Sub => FloatArithOp::Sub,
                    ArithBinOp::Mul => FloatArithOp::Mul,
                    ArithBinOp::Div => FloatArithOp::Div,
                    ArithBinOp::FloorDiv => FloatArithOp::FloorDiv,
                    ArithBinOp::Mod => FloatArithOp::Mod,
                    ArithBinOp::Pow => FloatArithOp::Pow,
                })
            } else {
                TypedBinOp::IntArith(match arith {
                    ArithBinOp::Add => IntArithOp::Add,
                    ArithBinOp::Sub => IntArithOp::Sub,
                    ArithBinOp::Mul => IntArithOp::Mul,
                    ArithBinOp::Div => unreachable!("ICE: int result for true division"),
                    ArithBinOp::FloorDiv => IntArithOp::FloorDiv,
                    ArithBinOp::Mod => IntArithOp::Mod,
                    ArithBinOp::Pow => IntArithOp::Pow,
                })
            }
        }
    }
}
