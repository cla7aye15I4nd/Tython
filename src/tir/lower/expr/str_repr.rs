use crate::tir::{builtin, TirExpr, TirExprKind, TirStmt, ValueType};

use crate::tir::lower::Lowering;

impl Lowering {
    /// Auto-generate str() for composite types (list, tuple).
    /// Pushes setup statements into `self.pre_stmts` and returns a
    /// `Var` expression pointing to the result string.
    pub(in crate::tir::lower) fn lower_str_auto(&mut self, arg: TirExpr) -> TirExpr {
        let result_var = self.fresh_internal("str_auto");
        let arg_ty = arg.ty.clone();
        match &arg_ty {
            ValueType::List(inner) => {
                self.lower_str_list(&result_var, arg, inner.as_ref().clone());
            }
            ValueType::Tuple(element_types) => {
                self.lower_str_tuple(&result_var, arg, element_types.clone());
            }
            _ => unreachable!("ICE: lower_str_auto called on non-composite type"),
        }
        TirExpr {
            kind: TirExprKind::Var(result_var),
            ty: ValueType::Str,
        }
    }

    /// Generate string building code for a list.
    /// Produces: result_var = "[" + join(repr(elem), ", ") + "]"
    fn lower_str_list(&mut self, result_var: &str, list_arg: TirExpr, elem_ty: ValueType) {
        let list_var = self.fresh_internal("str_list");
        let idx_var = self.fresh_internal("str_idx");
        let len_var = self.fresh_internal("str_len");
        let loop_var = self.fresh_internal("str_elem");
        let list_ty = list_arg.ty.clone();

        // let list_var = <list_arg>
        self.pre_stmts.push(TirStmt::Let {
            name: list_var.clone(),
            ty: list_ty,
            value: list_arg,
        });

        // let result_var = "["
        self.pre_stmts.push(TirStmt::Let {
            name: result_var.to_string(),
            ty: ValueType::Str,
            value: TirExpr {
                kind: TirExprKind::StrLiteral("[".to_string()),
                ty: ValueType::Str,
            },
        });

        // Build loop body
        let mut body = Vec::new();

        // if idx > 0: result_var = result_var + ", "
        let idx_gt_zero = TirExpr {
            kind: TirExprKind::IntGt(
                Box::new(TirExpr {
                    kind: TirExprKind::Var(idx_var.clone()),
                    ty: ValueType::Int,
                }),
                Box::new(TirExpr {
                    kind: TirExprKind::IntLiteral(0),
                    ty: ValueType::Int,
                }),
            ),
            ty: ValueType::Bool,
        };
        body.push(TirStmt::If {
            condition: idx_gt_zero,
            then_body: vec![TirStmt::Let {
                name: result_var.to_string(),
                ty: ValueType::Str,
                value: TirExpr {
                    kind: TirExprKind::ExternalCall {
                        func: builtin::BuiltinFn::StrConcat,
                        args: vec![
                            TirExpr {
                                kind: TirExprKind::Var(result_var.to_string()),
                                ty: ValueType::Str,
                            },
                            TirExpr {
                                kind: TirExprKind::StrLiteral(", ".to_string()),
                                ty: ValueType::Str,
                            },
                        ],
                    },
                    ty: ValueType::Str,
                },
            }],
            else_body: vec![],
        });

        // result_var = result_var + repr(elem)
        // Save/restore pre_stmts so that any nested str_auto code
        // (e.g. for list[list[int]]) goes into the loop body, not the outer scope.
        let saved_pre = std::mem::take(&mut self.pre_stmts);
        let elem_expr = TirExpr {
            kind: TirExprKind::Var(loop_var.clone()),
            ty: elem_ty.clone(),
        };
        let repr_expr = self.lower_repr_str_expr(elem_expr);
        let nested_pre = std::mem::take(&mut self.pre_stmts);
        self.pre_stmts = saved_pre;
        body.extend(nested_pre);
        body.push(TirStmt::Let {
            ty: ValueType::Str,
            name: result_var.to_string(),
            value: TirExpr {
                kind: TirExprKind::ExternalCall {
                    func: builtin::BuiltinFn::StrConcat,
                    args: vec![
                        TirExpr {
                            kind: TirExprKind::Var(result_var.to_string()),
                            ty: ValueType::Str,
                        },
                        repr_expr,
                    ],
                },
                ty: ValueType::Str,
            },
        });

        self.pre_stmts.push(TirStmt::ForList {
            loop_var,
            loop_var_ty: elem_ty,
            list_var,
            index_var: idx_var,
            len_var,
            body,
            else_body: vec![],
        });

        // result_var = result_var + "]"
        self.pre_stmts.push(TirStmt::Let {
            ty: ValueType::Str,
            name: result_var.to_string(),
            value: TirExpr {
                kind: TirExprKind::ExternalCall {
                    func: builtin::BuiltinFn::StrConcat,
                    args: vec![
                        TirExpr {
                            kind: TirExprKind::Var(result_var.to_string()),
                            ty: ValueType::Str,
                        },
                        TirExpr {
                            kind: TirExprKind::StrLiteral("]".to_string()),
                            ty: ValueType::Str,
                        },
                    ],
                },
                ty: ValueType::Str,
            },
        });
    }

