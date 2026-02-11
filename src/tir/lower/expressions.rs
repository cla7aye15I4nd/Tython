use anyhow::Result;
use pyo3::prelude::*;

use crate::ast::Type;
use crate::tir::{
    builtin, type_rules, CallResult, CallTarget, CastKind, LogicalOp, TirExpr, TirExprKind,
    TirStmt, TypedBinOp, ValueType,
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
                let op = Self::convert_binop(&ast_getattr!(node, "op"))?;

                let (final_left, final_right, result_ty) =
                    self.resolve_binop_types(line, op, left, right)?;

                Ok(TirExpr {
                    kind: TirExprKind::BinOp {
                        op,
                        left: Box::new(final_left),
                        right: Box::new(final_right),
                    },
                    ty: Self::to_value_type(&result_ty),
                })
            }

            "Compare" => {
                let left = self.lower_expr(&ast_getattr!(node, "left"))?;
                let ops_list = ast_get_list!(node, "ops");
                let comparators_list = ast_get_list!(node, "comparators");

                if ops_list.len() == 1 {
                    let op_node = ops_list.get_item(0)?;
                    let cmp_op = Self::convert_cmpop(&op_node)?;
                    let right = self.lower_expr(&comparators_list.get_item(0)?)?;
                    let (fl, fr) = self.promote_for_comparison(line, left, right)?;
                    return Ok(TirExpr {
                        kind: TirExprKind::Compare {
                            op: cmp_op,
                            left: Box::new(fl),
                            right: Box::new(fr),
                        },
                        ty: ValueType::Bool,
                    });
                }

                let mut comparisons: Vec<TirExpr> = Vec::new();
                let mut current_left = left;

                for i in 0..ops_list.len() {
                    let op_node = ops_list.get_item(i)?;
                    let cmp_op = Self::convert_cmpop(&op_node)?;
                    let right = self.lower_expr(&comparators_list.get_item(i)?)?;

                    let (fl, fr) =
                        self.promote_for_comparison(line, current_left.clone(), right.clone())?;

                    comparisons.push(TirExpr {
                        kind: TirExprKind::Compare {
                            op: cmp_op,
                            left: Box::new(fl),
                            right: Box::new(fr),
                        },
                        ty: ValueType::Bool,
                    });

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
                        field_name: attr_name,
                        field_index,
                    },
                    ty: field_ty,
                })
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

    // ── type promotion / binary ops ────────────────────────────────────

    pub(super) fn resolve_binop_types(
        &self,
        line: usize,
        op: TypedBinOp,
        left: TirExpr,
        right: TirExpr,
    ) -> Result<(TirExpr, TirExpr, Type)> {
        let left_ast = left.ty.to_type();
        let right_ast = right.ty.to_type();
        let rule = type_rules::lookup_binop(op, &left_ast, &right_ast).ok_or_else(|| {
            self.type_error(
                line,
                type_rules::binop_type_error_message(op, &left_ast, &right_ast),
            )
        })?;

        let final_left = Self::apply_coercion(left, rule.left_coercion);
        let final_right = Self::apply_coercion(right, rule.right_coercion);

        Ok((final_left, final_right, rule.result_type))
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
            // Promote whichever side is Int to Float; apply_coercion is a no-op on Float.
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
