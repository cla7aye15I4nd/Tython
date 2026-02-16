use anyhow::Result;

use crate::ast::Type;
use crate::tir::{
    ArithBinOp, BitwiseBinOp, CallResult, CastKind, RawBinOp, TirExpr, TirExprKind, ValueType,
};

use crate::tir::lower::Lowering;

// ── Validation (used by integration tests) ───────────────────────────

/// Check whether a binary operation is valid for the given operand types.
pub fn is_valid_binop(op: RawBinOp, left: &Type, right: &Type) -> bool {
    use Type::*;

    // Primitive arithmetic/bitwise (int, float)
    match op {
        RawBinOp::Bitwise(_) => {
            if matches!((left, right), (Int, Int)) {
                return true;
            }
        }
        RawBinOp::Arith(_) => {
            if matches!(
                (left, right),
                (Int, Int) | (Float, Float) | (Int, Float) | (Float, Int)
            ) {
                return true;
            }
        }
    }

    // Method-dispatched operations (ref types with dunder methods)
    has_method_binop(op, left, right)
}

/// Check whether a binary operation is supported via dunder method dispatch
/// for builtin ref types.
fn has_method_binop(op: RawBinOp, left: &Type, right: &Type) -> bool {
    use Type::*;
    match op {
        RawBinOp::Arith(ArithBinOp::Add) => {
            matches!(
                (left, right),
                (Str, Str) | (Bytes, Bytes) | (ByteArray, ByteArray)
            ) || matches!((left, right), (List(a), List(b)) if a == b)
        }
        RawBinOp::Arith(ArithBinOp::Mul) => matches!(
            (left, right),
            (Str, Int)
                | (Int, Str)
                | (Bytes, Int)
                | (Int, Bytes)
                | (ByteArray, Int)
                | (Int, ByteArray)
                | (List(_), Int)
                | (Int, List(_))
        ),
        _ => false,
    }
}

// ── Helpers for constructing TIR expression kinds ────────────────────

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
    // Callers only invoke this helper for int/float numeric promotion.
    TirExpr {
        kind: TirExprKind::Cast {
            kind: CastKind::IntToFloat,
            arg: Box::new(expr),
        },
        ty: ValueType::Float,
    }
}

// ── Binary Operation Lowering ────────────────────────────────────────

/// Map a binary operator to its (forward, reflected) dunder method names.
fn binop_to_dunder(raw_op: RawBinOp) -> (&'static str, &'static str) {
    match raw_op {
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
    }
}

