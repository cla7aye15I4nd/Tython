use anyhow::Result;
use pyo3::prelude::*;
use std::collections::HashMap;

use crate::ast::{ClassInfo, Type};
use crate::tir::{
    builtin, ArithBinOp, CallResult, CallTarget, CastKind, TirExpr, TirExprKind, TirStmt,
    TypedBinOp, ValueType,
};
use crate::{ast_get_list, ast_get_string, ast_getattr, ast_type_name};

use super::Lowering;

impl Lowering {
    pub(super) fn lower_stmt(&mut self, node: &Bound<PyAny>) -> Result<Vec<TirStmt>> {
        let node_type = ast_type_name!(node);
        let line = Self::get_line(node);

        match node_type.as_str() {
            "FunctionDef" => {
                Err(self.syntax_error(line, "nested function definitions are not supported"))
            }
            "ClassDef" => self.handle_class_def_stmt(node, line),
            "AnnAssign" => self.handle_ann_assign(node, line),
            "Assign" => self.handle_assign(node, line),
            "AugAssign" => self.handle_aug_assign(node, line),
            "Return" => self.handle_return(node, line),
            "Expr" => self.handle_expr_stmt(node, line),
            "If" => {
                let condition = self.lower_expr(&ast_getattr!(node, "test"))?;
                let then_body = self.lower_block(&ast_get_list!(node, "body"))?;
                let else_body = self.lower_block(&ast_get_list!(node, "orelse"))?;
                Ok(vec![TirStmt::If {
                    condition,
                    then_body,
                    else_body,
                }])
            }
            "While" => {
                let condition = self.lower_expr(&ast_getattr!(node, "test"))?;
                let body = self.lower_block(&ast_get_list!(node, "body"))?;
                Ok(vec![TirStmt::While { condition, body }])
            }
            "Break" => Ok(vec![TirStmt::Break]),
            "Continue" => Ok(vec![TirStmt::Continue]),
            "Assert" => self.handle_assert(node, line),
            _ => {
                Err(self.syntax_error(line, format!("unsupported statement type: `{}`", node_type)))
            }
        }
    }

    fn handle_class_def_stmt(&mut self, node: &Bound<PyAny>, _line: usize) -> Result<Vec<TirStmt>> {
        let raw_name = ast_get_string!(node, "name");
        let fn_name = self.current_function_name.as_deref().unwrap_or("_");
        let qualified = format!("{}${}${}", self.current_module_name, fn_name, raw_name);

        self.class_registry.insert(
            qualified.clone(),
            ClassInfo {
                name: qualified.clone(),
                fields: Vec::new(),
                methods: HashMap::new(),
                field_map: HashMap::new(),
            },
        );
        self.declare(raw_name, Type::Class(qualified.clone()));

        let body = ast_get_list!(node, "body");
        self.discover_classes(&body, &qualified)?;
        self.collect_class_definition(node, &qualified)?;
        self.collect_classes(&body, &qualified)?;

        let (class_infos, methods) = self.lower_class_def(node, &qualified)?;
        self.deferred_classes.extend(class_infos);
        self.deferred_functions.extend(methods);

        Ok(vec![])
    }

    fn handle_ann_assign(&mut self, node: &Bound<PyAny>, line: usize) -> Result<Vec<TirStmt>> {
        let target_node = ast_getattr!(node, "target");
        if ast_type_name!(target_node) != "Name" {
            return Err(self.syntax_error(line, "only simple variable assignments are supported"));
        }
        let target = ast_get_string!(target_node, "id");

        let annotation = ast_getattr!(node, "annotation");
        let annotated_ty = (!annotation.is_none())
            .then(|| self.convert_type_annotation(&annotation))
            .transpose()?;

        let value_node = ast_getattr!(node, "value");
        let tir_value = self.lower_expr(&value_node)?;

        let tir_value_ast_ty = tir_value.ty.to_type();
        if let Some(ref ann_ty) = annotated_ty {
            if ann_ty != &tir_value_ast_ty {
                return Err(self.type_error(
                    line,
                    format!(
                        "type mismatch: expected `{}`, got `{}`",
                        ann_ty, tir_value_ast_ty
                    ),
                ));
            }
        }

        let var_type = annotated_ty.unwrap_or(tir_value_ast_ty);
        self.declare(target.clone(), var_type.clone());

        Ok(vec![TirStmt::Let {
            name: target,
            ty: Self::to_value_type(&var_type),
            value: tir_value,
        }])
    }

    fn handle_assign(&mut self, node: &Bound<PyAny>, line: usize) -> Result<Vec<TirStmt>> {
        let targets_list = ast_get_list!(node, "targets");
        if targets_list.len() != 1 {
            return Err(self.syntax_error(line, "multiple assignment targets are not supported"));
        }

        let target_node = targets_list.get_item(0)?;
        match ast_type_name!(target_node).as_str() {
            "Name" => {
                let target = ast_get_string!(target_node, "id");
                let value_node = ast_getattr!(node, "value");
                let tir_value = self.lower_expr(&value_node)?;
                let var_type = tir_value.ty.to_type();
                self.declare(target.clone(), var_type);

                Ok(vec![TirStmt::Let {
                    name: target,
                    ty: tir_value.ty.clone(),
                    value: tir_value,
                }])
            }
            "Attribute" => self.lower_attribute_assign(&target_node, node, line),
            _ => {
                Err(self.syntax_error(line, "only variable or attribute assignments are supported"))
            }
        }
    }

