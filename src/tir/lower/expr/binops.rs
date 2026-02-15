use anyhow::Result;

use crate::ast::Type;
use crate::tir::{
    builtin, ArithBinOp, BitwiseBinOp, CastKind, RawBinOp, TirExpr, TirExprKind, ValueType,
};

use crate::tir::lower::Lowering;

// ── Validation (used by integration tests) ───────────────────────────

/// Check whether a binary operation is valid for the given operand types.
pub fn is_valid_binop(op: RawBinOp, left: &Type, right: &Type) -> bool {
    use Type::*;

    // Sequence operations (concat, repeat)
    match op {
        RawBinOp::Arith(ArithBinOp::Add) => match (left, right) {
            (Str, Str) | (Bytes, Bytes) | (ByteArray, ByteArray) => return true,
            (List(a), List(b)) if a == b => return true,
            _ => {}
        },
        RawBinOp::Arith(ArithBinOp::Mul) => match (left, right) {
            (Str, Int)
            | (Int, Str)
            | (Bytes, Int)
            | (Int, Bytes)
            | (ByteArray, Int)
            | (Int, ByteArray)
            | (List(_), Int)
            | (Int, List(_)) => return true,
            _ => {}
        },
        _ => {}
    }

    // Arithmetic/bitwise
    match op {
        RawBinOp::Bitwise(_) => matches!((left, right), (Int, Int)),
        RawBinOp::Arith(_) => matches!(
            (left, right),
            (Int, Int) | (Float, Float) | (Int, Float) | (Float, Int)
        ),
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

// ── Helpers for constructing TIR expression kinds ────────────────────

fn int_arith_kind(arith: ArithBinOp, left: TirExpr, right: TirExpr) -> TirExprKind {
    let (l, r) = (Box::new(left), Box::new(right));
    match arith {
        ArithBinOp::Add => TirExprKind::IntAdd(l, r),
        ArithBinOp::Sub => TirExprKind::IntSub(l, r),
        ArithBinOp::Mul => TirExprKind::IntMul(l, r),
        ArithBinOp::FloorDiv => TirExprKind::IntFloorDiv(l, r),
        ArithBinOp::Mod => TirExprKind::IntMod(l, r),
        ArithBinOp::Pow => TirExprKind::IntPow(l, r),
        ArithBinOp::Div => unreachable!("ICE: int result for true division"),
    }
}

fn float_arith_kind(arith: ArithBinOp, left: TirExpr, right: TirExpr) -> TirExprKind {
    let (l, r) = (Box::new(left), Box::new(right));
    match arith {
        ArithBinOp::Add => TirExprKind::FloatAdd(l, r),
        ArithBinOp::Sub => TirExprKind::FloatSub(l, r),
        ArithBinOp::Mul => TirExprKind::FloatMul(l, r),
        ArithBinOp::Div => TirExprKind::FloatDiv(l, r),
        ArithBinOp::FloorDiv => TirExprKind::FloatFloorDiv(l, r),
        ArithBinOp::Mod => TirExprKind::FloatMod(l, r),
        ArithBinOp::Pow => TirExprKind::FloatPow(l, r),
    }
}

fn bitwise_kind(bw: BitwiseBinOp, left: TirExpr, right: TirExpr) -> TirExprKind {
    let (l, r) = (Box::new(left), Box::new(right));
    match bw {
        BitwiseBinOp::BitAnd => TirExprKind::BitAnd(l, r),
        BitwiseBinOp::BitOr => TirExprKind::BitOr(l, r),
        BitwiseBinOp::BitXor => TirExprKind::BitXor(l, r),
        BitwiseBinOp::LShift => TirExprKind::LShift(l, r),
        BitwiseBinOp::RShift => TirExprKind::RShift(l, r),
    }
}

/// Cast an expression to Float if it is not already Float.
pub(in crate::tir::lower) fn coerce_to_float(expr: TirExpr) -> TirExpr {
    if expr.ty == ValueType::Float {
        return expr;
    }
    let cast_kind = match &expr.ty {
        ValueType::Int => CastKind::IntToFloat,
        ValueType::Bool => CastKind::BoolToFloat,
        _ => unreachable!("coerce_to_float called on non-numeric type"),
    };
    TirExpr {
        kind: TirExprKind::Cast {
            kind: cast_kind,
            arg: Box::new(expr),
        },
        ty: ValueType::Float,
    }
}

// ── Binary Operation Lowering ────────────────────────────────────────

impl Lowering {
    /// Resolve a binary operation directly into a TIR expression.
    pub(in crate::tir::lower) fn resolve_binop(
        &mut self,
        line: usize,
        raw_op: RawBinOp,
        left: TirExpr,
        right: TirExpr,
    ) -> Result<TirExpr> {
        // Class magic dispatch
        if let Some(class_expr) =
            self.try_lower_class_binop_magic(line, raw_op, left.clone(), right.clone())?
        {
            return Ok(class_expr);
        }

        // Sequence operations (concat, repeat)
        if let Some(seq_expr) = Self::try_lower_seq_binop(raw_op, left.clone(), right.clone()) {
            return Ok(seq_expr);
        }

        // Arithmetic/bitwise on primitives
        match raw_op {
            RawBinOp::Bitwise(bw) => {
                if left.ty != ValueType::Int || right.ty != ValueType::Int {
                    let left_ast = left.ty.to_type();
                    let right_ast = right.ty.to_type();
                    return Err(self.type_error(
                        line,
                        binop_type_error_message(raw_op, &left_ast, &right_ast),
                    ));
                }
                Ok(TirExpr {
                    kind: bitwise_kind(bw, left, right),
                    ty: ValueType::Int,
                })
            }
            RawBinOp::Arith(arith) => match (&left.ty, &right.ty) {
                (ValueType::Int, ValueType::Int) => {
                    if arith == ArithBinOp::Div {
                        // True division always produces Float
                        let fl = coerce_to_float(left);
                        let fr = coerce_to_float(right);
                        Ok(TirExpr {
                            kind: TirExprKind::FloatDiv(Box::new(fl), Box::new(fr)),
                            ty: ValueType::Float,
                        })
                    } else {
                        Ok(TirExpr {
                            kind: int_arith_kind(arith, left, right),
                            ty: ValueType::Int,
                        })
                    }
                }
                (ValueType::Float, ValueType::Float) => Ok(TirExpr {
                    kind: float_arith_kind(arith, left, right),
                    ty: ValueType::Float,
                }),
                (ValueType::Int, ValueType::Float) | (ValueType::Float, ValueType::Int) => {
                    let fl = coerce_to_float(left);
                    let fr = coerce_to_float(right);
                    Ok(TirExpr {
                        kind: float_arith_kind(arith, fl, fr),
                        ty: ValueType::Float,
                    })
                }
                _ => {
                    let left_ast = left.ty.to_type();
                    let right_ast = right.ty.to_type();
                    Err(self.type_error(
                        line,
                        binop_type_error_message(raw_op, &left_ast, &right_ast),
                    ))
                }
            },
        }
    }

    fn try_lower_seq_binop(raw_op: RawBinOp, left: TirExpr, right: TirExpr) -> Option<TirExpr> {
        match raw_op {
            RawBinOp::Arith(ArithBinOp::Add) => {
                let (func, ty) = match (&left.ty, &right.ty) {
                    (ValueType::Str, ValueType::Str) => {
                        (builtin::BuiltinFn::StrConcat, ValueType::Str)
                    }
                    (ValueType::Bytes, ValueType::Bytes) => {
                        (builtin::BuiltinFn::BytesConcat, ValueType::Bytes)
                    }
                    (ValueType::ByteArray, ValueType::ByteArray) => {
                        (builtin::BuiltinFn::ByteArrayConcat, ValueType::ByteArray)
                    }
                    (ValueType::List(a), ValueType::List(b)) if a == b => {
                        (builtin::BuiltinFn::ListConcat, left.ty.clone())
                    }
                    _ => return None,
                };
                Some(TirExpr {
                    kind: TirExprKind::ExternalCall {
                        func,
                        args: vec![left, right],
                    },
                    ty,
                })
            }
            RawBinOp::Arith(ArithBinOp::Mul) => {
                let (func, ty, args) = match (&left.ty, &right.ty) {
                    (ValueType::Str, ValueType::Int) => (
                        builtin::BuiltinFn::StrRepeat,
                        ValueType::Str,
                        vec![left, right],
                    ),
                    (ValueType::Int, ValueType::Str) => (
                        builtin::BuiltinFn::StrRepeat,
                        ValueType::Str,
                        vec![right, left],
                    ),
                    (ValueType::Bytes, ValueType::Int) => (
                        builtin::BuiltinFn::BytesRepeat,
                        ValueType::Bytes,
                        vec![left, right],
                    ),
                    (ValueType::Int, ValueType::Bytes) => (
                        builtin::BuiltinFn::BytesRepeat,
                        ValueType::Bytes,
                        vec![right, left],
                    ),
                    (ValueType::ByteArray, ValueType::Int) => (
                        builtin::BuiltinFn::ByteArrayRepeat,
                        ValueType::ByteArray,
                        vec![left, right],
                    ),
                    (ValueType::Int, ValueType::ByteArray) => (
                        builtin::BuiltinFn::ByteArrayRepeat,
                        ValueType::ByteArray,
                        vec![right, left],
                    ),
                    (ValueType::List(_), ValueType::Int) => {
                        let ty = left.ty.clone();
                        (builtin::BuiltinFn::ListRepeat, ty, vec![left, right])
                    }
                    (ValueType::Int, ValueType::List(_)) => {
                        let ty = right.ty.clone();
                        (builtin::BuiltinFn::ListRepeat, ty, vec![right, left])
                    }
                    _ => return None,
                };
                Some(TirExpr {
                    kind: TirExprKind::ExternalCall { func, args },
                    ty,
                })
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
        // Inline the op → (left_method, right_method) mapping
        let (left_method, right_method) = match raw_op {
            RawBinOp::Arith(ArithBinOp::Add) => ("__add__", "__radd__"),
            RawBinOp::Arith(ArithBinOp::Sub) => ("__sub__", "__rsub__"),
            RawBinOp::Arith(ArithBinOp::Mul) => ("__mul__", "__rmul__"),
            RawBinOp::Arith(ArithBinOp::Div) => ("__truediv__", "__rtruediv__"),
            RawBinOp::Arith(ArithBinOp::FloorDiv) => ("__floordiv__", "__rfloordiv__"),
            RawBinOp::Arith(ArithBinOp::Mod) => ("__mod__", "__rmod__"),
            RawBinOp::Arith(ArithBinOp::Pow) => ("__pow__", "__rpow__"),
            RawBinOp::Bitwise(BitwiseBinOp::BitAnd) => ("__and__", "__rand__"),
            RawBinOp::Bitwise(BitwiseBinOp::BitOr) => ("__or__", "__ror__"),
            RawBinOp::Bitwise(BitwiseBinOp::BitXor) => ("__xor__", "__rxor__"),
            RawBinOp::Bitwise(BitwiseBinOp::LShift) => ("__lshift__", "__rlshift__"),
            RawBinOp::Bitwise(BitwiseBinOp::RShift) => ("__rshift__", "__rrshift__"),
        };

        let mut found_class_side = false;

        if let ValueType::Class(class_name) = &left.ty {
            found_class_side = true;
            let class_info = self.lookup_class(line, class_name)?;
            if class_info.methods.contains_key(left_method) {
                return self
                    .lower_class_magic_method_with_args(
                        line,
                        left,
                        &[left_method],
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
            if class_info.methods.contains_key(right_method) {
                return self
                    .lower_class_magic_method_with_args(
                        line,
                        right,
                        &[right_method],
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
                    raw_op, left_method, right_method, left.ty, right.ty
                ),
            ));
        }

        Ok(None)
    }
}
