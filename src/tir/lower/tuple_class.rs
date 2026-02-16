use std::collections::HashMap;

use crate::ast::{ClassField, ClassInfo, ClassMethod, Type};
use crate::tir::{
    builtin, CmpIntrinsicOp, FunctionParam, IntrinsicOp, TirExpr, TirExprKind, TirFunction,
    TirStmt, ValueType,
};

use super::Lowering;

impl Lowering {
    /// Check whether a class name refers to an auto-generated tuple class.
    pub(in crate::tir::lower) fn is_tuple_class(&self, class_name: &str) -> bool {
        self.tuple_class_elements.contains_key(class_name)
    }

    /// Return the element types of a tuple class, or panic if not a tuple class.
    pub(in crate::tir::lower) fn tuple_element_types(&self, class_name: &str) -> &[ValueType] {
        &self.tuple_class_elements[class_name]
    }

    /// Generate statements that dynamically index into a homogeneous tuple class.
    ///
    /// Produces: `result_var = tuple_fields[idx_var]` as an if-else chain:
    ///   if idx == 0: result = GetField(obj, 0)
    ///   elif idx == 1: result = GetField(obj, 1)  ...
    pub(in crate::tir::lower) fn gen_tuple_dynamic_getitem_stmts(
        &self,
        result_var: &str,
        elem_ty: &ValueType,
        tuple_var: &str,
        tuple_ty: &ValueType,
        idx_var: &str,
        tuple_len: usize,
    ) -> Vec<TirStmt> {
        // Build nested if-else from the last field down to the first.
        // Match both positive indices [0..len-1] and negative indices [-len..-1].
        // Raise IndexError for anything outside that range.
        let class_name = match tuple_ty {
            ValueType::Class(n) => n.clone(),
            _ => unreachable!(),
        };
        let make_assign = |field_index: usize| -> TirStmt {
            TirStmt::Let {
                name: result_var.to_string(),
                ty: elem_ty.clone(),
                value: TirExpr {
                    kind: TirExprKind::GetField {
                        object: Box::new(TirExpr {
                            kind: TirExprKind::Var(tuple_var.to_string()),
                            ty: tuple_ty.clone(),
                        }),
                        class_name: class_name.clone(),
                        field_index,
                    },
                    ty: elem_ty.clone(),
                },
            }
        };

        let idx_expr = || TirExpr {
            kind: TirExprKind::Var(idx_var.to_string()),
            ty: ValueType::Int,
        };

        if tuple_len == 0 {
            return vec![TirStmt::Raise {
                exc_type_tag: Some(9), // IndexError
                message: Some(TirExpr {
                    kind: TirExprKind::StrLiteral("tuple index out of range".to_string()),
                    ty: ValueType::Str,
                }),
            }];
        }

        let mut else_body = vec![TirStmt::Raise {
            exc_type_tag: Some(9), // IndexError
            message: Some(TirExpr {
                kind: TirExprKind::StrLiteral("tuple index out of range".to_string()),
                ty: ValueType::Str,
            }),
        }];

        for i in (0..tuple_len).rev() {
            let pos_idx = i as i64;
            let neg_idx = pos_idx - tuple_len as i64;
            let pos_match = TirExpr {
                kind: TirExprKind::IntEq(
                    Box::new(idx_expr()),
                    Box::new(TirExpr {
                        kind: TirExprKind::IntLiteral(pos_idx),
                        ty: ValueType::Int,
                    }),
                ),
                ty: ValueType::Bool,
            };
            let neg_match = TirExpr {
                kind: TirExprKind::IntEq(
                    Box::new(idx_expr()),
                    Box::new(TirExpr {
                        kind: TirExprKind::IntLiteral(neg_idx),
                        ty: ValueType::Int,
                    }),
                ),
                ty: ValueType::Bool,
            };
            else_body = vec![TirStmt::If {
                condition: TirExpr {
                    kind: TirExprKind::LogicalOr(Box::new(pos_match), Box::new(neg_match)),
                    ty: ValueType::Bool,
                },
                then_body: vec![make_assign(i)],
                else_body,
            }];
        }

        else_body
    }

