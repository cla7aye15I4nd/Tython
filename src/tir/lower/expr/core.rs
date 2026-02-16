use anyhow::Result;
use pyo3::prelude::*;

use super::unaryops::class_unary_magic;
use crate::ast::Type;
use crate::tir::{
    builtin, CallResult, CallTarget, IntrinsicOp, LogicalOp, TirExpr, TirExprKind, TirStmt,
    ValueType,
};
use crate::{ast_get_list, ast_get_string, ast_getattr, ast_type_name};

use crate::tir::lower::Lowering;

impl Lowering {
    pub(in crate::tir::lower) fn lower_expr(&mut self, node: &Bound<PyAny>) -> Result<TirExpr> {
        let node_type = ast_type_name!(node);
        let line = Self::get_line(node);

        match node_type.as_str() {
            "Constant" => {
                let value = ast_getattr!(node, "value");
                if value.is_instance_of::<pyo3::types::PyBool>() {
                    let bool_val = value.extract::<bool>()?;
                    Ok(TirExpr {
                        kind: TirExprKind::BoolLiteral(bool_val),
                        ty: ValueType::Bool,
                    })
                } else if value.is_instance_of::<pyo3::types::PyBytes>() {
                    let bytes_val = value.extract::<Vec<u8>>()?;
                    Ok(TirExpr {
                        kind: TirExprKind::BytesLiteral(bytes_val),
                        ty: ValueType::Bytes,
                    })
                } else if let Ok(int_val) = value.extract::<i64>() {
                    Ok(TirExpr {
                        kind: TirExprKind::IntLiteral(int_val),
                        ty: ValueType::Int,
                    })
                } else if let Ok(float_val) = value.extract::<f64>() {
                    Ok(TirExpr {
                        kind: TirExprKind::FloatLiteral(float_val),
                        ty: ValueType::Float,
                    })
                } else if let Ok(string_val) = value.extract::<String>() {
                    Ok(TirExpr {
                        kind: TirExprKind::StrLiteral(string_val),
                        ty: ValueType::Str,
                    })
                } else {
                    Err(self.value_error(line, "unsupported constant type"))
                }
            }

            "Name" => {
                let id = ast_get_string!(node, "id");
                let ty = self
                    .lookup(&id)
                    .cloned()
                    .ok_or_else(|| self.name_error(line, format!("undefined variable `{}`", id)))?;

                if matches!(ty, Type::Module(_)) {
                    return Err(self.type_error(
                        line,
                        format!("module `{}` cannot be used as a value expression", ty),
                    ));
                }

                let vty = self.value_type_from_type(&ty);
                Ok(TirExpr {
                    kind: TirExprKind::Var(id),
                    ty: vty,
                })
            }

            "BinOp" => {
                let left = self.lower_expr(&ast_getattr!(node, "left"))?;
                let right = self.lower_expr(&ast_getattr!(node, "right"))?;
                let raw_op = Self::convert_binop(&ast_getattr!(node, "op"))?;
                self.resolve_binop(line, raw_op, left, right)
            }

            "Compare" => {
                let left = self.lower_expr(&ast_getattr!(node, "left"))?;
                let ops_list = ast_get_list!(node, "ops");
                let comparators_list = ast_get_list!(node, "comparators");
                let is_empty_list_literal = |n: &Bound<PyAny>| -> bool {
                    ast_type_name!(n) == "List" && ast_get_list!(n, "elts").is_empty()
                };

                if ops_list.len() == 1 {
                    let op_node = ops_list.get_item(0)?;
                    let cmp_op = Self::convert_cmpop(&op_node)?;
                    let right_node = comparators_list.get_item(0)?;
                    let right = if is_empty_list_literal(&right_node)
                        && matches!(left.ty, ValueType::List(_))
                    {
                        let saved_empty_list_hint = self.empty_list_hint.clone();
                        self.empty_list_hint = Some(left.ty.clone());
                        let lowered = self.lower_expr(&right_node)?;
                        self.empty_list_hint = saved_empty_list_hint;
                        lowered
                    } else {
                        self.lower_expr(&right_node)?
                    };
                    return self.lower_single_comparison(line, cmp_op, left, right);
                }

                let mut comparisons: Vec<TirExpr> = Vec::new();
                let mut current_left = left;

                for i in 0..ops_list.len() {
                    let op_node = ops_list.get_item(i)?;
                    let cmp_op = Self::convert_cmpop(&op_node)?;
                    let right_node = comparators_list.get_item(i)?;
                    let right = if is_empty_list_literal(&right_node)
                        && matches!(current_left.ty, ValueType::List(_))
                    {
                        let saved_empty_list_hint = self.empty_list_hint.clone();
                        self.empty_list_hint = Some(current_left.ty.clone());
                        let lowered = self.lower_expr(&right_node)?;
                        self.empty_list_hint = saved_empty_list_hint;
                        lowered
                    } else {
                        self.lower_expr(&right_node)?
                    };

                    comparisons.push(self.lower_single_comparison(
                        line,
                        cmp_op,
                        current_left.clone(),
                        right.clone(),
                    )?);

                    current_left = right;
                }

                let mut result = comparisons.remove(0);
                for cmp in comparisons {
                    result = TirExpr {
                        kind: TirExprKind::LogicalAnd(Box::new(result), Box::new(cmp)),
                        ty: ValueType::Bool,
                    };
                }

                Ok(result)
            }

            "UnaryOp" => {
                let op_node = ast_getattr!(node, "op");
                let op_type = ast_type_name!(op_node);
                let operand = self.lower_expr(&ast_getattr!(node, "operand"))?;

                let op = Self::convert_unaryop(&op_type);

                use crate::tir::UnaryOpKind::*;

                // `not` — truthiness for all types
                if op == Not {
                    if matches!(operand.ty, ValueType::Class(_)) {
                        let (method_name, expected_return_type, negate_result) =
                            class_unary_magic(op);
                        let mut expr = self.lower_class_magic_method_with_args(
                            line,
                            operand,
                            &[method_name],
                            expected_return_type,
                            "unary operator",
                            vec![],
                        )?;
                        if negate_result {
                            expr = TirExpr {
                                kind: TirExprKind::Not(Box::new(expr)),
                                ty: ValueType::Bool,
                            };
                        }
                        return Ok(expr);
                    }
                    let bool_expr =
                        self.lower_truthy_to_bool(line, operand, "unary `not` operand")?;
                    return Ok(TirExpr {
                        kind: TirExprKind::Not(Box::new(bool_expr)),
                        ty: ValueType::Bool,
                    });
                }

                // Primitive direct lowering
                match (op, &operand.ty) {
                    (Pos, ValueType::Int | ValueType::Float) => Ok(operand),
                    (Neg, ValueType::Int) => Ok(TirExpr {
                        kind: TirExprKind::IntNeg(Box::new(operand)),
                        ty: ValueType::Int,
                    }),
                    (Neg, ValueType::Float) => Ok(TirExpr {
                        kind: TirExprKind::FloatNeg(Box::new(operand)),
                        ty: ValueType::Float,
                    }),
                    (BitNot, ValueType::Int) => Ok(TirExpr {
                        kind: TirExprKind::BitNot(Box::new(operand)),
                        ty: ValueType::Int,
                    }),
                    // Non-primitive → method dispatch (__neg__, __pos__, __invert__)
                    _ => {
                        let (method_name, expected_return_type, _) = class_unary_magic(op);
                        self.dispatch_unary_method(line, operand, method_name, expected_return_type)
                    }
                }
            }

            "BoolOp" => {
                let op_node = ast_getattr!(node, "op");
                let op_type = ast_type_name!(op_node);
                let values_list = ast_get_list!(node, "values");

                let logical_op = match op_type.as_str() {
                    "And" => LogicalOp::And,
                    "Or" => LogicalOp::Or,
                    _ => {
                        return Err(self.syntax_error(
                            line,
                            format!("unsupported logical operator: `{}`", op_type),
                        ))
                    }
                };

                let mut exprs: Vec<TirExpr> = Vec::new();
                for val in values_list.iter() {
                    let raw = self.lower_expr(&val)?;
                    exprs.push(self.lower_truthy_to_bool(line, raw, "logical operator")?);
                }

                let mut result = exprs.remove(0);
                for operand in exprs {
                    result = TirExpr {
                        kind: match logical_op {
                            LogicalOp::And => {
                                TirExprKind::LogicalAnd(Box::new(result), Box::new(operand))
                            }
                            LogicalOp::Or => {
                                TirExprKind::LogicalOr(Box::new(result), Box::new(operand))
                            }
                        },
                        ty: ValueType::Bool,
                    };
                }

                Ok(result)
            }

            "Call" => match self.lower_call(node, line)? {
                CallResult::Expr(expr) => Ok(expr),
                CallResult::VoidStmt(_) => {
                    Err(self.type_error(line, "void function cannot be used as a value expression"))
                }
            },

            "Attribute" => {
                let value_node = ast_getattr!(node, "value");
                let attr_name = ast_get_string!(node, "attr");
                let obj_expr = self.lower_expr(&value_node)?;

                let class_name = match &obj_expr.ty {
                    ValueType::Class(name) => name.clone(),
                    other => {
                        return Err(self.type_error(
                            line,
                            format!("cannot access attribute on non-class type `{}`", other),
                        ))
                    }
                };

                let class_info = self.class_registry.get(&class_name).ok_or_else(|| {
                    self.name_error(line, format!("unknown class `{}`", class_name))
                })?;

                let field_index = *class_info.field_map.get(&attr_name).ok_or_else(|| {
                    self.attribute_error(
                        line,
                        format!("class `{}` has no field `{}`", class_name, attr_name),
                    )
                })?;

                let field_ty_ast = class_info.fields[field_index].ty.clone();
                let field_ty = self.value_type_from_type(&field_ty_ast);

                Ok(TirExpr {
                    kind: TirExprKind::GetField {
                        object: Box::new(obj_expr),
                        class_name,
                        field_index,
                    },
                    ty: field_ty,
                })
            }

            "List" => {
                let elts_list = ast_get_list!(node, "elts");
                if elts_list.is_empty() {
                    if let Some(ValueType::List(inner)) = self.empty_list_hint.clone() {
                        let elem_ty = (*inner).clone();
                        let ty = ValueType::List(Box::new(elem_ty.clone()));
                        return Ok(TirExpr {
                            kind: TirExprKind::ListLiteral {
                                element_type: elem_ty,
                                elements: vec![],
                            },
                            ty,
                        });
                    }
                    return Err(self
                        .syntax_error(line, "empty list literal `[]` requires a type annotation"));
                }
                let mut elements = Vec::new();
                for elt in elts_list.iter() {
                    elements.push(self.lower_expr(&elt)?);
                }
                let elem_ty = elements[0].ty.clone();
                for (i, elt) in elements.iter().enumerate().skip(1) {
                    if elt.ty != elem_ty {
                        return Err(self.type_error(
                            line,
                            format!(
                                "list literal element {} has type `{}`, expected `{}`",
                                i, elt.ty, elem_ty
                            ),
                        ));
                    }
                }
                Ok(TirExpr {
                    kind: TirExprKind::ListLiteral {
                        element_type: elem_ty.clone(),
                        elements,
                    },
                    ty: ValueType::List(Box::new(elem_ty)),
                })
            }
            "Tuple" => {
                let elts_list = ast_get_list!(node, "elts");
                let mut elements = Vec::with_capacity(elts_list.len());
                for elt in elts_list.iter() {
                    elements.push(self.lower_expr(&elt)?);
                }
                let element_types: Vec<ValueType> =
                    elements.iter().map(|elt| elt.ty.clone()).collect();
                let class_name = self.get_or_create_tuple_class(&element_types);
                let init_mangled = format!("{}$__init__", class_name);
                Ok(TirExpr {
                    kind: TirExprKind::Construct {
                        class_name: class_name.clone(),
                        init_mangled_name: init_mangled,
                        args: elements,
                    },
                    ty: ValueType::Class(class_name),
                })
            }
            "Dict" => {
                let keys_list = ast_get_list!(node, "keys");
                let values_list = ast_get_list!(node, "values");
                if keys_list.is_empty() {
                    return Err(self
                        .syntax_error(line, "empty dict literal `{}` requires a type annotation"));
                }

                let mut keys = Vec::with_capacity(keys_list.len());
                let mut values = Vec::with_capacity(values_list.len());
                for i in 0..keys_list.len() {
                    let key_node = keys_list.get_item(i)?;
                    if key_node.is_none() {
                        return Err(
                            self.syntax_error(line, "dict unpacking (`**other`) is not supported")
                        );
                    }
                    keys.push(self.lower_expr(&key_node)?);
                    values.push(self.lower_expr(&values_list.get_item(i)?)?);
                }

                let key_ty = keys[0].ty.clone();
                let value_ty = values[0].ty.clone();
                for (i, key) in keys.iter().enumerate().skip(1) {
                    if key.ty != key_ty {
                        return Err(self.type_error(
                            line,
                            format!(
                                "dict literal key {} has type `{}`, expected `{}`",
                                i, key.ty, key_ty
                            ),
                        ));
                    }
                }
                for (i, value) in values.iter().enumerate().skip(1) {
                    if value.ty != value_ty {
                        return Err(self.type_error(
                            line,
                            format!(
                                "dict literal value {} has type `{}`, expected `{}`",
                                i, value.ty, value_ty
                            ),
                        ));
                    }
                }

                let dict_ty = ValueType::Dict(Box::new(key_ty.clone()), Box::new(value_ty.clone()));
                self.require_intrinsic_eq_support();
                let key_eq_tag = self.register_intrinsic_instance(IntrinsicOp::Eq, &key_ty);
                let dict_var = self.fresh_internal("dict_lit");
                self.pre_stmts.push(TirStmt::Let {
                    name: dict_var.clone(),
                    ty: dict_ty.clone(),
                    value: TirExpr {
                        kind: TirExprKind::ExternalCall {
                            func: builtin::BuiltinFn::DictEmpty,
                            args: vec![],
                        },
                        ty: dict_ty.clone(),
                    },
                });
                for i in 0..keys.len() {
                    self.pre_stmts.push(TirStmt::VoidCall {
                        target: CallTarget::Builtin(builtin::BuiltinFn::DictSetByTag),
                        args: vec![
                            TirExpr {
                                kind: TirExprKind::Var(dict_var.clone()),
                                ty: dict_ty.clone(),
                            },
                            keys[i].clone(),
                            values[i].clone(),
                            TirExpr {
                                kind: TirExprKind::IntLiteral(key_eq_tag),
                                ty: ValueType::Int,
                            },
                        ],
                    });
                }
                Ok(TirExpr {
                    kind: TirExprKind::Var(dict_var),
                    ty: dict_ty,
                })
            }
            "Set" => {
                let elts_list = ast_get_list!(node, "elts");
                if elts_list.is_empty() {
                    return Err(
                        self.syntax_error(line, "empty set literal is not valid; use set()")
                    );
                }
                let mut elements = Vec::with_capacity(elts_list.len());
                for elt in elts_list.iter() {
                    elements.push(self.lower_expr(&elt)?);
                }
                let elem_ty = elements[0].ty.clone();
                for (i, elt) in elements.iter().enumerate().skip(1) {
                    if elt.ty != elem_ty {
                        return Err(self.type_error(
                            line,
                            format!(
                                "set literal element {} has type `{}`, expected `{}`",
                                i, elt.ty, elem_ty
                            ),
                        ));
                    }
                }

                self.require_intrinsic_eq_support();
                let eq_tag = self.register_intrinsic_instance(IntrinsicOp::Eq, &elem_ty);
                let set_ty = ValueType::Set(Box::new(elem_ty));
                let set_var = self.fresh_internal("set_lit");
                self.pre_stmts.push(TirStmt::Let {
                    name: set_var.clone(),
                    ty: set_ty.clone(),
                    value: TirExpr {
                        kind: TirExprKind::ExternalCall {
                            func: builtin::BuiltinFn::SetEmpty,
                            args: vec![],
                        },
                        ty: set_ty.clone(),
                    },
                });
                for elt in elements {
                    self.pre_stmts.push(TirStmt::VoidCall {
                        target: CallTarget::Builtin(builtin::BuiltinFn::SetAddByTag),
                        args: vec![
                            TirExpr {
                                kind: TirExprKind::Var(set_var.clone()),
                                ty: set_ty.clone(),
                            },
                            elt,
                            TirExpr {
                                kind: TirExprKind::IntLiteral(eq_tag),
                                ty: ValueType::Int,
                            },
                        ],
                    });
                }
                Ok(TirExpr {
                    kind: TirExprKind::Var(set_var),
                    ty: set_ty,
                })
            }

            "ListComp" => self.lower_list_comprehension(node, line),
            "GeneratorExp" => self.lower_list_comprehension(node, line),

            "JoinedStr" => self.lower_joined_str(node, line),

            "FormattedValue" => self.lower_formatted_value(node, line),

            "Subscript" => {
                let value_node = ast_getattr!(node, "value");
                let slice_node = ast_getattr!(node, "slice");
                let obj_expr = self.lower_expr(&value_node)?;

                match obj_expr.ty.clone() {
                    // List slicing — special syntax with no dunder equivalent
                    ValueType::List(inner) if ast_type_name!(slice_node) == "Slice" => {
                        let step_node = ast_getattr!(slice_node, "step");
                        if !step_node.is_none() {
                            let step_expr = self.lower_expr(&step_node)?;
                            if step_expr.ty != ValueType::Int
                                || !matches!(step_expr.kind, TirExprKind::IntLiteral(1))
                            {
                                return Err(
                                    self.syntax_error(line, "list slicing step is not supported")
                                );
                            }
                        }
                        let lower_node = ast_getattr!(slice_node, "lower");
                        let upper_node = ast_getattr!(slice_node, "upper");
                        let lower_expr = if lower_node.is_none() {
                            TirExpr {
                                kind: TirExprKind::IntLiteral(0),
                                ty: ValueType::Int,
                            }
                        } else {
                            let e = self.lower_expr(&lower_node)?;
                            if e.ty != ValueType::Int {
                                return Err(self.type_error(
                                    line,
                                    format!("list slice start must be `int`, got `{}`", e.ty),
                                ));
                            }
                            e
                        };
                        let upper_expr = if upper_node.is_none() {
                            TirExpr {
                                kind: TirExprKind::IntLiteral(i64::MAX),
                                ty: ValueType::Int,
                            }
                        } else {
                            let e = self.lower_expr(&upper_node)?;
                            if e.ty != ValueType::Int {
                                return Err(self.type_error(
                                    line,
                                    format!("list slice end must be `int`, got `{}`", e.ty),
                                ));
                            }
                            e
                        };
                        let out_ty = ValueType::List(inner.clone());
                        Ok(TirExpr {
                            kind: TirExprKind::ExternalCall {
                                func: builtin::BuiltinFn::ListSlice,
                                args: vec![obj_expr, lower_expr, upper_expr],
                            },
                            ty: out_ty,
                        })
                    }

                    // Tuple indexing — field access, not method dispatch
                    ValueType::Class(ref name) if self.is_tuple_class(name) => {
                        let elements = self.tuple_element_types(name).to_vec();
                        let index_expr = self.lower_expr(&slice_node)?;
                        if index_expr.ty != ValueType::Int {
                            return Err(self.type_error(
                                line,
                                format!("tuple index must be `int`, got `{}`", index_expr.ty),
                            ));
                        }
                        if let Some(raw_index) = Self::extract_static_int_literal(&slice_node) {
                            let len = elements.len() as i64;
                            let normalized = if raw_index < 0 {
                                len + raw_index
                            } else {
                                raw_index
                            };
                            if normalized < 0 || normalized >= len {
                                return Err(self.type_error(
                                    line,
                                    format!(
                                        "tuple index {} out of bounds for {}-element tuple",
                                        raw_index, len
                                    ),
                                ));
                            }
                            let idx = normalized as usize;
                            let elem_ty = elements[idx].clone();
                            Ok(TirExpr {
                                kind: TirExprKind::GetField {
                                    object: Box::new(obj_expr),
                                    class_name: name.clone(),
                                    field_index: idx,
                                },
                                ty: elem_ty,
                            })
                        } else {
                            let first = elements.first().ok_or_else(|| {
                                self.type_error(line, "cannot index empty tuple".to_string())
                            })?;
                            if elements.iter().any(|ty| ty != first) {
                                return Err(self.type_error(
                                    line,
                                    "tuple indexed by variable must have all elements of the same type"
                                        .to_string(),
                                ));
                            }
                            // Dynamic tuple index: generate if-else chain of GetField
                            let elem_ty = first.clone();
                            let tuple_len = elements.len();
                            let tuple_var = self.fresh_internal("dyn_tup");
                            let idx_tmp = self.fresh_internal("dyn_idx");
                            let result_var = self.fresh_internal("dyn_tup_elem");

                            let tuple_ty = obj_expr.ty.clone();
                            self.pre_stmts.push(TirStmt::Let {
                                name: tuple_var.clone(),
                                ty: tuple_ty.clone(),
                                value: obj_expr,
                            });
                            self.pre_stmts.push(TirStmt::Let {
                                name: idx_tmp.clone(),
                                ty: ValueType::Int,
                                value: index_expr,
                            });
                            // Initialize result variable
                            self.pre_stmts.push(TirStmt::Let {
                                name: result_var.clone(),
                                ty: elem_ty.clone(),
                                value: TirExpr {
                                    kind: TirExprKind::GetField {
                                        object: Box::new(TirExpr {
                                            kind: TirExprKind::Var(tuple_var.clone()),
                                            ty: tuple_ty.clone(),
                                        }),
                                        class_name: name.clone(),
                                        field_index: 0,
                                    },
                                    ty: elem_ty.clone(),
                                },
                            });
                            let switch_stmts = self.gen_tuple_dynamic_getitem_stmts(
                                &result_var,
                                &elem_ty,
                                &tuple_var,
                                &tuple_ty,
                                &idx_tmp,
                                tuple_len,
                            );
                            self.pre_stmts.extend(switch_stmts);

                            Ok(TirExpr {
                                kind: TirExprKind::Var(result_var),
                                ty: elem_ty,
                            })
                        }
                    }

                    // Everything else → __getitem__ method dispatch
                    _ => {
                        let index_expr = self.lower_expr(&slice_node)?;
                        match self.lower_method_call(
                            line,
                            obj_expr,
                            "__getitem__",
                            vec![index_expr],
                        )? {
                            CallResult::Expr(e) => Ok(e),
                            CallResult::VoidStmt(_) => unreachable!(),
                        }
                    }
                }
            }

            _ => Err(self.syntax_error(
                line,
                format!("unsupported expression type: `{}`", node_type),
            )),
        }
    }

    pub(in crate::tir::lower) fn extract_static_int_literal(node: &Bound<PyAny>) -> Option<i64> {
        match ast_type_name!(node).as_str() {
            "Constant" => {
                let value = ast_getattr!(node, "value");
                if value.is_instance_of::<pyo3::types::PyBool>() {
                    None
                } else {
                    value.extract::<i64>().ok()
                }
            }
            "UnaryOp" => {
                let op_node = ast_getattr!(node, "op");
                let operand = ast_getattr!(node, "operand");
                if ast_type_name!(op_node) != "USub" {
                    return None;
                }
                Self::extract_static_int_literal(&operand).map(|v| -v)
            }
            _ => None,
        }
    }
}