    fn handle_aug_assign(&mut self, node: &Bound<PyAny>, line: usize) -> Result<Vec<TirStmt>> {
        let target_node = ast_getattr!(node, "target");
        match ast_type_name!(target_node).as_str() {
            "Name" => {
                let target = ast_get_string!(target_node, "id");

                let target_ty = self.lookup(&target).cloned().ok_or_else(|| {
                    self.name_error(line, format!("undefined variable `{}`", target))
                })?;

                let op = Self::convert_binop(&ast_getattr!(node, "op"))?;
                let value_expr = self.lower_expr(&ast_getattr!(node, "value"))?;

                if op == TypedBinOp::Arith(ArithBinOp::Div) && target_ty == Type::Int {
                    return Err(self.type_error(
                        line,
                        format!("`/=` on `int` variable `{}` would change type to `float`; use `//=` for integer division", target),
                    ));
                }

                let target_ref = TirExpr {
                    kind: TirExprKind::Var(target.clone()),
                    ty: Self::to_value_type(&target_ty),
                };

                let (final_left, final_right, result_ty) =
                    self.resolve_binop_types(line, op, target_ref, value_expr)?;

                let result_vty = Self::to_value_type(&result_ty);
                let binop_expr = TirExpr {
                    kind: TirExprKind::BinOp {
                        op,
                        left: Box::new(final_left),
                        right: Box::new(final_right),
                    },
                    ty: result_vty.clone(),
                };

                self.declare(target.clone(), result_ty);

                Ok(vec![TirStmt::Let {
                    name: target,
                    ty: result_vty,
                    value: binop_expr,
                }])
            }
            "Attribute" => self.lower_attribute_aug_assign(&target_node, node, line),
            _ => Err(self.syntax_error(
                line,
                "only variable or attribute augmented assignments are supported",
            )),
        }
    }

    fn handle_return(&mut self, node: &Bound<PyAny>, line: usize) -> Result<Vec<TirStmt>> {
        let value_node = ast_getattr!(node, "value");
        if value_node.is_none() {
            if let Some(ref expected) = self.current_return_type {
                if *expected != Type::Unit {
                    return Err(self.type_error(
                        line,
                        format!("return without value, but function expects `{}`", expected),
                    ));
                }
            }
            Ok(vec![TirStmt::Return(None)])
        } else {
            let tir_expr = self.lower_expr(&value_node)?;
            if let Some(ref expected) = self.current_return_type {
                if *expected != tir_expr.ty.to_type() {
                    return Err(self.type_error(
                        line,
                        format!(
                            "return type mismatch: expected `{}`, got `{}`",
                            expected, tir_expr.ty
                        ),
                    ));
                }
            }
            Ok(vec![TirStmt::Return(Some(tir_expr))])
        }
    }

    fn handle_expr_stmt(&mut self, node: &Bound<PyAny>, line: usize) -> Result<Vec<TirStmt>> {
        let value_node = ast_getattr!(node, "value");

        if ast_type_name!(value_node) == "Call" {
            let func_node = ast_getattr!(value_node, "func");
            if ast_type_name!(func_node) == "Name" && ast_get_string!(func_node, "id") == "print" {
                return self.lower_print_stmt(&value_node);
            }

            let call_result = self.lower_call(&value_node, line)?;
            return match call_result {
                CallResult::Expr(expr) => Ok(vec![TirStmt::Expr(expr)]),
                CallResult::VoidStmt(stmt) => Ok(vec![stmt]),
            };
        }

        Ok(vec![TirStmt::Expr(self.lower_expr(&value_node)?)])
    }

    fn handle_assert(&mut self, node: &Bound<PyAny>, line: usize) -> Result<Vec<TirStmt>> {
        let test_node = ast_getattr!(node, "test");
        let condition = self.lower_expr(&test_node)?;

        let bool_condition = if condition.ty == ValueType::Bool {
            condition
        } else {
            match &condition.ty {
                ValueType::Int => TirExpr {
                    kind: TirExprKind::Cast {
                        kind: CastKind::IntToBool,
                        arg: Box::new(condition),
                    },
                    ty: ValueType::Bool,
                },
                ValueType::Float => TirExpr {
                    kind: TirExprKind::Cast {
                        kind: CastKind::FloatToBool,
                        arg: Box::new(condition),
                    },
                    ty: ValueType::Bool,
                },
                ValueType::Str | ValueType::Bytes | ValueType::ByteArray => {
                    let len_fn = match &condition.ty {
                        ValueType::Str => builtin::BuiltinFn::StrLen,
                        ValueType::Bytes => builtin::BuiltinFn::BytesLen,
                        ValueType::ByteArray => builtin::BuiltinFn::ByteArrayLen,
                        _ => unreachable!(),
                    };
                    let len_expr = TirExpr {
                        kind: TirExprKind::ExternalCall {
                            func: len_fn,
                            args: vec![condition],
                        },
                        ty: ValueType::Int,
                    };
                    TirExpr {
                        kind: TirExprKind::Cast {
                            kind: CastKind::IntToBool,
                            arg: Box::new(len_expr),
                        },
                        ty: ValueType::Bool,
                    }
                }
                _ => {
                    return Err(
                        self.type_error(line, format!("cannot use `{}` in assert", condition.ty))
                    )
                }
            }
        };

        Ok(vec![TirStmt::VoidCall {
            target: CallTarget::Builtin(builtin::BuiltinFn::Assert),
            args: vec![bool_condition],
        }])
    }