    /// Generate string building code for a tuple.
    /// Produces: result_var = "(" + join(repr(elem), ", ") + ")"
    fn lower_str_tuple(
        &mut self,
        result_var: &str,
        tuple_arg: TirExpr,
        element_types: Vec<ValueType>,
    ) {
        let tuple_var = self.fresh_internal("str_tuple");
        let tuple_ty = tuple_arg.ty.clone();

        // let tuple_var = <tuple_arg>
        self.pre_stmts.push(TirStmt::Let {
            name: tuple_var.clone(),
            ty: tuple_ty.clone(),
            value: tuple_arg,
        });

        // let result_var = "("
        self.pre_stmts.push(TirStmt::Let {
            name: result_var.to_string(),
            ty: ValueType::Str,
            value: TirExpr {
                kind: TirExprKind::StrLiteral("(".to_string()),
                ty: ValueType::Str,
            },
        });

        for (i, elem_ty) in element_types.iter().enumerate() {
            if i > 0 {
                // result_var = result_var + ", "
                self.pre_stmts.push(TirStmt::Let {
                    ty: ValueType::Str,
                    name: result_var.to_string(),
                    value: TirExpr {
                        kind: TirExprKind::ExternalCall {
                            func: builtin::BuiltinFn::StrConcat,
                            args: vec![
                                TirExpr {
                                    kind: TirExprKind::Var(result_var.to_string()),
                                    ty: ValueType::Str,
                                },
                                TirExpr {
                                    kind: TirExprKind::StrLiteral(", ".to_string()),
                                    ty: ValueType::Str,
                                },
                            ],
                        },
                        ty: ValueType::Str,
                    },
                });
            }

            let elem_expr = TirExpr {
                kind: TirExprKind::GetField {
                    object: Box::new(TirExpr {
                        kind: TirExprKind::Var(tuple_var.clone()),
                        ty: tuple_ty.clone(),
                    }),
                    field_index: i,
                },
                ty: elem_ty.clone(),
            };
            let repr_expr = self.lower_repr_str_expr(elem_expr);

            // result_var = result_var + repr(elem)
            self.pre_stmts.push(TirStmt::Let {
                name: result_var.to_string(),
                ty: ValueType::Str,
                value: TirExpr {
                    kind: TirExprKind::ExternalCall {
                        func: builtin::BuiltinFn::StrConcat,
                        args: vec![
                            TirExpr {
                                kind: TirExprKind::Var(result_var.to_string()),
                                ty: ValueType::Str,
                            },
                            repr_expr,
                        ],
                    },
                    ty: ValueType::Str,
                },
            });
        }

        // Trailing comma for single-element tuples
        let suffix = if element_types.len() == 1 { ",)" } else { ")" };
        self.pre_stmts.push(TirStmt::Let {
            ty: ValueType::Str,
            name: result_var.to_string(),
            value: TirExpr {
                kind: TirExprKind::ExternalCall {
                    func: builtin::BuiltinFn::StrConcat,
                    args: vec![
                        TirExpr {
                            kind: TirExprKind::Var(result_var.to_string()),
                            ty: ValueType::Str,
                        },
                        TirExpr {
                            kind: TirExprKind::StrLiteral(suffix.to_string()),
                            ty: ValueType::Str,
                        },
                    ],
                },
                ty: ValueType::Str,
            },
        });
    }

    /// Return a TirExpr of type Str representing the repr of a value.
    /// For composite types, generates code via pre_stmts.
    pub(in crate::tir::lower) fn lower_repr_str_expr(&mut self, arg: TirExpr) -> TirExpr {
        match &arg.ty {
            ValueType::Int => TirExpr {
                kind: TirExprKind::ExternalCall {
                    func: builtin::BuiltinFn::StrFromInt,
                    args: vec![arg],
                },
                ty: ValueType::Str,
            },
            ValueType::Float => TirExpr {
                kind: TirExprKind::ExternalCall {
                    func: builtin::BuiltinFn::StrFromFloat,
                    args: vec![arg],
                },
                ty: ValueType::Str,
            },
            ValueType::Bool => TirExpr {
                kind: TirExprKind::ExternalCall {
                    func: builtin::BuiltinFn::StrFromBool,
                    args: vec![arg],
                },
                ty: ValueType::Str,
            },
            ValueType::Str => TirExpr {
                kind: TirExprKind::ExternalCall {
                    func: builtin::BuiltinFn::ReprStr,
                    args: vec![arg],
                },
                ty: ValueType::Str,
            },
            ValueType::Bytes => TirExpr {
                kind: TirExprKind::ExternalCall {
                    func: builtin::BuiltinFn::StrFromBytes,
                    args: vec![arg],
                },
                ty: ValueType::Str,
            },
            ValueType::ByteArray => TirExpr {
                kind: TirExprKind::ExternalCall {
                    func: builtin::BuiltinFn::StrFromByteArray,
                    args: vec![arg],
                },
                ty: ValueType::Str,
            },
            ValueType::List(_) | ValueType::Tuple(_) => {
                // Recursive: auto-generate str for nested composite types
                self.lower_str_auto(arg)
            }
            _ => {
                // Fallback: use str() conversion if available
                TirExpr {
                    kind: TirExprKind::StrLiteral("<?>".to_string()),
                    ty: ValueType::Str,
                }
            }
        }
    }
}
