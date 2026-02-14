use anyhow::Result;

use crate::tir::{
    builtin, type_rules, CastKind, CmpOp, IntrinsicOp, OrderedCmpOp, TirExpr, TirExprKind,
    TypedCompare, ValueType,
};

use crate::tir::lower::expr::binops::Coercion;
use crate::tir::lower::Lowering;

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
        // `in` / `not in` — containment check (must be before seq comparison)
        if matches!(cmp_op, CmpOp::In | CmpOp::NotIn) {
            let contains_expr = match &right.ty {
                ValueType::List(inner) => {
                    self.require_list_leaf_eq_support(line, inner)?;
                    let eq_tag = self.register_intrinsic_instance(IntrinsicOp::Eq, inner);
                    TirExpr {
                        kind: TirExprKind::ExternalCall {
                            func: builtin::BuiltinFn::ListContainsByTag,
                            args: vec![
                                right,
                                left,
                                TirExpr {
                                    kind: TirExprKind::IntLiteral(eq_tag),
                                    ty: ValueType::Int,
                                },
                            ],
                        },
                        ty: ValueType::Bool,
                    }
                }
                ValueType::Dict(key_ty, _value_ty) => {
                    if left.ty != **key_ty {
                        return Err(self.type_error(
                            line,
                            format!(
                                "`in <dict>` requires key type `{}`, got `{}`",
                                key_ty, left.ty
                            ),
                        ));
                    }
                    TirExpr {
                        kind: TirExprKind::ExternalCall {
                            func: builtin::BuiltinFn::DictContains,
                            args: vec![right, left],
                        },
                        ty: ValueType::Bool,
                    }
                }
                ValueType::Set(elem_ty) => {
                    if left.ty != **elem_ty {
                        return Err(self.type_error(
                            line,
                            format!(
                                "`in <set>` requires element type `{}`, got `{}`",
                                elem_ty, left.ty
                            ),
                        ));
                    }
                    TirExpr {
                        kind: TirExprKind::ExternalCall {
                            func: builtin::BuiltinFn::SetContains,
                            args: vec![right, left],
                        },
                        ty: ValueType::Bool,
                    }
                }
                ValueType::Str => {
                    if left.ty != ValueType::Str {
                        return Err(self.type_error(
                            line,
                            format!("`in <str>` requires str operand, got `{}`", left.ty),
                        ));
                    }
                    TirExpr {
                        kind: TirExprKind::ExternalCall {
                            func: builtin::BuiltinFn::StrContains,
                            args: vec![right, left],
                        },
                        ty: ValueType::Bool,
                    }
                }
                _ => {
                    return Err(self
                        .type_error(line, format!("`in` not supported for type `{}`", right.ty)));
                }
            };
            if cmp_op == CmpOp::NotIn {
                return Ok(TirExpr {
                    kind: TirExprKind::Not(Box::new(contains_expr)),
                    ty: ValueType::Bool,
                });
            }
            return Ok(contains_expr);
        }

        // `is` / `is not` — identity (pointer equality for ref types, value equality for primitives)
        if matches!(cmp_op, CmpOp::Is | CmpOp::IsNot) {
            let ordered_op = if cmp_op == CmpOp::Is {
                OrderedCmpOp::Eq
            } else {
                OrderedCmpOp::NotEq
            };

            // For reference types, we'll compare pointers as integers in codegen
            // For primitive types, use direct comparison
            let typed_op = if left.ty.is_ref_type() {
                // Pointer comparison - codegen will convert to int and compare
                if cmp_op == CmpOp::Is {
                    TypedCompare::IntEq
                } else {
                    TypedCompare::IntNotEq
                }
            } else {
                type_rules::resolve_typed_compare(ordered_op, &left.ty)
            };

            return Ok(TirExpr {
                kind: typed_compare_to_kind(typed_op, left, right),
                ty: ValueType::Bool,
            });
        }

        // Sequence comparison: dispatch to runtime functions
        if let Some((eq_fn, cmp_fn)) = Self::seq_compare_builtins(&left.ty) {
            if left.ty != right.ty {
                return Err(self.type_error(
                    line,
                    format!(
                        "comparison operands must have compatible types: `{}` vs `{}`",
                        left.ty, right.ty
                    ),
                ));
            }
            return Ok(Self::build_seq_comparison(
                OrderedCmpOp::from_cmp_op(cmp_op),
                eq_fn,
                cmp_fn,
                left,
                right,
            ));
        }

        // List comparison (equality + lexicographic ordering)
        if matches!(&left.ty, ValueType::List(_)) && matches!(&right.ty, ValueType::List(_)) {
            if left.ty != right.ty {
                return Err(self.type_error(
                    line,
                    format!(
                        "comparison operands must have compatible types: `{}` vs `{}`",
                        left.ty, right.ty
                    ),
                ));
            }
            let ValueType::List(inner) = &left.ty else {
                unreachable!();
            };
            match cmp_op {
                CmpOp::Eq | CmpOp::NotEq => {
                    self.require_list_leaf_eq_support(line, inner)?;
                    let eq_expr = self.generate_list_eq(left, right);
                    if cmp_op == CmpOp::NotEq {
                        return Ok(TirExpr {
                            kind: TirExprKind::Not(Box::new(eq_expr)),
                            ty: ValueType::Bool,
                        });
                    }
                    return Ok(eq_expr);
                }
                CmpOp::Lt | CmpOp::LtEq | CmpOp::Gt | CmpOp::GtEq => {
                    self.require_list_leaf_lt_support(line, inner)?;
                    let lt_tag = self.register_intrinsic_instance(IntrinsicOp::Lt, inner);
                    let left_lt_right = TirExpr {
                        kind: TirExprKind::ExternalCall {
                            func: builtin::BuiltinFn::ListLtByTag,
                            args: vec![
                                left.clone(),
                                right.clone(),
                                TirExpr {
                                    kind: TirExprKind::IntLiteral(lt_tag),
                                    ty: ValueType::Int,
                                },
                            ],
                        },
                        ty: ValueType::Bool,
                    };
                    let right_lt_left = TirExpr {
                        kind: TirExprKind::ExternalCall {
                            func: builtin::BuiltinFn::ListLtByTag,
                            args: vec![
                                right.clone(),
                                left.clone(),
                                TirExpr {
                                    kind: TirExprKind::IntLiteral(lt_tag),
                                    ty: ValueType::Int,
                                },
                            ],
                        },
                        ty: ValueType::Bool,
                    };
                    let out = match cmp_op {
                        CmpOp::Lt => left_lt_right,
                        CmpOp::Gt => right_lt_left,
                        CmpOp::LtEq => TirExpr {
                            kind: TirExprKind::Not(Box::new(right_lt_left)),
                            ty: ValueType::Bool,
                        },
                        CmpOp::GtEq => TirExpr {
                            kind: TirExprKind::Not(Box::new(left_lt_right)),
                            ty: ValueType::Bool,
                        },
                        _ => unreachable!(),
                    };
                    return Ok(out);
                }
                _ => {}
            }
        }

        // Dict equality
        if matches!(&left.ty, ValueType::Dict(_, _)) && matches!(&right.ty, ValueType::Dict(_, _)) {
            if left.ty != right.ty {
                return Err(self.type_error(
                    line,
                    format!(
                        "comparison operands must have compatible types: `{}` vs `{}`",
                        left.ty, right.ty
                    ),
                ));
            }
            if cmp_op != CmpOp::Eq && cmp_op != CmpOp::NotEq {
                return Err(
                    self.type_error(line, "only `==` and `!=` are supported for dict comparison")
                );
            }
            let eq_expr = TirExpr {
                kind: TirExprKind::ExternalCall {
                    func: builtin::BuiltinFn::DictEq,
                    args: vec![left, right],
                },
                ty: ValueType::Bool,
            };
            if cmp_op == CmpOp::NotEq {
                return Ok(TirExpr {
                    kind: TirExprKind::Not(Box::new(eq_expr)),
                    ty: ValueType::Bool,
                });
            }
            return Ok(eq_expr);
        }

        // Set equality
        if matches!(&left.ty, ValueType::Set(_)) && matches!(&right.ty, ValueType::Set(_)) {
            if left.ty != right.ty {
                return Err(self.type_error(
                    line,
                    format!(
                        "comparison operands must have compatible types: `{}` vs `{}`",
                        left.ty, right.ty
                    ),
                ));
            }
            if cmp_op != CmpOp::Eq && cmp_op != CmpOp::NotEq {
                return Err(
                    self.type_error(line, "only `==` and `!=` are supported for set comparison")
                );
            }
            let eq_expr = TirExpr {
                kind: TirExprKind::ExternalCall {
                    func: builtin::BuiltinFn::SetEq,
                    args: vec![left, right],
                },
                ty: ValueType::Bool,
            };
            if cmp_op == CmpOp::NotEq {
                return Ok(TirExpr {
                    kind: TirExprKind::Not(Box::new(eq_expr)),
                    ty: ValueType::Bool,
                });
            }
            return Ok(eq_expr);
        }

        // Tuple equality
        if matches!(&left.ty, ValueType::Tuple(_)) && matches!(&right.ty, ValueType::Tuple(_)) {
            if left.ty != right.ty {
                return Err(self.type_error(
                    line,
                    format!(
                        "comparison operands must have compatible types: `{}` vs `{}`",
                        left.ty, right.ty
                    ),
                ));
            }
            if cmp_op != CmpOp::Eq && cmp_op != CmpOp::NotEq {
                return Err(self.type_error(
                    line,
                    "only `==` and `!=` are supported for tuple comparison",
                ));
            }
            let eq_expr = self.generate_tuple_eq(left, right);
            if cmp_op == CmpOp::NotEq {
                return Ok(TirExpr {
                    kind: TirExprKind::Not(Box::new(eq_expr)),
                    ty: ValueType::Bool,
                });
            }
            return Ok(eq_expr);
        }

        // Class comparison via intrinsic Eq/Lt dispatch
        if matches!(&left.ty, ValueType::Class(_)) && matches!(&right.ty, ValueType::Class(_)) {
            if left.ty != right.ty {
                return Err(self.type_error(
                    line,
                    format!(
                        "comparison operands must have compatible types: `{}` vs `{}`",
                        left.ty, right.ty
                    ),
                ));
            }
            let out = match cmp_op {
                CmpOp::Eq => {
                    if let ValueType::Class(class_name) = &left.ty {
                        self.require_class_magic_method(line, class_name, "__eq__")?;
                    }
                    self.register_intrinsic_instance(IntrinsicOp::Eq, &left.ty);
                    TirExpr {
                        kind: TirExprKind::IntrinsicCmp {
                            op: IntrinsicOp::Eq,
                            lhs: Box::new(left),
                            rhs: Box::new(right),
                        },
                        ty: ValueType::Bool,
                    }
                }
                CmpOp::NotEq => {
                    if let ValueType::Class(class_name) = &left.ty {
                        self.require_class_magic_method(line, class_name, "__eq__")?;
                    }
                    self.register_intrinsic_instance(IntrinsicOp::Eq, &left.ty);
                    let eq = TirExpr {
                        kind: TirExprKind::IntrinsicCmp {
                            op: IntrinsicOp::Eq,
                            lhs: Box::new(left),
                            rhs: Box::new(right),
                        },
                        ty: ValueType::Bool,
                    };
                    TirExpr {
                        kind: TirExprKind::Not(Box::new(eq)),
                        ty: ValueType::Bool,
                    }
                }
                CmpOp::Lt | CmpOp::LtEq | CmpOp::Gt | CmpOp::GtEq => {
                    if let ValueType::Class(class_name) = &left.ty {
                        self.require_class_magic_method(line, class_name, "__lt__")?;
                    }
                    self.register_intrinsic_instance(IntrinsicOp::Lt, &left.ty);
                    let left_lt_right = TirExpr {
                        kind: TirExprKind::IntrinsicCmp {
                            op: IntrinsicOp::Lt,
                            lhs: Box::new(left.clone()),
                            rhs: Box::new(right.clone()),
                        },
                        ty: ValueType::Bool,
                    };
                    let right_lt_left = TirExpr {
                        kind: TirExprKind::IntrinsicCmp {
                            op: IntrinsicOp::Lt,
                            lhs: Box::new(right.clone()),
                            rhs: Box::new(left.clone()),
                        },
                        ty: ValueType::Bool,
                    };
                    match cmp_op {
                        CmpOp::Lt => left_lt_right,
                        CmpOp::Gt => right_lt_left,
                        CmpOp::LtEq => TirExpr {
                            kind: TirExprKind::Not(Box::new(right_lt_left)),
                            ty: ValueType::Bool,
                        },
                        CmpOp::GtEq => TirExpr {
                            kind: TirExprKind::Not(Box::new(left_lt_right)),
                            ty: ValueType::Bool,
                        },
                        _ => unreachable!(),
                    }
                }
                _ => {
                    return Err(self.type_error(
                        line,
                        format!(
                            "comparison operator `{:?}` is not supported for class values",
                            cmp_op
                        ),
                    ));
                }
            };
            return Ok(out);
        }

        // Numeric comparison with optional promotion
        let (fl, fr) = self.promote_for_comparison(line, left, right)?;
        let ordered_op = OrderedCmpOp::from_cmp_op(cmp_op);

        // Resolve to typed comparison for primitive types
        let typed_op = type_rules::resolve_typed_compare(ordered_op, &fl.ty);

        Ok(TirExpr {
            kind: typed_compare_to_kind(typed_op, fl, fr),
            ty: ValueType::Bool,
        })
    }

    /// Generate equality check for two lists.
    /// Lowered as a generic list-equality call; codegen monomorphizes by element type.
    fn generate_list_eq(&mut self, left: TirExpr, right: TirExpr) -> TirExpr {
        let ValueType::List(inner) = &left.ty else {
            unreachable!();
        };
        let eq_tag = self.register_intrinsic_instance(IntrinsicOp::Eq, inner);
        TirExpr {
            kind: TirExprKind::ExternalCall {
                func: builtin::BuiltinFn::ListEqByTag,
                args: vec![
                    left,
                    right,
                    TirExpr {
                        kind: TirExprKind::IntLiteral(eq_tag),
                        ty: ValueType::Int,
                    },
                ],
            },
            ty: ValueType::Bool,
        }
    }

    /// Generate equality expression for two tuples.
    /// Produces: a[0] == b[0] && a[1] == b[1] && ... && a[N-1] == b[N-1]
    fn generate_tuple_eq(&mut self, left: TirExpr, right: TirExpr) -> TirExpr {
        let elements = match &left.ty {
            ValueType::Tuple(elems) => elems.clone(),
            _ => unreachable!(),
        };

        if elements.is_empty() {
            return TirExpr {
                kind: TirExprKind::BoolLiteral(true),
                ty: ValueType::Bool,
            };
        }

        let mut comparisons = Vec::new();
        let tuple_signature = self.get_or_register_tuple(&elements);

        for (i, elem_ty) in elements.iter().enumerate() {
            let left_elem = TirExpr {
                kind: TirExprKind::GetTupleField {
                    tuple: Box::new(left.clone()),
                    tuple_signature: tuple_signature.clone(),
                    field_index: i,
                },
                ty: elem_ty.clone(),
            };
            let right_elem = TirExpr {
                kind: TirExprKind::GetTupleField {
                    tuple: Box::new(right.clone()),
                    tuple_signature: tuple_signature.clone(),
                    field_index: i,
                },
                ty: elem_ty.clone(),
            };
            comparisons.push(self.generate_equality_check(left_elem, right_elem, elem_ty));
        }

        // Chain with LogicalAnd
        let mut result = comparisons.remove(0);
        for cmp in comparisons {
            result = TirExpr {
                kind: TirExprKind::LogicalAnd(Box::new(result), Box::new(cmp)),
                ty: ValueType::Bool,
            };
        }
        result
    }

    /// Recursive dispatch for generating equality checks based on type.
    fn generate_equality_check(
        &mut self,
        left: TirExpr,
        right: TirExpr,
        ty: &ValueType,
    ) -> TirExpr {
        match ty {
            ValueType::Int => TirExpr {
                kind: TirExprKind::IntEq(Box::new(left), Box::new(right)),
                ty: ValueType::Bool,
            },
            ValueType::Float => TirExpr {
                kind: TirExprKind::FloatEq(Box::new(left), Box::new(right)),
                ty: ValueType::Bool,
            },
            ValueType::Bool => TirExpr {
                kind: TirExprKind::BoolEq(Box::new(left), Box::new(right)),
                ty: ValueType::Bool,
            },
            ValueType::Tuple(_) => self.generate_tuple_eq(left, right),
            ValueType::List(_) => self.generate_list_eq(left, right),
            ValueType::Str => TirExpr {
                kind: TirExprKind::ExternalCall {
                    func: builtin::BuiltinFn::StrEq,
                    args: vec![left, right],
                },
                ty: ValueType::Bool,
            },
            ValueType::Bytes => TirExpr {
                kind: TirExprKind::ExternalCall {
                    func: builtin::BuiltinFn::BytesEq,
                    args: vec![left, right],
                },
                ty: ValueType::Bool,
            },
            ValueType::ByteArray => TirExpr {
                kind: TirExprKind::ExternalCall {
                    func: builtin::BuiltinFn::ByteArrayEq,
                    args: vec![left, right],
                },
                ty: ValueType::Bool,
            },
            ValueType::Class(_) => {
                self.register_intrinsic_instance(IntrinsicOp::Eq, ty);
                TirExpr {
                    kind: TirExprKind::IntrinsicCmp {
                        op: IntrinsicOp::Eq,
                        lhs: Box::new(left),
                        rhs: Box::new(right),
                    },
                    ty: ValueType::Bool,
                }
            }
            _ => {
                // Fallback: bitwise compare (works for primitives stored as i64)
                TirExpr {
                    kind: TirExprKind::IntEq(Box::new(left), Box::new(right)),
                    ty: ValueType::Bool,
                }
            }
        }
    }

    fn seq_compare_builtins(ty: &ValueType) -> Option<(builtin::BuiltinFn, builtin::BuiltinFn)> {
        match ty {
            ValueType::Str => Some((builtin::BuiltinFn::StrEq, builtin::BuiltinFn::StrCmp)),
            ValueType::Bytes => Some((builtin::BuiltinFn::BytesEq, builtin::BuiltinFn::BytesCmp)),
            ValueType::ByteArray => Some((
                builtin::BuiltinFn::ByteArrayEq,
                builtin::BuiltinFn::ByteArrayCmp,
            )),
            _ => None,
        }
    }

    fn build_seq_comparison(
        cmp_op: OrderedCmpOp,
        eq_fn: builtin::BuiltinFn,
        cmp_fn: builtin::BuiltinFn,
        left: TirExpr,
        right: TirExpr,
    ) -> TirExpr {
        let zero = TirExpr {
            kind: TirExprKind::IntLiteral(0),
            ty: ValueType::Int,
        };

        match cmp_op {
            OrderedCmpOp::Eq => {
                // str_eq returns 1 if equal, 0 if not — usable directly as Bool.
                TirExpr {
                    kind: TirExprKind::ExternalCall {
                        func: eq_fn,
                        args: vec![left, right],
                    },
                    ty: ValueType::Bool,
                }
            }
            OrderedCmpOp::NotEq => {
                // str_eq(a,b) == 0 means "not equal"
                let eq_call = TirExpr {
                    kind: TirExprKind::ExternalCall {
                        func: eq_fn,
                        args: vec![left, right],
                    },
                    ty: ValueType::Int,
                };
                TirExpr {
                    kind: TirExprKind::Not(Box::new(eq_call)),
                    ty: ValueType::Bool,
                }
            }
            ordered => {
                // str_cmp(a,b) <op> 0 — all comparisons are on Int values
                let cmp_call = TirExpr {
                    kind: TirExprKind::ExternalCall {
                        func: cmp_fn,
                        args: vec![left, right],
                    },
                    ty: ValueType::Int,
                };
                let typed_op = type_rules::resolve_typed_compare(ordered, &ValueType::Int);
                TirExpr {
                    kind: typed_compare_to_kind(typed_op, cmp_call, zero),
                    ty: ValueType::Bool,
                }
            }
        }
    }

    fn promote_for_comparison(
        &self,
        line: usize,
        left: TirExpr,
        right: TirExpr,
    ) -> Result<(TirExpr, TirExpr)> {
        if left.ty == right.ty {
            if left.ty.supports_ordering() {
                Ok((left, right))
            } else {
                Err(self.type_error(
                    line,
                    format!(
                        "type `{}` does not support ordering comparisons (no `__lt__`)",
                        left.ty
                    ),
                ))
            }
        } else if matches!(
            (&left.ty, &right.ty),
            (ValueType::Int, ValueType::Float) | (ValueType::Float, ValueType::Int)
        ) {
            Ok((
                Self::apply_coercion(left, Coercion::ToFloat),
                Self::apply_coercion(right, Coercion::ToFloat),
            ))
        } else {
            Err(self.type_error(
                line,
                format!(
                    "comparison operands must have compatible types: `{}` vs `{}`",
                    left.ty, right.ty
                ),
            ))
        }
    }

    pub(in crate::tir::lower) fn compute_cast_kind(from: &ValueType, to: &ValueType) -> CastKind {
        match (from, to) {
            (ValueType::Int, ValueType::Float) => CastKind::IntToFloat,
            (ValueType::Float, ValueType::Int) => CastKind::FloatToInt,
            (ValueType::Bool, ValueType::Float) => CastKind::BoolToFloat,
            (ValueType::Int, ValueType::Bool) => CastKind::IntToBool,
            (ValueType::Float, ValueType::Bool) => CastKind::FloatToBool,
            (ValueType::Bool, ValueType::Int) => CastKind::BoolToInt,
            _ => unreachable!("identity cast should have been eliminated"),
        }
    }
}
