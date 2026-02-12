use anyhow::Result;
use pyo3::prelude::*;

use crate::tir::{
    builtin, type_rules, ArithBinOp, CallResult, CallTarget, CastKind, CmpOp, LogicalOp,
    OrderedCmpOp, RawBinOp, TirExpr, TirExprKind, TirStmt, Type, ValueType,
};
use crate::{ast_get_list, ast_get_string, ast_getattr, ast_type_name};

use super::Lowering;

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
                        kind: TirExprKind::IntLiteral(if bool_val { 1 } else { 0 }),
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

                // If it's a function type and a known function definition,
                // produce a FuncRef (function pointer) expression.
                if let Type::Function {
                    ref params,
                    ref return_type,
                } = ty
                {
                    if let Some(mangled) = self.function_mangled_names.get(&id).cloned() {
                        let vt_params: Vec<ValueType> =
                            params.iter().map(Self::to_value_type).collect();
                        let vt_ret = Self::to_opt_value_type(return_type);
                        return Ok(TirExpr {
                            kind: TirExprKind::FuncRef {
                                mangled_name: mangled,
                            },
                            ty: ValueType::Function {
                                params: vt_params,
                                return_type: vt_ret.map(Box::new),
                            },
                        });
                    }
                }

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
                        kind: TirExprKind::LogicalOp {
                            op: LogicalOp::And,
                            left: Box::new(result),
                            right: Box::new(cmp),
                        },
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

                let rule =
                    type_rules::lookup_unaryop(op, &operand.ty.to_type()).ok_or_else(|| {
                        self.type_error(
                            line,
                            type_rules::unaryop_type_error_message(op, &operand.ty.to_type()),
                        )
                    })?;

                Ok(TirExpr {
                    kind: TirExprKind::UnaryOp {
                        op,
                        operand: Box::new(operand),
                    },
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
                        kind: TirExprKind::LogicalOp {
                            op: logical_op,
                            left: Box::new(result),
                            right: Box::new(operand),
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

            "ListComp" => self.lower_list_comprehension(node, line),

            "Subscript" => {
                let value_node = ast_getattr!(node, "value");
                let slice_node = ast_getattr!(node, "slice");
                let obj_expr = self.lower_expr(&value_node)?;

                match obj_expr.ty.clone() {
                    ValueType::List(inner) => {
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

                    self.lower_print_value_stmts(line, element_expr, stmts)?;
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
            ValueType::List(inner) => match inner.as_ref() {
                ValueType::Int => push_direct_print!(builtin::BuiltinFn::PrintListInt, arg),
                ValueType::Float => push_direct_print!(builtin::BuiltinFn::PrintListFloat, arg),
                ValueType::Bool => push_direct_print!(builtin::BuiltinFn::PrintListBool, arg),
                ValueType::Str => push_direct_print!(builtin::BuiltinFn::PrintListStr, arg),
                ValueType::Bytes => push_direct_print!(builtin::BuiltinFn::PrintListBytes, arg),
                ValueType::ByteArray => {
                    push_direct_print!(builtin::BuiltinFn::PrintListByteArray, arg)
                }
                _ => {
                    let list_var = self.fresh_internal("print_list");
                    let idx_var = self.fresh_internal("print_idx");
                    let len_var = self.fresh_internal("print_len");
                    let loop_var = self.fresh_internal("print_elem");
                    let list_ty = arg.ty.clone();
                    let loop_var_ty = inner.as_ref().clone();

                    stmts.push(TirStmt::Let {
                        name: list_var.clone(),
                        ty: list_ty,
                        value: arg,
                    });
                    Self::push_print_str_literal(stmts, "[");

                    let mut body = Vec::new();
                    let idx_gt_zero = TirExpr {
                        kind: TirExprKind::Compare {
                            op: OrderedCmpOp::Gt,
                            left: Box::new(TirExpr {
                                kind: TirExprKind::Var(idx_var.clone()),
                                ty: ValueType::Int,
                            }),
                            right: Box::new(TirExpr {
                                kind: TirExprKind::IntLiteral(0),
                                ty: ValueType::Int,
                            }),
                        },
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
                    self.lower_print_value_stmts(line, elem_expr, &mut body)?;

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
            },
            ValueType::Function { .. } => {
                Err(self.type_error(line, format!("cannot print value of type `{}`", arg.ty)))
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

        Ok(TirExpr {
            kind: TirExprKind::BinOp {
                op: typed_op,
                left: Box::new(final_left),
                right: Box::new(final_right),
            },
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
            (RawBinOp::Arith(ArithBinOp::Mul), ValueType::Str) => {
                Some(builtin::BuiltinFn::StrRepeat)
            }
            (RawBinOp::Arith(ArithBinOp::Mul), ValueType::Bytes) => {
                Some(builtin::BuiltinFn::BytesRepeat)
            }
            (RawBinOp::Arith(ArithBinOp::Mul), ValueType::ByteArray) => {
                Some(builtin::BuiltinFn::ByteArrayRepeat)
            }
            _ => None,
        }
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
                    kind: TirExprKind::UnaryOp {
                        op: crate::tir::UnaryOpKind::Not,
                        operand: Box::new(contains_expr),
                    },
                    ty: ValueType::Bool,
                });
            }
            return Ok(contains_expr);
        }

        // `is` / `is not` — identity (pointer equality for ref types, value equality for primitives)
        if matches!(cmp_op, CmpOp::Is | CmpOp::IsNot) {
            let eq_op = if cmp_op == CmpOp::Is {
                OrderedCmpOp::Eq
            } else {
                OrderedCmpOp::NotEq
            };
            return Ok(TirExpr {
                kind: TirExprKind::Compare {
                    op: eq_op,
                    left: Box::new(left),
                    right: Box::new(right),
                },
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
                    kind: TirExprKind::Compare {
                        op: OrderedCmpOp::Eq,
                        left: Box::new(eq_expr),
                        right: Box::new(TirExpr {
                            kind: TirExprKind::IntLiteral(0),
                            ty: ValueType::Int,
                        }),
                    },
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
                    kind: TirExprKind::Compare {
                        op: OrderedCmpOp::Eq,
                        left: Box::new(eq_expr),
                        right: Box::new(TirExpr {
                            kind: TirExprKind::IntLiteral(0),
                            ty: ValueType::Int,
                        }),
                    },
                    ty: ValueType::Bool,
                });
            }
            return Ok(eq_expr);
        }

        // Numeric comparison with optional promotion
        let (fl, fr) = self.promote_for_comparison(line, left, right)?;
        Ok(TirExpr {
            kind: TirExprKind::Compare {
                op: OrderedCmpOp::from_cmp_op(cmp_op),
                left: Box::new(fl),
                right: Box::new(fr),
            },
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
                    kind: TirExprKind::IntLiteral(1),
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
                kind: TirExprKind::Compare {
                    op: OrderedCmpOp::Eq,
                    left: Box::new(elem_eq),
                    right: Box::new(TirExpr {
                        kind: TirExprKind::IntLiteral(0),
                        ty: ValueType::Int,
                    }),
                },
                ty: ValueType::Bool,
            },
            then_body: vec![
                TirStmt::Let {
                    name: result_var.clone(),
                    ty: ValueType::Bool,
                    value: TirExpr {
                        kind: TirExprKind::IntLiteral(0),
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
                kind: TirExprKind::Compare {
                    op: OrderedCmpOp::NotEq,
                    left: Box::new(TirExpr {
                        kind: TirExprKind::Var(len_a_var),
                        ty: ValueType::Int,
                    }),
                    right: Box::new(TirExpr {
                        kind: TirExprKind::Var(len_b_var.clone()),
                        ty: ValueType::Int,
                    }),
                },
                ty: ValueType::Bool,
            },
            then_body: vec![TirStmt::Let {
                name: result_var.clone(),
                ty: ValueType::Bool,
                value: TirExpr {
                    kind: TirExprKind::IntLiteral(0),
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
                kind: TirExprKind::IntLiteral(1),
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
                kind: TirExprKind::LogicalOp {
                    op: LogicalOp::And,
                    left: Box::new(result),
                    right: Box::new(cmp),
                },
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
                kind: TirExprKind::Compare {
                    op: OrderedCmpOp::Eq,
                    left: Box::new(left),
                    right: Box::new(right),
                },
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
                    kind: TirExprKind::Compare {
                        op: OrderedCmpOp::Eq,
                        left: Box::new(left),
                        right: Box::new(right),
                    },
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
                    kind: TirExprKind::Compare {
                        op: OrderedCmpOp::Eq,
                        left: Box::new(eq_call),
                        right: Box::new(zero),
                    },
                    ty: ValueType::Bool,
                }
            }
            ordered => {
                // str_cmp(a,b) <op> 0
                let cmp_call = TirExpr {
                    kind: TirExprKind::ExternalCall {
                        func: cmp_fn,
                        args: vec![left, right],
                    },
                    ty: ValueType::Int,
                };
                TirExpr {
                    kind: TirExprKind::Compare {
                        op: ordered,
                        left: Box::new(cmp_call),
                        right: Box::new(zero),
                    },
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
            if ast_type_name!(target) != "Name" {
                return Err(
                    self.syntax_error(line, "comprehension target must be a simple variable")
                );
            }
            let var_name = ast_get_string!(target, "id");
            let iter_node = ast_getattr!(gen, "iter");

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

            // Lower if conditions
            let ifs_list = ast_get_list!(gen, "ifs");
            let mut if_conds = Vec::new();
            for if_node in ifs_list.iter() {
                if_conds.push(self.lower_expr(&if_node)?);
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
                        kind: TirExprKind::LogicalOp {
                            op: LogicalOp::And,
                            left: Box::new(a),
                            right: Box::new(b),
                        },
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
}
