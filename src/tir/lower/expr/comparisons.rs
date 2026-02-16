use anyhow::Result;

use crate::tir::{CallResult, CmpOp, OrderedCmpOp, TirExpr, TirExprKind, TypedCompare, ValueType};

use super::binops::coerce_to_float;
use crate::tir::lower::Lowering;

/// Convert an ordered comparison operator + operand type into a fully-typed `TypedCompare`.
fn resolve_typed_compare(op: OrderedCmpOp, operand_ty: &ValueType) -> Result<TypedCompare> {
    use OrderedCmpOp::*;
    use ValueType::*;

    match operand_ty {
        Int => match op {
            Eq => Ok(TypedCompare::IntEq),
            NotEq => Ok(TypedCompare::IntNotEq),
            Lt => Ok(TypedCompare::IntLt),
            LtEq => Ok(TypedCompare::IntLtEq),
            Gt => Ok(TypedCompare::IntGt),
            GtEq => Ok(TypedCompare::IntGtEq),
        },
        Float => match op {
            Eq => Ok(TypedCompare::FloatEq),
            NotEq => Ok(TypedCompare::FloatNotEq),
            Lt => Ok(TypedCompare::FloatLt),
            LtEq => Ok(TypedCompare::FloatLtEq),
            Gt => Ok(TypedCompare::FloatGt),
            GtEq => Ok(TypedCompare::FloatGtEq),
        },
        Bool => match op {
            Eq => Ok(TypedCompare::BoolEq),
            NotEq => Ok(TypedCompare::BoolNotEq),
            _ => Err(anyhow::anyhow!("`{op:?}` is not supported for type `bool`")),
        },
        _ => Err(anyhow::anyhow!(
            "internal error: comparison dispatch for type `{operand_ty}` not supported"
        )),
    }
}

/// Helper to convert TypedCompare to the appropriate TirExprKind variant
fn typed_compare_to_kind(op: TypedCompare, left: TirExpr, right: TirExpr) -> TirExprKind {
    match op {
        TypedCompare::IntEq => TirExprKind::IntEq(Box::new(left), Box::new(right)),
        TypedCompare::IntNotEq => TirExprKind::IntNotEq(Box::new(left), Box::new(right)),
        TypedCompare::IntLt => TirExprKind::IntLt(Box::new(left), Box::new(right)),
        TypedCompare::IntLtEq => TirExprKind::IntLtEq(Box::new(left), Box::new(right)),
        TypedCompare::IntGt => TirExprKind::IntGt(Box::new(left), Box::new(right)),
        TypedCompare::IntGtEq => TirExprKind::IntGtEq(Box::new(left), Box::new(right)),
        TypedCompare::FloatEq => TirExprKind::FloatEq(Box::new(left), Box::new(right)),
        TypedCompare::FloatNotEq => TirExprKind::FloatNotEq(Box::new(left), Box::new(right)),
        TypedCompare::FloatLt => TirExprKind::FloatLt(Box::new(left), Box::new(right)),
        TypedCompare::FloatLtEq => TirExprKind::FloatLtEq(Box::new(left), Box::new(right)),
        TypedCompare::FloatGt => TirExprKind::FloatGt(Box::new(left), Box::new(right)),
        TypedCompare::FloatGtEq => TirExprKind::FloatGtEq(Box::new(left), Box::new(right)),
        TypedCompare::BoolEq => TirExprKind::BoolEq(Box::new(left), Box::new(right)),
        TypedCompare::BoolNotEq => TirExprKind::BoolNotEq(Box::new(left), Box::new(right)),
    }
}

impl Lowering {
    pub(in crate::tir::lower) fn lower_single_comparison(
        &mut self,
        line: usize,
        cmp_op: CmpOp,
        left: TirExpr,
        right: TirExpr,
    ) -> Result<TirExpr> {
        // `in` / `not in` — containment check via __contains__
        if matches!(cmp_op, CmpOp::In | CmpOp::NotIn) {
            return self.resolve_contains(line, cmp_op, left, right);
        }

        // `is` / `is not` — identity (pointer equality for ref types, value equality for primitives)
        if matches!(cmp_op, CmpOp::Is | CmpOp::IsNot) {
            let typed_op = if left.ty.is_ref_type() {
                if cmp_op == CmpOp::Is {
                    TypedCompare::IntEq
                } else {
                    TypedCompare::IntNotEq
                }
            } else {
                let ordered_op = if cmp_op == CmpOp::Is {
                    OrderedCmpOp::Eq
                } else {
                    OrderedCmpOp::NotEq
                };
                resolve_typed_compare(ordered_op, &left.ty)?
            };

            return Ok(TirExpr {
                kind: typed_compare_to_kind(typed_op, left, right),
                ty: ValueType::Bool,
            });
        }

        // Primitive comparison (Int, Float, Bool) with optional promotion
        match (&left.ty, &right.ty) {
            (ValueType::Int, ValueType::Int)
            | (ValueType::Float, ValueType::Float)
            | (ValueType::Bool, ValueType::Bool) => {
                let ordered_op = OrderedCmpOp::from_cmp_op(cmp_op);
                let typed_op = resolve_typed_compare(ordered_op, &left.ty)?;
                Ok(TirExpr {
                    kind: typed_compare_to_kind(typed_op, left, right),
                    ty: ValueType::Bool,
                })
            }
            (ValueType::Int, ValueType::Float) | (ValueType::Float, ValueType::Int) => {
                let fl = coerce_to_float(left);
                let fr = coerce_to_float(right);
                let ordered_op = OrderedCmpOp::from_cmp_op(cmp_op);
                let typed_op = resolve_typed_compare(ordered_op, &ValueType::Float)?;
                Ok(TirExpr {
                    kind: typed_compare_to_kind(typed_op, fl, fr),
                    ty: ValueType::Bool,
                })
            }

            // Everything else — method dispatch
            _ => self.resolve_method_comparison(line, cmp_op, left, right),
        }
    }