    // ── attribute assignment ───────────────────────────────────────────

    fn lower_attribute_assign(
        &mut self,
        target_node: &Bound<PyAny>,
        assign_node: &Bound<PyAny>,
        line: usize,
    ) -> Result<Vec<TirStmt>> {
        let obj_node = ast_getattr!(target_node, "value");
        let field_name = ast_get_string!(target_node, "attr");
        let obj_expr = self.lower_expr(&obj_node)?;

        let class_name = match &obj_expr.ty {
            ValueType::Class(name) => name.clone(),
            other => {
                return Err(self.type_error(
                    line,
                    format!("cannot set attribute on non-class type `{}`", other),
                ))
            }
        };

        let class_info = self.lookup_class(line, &class_name)?;
        let field_index = self.lookup_field_index(line, &class_info, &field_name)?;
        let field = &class_info.fields[field_index];

        // Enforce reference-type field immutability outside __init__
        if field.ty.is_reference_type() {
            let inside_init = self.current_class.as_ref() == Some(&class_name)
                && self
                    .current_function_name
                    .as_ref()
                    .map(|n| n.ends_with(".__init__"))
                    .unwrap_or(false);
            let is_self = matches!(&obj_expr.kind, TirExprKind::Var(name) if name == "self");

            if !(inside_init && is_self) {
                return Err(self.type_error(
                    line,
                    format!(
                        "cannot reassign reference field `{}.{}` of type `{}` outside of __init__",
                        class_name, field_name, field.ty
                    ),
                ));
            }
        }

        let value_node = ast_getattr!(assign_node, "value");
        let tir_value = self.lower_expr(&value_node)?;

        if tir_value.ty.to_type() != field.ty {
            return Err(self.type_error(
                line,
                format!(
                    "cannot assign `{}` to field `{}.{}` of type `{}`",
                    tir_value.ty, class_name, field_name, field.ty
                ),
            ));
        }

        Ok(vec![TirStmt::SetField {
            object: obj_expr,
            field_name,
            field_index,
            value: tir_value,
        }])
    }

    fn lower_attribute_aug_assign(
        &mut self,
        target_node: &Bound<PyAny>,
        aug_node: &Bound<PyAny>,
        line: usize,
    ) -> Result<Vec<TirStmt>> {
        let obj_node = ast_getattr!(target_node, "value");
        let field_name = ast_get_string!(target_node, "attr");
        let obj_expr = self.lower_expr(&obj_node)?;

        let class_name = match &obj_expr.ty {
            ValueType::Class(name) => name.clone(),
            other => {
                return Err(self.type_error(
                    line,
                    format!("cannot set attribute on non-class type `{}`", other),
                ))
            }
        };

        let class_info = self.lookup_class(line, &class_name)?;
        let field_index = self.lookup_field_index(line, &class_info, &field_name)?;
        let field = &class_info.fields[field_index];
        let field_vty = Self::to_value_type(&field.ty);

        // Read current field value
        let current_val = TirExpr {
            kind: TirExprKind::GetField {
                object: Box::new(obj_expr.clone()),
                field_name: field_name.clone(),
                field_index,
            },
            ty: field_vty,
        };

        let op = Self::convert_binop(&ast_getattr!(aug_node, "op"))?;
        let rhs = self.lower_expr(&ast_getattr!(aug_node, "value"))?;

        let (final_left, final_right, result_ty) =
            self.resolve_binop_types(line, op, current_val, rhs)?;

        if result_ty != field.ty {
            return Err(self.type_error(
                line,
                format!(
                    "augmented assignment would change field `{}.{}` type from `{}` to `{}`",
                    class_name, field_name, field.ty, result_ty
                ),
            ));
        }

        let result_vty = Self::to_value_type(&result_ty);
        let binop_expr = TirExpr {
            kind: TirExprKind::BinOp {
                op,
                left: Box::new(final_left),
                right: Box::new(final_right),
            },
            ty: result_vty,
        };

        Ok(vec![TirStmt::SetField {
            object: obj_expr,
            field_name,
            field_index,
            value: binop_expr,
        }])
    }
}
