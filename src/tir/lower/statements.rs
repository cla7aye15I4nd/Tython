use anyhow::Result;
use pyo3::prelude::*;
use std::collections::HashMap;

use crate::ast::{ClassInfo, Type};
use crate::tir::{
    builtin, ArithBinOp, CallResult, CallTarget, CastKind, RawBinOp, TirExpr, TirExprKind, TirStmt,
    ValueType,
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
            "For" => self.handle_for(node, line),
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

        // Handle empty list literal `[]` with type annotation
        if ast_type_name!(value_node) == "List" {
            let elts = ast_get_list!(value_node, "elts");
            if elts.is_empty() {
                let list_ty = annotated_ty.ok_or_else(|| {
                    self.syntax_error(line, "empty list literal `[]` requires a type annotation")
                })?;
                let inner_ty = match &list_ty {
                    Type::List(inner) => Self::to_value_type(inner),
                    _ => {
                        return Err(self.type_error(
                            line,
                            format!("type mismatch: expected `list[...]`, got `{}`", list_ty),
                        ))
                    }
                };
                let vty = ValueType::List(Box::new(inner_ty.clone()));
                self.declare(target.clone(), list_ty);
                return Ok(vec![TirStmt::Let {
                    name: target,
                    ty: vty.clone(),
                    value: TirExpr {
                        kind: TirExprKind::ListLiteral {
                            element_type: inner_ty,
                            elements: vec![],
                        },
                        ty: vty,
                    },
                }]);
            }
        }

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
            "Subscript" => self.lower_subscript_assign(&target_node, node, line),
            _ => Err(self.syntax_error(
                line,
                "only variable, attribute, or subscript assignments are supported",
            )),
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

                if op == RawBinOp::Arith(ArithBinOp::Div) && target_ty == Type::Int {
                    return Err(self.type_error(
                        line,
                        format!("`/=` on `int` variable `{}` would change type to `float`; use `//=` for integer division", target),
                    ));
                }

                let target_ref = TirExpr {
                    kind: TirExprKind::Var(target.clone()),
                    ty: Self::to_value_type(&target_ty),
                };

                let binop_expr = self.resolve_binop(line, op, target_ref, value_expr)?;
                self.declare(target.clone(), binop_expr.ty.to_type());

                Ok(vec![TirStmt::Let {
                    name: target,
                    ty: binop_expr.ty.clone(),
                    value: binop_expr,
                }])
            }
            "Attribute" => self.lower_attribute_aug_assign(&target_node, node, line),
            "Subscript" => self.lower_subscript_aug_assign(&target_node, node, line),
            _ => Err(self.syntax_error(
                line,
                "only variable, attribute, or subscript augmented assignments are supported",
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
                CallResult::VoidStmt(stmt) => Ok(vec![*stmt]),
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
                ValueType::Str
                | ValueType::Bytes
                | ValueType::ByteArray
                | ValueType::List(_)
                | ValueType::Tuple(_) => {
                    let len_fn = match &condition.ty {
                        ValueType::Str => builtin::BuiltinFn::StrLen,
                        ValueType::Bytes => builtin::BuiltinFn::BytesLen,
                        ValueType::ByteArray => builtin::BuiltinFn::ByteArrayLen,
                        ValueType::List(_) => builtin::BuiltinFn::ListLen,
                        ValueType::Tuple(elements) => {
                            let len_expr = TirExpr {
                                kind: TirExprKind::IntLiteral(elements.len() as i64),
                                ty: ValueType::Int,
                            };
                            return Ok(vec![TirStmt::VoidCall {
                                target: CallTarget::Builtin(builtin::BuiltinFn::Assert),
                                args: vec![TirExpr {
                                    kind: TirExprKind::Cast {
                                        kind: CastKind::IntToBool,
                                        arg: Box::new(len_expr),
                                    },
                                    ty: ValueType::Bool,
                                }],
                            }]);
                        }
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

    fn handle_for(&mut self, node: &Bound<PyAny>, line: usize) -> Result<Vec<TirStmt>> {
        let target_node = ast_getattr!(node, "target");
        if ast_type_name!(target_node) != "Name" {
            return Err(self.syntax_error(line, "for-loop target must be a simple variable name"));
        }
        let loop_var = ast_get_string!(target_node, "id");

        let iter_node = ast_getattr!(node, "iter");
        if ast_type_name!(iter_node) != "Call" {
            return Err(self.syntax_error(line, "only `for ... in range(...)` is supported"));
        }

        let func_node = ast_getattr!(iter_node, "func");
        if ast_type_name!(func_node) != "Name" || ast_get_string!(func_node, "id") != "range" {
            return Err(self.syntax_error(line, "only `for ... in range(...)` is supported"));
        }

        let args_list = ast_get_list!(iter_node, "args");
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

        let (start_expr, stop_expr, step_expr) = match args.len() {
            1 => (
                TirExpr {
                    kind: TirExprKind::IntLiteral(0),
                    ty: ValueType::Int,
                },
                args[0].clone(),
                TirExpr {
                    kind: TirExprKind::IntLiteral(1),
                    ty: ValueType::Int,
                },
            ),
            2 => (
                args[0].clone(),
                args[1].clone(),
                TirExpr {
                    kind: TirExprKind::IntLiteral(1),
                    ty: ValueType::Int,
                },
            ),
            3 => (args[0].clone(), args[1].clone(), args[2].clone()),
            _ => unreachable!(),
        };

        let start_name = self.fresh_internal("range_start");
        let stop_name = self.fresh_internal("range_stop");
        let step_name = self.fresh_internal("range_step");

        self.declare(start_name.clone(), Type::Int);
        self.declare(stop_name.clone(), Type::Int);
        self.declare(step_name.clone(), Type::Int);
        self.declare(loop_var.clone(), Type::Int);

        let body = self.lower_block(&ast_get_list!(node, "body"))?;
        let orelse = ast_get_list!(node, "orelse");
        if !orelse.is_empty() {
            return Err(self.syntax_error(line, "for-else is not supported"));
        }

        Ok(vec![
            TirStmt::Let {
                name: start_name.clone(),
                ty: ValueType::Int,
                value: start_expr,
            },
            TirStmt::Let {
                name: stop_name.clone(),
                ty: ValueType::Int,
                value: stop_expr,
            },
            TirStmt::Let {
                name: step_name.clone(),
                ty: ValueType::Int,
                value: step_expr,
            },
            TirStmt::ForRange {
                loop_var,
                start_var: start_name,
                stop_var: stop_name,
                step_var: step_name,
                body,
            },
        ])
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
            class_name,
            field_index,
            value: tir_value,
        }])
    }

    // ── subscript assignment ──────────────────────────────────────────

    fn lower_subscript_assign(
        &mut self,
        target_node: &Bound<PyAny>,
        assign_node: &Bound<PyAny>,
        line: usize,
    ) -> Result<Vec<TirStmt>> {
        let value_node_target = ast_getattr!(target_node, "value");
        let slice_node = ast_getattr!(target_node, "slice");
        let value_node = ast_getattr!(assign_node, "value");

        let list_expr = self.lower_expr(&value_node_target)?;
        match &list_expr.ty {
            ValueType::List(inner) => {
                let index_expr = self.lower_expr(&slice_node)?;
                if index_expr.ty != ValueType::Int {
                    return Err(self.type_error(
                        line,
                        format!("list index must be `int`, got `{}`", index_expr.ty),
                    ));
                }
                let tir_value = self.lower_expr(&value_node)?;
                let expected_elem_ty = inner.as_ref();
                if &tir_value.ty != expected_elem_ty {
                    return Err(self.type_error(
                        line,
                        format!(
                            "cannot assign `{}` to element of `list[{}]`",
                            tir_value.ty, expected_elem_ty
                        ),
                    ));
                }
                Ok(vec![TirStmt::ListSet {
                    list: list_expr,
                    index: index_expr,
                    value: tir_value,
                }])
            }
            ValueType::Tuple(_) => {
                Err(self.type_error(line, "tuple does not support index assignment".to_string()))
            }
            other => Err(self.type_error(
                line,
                format!("type `{}` does not support index assignment", other),
            )),
        }
    }

    fn lower_subscript_aug_assign(
        &mut self,
        target_node: &Bound<PyAny>,
        aug_node: &Bound<PyAny>,
        line: usize,
    ) -> Result<Vec<TirStmt>> {
        let value_node = ast_getattr!(target_node, "value");
        let slice_node = ast_getattr!(target_node, "slice");

        let list_expr = self.lower_expr(&value_node)?;
        match &list_expr.ty {
            ValueType::List(inner) => {
                let index_expr = self.lower_expr(&slice_node)?;
                if index_expr.ty != ValueType::Int {
                    return Err(self.type_error(
                        line,
                        format!("list index must be `int`, got `{}`", index_expr.ty),
                    ));
                }
                let elem_ty = inner.as_ref().clone();

                let current_val = TirExpr {
                    kind: TirExprKind::ExternalCall {
                        func: builtin::BuiltinFn::ListGet,
                        args: vec![list_expr.clone(), index_expr.clone()],
                    },
                    ty: elem_ty,
                };

                let op = Self::convert_binop(&ast_getattr!(aug_node, "op"))?;
                let rhs = self.lower_expr(&ast_getattr!(aug_node, "value"))?;
                let binop_expr = self.resolve_binop(line, op, current_val, rhs)?;

                if &binop_expr.ty != inner.as_ref() {
                    return Err(self.type_error(
                        line,
                        format!(
                            "augmented assignment would change list element type from `{}` to `{}`",
                            inner, binop_expr.ty
                        ),
                    ));
                }

                Ok(vec![TirStmt::ListSet {
                    list: list_expr,
                    index: index_expr,
                    value: binop_expr,
                }])
            }
            ValueType::Tuple(_) => {
                Err(self.type_error(line, "tuple does not support index assignment".to_string()))
            }
            other => Err(self.type_error(
                line,
                format!("type `{}` does not support index assignment", other),
            )),
        }
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
                class_name: class_name.clone(),
                field_index,
            },
            ty: field_vty,
        };

        let op = Self::convert_binop(&ast_getattr!(aug_node, "op"))?;
        let rhs = self.lower_expr(&ast_getattr!(aug_node, "value"))?;

        let binop_expr = self.resolve_binop(line, op, current_val, rhs)?;

        if binop_expr.ty.to_type() != field.ty {
            return Err(self.type_error(
                line,
                format!(
                    "augmented assignment would change field `{}.{}` type from `{}` to `{}`",
                    class_name, field_name, field.ty, binop_expr.ty
                ),
            ));
        }

        Ok(vec![TirStmt::SetField {
            object: obj_expr,
            class_name,
            field_index,
            value: binop_expr,
        }])
    }
}
