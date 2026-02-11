use anyhow::Result;
use pyo3::prelude::*;

use crate::tir::{
    builtin, type_rules, ArithBinOp, CallResult, CallTarget, CastKind, CmpOp, LogicalOp, RawBinOp,
    TirExpr, TirExprKind, TirStmt, ValueType,
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

                let op = Self::convert_unaryop(&op_type)?;

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

            "Subscript" => {
                let value_node = ast_getattr!(node, "value");
                let slice_node = ast_getattr!(node, "slice");
                let obj_expr = self.lower_expr(&value_node)?;

                match &obj_expr.ty {
                    ValueType::List(inner) => {
                        let index_expr = self.lower_expr(&slice_node)?;
                        if index_expr.ty != ValueType::Int {
                            return Err(self.type_error(
                                line,
                                format!("list index must be `int`, got `{}`", index_expr.ty),
                            ));
                        }
                        let elem_ty = inner.as_ref().clone();
                        Ok(TirExpr {
                            kind: TirExprKind::ExternalCall {
                                func: builtin::BuiltinFn::ListGet,
                                args: vec![obj_expr, index_expr],
                            },
                            ty: elem_ty,
                        })
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

    // ── print statement ────────────────────────────────────────────────

    pub(super) fn lower_print_stmt(&mut self, call_node: &Bound<PyAny>) -> Result<Vec<TirStmt>> {
        let line = Self::get_line(call_node);
        let args_list = ast_get_list!(call_node, "args");

        let mut tir_args = Vec::new();
        for arg in args_list.iter() {
            tir_args.push(self.lower_expr(&arg)?);
        }

        if tir_args.is_empty() {
            return Ok(vec![TirStmt::VoidCall {
                target: CallTarget::Builtin(builtin::BuiltinFn::PrintNewline),
                args: vec![],
            }]);
        }

        let mut stmts = Vec::new();
        for (i, arg) in tir_args.into_iter().enumerate() {
            if i > 0 {
                stmts.push(TirStmt::VoidCall {
                    target: CallTarget::Builtin(builtin::BuiltinFn::PrintSpace),
                    args: vec![],
                });
            }
            let print_fn = builtin::resolve_print(&arg.ty).ok_or_else(|| {
                self.type_error(line, format!("cannot print value of type `{}`", arg.ty))
            })?;
            stmts.push(TirStmt::VoidCall {
                target: CallTarget::Builtin(print_fn),
                args: vec![arg],
            });
        }
        stmts.push(TirStmt::VoidCall {
            target: CallTarget::Builtin(builtin::BuiltinFn::PrintNewline),
            args: vec![],
        });

        Ok(stmts)
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
        &self,
        line: usize,
        cmp_op: CmpOp,
        left: TirExpr,
        right: TirExpr,
    ) -> Result<TirExpr> {
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
                cmp_op, eq_fn, cmp_fn, left, right,
            ));
        }

        // Numeric comparison with optional promotion
        let (fl, fr) = self.promote_for_comparison(line, left, right)?;
        Ok(TirExpr {
            kind: TirExprKind::Compare {
                op: cmp_op,
                left: Box::new(fl),
                right: Box::new(fr),
            },
            ty: ValueType::Bool,
        })
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
        cmp_op: CmpOp,
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
            CmpOp::Eq => {
                // str_eq returns 1 if equal, 0 if not — usable directly as Bool.
                TirExpr {
                    kind: TirExprKind::ExternalCall {
                        func: eq_fn,
                        args: vec![left, right],
                    },
                    ty: ValueType::Bool,
                }
            }
            CmpOp::NotEq => {
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
                        op: CmpOp::Eq,
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
            Ok((left, right))
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
}
