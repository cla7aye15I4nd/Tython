use anyhow::Result;

use crate::ast::Type;
use crate::tir::{
    builtin, ArithBinOp, BitwiseBinOp, CastKind, FloatArithOp, IntArithOp, RawBinOp, TirExpr,
    TirExprKind, TypedBinOp, ValueType,
};

use crate::tir::lower::Lowering;

// ── Type Rules for Binary Operations ────────────────────────────────

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClassBinOpMagicRule {
    pub left_method: &'static str,
    pub right_method: &'static str,
}

pub fn lookup_class_binop_magic(op: RawBinOp) -> Option<ClassBinOpMagicRule> {
    use ArithBinOp::*;
    use BitwiseBinOp::*;
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

/// Look up the type rule for a binary operation.
/// Returns `None` if the (op, left, right) combination is invalid.
pub fn lookup_binop(op: RawBinOp, left: &Type, right: &Type) -> Option<BinOpRule> {
    use Type::*;

    // Helper to create rules
    let same = |ty| BinOpRule {
        left_coercion: Coercion::None,
        right_coercion: Coercion::None,
        result_type: ty,
    };
    let both_to_float = || BinOpRule {
        left_coercion: Coercion::ToFloat,
        right_coercion: Coercion::ToFloat,
        result_type: Float,
    };
    let left_to_float = || BinOpRule {
        left_coercion: Coercion::ToFloat,
        right_coercion: Coercion::None,
        result_type: Float,
    };
    let right_to_float = || BinOpRule {
        left_coercion: Coercion::None,
        right_coercion: Coercion::ToFloat,
        result_type: Float,
    };

    // Sequence operations (concat, repeat)
    match op {
        RawBinOp::Arith(ArithBinOp::Add) => match (left, right) {
            (Str, Str) => return Some(same(Str)),
            (Bytes, Bytes) => return Some(same(Bytes)),
            (ByteArray, ByteArray) => return Some(same(ByteArray)),
            (List(a), List(b)) if a == b => return Some(same(List(a.clone()))),
            _ => {}
        },
        RawBinOp::Arith(ArithBinOp::Mul) => match (left, right) {
            (Str, Int) | (Int, Str) => return Some(same(Str)),
            (Bytes, Int) | (Int, Bytes) => return Some(same(Bytes)),
            (ByteArray, Int) | (Int, ByteArray) => return Some(same(ByteArray)),
            (List(inner), Int) | (Int, List(inner)) => return Some(same(List(inner.clone()))),
            _ => {}
        },
        _ => {}
    }

    // Arithmetic/bitwise operations
    match op {
        RawBinOp::Bitwise(_) => match (left, right) {
            (Int, Int) => Some(same(Int)),
            _ => None,
        },
        // True division: always produces Float (even Int / Int)
        RawBinOp::Arith(ArithBinOp::Div) => match (left, right) {
            (Int, Int) => Some(both_to_float()),
            (Float, Float) => Some(same(Float)),
            (Int, Float) => Some(left_to_float()),
            (Float, Int) => Some(right_to_float()),
            _ => None,
        },
        // All other arithmetic: standard numeric rules
        RawBinOp::Arith(_) => match (left, right) {
            (Int, Int) => Some(same(Int)),
            (Float, Float) => Some(same(Float)),
            (Int, Float) => Some(left_to_float()),
            (Float, Int) => Some(right_to_float()),
            _ => None,
        },
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

/// Convert a raw binary operator + result type into a fully-typed `TypedBinOp`.
/// Must only be called after `lookup_binop` has validated the combination.
pub fn resolve_typed_binop(raw_op: RawBinOp, result_ty: &Type) -> TypedBinOp {
    use Type::*;

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

// ── Binary Operation Lowering ────────────────────────────────────────

impl Lowering {
    /// Resolve a binary operation into a TIR expression.
    /// Sequence operations (concat, repeat) become `ExternalCall`;
    /// arithmetic/bitwise operations become `BinOp`.
    pub(in crate::tir::lower) fn resolve_binop(
        &mut self,
        line: usize,
        raw_op: RawBinOp,
        left: TirExpr,
        right: TirExpr,
    ) -> Result<TirExpr> {
        if let Some(class_expr) =
            self.try_lower_class_binop_magic(line, raw_op, left.clone(), right.clone())?
        {
            return Ok(class_expr);
        }

        let left_ast = left.ty.to_type();
        let right_ast = right.ty.to_type();
        let rule = lookup_binop(raw_op, &left_ast, &right_ast).ok_or_else(|| {
            self.type_error(
                line,
                binop_type_error_message(raw_op, &left_ast, &right_ast),
            )
        })?;

        let result_vty = self.value_type_from_type(&rule.result_type);

        // Sequence operations → ExternalCall
        if let Some(func) = Self::resolve_seq_binop(raw_op, &result_vty) {
            let args = if matches!(raw_op, RawBinOp::Arith(ArithBinOp::Mul))
                && left.ty == ValueType::Int
            {
                // Repeat with int on left: normalize to (seq, int)
                vec![right, left]
            } else {
                vec![left, right]
            };
            return Ok(TirExpr {
                kind: TirExprKind::ExternalCall { func, args },
                ty: result_vty,
            });
        }

        // Arithmetic/bitwise → BinOp
        let typed_op = resolve_typed_binop(raw_op, &rule.result_type);
        let final_left = Self::apply_coercion(left, rule.left_coercion);
        let final_right = Self::apply_coercion(right, rule.right_coercion);

        // Construct typed operation variant based on TypedBinOp
        let kind = match typed_op {
            crate::tir::TypedBinOp::IntArith(crate::tir::IntArithOp::Add) => {
                TirExprKind::IntAdd(Box::new(final_left), Box::new(final_right))
            }
            crate::tir::TypedBinOp::IntArith(crate::tir::IntArithOp::Sub) => {
                TirExprKind::IntSub(Box::new(final_left), Box::new(final_right))
            }
            crate::tir::TypedBinOp::IntArith(crate::tir::IntArithOp::Mul) => {
                TirExprKind::IntMul(Box::new(final_left), Box::new(final_right))
            }
            crate::tir::TypedBinOp::IntArith(crate::tir::IntArithOp::FloorDiv) => {
                TirExprKind::IntFloorDiv(Box::new(final_left), Box::new(final_right))
            }
            crate::tir::TypedBinOp::IntArith(crate::tir::IntArithOp::Mod) => {
                TirExprKind::IntMod(Box::new(final_left), Box::new(final_right))
            }
            crate::tir::TypedBinOp::IntArith(crate::tir::IntArithOp::Pow) => {
                TirExprKind::IntPow(Box::new(final_left), Box::new(final_right))
            }
            crate::tir::TypedBinOp::FloatArith(crate::tir::FloatArithOp::Add) => {
                TirExprKind::FloatAdd(Box::new(final_left), Box::new(final_right))
            }
            crate::tir::TypedBinOp::FloatArith(crate::tir::FloatArithOp::Sub) => {
                TirExprKind::FloatSub(Box::new(final_left), Box::new(final_right))
            }
            crate::tir::TypedBinOp::FloatArith(crate::tir::FloatArithOp::Mul) => {
                TirExprKind::FloatMul(Box::new(final_left), Box::new(final_right))
            }
            crate::tir::TypedBinOp::FloatArith(crate::tir::FloatArithOp::Div) => {
                TirExprKind::FloatDiv(Box::new(final_left), Box::new(final_right))
            }
            crate::tir::TypedBinOp::FloatArith(crate::tir::FloatArithOp::FloorDiv) => {
                TirExprKind::FloatFloorDiv(Box::new(final_left), Box::new(final_right))
            }
            crate::tir::TypedBinOp::FloatArith(crate::tir::FloatArithOp::Mod) => {
                TirExprKind::FloatMod(Box::new(final_left), Box::new(final_right))
            }
            crate::tir::TypedBinOp::FloatArith(crate::tir::FloatArithOp::Pow) => {
                TirExprKind::FloatPow(Box::new(final_left), Box::new(final_right))
            }
            crate::tir::TypedBinOp::Bitwise(crate::tir::BitwiseBinOp::BitAnd) => {
                TirExprKind::BitAnd(Box::new(final_left), Box::new(final_right))
            }
            crate::tir::TypedBinOp::Bitwise(crate::tir::BitwiseBinOp::BitOr) => {
                TirExprKind::BitOr(Box::new(final_left), Box::new(final_right))
            }
            crate::tir::TypedBinOp::Bitwise(crate::tir::BitwiseBinOp::BitXor) => {
                TirExprKind::BitXor(Box::new(final_left), Box::new(final_right))
            }
            crate::tir::TypedBinOp::Bitwise(crate::tir::BitwiseBinOp::LShift) => {
                TirExprKind::LShift(Box::new(final_left), Box::new(final_right))
            }
            crate::tir::TypedBinOp::Bitwise(crate::tir::BitwiseBinOp::RShift) => {
                TirExprKind::RShift(Box::new(final_left), Box::new(final_right))
            }
        };
        Ok(TirExpr {
            kind,
            ty: result_vty,
        })
    }

    fn resolve_seq_binop(raw_op: RawBinOp, result_ty: &ValueType) -> Option<builtin::BuiltinFn> {
        match (raw_op, result_ty) {
            (RawBinOp::Arith(ArithBinOp::Add), ValueType::Str) => {
                Some(builtin::BuiltinFn::StrConcat)
            }
            (RawBinOp::Arith(ArithBinOp::Add), ValueType::Bytes) => {
                Some(builtin::BuiltinFn::BytesConcat)
            }
            (RawBinOp::Arith(ArithBinOp::Add), ValueType::ByteArray) => {
                Some(builtin::BuiltinFn::ByteArrayConcat)
            }
            (RawBinOp::Arith(ArithBinOp::Add), ValueType::List(_)) => {
                Some(builtin::BuiltinFn::ListConcat)
            }
            (RawBinOp::Arith(ArithBinOp::Mul), ValueType::Str) => {
                Some(builtin::BuiltinFn::StrRepeat)
            }
            (RawBinOp::Arith(ArithBinOp::Mul), ValueType::Bytes) => {
                Some(builtin::BuiltinFn::BytesRepeat)
            }
            (RawBinOp::Arith(ArithBinOp::Mul), ValueType::ByteArray) => {
                Some(builtin::BuiltinFn::ByteArrayRepeat)
            }
            (RawBinOp::Arith(ArithBinOp::Mul), ValueType::List(_)) => {
                Some(builtin::BuiltinFn::ListRepeat)
            }
            _ => None,
        }
    }

    fn try_lower_class_binop_magic(
        &mut self,
        line: usize,
        raw_op: RawBinOp,
        left: TirExpr,
        right: TirExpr,
    ) -> Result<Option<TirExpr>> {
        let magic =
            lookup_class_binop_magic(raw_op).expect("ICE: missing class binop magic mapping");

        let mut found_class_side = false;

        if let ValueType::Class(class_name) = &left.ty {
            found_class_side = true;
            let class_info = self.lookup_class(line, class_name)?;
            if class_info.methods.contains_key(magic.left_method) {
                return self
                    .lower_class_magic_method_with_args(
                        line,
                        left,
                        &[magic.left_method],
                        None,
                        "binary operator",
                        vec![right],
                    )
                    .map(Some);
            }
        }

        if let ValueType::Class(class_name) = &right.ty {
            found_class_side = true;
            let class_info = self.lookup_class(line, class_name)?;
            if class_info.methods.contains_key(magic.right_method) {
                return self
                    .lower_class_magic_method_with_args(
                        line,
                        right,
                        &[magic.right_method],
                        None,
                        "binary operator",
                        vec![left],
                    )
                    .map(Some);
            }
        }

        if found_class_side {
            return Err(self.type_error(
                line,
                format!(
                    "operator `{}` requires class magic methods `{}` or `{}` for operand types `{}` and `{}`",
                    raw_op, magic.left_method, magic.right_method, left.ty, right.ty
                ),
            ));
        }

        Ok(None)
    }

    pub(in crate::tir::lower) fn apply_coercion(expr: TirExpr, coercion: Coercion) -> TirExpr {
        match coercion {
            Coercion::None => expr,
            Coercion::ToFloat => {
                if expr.ty == ValueType::Float {
                    expr
                } else {
                    let cast_kind = match &expr.ty {
                        ValueType::Int => CastKind::IntToFloat,
                        ValueType::Bool => CastKind::BoolToFloat,
                        _ => unreachable!(),
                    };
                    TirExpr {
                        kind: TirExprKind::Cast {
                            kind: cast_kind,
                            arg: Box::new(expr),
                        },
                        ty: ValueType::Float,
                    }
                }
            }
        }
    }
}