    /// Get-or-create a tuple class for the given element types.
    /// Returns the qualified class name (e.g. `__tuple$int|str|bool`).
    pub(in crate::tir::lower) fn get_or_create_tuple_class(
        &mut self,
        element_types: &[ValueType],
    ) -> String {
        let key: String = element_types
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("|");
        let class_name = format!("__tuple${}", key);

        if self.tuple_class_elements.contains_key(&class_name) {
            return class_name;
        }

        // Store element types before generating methods (avoids borrow issues)
        self.tuple_class_elements
            .insert(class_name.clone(), element_types.to_vec());

        // Build ClassInfo
        let mut fields = Vec::with_capacity(element_types.len());
        let mut field_map = HashMap::new();
        for (i, vty) in element_types.iter().enumerate() {
            let field_name = format!("_{}", i);
            fields.push(ClassField {
                name: field_name.clone(),
                ty: vty.to_type(),
                index: i,
            });
            field_map.insert(field_name, i);
        }

        let init_mangled = format!("{}$__init__", class_name);
        let new_mangled = format!("{}$new", class_name);
        let eq_mangled = format!("{}$__eq__", class_name);
        let str_mangled = format!("{}$__str__", class_name);
        let repr_mangled = format!("{}$__repr__", class_name);
        let len_mangled = format!("{}$__len__", class_name);
        let bool_mangled = format!("{}$__bool__", class_name);

        let init_params: Vec<Type> = element_types.iter().map(|vty| vty.to_type()).collect();

        let mut methods = HashMap::new();
        methods.insert(
            "__init__".to_string(),
            ClassMethod {
                name: "__init__".to_string(),
                params: init_params.clone(),
                return_type: Type::Unit,
                mangled_name: init_mangled.clone(),
            },
        );
        methods.insert(
            "new".to_string(),
            ClassMethod {
                name: "new".to_string(),
                params: init_params,
                return_type: Type::Class(class_name.clone()),
                mangled_name: new_mangled.clone(),
            },
        );
        methods.insert(
            "__eq__".to_string(),
            ClassMethod {
                name: "__eq__".to_string(),
                params: vec![Type::Class(class_name.clone())],
                return_type: Type::Bool,
                mangled_name: eq_mangled.clone(),
            },
        );
        methods.insert(
            "__str__".to_string(),
            ClassMethod {
                name: "__str__".to_string(),
                params: vec![],
                return_type: Type::Str,
                mangled_name: str_mangled.clone(),
            },
        );
        methods.insert(
            "__repr__".to_string(),
            ClassMethod {
                name: "__repr__".to_string(),
                params: vec![],
                return_type: Type::Str,
                mangled_name: repr_mangled.clone(),
            },
        );
        methods.insert(
            "__len__".to_string(),
            ClassMethod {
                name: "__len__".to_string(),
                params: vec![],
                return_type: Type::Int,
                mangled_name: len_mangled.clone(),
            },
        );
        methods.insert(
            "__bool__".to_string(),
            ClassMethod {
                name: "__bool__".to_string(),
                params: vec![],
                return_type: Type::Bool,
                mangled_name: bool_mangled.clone(),
            },
        );

        let class_info = ClassInfo {
            name: class_name.clone(),
            fields,
            methods,
            field_map,
        };

        self.class_registry
            .insert(class_name.clone(), class_info.clone());

        // Generate all TIR functions for this tuple class
        let class_vty = ValueType::Class(class_name.clone());
        let elem_types = element_types.to_vec();

        // __init__
        self.deferred_functions
            .push(Self::gen_tuple_init(&class_name, &elem_types));
        // new (factory)
        self.deferred_functions
            .push(Self::gen_tuple_new(&class_name, &elem_types));
        // __len__
        self.deferred_functions
            .push(Self::gen_tuple_len(&class_name, elem_types.len()));
        // __bool__
        self.deferred_functions
            .push(Self::gen_tuple_bool(&class_name, elem_types.len()));

        // __eq__ (needs mutable self for intrinsic registration)
        let eq_fn = self.gen_tuple_eq_method(&class_name, &elem_types);
        self.deferred_functions.push(eq_fn);

        // __repr__ (needs mutable self for pre_stmts/fresh_internal)
        let repr_fn = self.gen_tuple_repr(&class_name, &elem_types);
        self.deferred_functions.push(repr_fn);

        // __str__ — delegates to __repr__ for tuples (Python semantics)
        self.deferred_functions.push(TirFunction {
            name: str_mangled,
            params: vec![FunctionParam::new("self".to_string(), class_vty.clone())],
            return_type: Some(ValueType::Str),
            body: vec![TirStmt::Return(Some(TirExpr {
                kind: TirExprKind::Call {
                    func: repr_mangled,
                    args: vec![TirExpr {
                        kind: TirExprKind::Var("self".to_string()),
                        ty: class_vty.clone(),
                    }],
                },
                ty: ValueType::Str,
            }))],
        });

        // Register this tuple class as a deferred class so it appears in the output
        self.deferred_classes.push(class_info);

        class_name
    }

    // ── __init__ ──────────────────────────────────────────────────────

