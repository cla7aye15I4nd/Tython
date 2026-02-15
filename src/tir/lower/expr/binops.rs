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

fn is_arith_primitive(ty: &ValueType) -> bool {
    matches!(ty, ValueType::Int | ValueType::Float)
}

impl Lowering {
    /// Resolve a binary operation directly into a TIR expression.
    pub(in crate::tir::lower) fn resolve_binop(
        &mut self,
        line: usize,
        raw_op: RawBinOp,
        left: TirExpr,
        right: TirExpr,
    ) -> Result<TirExpr> {
        if is_arith_primitive(&left.ty) && is_arith_primitive(&right.ty) {
            return self.resolve_primitive_binop(line, raw_op, left, right);
        }
        self.resolve_method_binop(line, raw_op, left, right)
    }

    /// Handle arithmetic/bitwise on int/float primitives.
    fn resolve_primitive_binop(
        &mut self,
        line: usize,
        raw_op: RawBinOp,
        left: TirExpr,
        right: TirExpr,
    ) -> Result<TirExpr> {
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
                _ => unreachable!("resolve_primitive_binop called with non-primitive types"),
            },
        }
    }

    /// Dispatch a binary operation through method calls (dunder methods).
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
    /// Returns `Ok(Some(expr))` if the method exists, `Ok(None)` if it doesn't,
    /// or propagates type errors from method call validation.
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
            ty if !is_arith_primitive(ty) => {
                match self.lower_method_call(line, obj.clone(), method, vec![arg.clone()]) {
                    Ok(CallResult::Expr(e)) => Ok(Some(e)),
                    Ok(CallResult::VoidStmt(_)) => {
                        unreachable!("binop dunder methods must return a value")
                    }
                    Err(_) => Ok(None),
                }
            }
            _ => Ok(None),
        }
    }
}