    /// Dispatch `in` / `not in` via `__contains__` on the right operand.
    fn resolve_contains(
        &mut self,
        line: usize,
        cmp_op: CmpOp,
        left: TirExpr,
        right: TirExpr,
    ) -> Result<TirExpr> {
        let contains_expr = self
            .try_comparison_dispatch(line, &right, "__contains__", &left)?
            .ok_or_else(|| {
                self.type_error(line, format!("`in` not supported for type `{}`", right.ty))
            })?;

        if cmp_op == CmpOp::NotIn {
            Ok(TirExpr {
                kind: TirExprKind::Not(Box::new(contains_expr)),
                ty: ValueType::Bool,
            })
        } else {
            Ok(contains_expr)
        }
    }

    /// Dispatch ordered comparisons via `__eq__` / `__lt__` methods.
    fn resolve_method_comparison(
        &mut self,
        line: usize,
        cmp_op: CmpOp,
        left: TirExpr,
        right: TirExpr,
    ) -> Result<TirExpr> {
        if left.ty != right.ty {
            return Err(self.type_error(
                line,
                format!(
                    "comparison operands must have compatible types: `{}` vs `{}`",
                    left.ty, right.ty
                ),
            ));
        }

        match cmp_op {
            CmpOp::Eq | CmpOp::NotEq => {
                let eq_result = self.try_comparison_dispatch(line, &left, "__eq__", &right)?;
                let eq_expr = match eq_result {
                    Some(e) => e,
                    None => {
                        // No __eq__ → fall back to identity comparison
                        let typed_op = if left.ty.is_ref_type() {
                            if cmp_op == CmpOp::Eq {
                                TypedCompare::IntEq
                            } else {
                                TypedCompare::IntNotEq
                            }
                        } else {
                            return Err(self.type_error(
                                line,
                                format!("type `{}` does not support equality comparison", left.ty),
                            ));
                        };
                        return Ok(TirExpr {
                            kind: typed_compare_to_kind(typed_op, left, right),
                            ty: ValueType::Bool,
                        });
                    }
                };
                if cmp_op == CmpOp::NotEq {
                    Ok(TirExpr {
                        kind: TirExprKind::Not(Box::new(eq_expr)),
                        ty: ValueType::Bool,
                    })
                } else {
                    Ok(eq_expr)
                }
            }
            CmpOp::Lt | CmpOp::LtEq | CmpOp::Gt | CmpOp::GtEq => {
                let dispatch_lt = |this: &mut Self, a: &TirExpr, b: &TirExpr| -> Result<TirExpr> {
                    this.try_comparison_dispatch(line, a, "__lt__", b)?
                        .ok_or_else(|| {
                            this.type_error(
                                line,
                                format!(
                                    "type `{}` does not support ordering comparisons (no `__lt__`)",
                                    a.ty
                                ),
                            )
                        })
                };
                match cmp_op {
                    CmpOp::Lt => dispatch_lt(self, &left, &right),
                    CmpOp::Gt => dispatch_lt(self, &right, &left),
                    CmpOp::LtEq => {
                        let gt = dispatch_lt(self, &right, &left)?;
                        Ok(TirExpr {
                            kind: TirExprKind::Not(Box::new(gt)),
                            ty: ValueType::Bool,
                        })
                    }
                    CmpOp::GtEq => {
                        let lt = dispatch_lt(self, &left, &right)?;
                        Ok(TirExpr {
                            kind: TirExprKind::Not(Box::new(lt)),
                            ty: ValueType::Bool,
                        })
                    }
                    _ => unreachable!(),
                }
            }
            _ => Err(self.type_error(
                line,
                format!(
                    "comparison operator `{:?}` is not supported for type `{}`",
                    cmp_op, left.ty
                ),
            )),
        }
    }

    /// Try to dispatch a comparison operation on a single operand via its dunder method.
    fn try_comparison_dispatch(
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
                        Some(ValueType::Bool),
                        "comparison operator",
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
                Ok(CallResult::VoidStmt(_)) => Ok(None),
                Err(e) => Err(e),
            },
        }
    }
}