    fn gen_tuple_init(class_name: &str, elem_types: &[ValueType]) -> TirFunction {
        let class_vty = ValueType::Class(class_name.to_string());
        let mangled = format!("{}$__init__", class_name);

        let mut params = vec![FunctionParam::new("self".to_string(), class_vty.clone())];
        for (i, vty) in elem_types.iter().enumerate() {
            params.push(FunctionParam::new(format!("_{}", i), vty.clone()));
        }

        let mut body = Vec::new();
        for (i, vty) in elem_types.iter().enumerate() {
            body.push(TirStmt::SetField {
                object: TirExpr {
                    kind: TirExprKind::Var("self".to_string()),
                    ty: class_vty.clone(),
                },
                class_name: class_name.to_string(),
                field_index: i,
                value: TirExpr {
                    kind: TirExprKind::Var(format!("_{}", i)),
                    ty: vty.clone(),
                },
            });
        }

        TirFunction {
            name: mangled,
            params,
            return_type: None,
            body,
        }
    }

    // ── new (factory) ─────────────────────────────────────────────────

    fn gen_tuple_new(class_name: &str, elem_types: &[ValueType]) -> TirFunction {
        let class_vty = ValueType::Class(class_name.to_string());
        let new_mangled = format!("{}$new", class_name);
        let init_mangled = format!("{}$__init__", class_name);

        let params: Vec<FunctionParam> = elem_types
            .iter()
            .enumerate()
            .map(|(i, vty)| FunctionParam::new(format!("_{}", i), vty.clone()))
            .collect();

        let args: Vec<TirExpr> = params
            .iter()
            .map(|p| TirExpr {
                kind: TirExprKind::Var(p.name.clone()),
                ty: p.ty.clone(),
            })
            .collect();

        TirFunction {
            name: new_mangled,
            params,
            return_type: Some(class_vty.clone()),
            body: vec![TirStmt::Return(Some(TirExpr {
                kind: TirExprKind::Construct {
                    class_name: class_name.to_string(),
                    init_mangled_name: init_mangled,
                    args,
                },
                ty: class_vty,
            }))],
        }
    }

    // ── __len__ ───────────────────────────────────────────────────────

    fn gen_tuple_len(class_name: &str, len: usize) -> TirFunction {
        let class_vty = ValueType::Class(class_name.to_string());
        TirFunction {
            name: format!("{}$__len__", class_name),
            params: vec![FunctionParam::new("self".to_string(), class_vty)],
            return_type: Some(ValueType::Int),
            body: vec![TirStmt::Return(Some(TirExpr {
                kind: TirExprKind::IntLiteral(len as i64),
                ty: ValueType::Int,
            }))],
        }
    }

    // ── __bool__ ──────────────────────────────────────────────────────

    fn gen_tuple_bool(class_name: &str, len: usize) -> TirFunction {
        let class_vty = ValueType::Class(class_name.to_string());
        TirFunction {
            name: format!("{}$__bool__", class_name),
            params: vec![FunctionParam::new("self".to_string(), class_vty)],
            return_type: Some(ValueType::Bool),
            body: vec![TirStmt::Return(Some(TirExpr {
                kind: TirExprKind::BoolLiteral(len > 0),
                ty: ValueType::Bool,
            }))],
        }
    }

    // ── __eq__ ────────────────────────────────────────────────────────

    fn gen_tuple_eq_method(&mut self, class_name: &str, elem_types: &[ValueType]) -> TirFunction {
        let class_vty = ValueType::Class(class_name.to_string());
        let mangled = format!("{}$__eq__", class_name);

        let params = vec![
            FunctionParam::new("self".to_string(), class_vty.clone()),
            FunctionParam::new("other".to_string(), class_vty.clone()),
        ];

        // Empty tuple: always equal
        if elem_types.is_empty() {
            return TirFunction {
                name: mangled,
                params,
                return_type: Some(ValueType::Bool),
                body: vec![TirStmt::Return(Some(TirExpr {
                    kind: TirExprKind::BoolLiteral(true),
                    ty: ValueType::Bool,
                }))],
            };
        }

        // Build element-wise comparisons for each tuple field in order.
        let mut comparisons = Vec::new();
        for (i, elem_ty) in elem_types.iter().enumerate() {
            let self_field = TirExpr {
                kind: TirExprKind::GetField {
                    object: Box::new(TirExpr {
                        kind: TirExprKind::Var("self".to_string()),
                        ty: class_vty.clone(),
                    }),
                    class_name: class_name.to_string(),
                    field_index: i,
                },
                ty: elem_ty.clone(),
            };
            let other_field = TirExpr {
                kind: TirExprKind::GetField {
                    object: Box::new(TirExpr {
                        kind: TirExprKind::Var("other".to_string()),
                        ty: class_vty.clone(),
                    }),
                    class_name: class_name.to_string(),
                    field_index: i,
                },
                ty: elem_ty.clone(),
            };
            comparisons.push(self.gen_equality_expr(self_field, other_field, elem_ty));
        }

        // Chain with LogicalAnd
        let mut result = comparisons.remove(0);
        for cmp in comparisons {
            result = TirExpr {
                kind: TirExprKind::LogicalAnd(Box::new(result), Box::new(cmp)),
                ty: ValueType::Bool,
            };
        }

        TirFunction {
            name: mangled,
            params,
            return_type: Some(ValueType::Bool),
            body: vec![TirStmt::Return(Some(result))],
        }
    }

