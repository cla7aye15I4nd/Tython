use anyhow::Result;

use crate::tir::{
    builtin, type_rules, CastKind, CmpOp, OrderedCmpOp, TirExpr, TirExprKind, TirStmt,
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
                ValueType::List(_inner) => TirExpr {
                    kind: TirExprKind::ExternalCall {
                        func: builtin::BuiltinFn::ListContains,
                        args: vec![right, left],
                    },
                    ty: ValueType::Bool,
                },
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

        // List equality
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
            if cmp_op != CmpOp::Eq && cmp_op != CmpOp::NotEq {
                return Err(
                    self.type_error(line, "only `==` and `!=` are supported for list comparison")
                );
            }
            let eq_expr = self.generate_list_eq(left, right);
            if cmp_op == CmpOp::NotEq {
                return Ok(TirExpr {
                    kind: TirExprKind::Not(Box::new(eq_expr)),
                    ty: ValueType::Bool,
                });
            }
            return Ok(eq_expr);
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
    /// For primitive inner types, uses ListEqShallow.
    /// For nested lists with primitive leaves, uses ListEqDeep.
    /// For complex inner types (tuples), generates a comparison loop.
    fn generate_list_eq(&mut self, left: TirExpr, right: TirExpr) -> TirExpr {
        let inner = match &left.ty {
            ValueType::List(inner) => (**inner).clone(),
            _ => unreachable!(),
        };

        // Check if we can use shallow comparison (primitive elements)
        if matches!(inner, ValueType::Int | ValueType::Float | ValueType::Bool) {
            return TirExpr {
                kind: TirExprKind::ExternalCall {
                    func: builtin::BuiltinFn::ListEqShallow,
                    args: vec![left, right],
                },
                ty: ValueType::Bool,
            };
        }

        // Check for nested lists → ListEqDeep with computed depth
        if let Some(depth) = Self::list_nesting_depth(&inner) {
            return TirExpr {
                kind: TirExprKind::ExternalCall {
                    func: builtin::BuiltinFn::ListEqDeep,
                    args: vec![
                        left,
                        right,
                        TirExpr {
                            kind: TirExprKind::IntLiteral(depth),
                            ty: ValueType::Int,
                        },
                    ],
                },
                ty: ValueType::Bool,
            };
        }

        // Complex inner type (tuples, etc.) — generate comparison loop
        self.generate_list_eq_loop(left, right, &inner)
    }

    /// Returns Some(depth) if the type is a list nesting ending in primitives.
    /// e.g., list[int] → Some(0), list[list[int]] → Some(1), list[list[list[int]]] → Some(2)
    /// Returns None for non-list or lists containing non-primitive leaves.
    fn list_nesting_depth(ty: &ValueType) -> Option<i64> {
        match ty {
            ValueType::Int | ValueType::Float | ValueType::Bool => Some(0),
            ValueType::List(inner) => Self::list_nesting_depth(inner).map(|d| d + 1),
            _ => None,
        }
    }

    /// Generate a loop-based list equality check for complex inner types.
    /// Pushes setup + loop stmts into self.pre_stmts, returns a Var expression.
    fn generate_list_eq_loop(
        &mut self,
        left: TirExpr,
        right: TirExpr,
        inner_ty: &ValueType,
    ) -> TirExpr {
        let result_var = self.fresh_internal("listeq_res");
        let left_var = self.fresh_internal("listeq_a");
        let right_var = self.fresh_internal("listeq_b");
        let len_a_var = self.fresh_internal("listeq_len_a");
        let len_b_var = self.fresh_internal("listeq_len_b");
        let idx_var = self.fresh_internal("listeq_idx");
        let stop_var = self.fresh_internal("listeq_stop");
        let step_var = self.fresh_internal("listeq_step");
        let start_var = self.fresh_internal("listeq_start");
        let elem_a_var = self.fresh_internal("listeq_ea");
        let elem_b_var = self.fresh_internal("listeq_eb");

        let left_ty = left.ty.clone();
        let right_ty = right.ty.clone();

        // Let result = 1 (true)
        let mut stmts = vec![
            TirStmt::Let {
                name: result_var.clone(),
                ty: ValueType::Bool,
                value: TirExpr {
                    kind: TirExprKind::BoolLiteral(true),
                    ty: ValueType::Bool,
                },
            },
            TirStmt::Let {
                name: left_var.clone(),
                ty: left_ty.clone(),
                value: left,
            },
            TirStmt::Let {
                name: right_var.clone(),
                ty: right_ty.clone(),
                value: right,
            },
            // len_a = list_len(left)
            TirStmt::Let {
                name: len_a_var.clone(),
                ty: ValueType::Int,
                value: TirExpr {
                    kind: TirExprKind::ExternalCall {
                        func: builtin::BuiltinFn::ListLen,
                        args: vec![TirExpr {
                            kind: TirExprKind::Var(left_var.clone()),
                            ty: left_ty.clone(),
                        }],
                    },
                    ty: ValueType::Int,
                },
            },
            // len_b = list_len(right)
            TirStmt::Let {
                name: len_b_var.clone(),
                ty: ValueType::Int,
                value: TirExpr {
                    kind: TirExprKind::ExternalCall {
                        func: builtin::BuiltinFn::ListLen,
                        args: vec![TirExpr {
                            kind: TirExprKind::Var(right_var.clone()),
                            ty: right_ty.clone(),
                        }],
                    },
                    ty: ValueType::Int,
                },
            },
        ];

        // Build the element comparison expression
        let elem_a_expr = TirExpr {
            kind: TirExprKind::Var(elem_a_var.clone()),
            ty: inner_ty.clone(),
        };
        let elem_b_expr = TirExpr {
            kind: TirExprKind::Var(elem_b_var.clone()),
            ty: inner_ty.clone(),
        };
        let elem_eq = self.generate_equality_check(elem_a_expr, elem_b_expr, inner_ty);
        // Drain any pre_stmts generated by nested equality checks
        let nested_pre = std::mem::take(&mut self.pre_stmts);

        // for i in range(0, len_a):
        //   elem_a = list_get(left, i)
        //   elem_b = list_get(right, i)
        //   if !(elem_a == elem_b): result = 0; break
        let idx_expr = TirExpr {
            kind: TirExprKind::Var(idx_var.clone()),
            ty: ValueType::Int,
        };

        let mut loop_body = vec![
            TirStmt::Let {
                name: elem_a_var,
                ty: inner_ty.clone(),
                value: TirExpr {
                    kind: TirExprKind::ExternalCall {
                        func: builtin::BuiltinFn::ListGet,
                        args: vec![
                            TirExpr {
                                kind: TirExprKind::Var(left_var),
                                ty: left_ty,
                            },
                            idx_expr.clone(),
                        ],
                    },
                    ty: inner_ty.clone(),
                },
            },
            TirStmt::Let {
                name: elem_b_var,
                ty: inner_ty.clone(),
                value: TirExpr {
                    kind: TirExprKind::ExternalCall {
                        func: builtin::BuiltinFn::ListGet,
                        args: vec![
                            TirExpr {
                                kind: TirExprKind::Var(right_var),
                                ty: right_ty,
                            },
                            idx_expr,
                        ],
                    },
                    ty: inner_ty.clone(),
                },
            },
        ];
        // Add any nested pre_stmts
        loop_body.extend(nested_pre);
        // if not eq: result = false; break
        loop_body.push(TirStmt::If {
            condition: TirExpr {
                kind: TirExprKind::Not(Box::new(elem_eq)),
                ty: ValueType::Bool,
            },
            then_body: vec![
                TirStmt::Let {
                    name: result_var.clone(),
                    ty: ValueType::Bool,
                    value: TirExpr {
                        kind: TirExprKind::BoolLiteral(false),
                        ty: ValueType::Bool,
                    },
                },
                TirStmt::Break,
            ],
            else_body: vec![],
        });

        // if len_a != len_b: result = 0
        // else: for loop
        stmts.push(TirStmt::If {
            condition: TirExpr {
                kind: TirExprKind::IntNotEq(
                    Box::new(TirExpr {
                        kind: TirExprKind::Var(len_a_var),
                        ty: ValueType::Int,
                    }),
                    Box::new(TirExpr {
                        kind: TirExprKind::Var(len_b_var.clone()),
                        ty: ValueType::Int,
                    }),
                ),
                ty: ValueType::Bool,
            },
            then_body: vec![TirStmt::Let {
                name: result_var.clone(),
                ty: ValueType::Bool,
                value: TirExpr {
                    kind: TirExprKind::BoolLiteral(false),
                    ty: ValueType::Bool,
                },
            }],
            else_body: vec![
                TirStmt::Let {
                    name: start_var.clone(),
                    ty: ValueType::Int,
                    value: TirExpr {
                        kind: TirExprKind::IntLiteral(0),
                        ty: ValueType::Int,
                    },
                },
                TirStmt::Let {
                    name: stop_var.clone(),
                    ty: ValueType::Int,
                    value: TirExpr {
                        kind: TirExprKind::Var(len_b_var),
                        ty: ValueType::Int,
                    },
                },
                TirStmt::Let {
                    name: step_var.clone(),
                    ty: ValueType::Int,
                    value: TirExpr {
                        kind: TirExprKind::IntLiteral(1),
                        ty: ValueType::Int,
                    },
                },
                TirStmt::ForRange {
                    loop_var: idx_var,
                    start_var,
                    stop_var,
                    step_var,
                    body: loop_body,
                    else_body: vec![],
                },
            ],
        });

        self.pre_stmts.extend(stmts);

        TirExpr {
            kind: TirExprKind::Var(result_var),
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
