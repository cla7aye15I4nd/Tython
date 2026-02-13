use anyhow::Result;
use pyo3::prelude::*;

use crate::tir::{
    builtin, type_rules, ArithBinOp, BitwiseBinOp, CallResult, CallTarget, CastKind, CmpOp,
    FloatArithOp, IntArithOp, LogicalOp, OrderedCmpOp, RawBinOp, TirExpr, TirExprKind, TirStmt,
    Type, TypedBinOp, TypedCompare, ValueType,
};
use crate::{ast_get_int, ast_get_list, ast_get_string, ast_getattr, ast_type_name};

use super::Lowering;

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
    pub(super) fn lower_expr(&mut self, node: &Bound<PyAny>) -> Result<TirExpr> {
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

                let vty = Self::to_value_type(&ty);
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

                if ops_list.len() == 1 {
                    let op_node = ops_list.get_item(0)?;
                    let cmp_op = Self::convert_cmpop(&op_node)?;
                    let right = self.lower_expr(&comparators_list.get_item(0)?)?;
                    return self.lower_single_comparison(line, cmp_op, left, right);
                }

                let mut comparisons: Vec<TirExpr> = Vec::new();
                let mut current_left = left;

                for i in 0..ops_list.len() {
                    let op_node = ops_list.get_item(i)?;
                    let cmp_op = Self::convert_cmpop(&op_node)?;
                    let right = self.lower_expr(&comparators_list.get_item(i)?)?;

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

                if matches!(operand.ty, ValueType::Class(_)) {
                    let magic = type_rules::lookup_class_unary_magic(op)
                        .expect("ICE: missing class unary magic mapping");
                    let mut expr = self.lower_class_magic_method_with_args(
                        line,
                        operand,
                        &[magic.method_name],
                        magic.expected_return_type,
                        "unary operator",
                        vec![],
                    )?;
                    if magic.negate_result {
                        expr = TirExpr {
                            kind: TirExprKind::Not(Box::new(expr)),
                            ty: ValueType::Bool,
                        };
                    }
                    return Ok(expr);
                }

                let rule =
                    type_rules::lookup_unaryop(op, &operand.ty.to_type()).ok_or_else(|| {
                        self.type_error(
                            line,
                            type_rules::unaryop_type_error_message(op, &operand.ty.to_type()),
                        )
                    })?;

                // Handle Pos (unary +) as a no-op
                if op == crate::tir::UnaryOpKind::Pos {
                    return Ok(operand);
                }

                let lowered_operand = if op == crate::tir::UnaryOpKind::Not {
                    self.lower_truthy_to_bool(line, operand, "unary `not` operand")?
                } else {
                    operand
                };

                // Resolve to typed operation
                let typed_op = type_rules::resolve_typed_unaryop(op, &lowered_operand.ty.to_type())
                    .expect("ICE: resolve_typed_unaryop failed after successful lookup");

                // Construct typed operation variant
                let kind = match typed_op {
                    crate::tir::TypedUnaryOp::IntNeg => {
                        TirExprKind::IntNeg(Box::new(lowered_operand))
                    }
                    crate::tir::TypedUnaryOp::FloatNeg => {
                        TirExprKind::FloatNeg(Box::new(lowered_operand))
                    }
                    crate::tir::TypedUnaryOp::Not => TirExprKind::Not(Box::new(lowered_operand)),
                    crate::tir::TypedUnaryOp::BitNot => {
                        TirExprKind::BitNot(Box::new(lowered_operand))
                    }
                };

                Ok(TirExpr {
                    kind,
                    ty: Self::to_value_type(&rule.result_type),
                })
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
                    exprs.push(self.lower_expr(&val)?);
                }

                let result_ty = exprs[0].ty.clone();
                for (i, e) in exprs.iter().enumerate().skip(1) {
                    if e.ty != result_ty {
                        return Err(self.type_error(
                            line,
                            format!(
                                "all operands of `{}` must have the same type: operand {} is `{}`, expected `{}`",
                                op_type, i, e.ty, result_ty
                            ),
                        ));
                    }
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
                        ty: result_ty.clone(),
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

                let field_ty = Self::to_value_type(&class_info.fields[field_index].ty);

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
                Ok(TirExpr {
                    kind: TirExprKind::TupleLiteral {
                        elements,
                        element_types: element_types.clone(),
                    },
                    ty: ValueType::Tuple(element_types),
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
                        target: CallTarget::Builtin(builtin::BuiltinFn::DictSet),
                        args: vec![
                            TirExpr {
                                kind: TirExprKind::Var(dict_var.clone()),
                                ty: dict_ty.clone(),
                            },
                            keys[i].clone(),
                            values[i].clone(),
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
                        target: CallTarget::Builtin(builtin::BuiltinFn::SetAdd),
                        args: vec![
                            TirExpr {
                                kind: TirExprKind::Var(set_var.clone()),
                                ty: set_ty.clone(),
                            },
                            elt,
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
                    ValueType::List(inner) => {
                        if ast_type_name!(slice_node) == "Slice" {
                            let step_node = ast_getattr!(slice_node, "step");
                            if !step_node.is_none() {
                                let step_expr = self.lower_expr(&step_node)?;
                                if step_expr.ty != ValueType::Int
                                    || !matches!(step_expr.kind, TirExprKind::IntLiteral(1))
                                {
                                    return Err(self
                                        .syntax_error(line, "list slicing step is not supported"));
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
                        } else {
                            let index_expr = self.lower_expr(&slice_node)?;
                            if index_expr.ty != ValueType::Int {
                                return Err(self.type_error(
                                    line,
                                    format!("list index must be `int`, got `{}`", index_expr.ty),
                                ));
                            }
                            let elem_ty = (*inner).clone();
                            Ok(TirExpr {
                                kind: TirExprKind::ExternalCall {
                                    func: builtin::BuiltinFn::ListGet,
                                    args: vec![obj_expr, index_expr],
                                },
                                ty: elem_ty,
                            })
                        }
                    }
                    ValueType::Dict(key_ty, value_ty) => {
                        let index_expr = self.lower_expr(&slice_node)?;
                        if index_expr.ty != *key_ty {
                            return Err(self.type_error(
                                line,
                                format!(
                                    "dict key index must be `{}`, got `{}`",
                                    key_ty, index_expr.ty
                                ),
                            ));
                        }
                        Ok(TirExpr {
                            kind: TirExprKind::ExternalCall {
                                func: builtin::BuiltinFn::DictGet,
                                args: vec![obj_expr, index_expr],
                            },
                            ty: (*value_ty).clone(),
                        })
                    }
                    ValueType::Tuple(elements) => {
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
                                kind: TirExprKind::TupleGet {
                                    tuple: Box::new(obj_expr),
                                    index: idx,
                                    element_types: elements.clone(),
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
                            Ok(TirExpr {
                                kind: TirExprKind::TupleGetDynamic {
                                    tuple: Box::new(obj_expr),
                                    index: Box::new(index_expr),
                                    len: elements.len(),
                                    element_types: elements.clone(),
                                },
                                ty: first.clone(),
                            })
                        }
                    }
                    other => {
                        Err(self.type_error(line, format!("type `{}` is not subscriptable", other)))
                    }
                }
            }

            _ => Err(self.syntax_error(
                line,
                format!("unsupported expression type: `{}`", node_type),
            )),
        }
    }

    fn lower_joined_str(&mut self, node: &Bound<PyAny>, line: usize) -> Result<TirExpr> {
        let values = ast_get_list!(node, "values");
        let mut result = TirExpr {
            kind: TirExprKind::StrLiteral(String::new()),
            ty: ValueType::Str,
        };

        for part in values.iter() {
            let part_expr = match ast_type_name!(part).as_str() {
                "Constant" => {
                    let value = ast_getattr!(part, "value");
                    let s = value.extract::<String>().map_err(|_| {
                        self.syntax_error(line, "f-string constants must be string literals")
                    })?;
                    TirExpr {
                        kind: TirExprKind::StrLiteral(s),
                        ty: ValueType::Str,
                    }
                }
                "FormattedValue" => self.lower_formatted_value(&part, line)?,
                other => {
                    return Err(self
                        .syntax_error(line, format!("unsupported f-string segment `{}`", other)))
                }
            };

            result = TirExpr {
                kind: TirExprKind::ExternalCall {
                    func: builtin::BuiltinFn::StrConcat,
                    args: vec![result, part_expr],
                },
                ty: ValueType::Str,
            };
        }

        Ok(result)
    }

    fn lower_formatted_value(&mut self, node: &Bound<PyAny>, line: usize) -> Result<TirExpr> {
        let value_expr = self.lower_expr(&ast_getattr!(node, "value"))?;
        let conversion = ast_get_int!(node, "conversion", i64);

        // Parse and evaluate format spec for compatibility, but ignore formatting details for now.
        let format_spec = ast_getattr!(node, "format_spec");
        if !format_spec.is_none() {
            let spec_expr = match ast_type_name!(format_spec).as_str() {
                "JoinedStr" => self.lower_joined_str(&format_spec, line)?,
                "Constant" => self.lower_expr(&format_spec)?,
                other => {
                    return Err(self.syntax_error(
                        line,
                        format!("unsupported f-string format spec `{}`", other),
                    ))
                }
            };
            if spec_expr.ty != ValueType::Str {
                return Err(self.type_error(
                    line,
                    format!("f-string format spec must be `str`, got `{}`", spec_expr.ty),
                ));
            }
            let tmp = self.fresh_internal("fstr_spec");
            self.pre_stmts.push(TirStmt::Let {
                name: tmp,
                ty: ValueType::Str,
                value: spec_expr,
            });
        }

        match conversion {
            -1 | 115 => self.lower_fstring_convert(line, "str", value_expr),
            114 | 97 => self.lower_fstring_convert(line, "repr", value_expr),
            other => Err(self.syntax_error(
                line,
                format!("unsupported f-string conversion code `{}`", other),
            )),
        }
    }

    fn lower_fstring_convert(&mut self, line: usize, name: &str, arg: TirExpr) -> Result<TirExpr> {
        let arg_types: Vec<&ValueType> = vec![&arg.ty];
        let rule = type_rules::lookup_builtin_call(name, &arg_types).ok_or_else(|| {
            self.type_error(
                line,
                format!(
                    "f-string conversion `{}` is not defined for type `{}`",
                    name, arg.ty
                ),
            )
        })?;

        if let type_rules::BuiltinCallRule::ClassMagic {
            method_names,
            return_type,
        } = rule
        {
            return self.lower_class_magic_method(line, arg, method_names, return_type, name);
        }

        if matches!(rule, type_rules::BuiltinCallRule::StrAuto) {
            return Ok(self.lower_str_auto(arg));
        }
        if matches!(rule, type_rules::BuiltinCallRule::ReprAuto) {
            return Ok(self.lower_repr_str_expr(arg));
        }

        match Self::lower_builtin_rule(rule, vec![arg]) {
            CallResult::Expr(expr) => Ok(expr),
            CallResult::VoidStmt(_) => Err(self.type_error(
                line,
                format!("f-string conversion `{}` produced no value", name),
            )),
        }
    }

    fn extract_static_int_literal(node: &Bound<PyAny>) -> Option<i64> {
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

    // ── print statement ────────────────────────────────────────────────

    pub(super) fn lower_print_stmt(&mut self, call_node: &Bound<PyAny>) -> Result<Vec<TirStmt>> {
        let line = Self::get_line(call_node);
        let args_list = ast_get_list!(call_node, "args");

        let mut tir_args = Vec::new();
        for arg in args_list.iter() {
            tir_args.push(self.lower_expr(&arg)?);
        }

        let mut stmts = Vec::new();
        for (i, arg) in tir_args.into_iter().enumerate() {
            if i > 0 {
                stmts.push(TirStmt::VoidCall {
                    target: CallTarget::Builtin(builtin::BuiltinFn::PrintSpace),
                    args: vec![],
                });
            }
            self.lower_print_value_stmts(line, arg, &mut stmts)?;
        }
        stmts.push(TirStmt::VoidCall {
            target: CallTarget::Builtin(builtin::BuiltinFn::PrintNewline),
            args: vec![],
        });

        Ok(stmts)
    }

    fn push_print_str_literal(stmts: &mut Vec<TirStmt>, s: impl Into<String>) {
        stmts.push(TirStmt::VoidCall {
            target: CallTarget::Builtin(builtin::BuiltinFn::PrintStr),
            args: vec![TirExpr {
                kind: TirExprKind::StrLiteral(s.into()),
                ty: ValueType::Str,
            }],
        });
    }

    fn lower_print_class_as_str(&self, line: usize, object: TirExpr) -> Result<TirExpr> {
        let rule = type_rules::lookup_builtin_call("str", &[&object.ty])
            .expect("ICE: missing builtin rule for str() on class");
        match rule {
            type_rules::BuiltinCallRule::ClassMagic {
                method_names,
                return_type,
            } => self.lower_class_magic_method(line, object, method_names, return_type, "str"),
            _ => unreachable!("ICE: str() on class should resolve to ClassMagic"),
        }
    }

    fn lower_print_value_stmts(
        &mut self,
        line: usize,
        arg: TirExpr,
        stmts: &mut Vec<TirStmt>,
    ) -> Result<()> {
        macro_rules! push_direct_print {
            ($fn_name:expr, $value:expr) => {{
                stmts.push(TirStmt::VoidCall {
                    target: CallTarget::Builtin($fn_name),
                    args: vec![$value],
                });
                Ok(())
            }};
        }

        match &arg.ty {
            ValueType::Tuple(element_types) => {
                let tuple_var = self.fresh_internal("print_tuple");
                let tuple_ty = arg.ty.clone();
                let tuple_element_types = element_types.clone();

                stmts.push(TirStmt::Let {
                    name: tuple_var.clone(),
                    ty: tuple_ty.clone(),
                    value: arg,
                });

                Self::push_print_str_literal(stmts, "(");

                for (i, element_ty) in tuple_element_types.iter().enumerate() {
                    if i > 0 {
                        Self::push_print_str_literal(stmts, ", ");
                    }

                    let element_expr = TirExpr {
                        kind: TirExprKind::TupleGet {
                            tuple: Box::new(TirExpr {
                                kind: TirExprKind::Var(tuple_var.clone()),
                                ty: tuple_ty.clone(),
                            }),
                            index: i,
                            element_types: tuple_element_types.clone(),
                        },
                        ty: element_ty.clone(),
                    };

                    self.lower_print_repr_stmts(line, element_expr, stmts)?;
                }

                if tuple_element_types.len() == 1 {
                    Self::push_print_str_literal(stmts, ",");
                }
                Self::push_print_str_literal(stmts, ")");
                Ok(())
            }
            ValueType::Class(_) => {
                let print_arg = self.lower_print_class_as_str(line, arg)?;
                self.lower_print_value_stmts(line, print_arg, stmts)
            }
            ValueType::Float => push_direct_print!(builtin::BuiltinFn::PrintFloat, arg),
            ValueType::Bool => push_direct_print!(builtin::BuiltinFn::PrintBool, arg),
            ValueType::Int => push_direct_print!(builtin::BuiltinFn::PrintInt, arg),
            ValueType::Str => push_direct_print!(builtin::BuiltinFn::PrintStr, arg),
            ValueType::Bytes => push_direct_print!(builtin::BuiltinFn::PrintBytes, arg),
            ValueType::ByteArray => push_direct_print!(builtin::BuiltinFn::PrintByteArray, arg),
            ValueType::List(_) => {
                let inner_ty = match arg.ty.clone() {
                    ValueType::List(inner) => *inner,
                    _ => unreachable!(),
                };
                self.lower_print_list_stmts(line, arg, inner_ty, stmts)
            }
            ValueType::Function { .. } => {
                Err(self.type_error(line, format!("cannot print value of type `{}`", arg.ty)))
            }
            ValueType::Dict(_, _) | ValueType::Set(_) => {
                Err(self.type_error(line, format!("cannot print value of type `{}`", arg.ty)))
            }
        }
    }

    /// Auto-generate list printing for any element type by iterating and
    /// printing each element's repr. Replaces per-type C++ print_list_*
    /// runtime functions.
    fn lower_print_list_stmts(
        &mut self,
        line: usize,
        arg: TirExpr,
        loop_var_ty: ValueType,
        stmts: &mut Vec<TirStmt>,
    ) -> Result<()> {
        let list_var = self.fresh_internal("print_list");
        let idx_var = self.fresh_internal("print_idx");
        let len_var = self.fresh_internal("print_len");
        let loop_var = self.fresh_internal("print_elem");
        let list_ty = arg.ty.clone();

        stmts.push(TirStmt::Let {
            name: list_var.clone(),
            ty: list_ty,
            value: arg,
        });
        Self::push_print_str_literal(stmts, "[");

        let mut body = Vec::new();
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
            then_body: vec![TirStmt::VoidCall {
                target: CallTarget::Builtin(builtin::BuiltinFn::PrintStr),
                args: vec![TirExpr {
                    kind: TirExprKind::StrLiteral(", ".to_string()),
                    ty: ValueType::Str,
                }],
            }],
            else_body: vec![],
        });

        let elem_expr = TirExpr {
            kind: TirExprKind::Var(loop_var.clone()),
            ty: loop_var_ty.clone(),
        };
        self.lower_print_repr_stmts(line, elem_expr, &mut body)?;

        stmts.push(TirStmt::ForList {
            loop_var,
            loop_var_ty,
            list_var,
            index_var: idx_var,
            len_var,
            body,
            else_body: vec![],
        });
        Self::push_print_str_literal(stmts, "]");
        Ok(())
    }

    /// Print the repr of a value (used inside list/tuple printing).
    /// Strings are wrapped in quotes; all other types delegate to
    /// `lower_print_value_stmts` (which already outputs the repr form
    /// for bytes, bytearray, nested lists, etc.).
    fn lower_print_repr_stmts(
        &mut self,
        line: usize,
        arg: TirExpr,
        stmts: &mut Vec<TirStmt>,
    ) -> Result<()> {
        match &arg.ty {
            ValueType::Str => {
                Self::push_print_str_literal(stmts, "'");
                stmts.push(TirStmt::VoidCall {
                    target: CallTarget::Builtin(builtin::BuiltinFn::PrintStr),
                    args: vec![arg],
                });
                Self::push_print_str_literal(stmts, "'");
                Ok(())
            }
            _ => self.lower_print_value_stmts(line, arg, stmts),
        }
    }

    // ── str() auto-generation ────────────────────────────────────────────

    /// Auto-generate str() for composite types (list, tuple).
    /// Pushes setup statements into `self.pre_stmts` and returns a
    /// `Var` expression pointing to the result string.
    pub(super) fn lower_str_auto(&mut self, arg: TirExpr) -> TirExpr {
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
                kind: TirExprKind::TupleGet {
                    tuple: Box::new(TirExpr {
                        kind: TirExprKind::Var(tuple_var.clone()),
                        ty: tuple_ty.clone(),
                    }),
                    index: i,
                    element_types: element_types.clone(),
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
    pub(super) fn lower_repr_str_expr(&mut self, arg: TirExpr) -> TirExpr {
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

    // ── binary ops ───────────────────────────────────────────────────────

    /// Resolve a binary operation into a TIR expression.
    /// Sequence operations (concat, repeat) become `ExternalCall`;
    /// arithmetic/bitwise operations become `BinOp`.
    pub(super) fn resolve_binop(
        &self,
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
        let rule = type_rules::lookup_binop(raw_op, &left_ast, &right_ast).ok_or_else(|| {
            self.type_error(
                line,
                type_rules::binop_type_error_message(raw_op, &left_ast, &right_ast),
            )
        })?;

        let result_vty = Self::to_value_type(&rule.result_type);

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
        let typed_op = type_rules::resolve_typed_binop(raw_op, &rule.result_type);
        let final_left = Self::apply_coercion(left, rule.left_coercion);
        let final_right = Self::apply_coercion(right, rule.right_coercion);

        // Construct typed operation variant based on TypedBinOp
        let kind = match typed_op {
            TypedBinOp::IntArith(IntArithOp::Add) => {
                TirExprKind::IntAdd(Box::new(final_left), Box::new(final_right))
            }
            TypedBinOp::IntArith(IntArithOp::Sub) => {
                TirExprKind::IntSub(Box::new(final_left), Box::new(final_right))
            }
            TypedBinOp::IntArith(IntArithOp::Mul) => {
                TirExprKind::IntMul(Box::new(final_left), Box::new(final_right))
            }
            TypedBinOp::IntArith(IntArithOp::FloorDiv) => {
                TirExprKind::IntFloorDiv(Box::new(final_left), Box::new(final_right))
            }
            TypedBinOp::IntArith(IntArithOp::Mod) => {
                TirExprKind::IntMod(Box::new(final_left), Box::new(final_right))
            }
            TypedBinOp::IntArith(IntArithOp::Pow) => {
                TirExprKind::IntPow(Box::new(final_left), Box::new(final_right))
            }
            TypedBinOp::FloatArith(FloatArithOp::Add) => {
                TirExprKind::FloatAdd(Box::new(final_left), Box::new(final_right))
            }
            TypedBinOp::FloatArith(FloatArithOp::Sub) => {
                TirExprKind::FloatSub(Box::new(final_left), Box::new(final_right))
            }
            TypedBinOp::FloatArith(FloatArithOp::Mul) => {
                TirExprKind::FloatMul(Box::new(final_left), Box::new(final_right))
            }
            TypedBinOp::FloatArith(FloatArithOp::Div) => {
                TirExprKind::FloatDiv(Box::new(final_left), Box::new(final_right))
            }
            TypedBinOp::FloatArith(FloatArithOp::FloorDiv) => {
                TirExprKind::FloatFloorDiv(Box::new(final_left), Box::new(final_right))
            }
            TypedBinOp::FloatArith(FloatArithOp::Mod) => {
                TirExprKind::FloatMod(Box::new(final_left), Box::new(final_right))
            }
            TypedBinOp::FloatArith(FloatArithOp::Pow) => {
                TirExprKind::FloatPow(Box::new(final_left), Box::new(final_right))
            }
            TypedBinOp::Bitwise(BitwiseBinOp::BitAnd) => {
                TirExprKind::BitAnd(Box::new(final_left), Box::new(final_right))
            }
            TypedBinOp::Bitwise(BitwiseBinOp::BitOr) => {
                TirExprKind::BitOr(Box::new(final_left), Box::new(final_right))
            }
            TypedBinOp::Bitwise(BitwiseBinOp::BitXor) => {
                TirExprKind::BitXor(Box::new(final_left), Box::new(final_right))
            }
            TypedBinOp::Bitwise(BitwiseBinOp::LShift) => {
                TirExprKind::LShift(Box::new(final_left), Box::new(final_right))
            }
            TypedBinOp::Bitwise(BitwiseBinOp::RShift) => {
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
        &self,
        line: usize,
        raw_op: RawBinOp,
        left: TirExpr,
        right: TirExpr,
    ) -> Result<Option<TirExpr>> {
        let magic = type_rules::lookup_class_binop_magic(raw_op)
            .expect("ICE: missing class binop magic mapping");

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

    fn apply_coercion(expr: TirExpr, coercion: type_rules::Coercion) -> TirExpr {
        match coercion {
            type_rules::Coercion::None => expr,
            type_rules::Coercion::ToFloat => {
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

    // ── comparisons ──────────────────────────────────────────────────────

    fn lower_single_comparison(
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

        for (i, elem_ty) in elements.iter().enumerate() {
            let left_elem = TirExpr {
                kind: TirExprKind::TupleGet {
                    tuple: Box::new(left.clone()),
                    index: i,
                    element_types: elements.clone(),
                },
                ty: elem_ty.clone(),
            };
            let right_elem = TirExpr {
                kind: TirExprKind::TupleGet {
                    tuple: Box::new(right.clone()),
                    index: i,
                    element_types: elements.clone(),
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
            ValueType::Int | ValueType::Float | ValueType::Bool => TirExpr {
                kind: TirExprKind::IntEq(Box::new(left), Box::new(right)),
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
                Self::apply_coercion(left, type_rules::Coercion::ToFloat),
                Self::apply_coercion(right, type_rules::Coercion::ToFloat),
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

    // ── cast kind computation ──────────────────────────────────────────

    pub(super) fn compute_cast_kind(from: &ValueType, to: &ValueType) -> CastKind {
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

    // ── list comprehension ───────────────────────────────────────────

    fn lower_list_comprehension(&mut self, node: &Bound<PyAny>, line: usize) -> Result<TirExpr> {
        let elt_node = ast_getattr!(node, "elt");
        let generators = ast_get_list!(node, "generators");
        self.lower_comp_impl(&elt_node, &generators, line)
    }

    /// Shared implementation for ListComp and GeneratorExp lowering.
    /// Returns a Var expression pointing to the result list.
    /// Emits setup+loop stmts into self.pre_stmts.
    pub(super) fn lower_comp_impl(
        &mut self,
        elt_node: &Bound<PyAny>,
        generators: &Bound<pyo3::types::PyList>,
        line: usize,
    ) -> Result<TirExpr> {
        self.push_scope();

        // Phase 1: Parse generators — lower iter exprs, declare loop vars, lower ifs
        let mut gen_infos = Vec::new();
        for gen in generators.iter() {
            let target = ast_getattr!(gen, "target");
            let iter_node = ast_getattr!(gen, "iter");
            let target_type = ast_type_name!(target);

            let (var_name, gen_kind) = if target_type == "Name" {
                let var_name = ast_get_string!(target, "id");
                let gen_kind = if ast_type_name!(iter_node) == "Call" {
                    let func_node = ast_getattr!(iter_node, "func");
                    if ast_type_name!(func_node) == "Name"
                        && ast_get_string!(func_node, "id") == "range"
                    {
                        let args_list = ast_get_list!(iter_node, "args");
                        let (start, stop, step) = self.parse_range_args(&args_list, line)?;
                        self.declare(var_name.clone(), Type::Int);
                        GenKind::Range { start, stop, step }
                    } else {
                        let iter_expr = self.lower_expr(&iter_node)?;
                        self.gen_kind_from_expr(line, &var_name, iter_expr)?
                    }
                } else {
                    let iter_expr = self.lower_expr(&iter_node)?;
                    self.gen_kind_from_expr(line, &var_name, iter_expr)?
                };
                (var_name, gen_kind)
            } else if target_type == "Tuple" && ast_type_name!(iter_node) == "Call" {
                let names = {
                    let elts = ast_get_list!(target, "elts");
                    let mut names = Vec::with_capacity(elts.len());
                    for elt in elts.iter() {
                        if ast_type_name!(elt) != "Name" {
                            return Err(self.syntax_error(
                                line,
                                "comprehension tuple target must contain only variable names",
                            ));
                        }
                        names.push(ast_get_string!(elt, "id"));
                    }
                    names
                };
                if names.len() != 2 {
                    return Err(self.syntax_error(
                        line,
                        "comprehension tuple target currently requires exactly two variables",
                    ));
                }
                let func_node = ast_getattr!(iter_node, "func");
                if ast_type_name!(func_node) != "Name" {
                    return Err(self.syntax_error(
                        line,
                        "comprehension tuple target is only supported with zip(...) or enumerate(...)",
                    ));
                }
                let func_name = ast_get_string!(func_node, "id");
                let args = ast_get_list!(iter_node, "args");
                match func_name.as_str() {
                    "zip" => {
                        if args.len() != 2 {
                            return Err(self.type_error(
                                line,
                                format!("zip() expects 2 arguments, got {}", args.len()),
                            ));
                        }
                        let left_expr = self.lower_expr(&args.get_item(0)?)?;
                        let right_expr = self.lower_expr(&args.get_item(1)?)?;
                        let (left_elem, right_elem) = match (&left_expr.ty, &right_expr.ty) {
                            (ValueType::List(a), ValueType::List(b)) => {
                                ((**a).clone(), (**b).clone())
                            }
                            _ => {
                                return Err(self.type_error(
                                    line,
                                    "zip() in comprehension requires list arguments",
                                ))
                            }
                        };
                        self.declare(names[0].clone(), left_elem.to_type());
                        self.declare(names[1].clone(), right_elem.to_type());
                        (
                            names[0].clone(),
                            GenKind::Zip2 {
                                left_name: names[0].clone(),
                                right_name: names[1].clone(),
                                left_expr,
                                right_expr,
                                left_elem,
                                right_elem,
                            },
                        )
                    }
                    "enumerate" => {
                        if args.len() != 1 {
                            return Err(self.type_error(
                                line,
                                format!("enumerate() expects 1 argument, got {}", args.len()),
                            ));
                        }
                        let list_expr = self.lower_expr(&args.get_item(0)?)?;
                        let elem_ty = match &list_expr.ty {
                            ValueType::List(inner) => (**inner).clone(),
                            _ => {
                                return Err(self.type_error(
                                    line,
                                    "enumerate() in comprehension requires a list argument",
                                ))
                            }
                        };
                        self.declare(names[0].clone(), Type::Int);
                        self.declare(names[1].clone(), elem_ty.to_type());
                        (
                            names[0].clone(),
                            GenKind::Enumerate {
                                idx_name: names[0].clone(),
                                value_name: names[1].clone(),
                                list_expr,
                                elem_ty,
                            },
                        )
                    }
                    _ => {
                        return Err(self.syntax_error(
                            line,
                            "comprehension tuple target is only supported with zip(...) or enumerate(...)",
                        ))
                    }
                }
            } else {
                return Err(
                    self.syntax_error(line, "comprehension target must be a variable or tuple")
                );
            };

            // Lower if conditions
            let ifs_list = ast_get_list!(gen, "ifs");
            let mut if_conds = Vec::new();
            for if_node in ifs_list.iter() {
                let cond = self.lower_expr(&if_node)?;
                if_conds.push(self.lower_truthy_to_bool(
                    line,
                    cond,
                    "comprehension filter condition",
                )?);
            }

            gen_infos.push(GenInfo {
                var_name,
                kind: gen_kind,
                if_conds,
            });
        }

        // Phase 2: Lower the elt expression
        let elt_expr = self.lower_expr(elt_node)?;
        let elt_pre = std::mem::take(&mut self.pre_stmts);
        let elem_ty = elt_expr.ty.clone();

        self.pop_scope();

        // Phase 3: Build the imperative structure
        let list_var = self.fresh_internal("listcomp");
        let list_ty = ValueType::List(Box::new(elem_ty.clone()));
        self.declare(list_var.clone(), list_ty.to_type());

        // Innermost body: pre_stmts from elt + append
        let append_stmt = TirStmt::VoidCall {
            target: CallTarget::Builtin(builtin::BuiltinFn::ListAppend),
            args: vec![
                TirExpr {
                    kind: TirExprKind::Var(list_var.clone()),
                    ty: list_ty.clone(),
                },
                elt_expr,
            ],
        };
        let mut body: Vec<TirStmt> = elt_pre;
        body.push(append_stmt);

        // Build from inside out
        for gen_info in gen_infos.iter().rev() {
            // Apply if conditions for this generator
            if !gen_info.if_conds.is_empty() {
                let combined = gen_info
                    .if_conds
                    .iter()
                    .cloned()
                    .reduce(|a, b| TirExpr {
                        kind: TirExprKind::LogicalAnd(Box::new(a), Box::new(b)),
                        ty: ValueType::Bool,
                    })
                    .unwrap();
                body = vec![TirStmt::If {
                    condition: combined,
                    then_body: body,
                    else_body: vec![],
                }];
            }

            // Wrap in for-loop
            body = self.build_comp_for_loop(&gen_info.var_name, &gen_info.kind, body);
        }

        // Emit: create empty list + loop stmts
        let mut stmts = vec![TirStmt::Let {
            name: list_var.clone(),
            ty: list_ty.clone(),
            value: TirExpr {
                kind: TirExprKind::ListLiteral {
                    element_type: elem_ty,
                    elements: vec![],
                },
                ty: list_ty.clone(),
            },
        }];
        stmts.extend(body);

        self.pre_stmts.extend(stmts);

        Ok(TirExpr {
            kind: TirExprKind::Var(list_var),
            ty: list_ty,
        })
    }

    fn parse_range_args(
        &mut self,
        args_list: &Bound<pyo3::types::PyList>,
        line: usize,
    ) -> Result<(TirExpr, TirExpr, TirExpr)> {
        if args_list.is_empty() || args_list.len() > 3 {
            return Err(self.type_error(
                line,
                format!("range() expects 1 to 3 arguments, got {}", args_list.len()),
            ));
        }
        let mut args = Vec::new();
        for arg in args_list.iter() {
            let expr = self.lower_expr(&arg)?;
            if expr.ty != ValueType::Int {
                return Err(self.type_error(
                    line,
                    format!("range() arguments must be `int`, got `{}`", expr.ty),
                ));
            }
            args.push(expr);
        }
        Ok(match args.len() {
            1 => (
                TirExpr {
                    kind: TirExprKind::IntLiteral(0),
                    ty: ValueType::Int,
                },
                args.remove(0),
                TirExpr {
                    kind: TirExprKind::IntLiteral(1),
                    ty: ValueType::Int,
                },
            ),
            2 => {
                let stop = args.remove(1);
                let start = args.remove(0);
                (
                    start,
                    stop,
                    TirExpr {
                        kind: TirExprKind::IntLiteral(1),
                        ty: ValueType::Int,
                    },
                )
            }
            3 => {
                let step = args.remove(2);
                let stop = args.remove(1);
                let start = args.remove(0);
                (start, stop, step)
            }
            _ => unreachable!(),
        })
    }

    fn gen_kind_from_expr(
        &mut self,
        line: usize,
        var_name: &str,
        iter_expr: TirExpr,
    ) -> Result<GenKind> {
        match &iter_expr.ty.clone() {
            ValueType::List(inner) => {
                let elem_ty = (**inner).clone();
                self.declare(var_name.to_string(), elem_ty.to_type());
                Ok(GenKind::List {
                    list_expr: iter_expr,
                    elem_ty,
                })
            }
            ValueType::Str => {
                self.declare(var_name.to_string(), Type::Str);
                Ok(GenKind::Str {
                    str_expr: iter_expr,
                    elem_ty: ValueType::Str,
                })
            }
            ValueType::Class(class_name) => {
                let class_info = self.lookup_class(line, class_name)?;
                let iter_method = class_info.methods.get("__iter__").ok_or_else(|| {
                    self.type_error(
                        line,
                        format!("class `{}` does not implement `__iter__`", class_name),
                    )
                })?;
                let iter_class_name = match &iter_method.return_type {
                    Type::Class(name) => name.clone(),
                    _ => {
                        return Err(self.type_error(
                            line,
                            format!("`{}`.__iter__() must return a class instance", class_name),
                        ))
                    }
                };
                let iter_mangled = iter_method.mangled_name.clone();
                let iter_class_info = self.lookup_class(line, &iter_class_name)?;
                let next_method = iter_class_info.methods.get("__next__").ok_or_else(|| {
                    self.type_error(
                        line,
                        format!(
                            "iterator class `{}` does not implement `__next__`",
                            iter_class_name
                        ),
                    )
                })?;
                let elem_ty = Self::to_value_type(&next_method.return_type);
                let next_mangled = next_method.mangled_name.clone();
                self.declare(var_name.to_string(), next_method.return_type.clone());
                Ok(GenKind::ClassIter {
                    obj_expr: iter_expr,
                    iter_mangled,
                    iter_class: iter_class_name,
                    next_mangled,
                    elem_ty,
                })
            }
            ValueType::Tuple(elements) => {
                let first = elements
                    .first()
                    .ok_or_else(|| self.type_error(line, "cannot iterate over empty tuple"))?;
                if elements.iter().any(|ty| ty != first) {
                    return Err(self.type_error(
                        line,
                        "for-in over tuple requires all elements to have the same type",
                    ));
                }
                let elem_ty = first.clone();
                let len = elements.len();
                self.declare(var_name.to_string(), elem_ty.to_type());
                Ok(GenKind::Tuple {
                    tuple_expr: iter_expr,
                    elem_ty,
                    len,
                })
            }
            other => Err(self.type_error(
                line,
                format!("cannot iterate over `{}` in comprehension", other),
            )),
        }
    }

    fn build_comp_for_loop(
        &mut self,
        var_name: &str,
        kind: &GenKind,
        body: Vec<TirStmt>,
    ) -> Vec<TirStmt> {
        match kind {
            GenKind::Range { start, stop, step } => {
                let start_name = self.fresh_internal("comp_start");
                let stop_name = self.fresh_internal("comp_stop");
                let step_name = self.fresh_internal("comp_step");

                vec![
                    TirStmt::Let {
                        name: start_name.clone(),
                        ty: ValueType::Int,
                        value: start.clone(),
                    },
                    TirStmt::Let {
                        name: stop_name.clone(),
                        ty: ValueType::Int,
                        value: stop.clone(),
                    },
                    TirStmt::Let {
                        name: step_name.clone(),
                        ty: ValueType::Int,
                        value: step.clone(),
                    },
                    TirStmt::ForRange {
                        loop_var: var_name.to_string(),
                        start_var: start_name,
                        stop_var: stop_name,
                        step_var: step_name,
                        body,
                        else_body: vec![],
                    },
                ]
            }
            GenKind::List { list_expr, elem_ty } => {
                let list_var = self.fresh_internal("comp_list");
                let idx_var = self.fresh_internal("comp_idx");
                let len_var = self.fresh_internal("comp_len");

                vec![
                    TirStmt::Let {
                        name: list_var.clone(),
                        ty: list_expr.ty.clone(),
                        value: list_expr.clone(),
                    },
                    TirStmt::ForList {
                        loop_var: var_name.to_string(),
                        loop_var_ty: elem_ty.clone(),
                        list_var,
                        index_var: idx_var,
                        len_var,
                        body,
                        else_body: vec![],
                    },
                ]
            }
            GenKind::Str { str_expr, elem_ty } => {
                let str_var = self.fresh_internal("comp_str");
                let idx_var = self.fresh_internal("comp_str_idx");
                let len_var = self.fresh_internal("comp_str_len");
                let start_var = self.fresh_internal("comp_str_start");
                let stop_var = self.fresh_internal("comp_str_stop");
                let step_var = self.fresh_internal("comp_str_step");

                let mut full_body = vec![TirStmt::Let {
                    name: var_name.to_string(),
                    ty: elem_ty.clone(),
                    value: TirExpr {
                        kind: TirExprKind::ExternalCall {
                            func: builtin::BuiltinFn::StrGetChar,
                            args: vec![
                                TirExpr {
                                    kind: TirExprKind::Var(str_var.clone()),
                                    ty: ValueType::Str,
                                },
                                TirExpr {
                                    kind: TirExprKind::Var(idx_var.clone()),
                                    ty: ValueType::Int,
                                },
                            ],
                        },
                        ty: elem_ty.clone(),
                    },
                }];
                full_body.extend(body);

                vec![
                    TirStmt::Let {
                        name: str_var.clone(),
                        ty: ValueType::Str,
                        value: str_expr.clone(),
                    },
                    TirStmt::Let {
                        name: len_var.clone(),
                        ty: ValueType::Int,
                        value: TirExpr {
                            kind: TirExprKind::ExternalCall {
                                func: builtin::BuiltinFn::StrLen,
                                args: vec![TirExpr {
                                    kind: TirExprKind::Var(str_var),
                                    ty: ValueType::Str,
                                }],
                            },
                            ty: ValueType::Int,
                        },
                    },
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
                            kind: TirExprKind::Var(len_var),
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
                        body: full_body,
                        else_body: vec![],
                    },
                ]
            }
            GenKind::ClassIter {
                obj_expr,
                iter_mangled,
                iter_class,
                next_mangled,
                elem_ty,
                ..
            } => {
                let obj_var = self.fresh_internal("comp_obj");
                let iter_var = self.fresh_internal("comp_iter");

                vec![
                    TirStmt::Let {
                        name: obj_var.clone(),
                        ty: obj_expr.ty.clone(),
                        value: obj_expr.clone(),
                    },
                    TirStmt::Let {
                        name: iter_var.clone(),
                        ty: ValueType::Class(iter_class.clone()),
                        value: TirExpr {
                            kind: TirExprKind::Call {
                                func: iter_mangled.clone(),
                                args: vec![TirExpr {
                                    kind: TirExprKind::Var(obj_var),
                                    ty: obj_expr.ty.clone(),
                                }],
                            },
                            ty: ValueType::Class(iter_class.clone()),
                        },
                    },
                    TirStmt::ForIter {
                        loop_var: var_name.to_string(),
                        loop_var_ty: elem_ty.clone(),
                        iterator_var: iter_var,
                        iterator_class: iter_class.clone(),
                        next_mangled: next_mangled.clone(),
                        body,
                        else_body: vec![],
                    },
                ]
            }
            GenKind::Tuple {
                tuple_expr,
                elem_ty,
                len,
            } => {
                let tuple_var = self.fresh_internal("comp_tuple");
                let idx_var = self.fresh_internal("comp_tuple_idx");
                let start_name = self.fresh_internal("comp_start");
                let stop_name = self.fresh_internal("comp_stop");
                let step_name = self.fresh_internal("comp_step");

                // Prepend: var_name = tuple[idx_var]
                let tuple_element_types = match &tuple_expr.ty {
                    ValueType::Tuple(types) => types.clone(),
                    _ => vec![elem_ty.clone(); *len],
                };
                let mut full_body = vec![TirStmt::Let {
                    name: var_name.to_string(),
                    ty: elem_ty.clone(),
                    value: TirExpr {
                        kind: TirExprKind::TupleGetDynamic {
                            tuple: Box::new(TirExpr {
                                kind: TirExprKind::Var(tuple_var.clone()),
                                ty: tuple_expr.ty.clone(),
                            }),
                            index: Box::new(TirExpr {
                                kind: TirExprKind::Var(idx_var.clone()),
                                ty: ValueType::Int,
                            }),
                            len: *len,
                            element_types: tuple_element_types,
                        },
                        ty: elem_ty.clone(),
                    },
                }];
                full_body.extend(body);

                vec![
                    TirStmt::Let {
                        name: tuple_var,
                        ty: tuple_expr.ty.clone(),
                        value: tuple_expr.clone(),
                    },
                    TirStmt::Let {
                        name: start_name.clone(),
                        ty: ValueType::Int,
                        value: TirExpr {
                            kind: TirExprKind::IntLiteral(0),
                            ty: ValueType::Int,
                        },
                    },
                    TirStmt::Let {
                        name: stop_name.clone(),
                        ty: ValueType::Int,
                        value: TirExpr {
                            kind: TirExprKind::IntLiteral(*len as i64),
                            ty: ValueType::Int,
                        },
                    },
                    TirStmt::Let {
                        name: step_name.clone(),
                        ty: ValueType::Int,
                        value: TirExpr {
                            kind: TirExprKind::IntLiteral(1),
                            ty: ValueType::Int,
                        },
                    },
                    TirStmt::ForRange {
                        loop_var: idx_var,
                        start_var: start_name,
                        stop_var: stop_name,
                        step_var: step_name,
                        body: full_body,
                        else_body: vec![],
                    },
                ]
            }
            GenKind::Zip2 {
                left_name,
                right_name,
                left_expr,
                right_expr,
                left_elem,
                right_elem,
            } => {
                let left_var = self.fresh_internal("comp_zip_left");
                let right_var = self.fresh_internal("comp_zip_right");
                let len_left_var = self.fresh_internal("comp_zip_len_left");
                let len_right_var = self.fresh_internal("comp_zip_len_right");
                let start_var = self.fresh_internal("comp_zip_start");
                let stop_var = self.fresh_internal("comp_zip_stop");
                let step_var = self.fresh_internal("comp_zip_step");
                let idx_var = self.fresh_internal("comp_zip_idx");

                let left_ty = left_expr.ty.clone();
                let right_ty = right_expr.ty.clone();
                let mut full_body = vec![
                    TirStmt::Let {
                        name: left_name.clone(),
                        ty: left_elem.clone(),
                        value: TirExpr {
                            kind: TirExprKind::ExternalCall {
                                func: builtin::BuiltinFn::ListGet,
                                args: vec![
                                    TirExpr {
                                        kind: TirExprKind::Var(left_var.clone()),
                                        ty: left_ty.clone(),
                                    },
                                    TirExpr {
                                        kind: TirExprKind::Var(idx_var.clone()),
                                        ty: ValueType::Int,
                                    },
                                ],
                            },
                            ty: left_elem.clone(),
                        },
                    },
                    TirStmt::Let {
                        name: right_name.clone(),
                        ty: right_elem.clone(),
                        value: TirExpr {
                            kind: TirExprKind::ExternalCall {
                                func: builtin::BuiltinFn::ListGet,
                                args: vec![
                                    TirExpr {
                                        kind: TirExprKind::Var(right_var.clone()),
                                        ty: right_ty.clone(),
                                    },
                                    TirExpr {
                                        kind: TirExprKind::Var(idx_var.clone()),
                                        ty: ValueType::Int,
                                    },
                                ],
                            },
                            ty: right_elem.clone(),
                        },
                    },
                ];
                full_body.extend(body);

                vec![
                    TirStmt::Let {
                        name: left_var.clone(),
                        ty: left_ty.clone(),
                        value: left_expr.clone(),
                    },
                    TirStmt::Let {
                        name: right_var.clone(),
                        ty: right_ty.clone(),
                        value: right_expr.clone(),
                    },
                    TirStmt::Let {
                        name: len_left_var.clone(),
                        ty: ValueType::Int,
                        value: TirExpr {
                            kind: TirExprKind::ExternalCall {
                                func: builtin::BuiltinFn::ListLen,
                                args: vec![TirExpr {
                                    kind: TirExprKind::Var(left_var),
                                    ty: left_ty,
                                }],
                            },
                            ty: ValueType::Int,
                        },
                    },
                    TirStmt::Let {
                        name: len_right_var.clone(),
                        ty: ValueType::Int,
                        value: TirExpr {
                            kind: TirExprKind::ExternalCall {
                                func: builtin::BuiltinFn::ListLen,
                                args: vec![TirExpr {
                                    kind: TirExprKind::Var(right_var),
                                    ty: right_ty,
                                }],
                            },
                            ty: ValueType::Int,
                        },
                    },
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
                            kind: TirExprKind::ExternalCall {
                                func: builtin::BuiltinFn::MinInt,
                                args: vec![
                                    TirExpr {
                                        kind: TirExprKind::Var(len_left_var),
                                        ty: ValueType::Int,
                                    },
                                    TirExpr {
                                        kind: TirExprKind::Var(len_right_var),
                                        ty: ValueType::Int,
                                    },
                                ],
                            },
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
                        body: full_body,
                        else_body: vec![],
                    },
                ]
            }
            GenKind::Enumerate {
                idx_name,
                value_name,
                list_expr,
                elem_ty,
            } => {
                let list_var = self.fresh_internal("comp_enum_list");
                let len_var = self.fresh_internal("comp_enum_len");
                let start_var = self.fresh_internal("comp_enum_start");
                let stop_var = self.fresh_internal("comp_enum_stop");
                let step_var = self.fresh_internal("comp_enum_step");
                let idx_var = self.fresh_internal("comp_enum_idx");

                let list_ty = list_expr.ty.clone();
                let mut full_body = vec![
                    TirStmt::Let {
                        name: idx_name.clone(),
                        ty: ValueType::Int,
                        value: TirExpr {
                            kind: TirExprKind::Var(idx_var.clone()),
                            ty: ValueType::Int,
                        },
                    },
                    TirStmt::Let {
                        name: value_name.clone(),
                        ty: elem_ty.clone(),
                        value: TirExpr {
                            kind: TirExprKind::ExternalCall {
                                func: builtin::BuiltinFn::ListGet,
                                args: vec![
                                    TirExpr {
                                        kind: TirExprKind::Var(list_var.clone()),
                                        ty: list_ty.clone(),
                                    },
                                    TirExpr {
                                        kind: TirExprKind::Var(idx_var.clone()),
                                        ty: ValueType::Int,
                                    },
                                ],
                            },
                            ty: elem_ty.clone(),
                        },
                    },
                ];
                full_body.extend(body);

                vec![
                    TirStmt::Let {
                        name: list_var.clone(),
                        ty: list_ty.clone(),
                        value: list_expr.clone(),
                    },
                    TirStmt::Let {
                        name: len_var.clone(),
                        ty: ValueType::Int,
                        value: TirExpr {
                            kind: TirExprKind::ExternalCall {
                                func: builtin::BuiltinFn::ListLen,
                                args: vec![TirExpr {
                                    kind: TirExprKind::Var(list_var),
                                    ty: list_ty,
                                }],
                            },
                            ty: ValueType::Int,
                        },
                    },
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
                            kind: TirExprKind::Var(len_var),
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
                        body: full_body,
                        else_body: vec![],
                    },
                ]
            }
        }
    }
}

struct GenInfo {
    var_name: String,
    kind: GenKind,
    if_conds: Vec<TirExpr>,
}

enum GenKind {
    Range {
        start: TirExpr,
        stop: TirExpr,
        step: TirExpr,
    },
    List {
        list_expr: TirExpr,
        elem_ty: ValueType,
    },
    Str {
        str_expr: TirExpr,
        elem_ty: ValueType,
    },
    ClassIter {
        obj_expr: TirExpr,
        iter_mangled: String,
        iter_class: String,
        next_mangled: String,
        elem_ty: ValueType,
    },
    Tuple {
        tuple_expr: TirExpr,
        elem_ty: ValueType,
        len: usize,
    },
    Zip2 {
        left_name: String,
        right_name: String,
        left_expr: TirExpr,
        right_expr: TirExpr,
        left_elem: ValueType,
        right_elem: ValueType,
    },
    Enumerate {
        idx_name: String,
        value_name: String,
        list_expr: TirExpr,
        elem_ty: ValueType,
    },
}