    /// Generate an equality expression for two values of the given type.
    fn gen_equality_expr(&mut self, left: TirExpr, right: TirExpr, ty: &ValueType) -> TirExpr {
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
            ValueType::List(inner) => {
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
            ValueType::Class(_) => {
                self.register_intrinsic_instance(IntrinsicOp::Eq, ty);
                TirExpr {
                    kind: TirExprKind::IntrinsicCmp {
                        op: CmpIntrinsicOp::Eq,
                        lhs: Box::new(left),
                        rhs: Box::new(right),
                    },
                    ty: ValueType::Bool,
                }
            }
            // For other types, use intrinsic dispatch as fallback
            _ => {
                self.register_intrinsic_instance(IntrinsicOp::Eq, ty);
                TirExpr {
                    kind: TirExprKind::IntrinsicCmp {
                        op: CmpIntrinsicOp::Eq,
                        lhs: Box::new(left),
                        rhs: Box::new(right),
                    },
                    ty: ValueType::Bool,
                }
            }
        }
    }

    // ── __repr__ ──────────────────────────────────────────────────────

    fn gen_tuple_repr(&mut self, class_name: &str, elem_types: &[ValueType]) -> TirFunction {
        let class_vty = ValueType::Class(class_name.to_string());
        let mangled = format!("{}$__repr__", class_name);

        let params = vec![FunctionParam::new("self".to_string(), class_vty.clone())];

        // Empty tuple: "()"
        if elem_types.is_empty() {
            return TirFunction {
                name: mangled,
                params,
                return_type: Some(ValueType::Str),
                body: vec![TirStmt::Return(Some(TirExpr {
                    kind: TirExprKind::StrLiteral("()".to_string()),
                    ty: ValueType::Str,
                }))],
            };
        }

        let result_var = self.fresh_internal("tuple_repr");
        let mut body = Vec::new();

        // let result = "("
        body.push(TirStmt::Let {
            name: result_var.clone(),
            ty: ValueType::Str,
            value: TirExpr {
                kind: TirExprKind::StrLiteral("(".to_string()),
                ty: ValueType::Str,
            },
        });

        for (i, elem_ty) in elem_types.iter().enumerate() {
            if i > 0 {
                body.push(TirStmt::Let {
                    name: result_var.clone(),
                    ty: ValueType::Str,
                    value: TirExpr {
                        kind: TirExprKind::ExternalCall {
                            func: builtin::BuiltinFn::StrConcat,
                            args: vec![
                                TirExpr {
                                    kind: TirExprKind::Var(result_var.clone()),
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

            let field_expr = TirExpr {
                kind: TirExprKind::GetField {
                    object: Box::new(TirExpr {
                        kind: TirExprKind::Var("self".to_string()),
                        ty: class_vty.clone(),
                    }),
                    class_name: class_name.to_string(),
                    field_index: i,
                },
                ty: elem_ty.clone(),
            };

            let repr_expr = self
                .lower_repr_str_expr(0, field_expr)
                .expect("ICE: all tuple element types support __repr__");

            body.push(TirStmt::Let {
                name: result_var.clone(),
                ty: ValueType::Str,
                value: TirExpr {
                    kind: TirExprKind::ExternalCall {
                        func: builtin::BuiltinFn::StrConcat,
                        args: vec![
                            TirExpr {
                                kind: TirExprKind::Var(result_var.clone()),
                                ty: ValueType::Str,
                            },
                            repr_expr,
                        ],
                    },
                    ty: ValueType::Str,
                },
            });
        }

        // Trailing comma for single-element tuples, then close paren
        let suffix = if elem_types.len() == 1 { ",)" } else { ")" };
        body.push(TirStmt::Let {
            name: result_var.clone(),
            ty: ValueType::Str,
            value: TirExpr {
                kind: TirExprKind::ExternalCall {
                    func: builtin::BuiltinFn::StrConcat,
                    args: vec![
                        TirExpr {
                            kind: TirExprKind::Var(result_var.clone()),
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

        // return result
        body.push(TirStmt::Return(Some(TirExpr {
            kind: TirExprKind::Var(result_var),
            ty: ValueType::Str,
        })));

        TirFunction {
            name: mangled,
            params,
            return_type: Some(ValueType::Str),
            body,
        }
    }
}