impl Lowering {
    /// Resolve a binary operation into a TIR expression.
    /// Primitives (int/float) are handled directly; everything else dispatches
    /// through dunder methods.
    pub(in crate::tir::lower) fn resolve_binop(
        &mut self,
        line: usize,
        raw_op: RawBinOp,
        left: TirExpr,
        right: TirExpr,
    ) -> Result<TirExpr> {
        match (raw_op, &left.ty, &right.ty) {
            // ── Bitwise: int × int ──────────────────────────────────────
            (RawBinOp::Bitwise(bw), ValueType::Int, ValueType::Int) => Ok(TirExpr {
                kind: bitwise_kind(bw, left, right),
                ty: ValueType::Int,
            }),

            // ── Arithmetic: int × int ───────────────────────────────────
            (RawBinOp::Arith(ArithBinOp::Div), ValueType::Int, ValueType::Int) => {
                let fl = coerce_to_float(left);
                let fr = coerce_to_float(right);
                Ok(TirExpr {
                    kind: TirExprKind::FloatDiv(Box::new(fl), Box::new(fr)),
                    ty: ValueType::Float,
                })
            }
            (RawBinOp::Arith(ArithBinOp::Add), ValueType::Int, ValueType::Int) => Ok(TirExpr {
                kind: TirExprKind::IntAdd(Box::new(left), Box::new(right)),
                ty: ValueType::Int,
            }),
            (RawBinOp::Arith(ArithBinOp::Sub), ValueType::Int, ValueType::Int) => Ok(TirExpr {
                kind: TirExprKind::IntSub(Box::new(left), Box::new(right)),
                ty: ValueType::Int,
            }),
            (RawBinOp::Arith(ArithBinOp::Mul), ValueType::Int, ValueType::Int) => Ok(TirExpr {
                kind: TirExprKind::IntMul(Box::new(left), Box::new(right)),
                ty: ValueType::Int,
            }),
            (RawBinOp::Arith(ArithBinOp::FloorDiv), ValueType::Int, ValueType::Int) => {
                Ok(TirExpr {
                    kind: TirExprKind::IntFloorDiv(Box::new(left), Box::new(right)),
                    ty: ValueType::Int,
                })
            }
            (RawBinOp::Arith(ArithBinOp::Mod), ValueType::Int, ValueType::Int) => Ok(TirExpr {
                kind: TirExprKind::IntMod(Box::new(left), Box::new(right)),
                ty: ValueType::Int,
            }),
            (RawBinOp::Arith(ArithBinOp::Pow), ValueType::Int, ValueType::Int) => Ok(TirExpr {
                kind: TirExprKind::IntPow(Box::new(left), Box::new(right)),
                ty: ValueType::Int,
            }),

            // ── Arithmetic: float × float ───────────────────────────────
            (RawBinOp::Arith(arith), ValueType::Float, ValueType::Float) => Ok(TirExpr {
                kind: float_arith_kind(arith, left, right),
                ty: ValueType::Float,
            }),

            // ── Arithmetic: mixed int/float ─────────────────────────────
            (RawBinOp::Arith(arith), ValueType::Int, ValueType::Float)
            | (RawBinOp::Arith(arith), ValueType::Float, ValueType::Int) => {
                let fl = coerce_to_float(left);
                let fr = coerce_to_float(right);
                Ok(TirExpr {
                    kind: float_arith_kind(arith, fl, fr),
                    ty: ValueType::Float,
                })
            }

            // ── Everything else: method dispatch ────────────────────────
            _ => self.resolve_method_binop(line, raw_op, left, right),
        }
    }

    /// Dispatch a binary operation through dunder methods.
    /// Handles both user-defined classes and builtin ref types (str, list, etc.).
    fn resolve_method_binop(
        &mut self,
        line: usize,
        raw_op: RawBinOp,
        left: TirExpr,
        right: TirExpr,
    ) -> Result<TirExpr> {
        let (left_method, right_method) = binop_to_dunder(raw_op);

        // Try left operand's forward method
        if let Some(expr) = self.try_binop_dispatch(line, &left, left_method, &right)? {
            return Ok(expr);
        }

        // Try right operand's reflected method
        if let Some(expr) = self.try_binop_dispatch(line, &right, right_method, &left)? {
            return Ok(expr);
        }

        Err(self.type_error(
            line,
            format!(
                "unsupported operand type(s) for `{}`: `{}` and `{}`",
                raw_op, left.ty, right.ty
            ),
        ))
    }

    /// Try to dispatch a binary operation on a single operand via its dunder method.
    fn try_binop_dispatch(
        &mut self,
        line: usize,
        obj: &TirExpr,
        method: &str,
        arg: &TirExpr,
    ) -> Result<Option<TirExpr>> {
        match &obj.ty {
            ValueType::Class(class_name) => {
                let class_info = self.lookup_class(line, class_name)?;
                if class_info.methods.contains_key(method) {
                    self.lower_class_magic_method_with_args(
                        line,
                        obj.clone(),
                        &[method],
                        None,
                        "binary operator",
                        vec![arg.clone()],
                    )
                    .map(Some)
                } else {
                    Ok(None)
                }
            }
            ValueType::Int | ValueType::Float | ValueType::Bool => Ok(None),
            _ => match self.lower_method_call(line, obj.clone(), method, vec![arg.clone()]) {
                Ok(CallResult::Expr(e)) => Ok(Some(e)),
                _ => Ok(None),
            },
        }
    }
}
