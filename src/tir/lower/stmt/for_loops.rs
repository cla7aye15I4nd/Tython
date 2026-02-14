use anyhow::Result;
use pyo3::prelude::*;

use crate::ast::Type;
use crate::tir::{builtin, TirExpr, TirExprKind, TirStmt, ValueType};
use crate::{ast_get_list, ast_get_string, ast_getattr, ast_type_name};

use crate::tir::lower::Lowering;

impl Lowering {
    pub(in crate::tir::lower) fn handle_for(
        &mut self,
        node: &Bound<PyAny>,
        line: usize,
    ) -> Result<Vec<TirStmt>> {
        let target_node = ast_getattr!(node, "target");
        let target_type = ast_type_name!(target_node);
        if target_type == "Tuple" {
            let target_names = self.extract_for_target_tuple_names(line, &target_node)?;
            let iter_node = ast_getattr!(node, "iter");
            if ast_type_name!(iter_node) == "Call" {
                let func_node = ast_getattr!(iter_node, "func");
                if ast_type_name!(func_node) == "Name" {
                    let func_name = ast_get_string!(func_node, "id");
                    if func_name == "zip" {
                        return self.handle_for_zip_unpack(node, line, &target_names, &iter_node);
                    }
                    if func_name == "enumerate" {
                        return self.handle_for_enumerate_unpack(
                            node,
                            line,
                            &target_names,
                            &iter_node,
                        );
                    }
                }
            }
            return Err(self.syntax_error(
                line,
                "tuple-unpack for-loop target is only supported with zip(...) or enumerate(...)",
            ));
        }
        if target_type != "Name" {
            return Err(self.syntax_error(line, "for-loop target must be a variable or tuple"));
        }
        let loop_var = ast_get_string!(target_node, "id");

        let iter_node = ast_getattr!(node, "iter");

        // Check for range(...) first
        if ast_type_name!(iter_node) == "Call" {
            let func_node = ast_getattr!(iter_node, "func");
            if ast_type_name!(func_node) == "Name" && ast_get_string!(func_node, "id") == "range" {
                return self.handle_for_range(node, line, &loop_var, &iter_node);
            }
        }

        // General iterable: evaluate the iterator expression
        let iterable_expr = self.lower_expr(&iter_node)?;

        match iterable_expr.ty.clone() {
            ValueType::List(inner) => {
                self.handle_for_list(node, line, &loop_var, iterable_expr, (*inner).clone())
            }
            ValueType::Str => self.handle_for_str(node, line, &loop_var, iterable_expr),
            ValueType::Bytes => self.handle_for_bytes(node, line, &loop_var, iterable_expr),
            ValueType::ByteArray => self.handle_for_bytearray(node, line, &loop_var, iterable_expr),
            ValueType::Class(ref name) if self.is_tuple_class(name) => {
                let elements = self.tuple_element_types(name).to_vec();
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
                let tuple_len = elements.len();
                self.handle_for_tuple(node, line, &loop_var, iterable_expr, elem_ty, tuple_len)
            }
            ValueType::Class(class_name) => {
                self.handle_for_class_iter(node, line, &loop_var, iterable_expr, &class_name)
            }
            other => Err(self.type_error(line, format!("cannot iterate over `{}`", other))),
        }
    }

    fn extract_for_target_tuple_names(
        &self,
        line: usize,
        target_node: &Bound<PyAny>,
    ) -> Result<Vec<String>> {
        let elts = ast_get_list!(target_node, "elts");
        if elts.is_empty() {
            return Err(self.syntax_error(line, "empty tuple target in for-loop is invalid"));
        }
        let mut names = Vec::with_capacity(elts.len());
        for elt in elts.iter() {
            if ast_type_name!(elt) != "Name" {
                return Err(self.syntax_error(
                    line,
                    "for-loop tuple target must contain only simple variable names",
                ));
            }
            names.push(ast_get_string!(elt, "id"));
        }
        Ok(names)
    }

    fn make_list_get_expr(
        &self,
        list_var: &str,
        list_ty: ValueType,
        idx_var: &str,
        elem_ty: ValueType,
    ) -> TirExpr {
        TirExpr {
            kind: TirExprKind::ExternalCall {
                func: builtin::BuiltinFn::ListGet,
                args: vec![
                    TirExpr {
                        kind: TirExprKind::Var(list_var.to_string()),
                        ty: list_ty,
                    },
                    TirExpr {
                        kind: TirExprKind::Var(idx_var.to_string()),
                        ty: ValueType::Int,
                    },
                ],
            },
            ty: elem_ty,
        }
    }

    fn handle_for_zip_unpack(
        &mut self,
        node: &Bound<PyAny>,
        line: usize,
        target_names: &[String],
        iter_node: &Bound<PyAny>,
    ) -> Result<Vec<TirStmt>> {
        if target_names.len() != 2 {
            return Err(self.syntax_error(
                line,
                "zip(...) unpack currently requires exactly two target variables",
            ));
        }
        let args = ast_get_list!(iter_node, "args");
        if args.len() != 2 {
            return Err(self.type_error(
                line,
                format!("zip() expects 2 arguments, got {}", args.len()),
            ));
        }

        let left = self.lower_expr(&args.get_item(0)?)?;
        let right = self.lower_expr(&args.get_item(1)?)?;
        let left_ty = left.ty.clone();
        let right_ty = right.ty.clone();
        let (left_elem, right_elem) = match (&left.ty, &right.ty) {
            (ValueType::List(a), ValueType::List(b)) => ((**a).clone(), (**b).clone()),
            _ => {
                return Err(self.type_error(
                    line,
                    "zip() in for-loop unpack currently requires list arguments",
                ))
            }
        };

        let left_var = self.fresh_internal("zip_left");
        let right_var = self.fresh_internal("zip_right");
        let len_left_var = self.fresh_internal("zip_len_left");
        let len_right_var = self.fresh_internal("zip_len_right");
        let start_var = self.fresh_internal("zip_start");
        let stop_var = self.fresh_internal("zip_stop");
        let step_var = self.fresh_internal("zip_step");
        let idx_var = self.fresh_internal("zip_idx");

        self.declare(left_var.clone(), left.ty.to_type());
        self.declare(right_var.clone(), right.ty.to_type());
        self.declare(len_left_var.clone(), Type::Int);
        self.declare(len_right_var.clone(), Type::Int);
        self.declare(start_var.clone(), Type::Int);
        self.declare(stop_var.clone(), Type::Int);
        self.declare(step_var.clone(), Type::Int);
        self.declare(idx_var.clone(), Type::Int);
        self.declare(target_names[0].clone(), left_elem.to_type());
        self.declare(target_names[1].clone(), right_elem.to_type());

        let mut body = vec![
            TirStmt::Let {
                name: target_names[0].clone(),
                ty: left_elem.clone(),
                value: self.make_list_get_expr(&left_var, left.ty.clone(), &idx_var, left_elem),
            },
            TirStmt::Let {
                name: target_names[1].clone(),
                ty: right_elem.clone(),
                value: self.make_list_get_expr(&right_var, right.ty.clone(), &idx_var, right_elem),
            },
        ];
        body.extend(self.lower_block_in_current_scope(&ast_get_list!(node, "body"))?);
        let else_body = self.lower_block_in_current_scope(&ast_get_list!(node, "orelse"))?;

        Ok(vec![
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
                body,
                else_body,
            },
        ])
    }

    fn handle_for_enumerate_unpack(
        &mut self,
        node: &Bound<PyAny>,
        line: usize,
        target_names: &[String],
        iter_node: &Bound<PyAny>,
    ) -> Result<Vec<TirStmt>> {
        if target_names.len() != 2 {
            return Err(self.syntax_error(
                line,
                "enumerate(...) unpack currently requires exactly two target variables",
            ));
        }
        let args = ast_get_list!(iter_node, "args");
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
                    "enumerate() in for-loop unpack requires a list argument",
                ))
            }
        };

        let list_var = self.fresh_internal("enum_list");
        let len_var = self.fresh_internal("enum_len");
        let start_var = self.fresh_internal("enum_start");
        let stop_var = self.fresh_internal("enum_stop");
        let step_var = self.fresh_internal("enum_step");
        let idx_var = self.fresh_internal("enum_idx");

        self.declare(list_var.clone(), list_expr.ty.to_type());
        self.declare(len_var.clone(), Type::Int);
        self.declare(start_var.clone(), Type::Int);
        self.declare(stop_var.clone(), Type::Int);
        self.declare(step_var.clone(), Type::Int);
        self.declare(idx_var.clone(), Type::Int);
        self.declare(target_names[0].clone(), Type::Int);
        self.declare(target_names[1].clone(), elem_ty.to_type());

        let mut body = vec![
            TirStmt::Let {
                name: target_names[0].clone(),
                ty: ValueType::Int,
                value: TirExpr {
                    kind: TirExprKind::Var(idx_var.clone()),
                    ty: ValueType::Int,
                },
            },
            TirStmt::Let {
                name: target_names[1].clone(),
                ty: elem_ty.clone(),
                value: self.make_list_get_expr(&list_var, list_expr.ty.clone(), &idx_var, elem_ty),
            },
        ];
        body.extend(self.lower_block_in_current_scope(&ast_get_list!(node, "body"))?);
        let else_body = self.lower_block_in_current_scope(&ast_get_list!(node, "orelse"))?;

        Ok(vec![
            TirStmt::Let {
                name: list_var.clone(),
                ty: list_expr.ty.clone(),
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
                            ty: list_expr.ty.clone(),
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
                body,
                else_body,
            },
        ])
    }

    fn handle_for_range(
        &mut self,
        node: &Bound<PyAny>,
        line: usize,
        loop_var: &str,
        iter_node: &Bound<PyAny>,
    ) -> Result<Vec<TirStmt>> {
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

        let (start_expr, stop_expr, step_expr) = if args.len() == 1 {
            (
                TirExpr {
                    kind: TirExprKind::IntLiteral(0),
                    ty: ValueType::Int,
                },
                args[0].clone(),
                TirExpr {
                    kind: TirExprKind::IntLiteral(1),
                    ty: ValueType::Int,
                },
            )
        } else if args.len() == 2 {
            (
                args[0].clone(),
                args[1].clone(),
                TirExpr {
                    kind: TirExprKind::IntLiteral(1),
                    ty: ValueType::Int,
                },
            )
        } else if args.len() == 3 {
            (args[0].clone(), args[1].clone(), args[2].clone())
        } else {
            return Err(self.type_error(
                line,
                format!("range() expects 1 to 3 arguments, got {}", args.len()),
            ));
        };

        let start_name = self.fresh_internal("range_start");
        let stop_name = self.fresh_internal("range_stop");
        let step_name = self.fresh_internal("range_step");

        self.declare(start_name.clone(), Type::Int);
        self.declare(stop_name.clone(), Type::Int);
        self.declare(step_name.clone(), Type::Int);
        self.declare(loop_var.to_string(), Type::Int);

        let body = self.lower_block_in_current_scope(&ast_get_list!(node, "body"))?;
        let else_body = self.lower_block_in_current_scope(&ast_get_list!(node, "orelse"))?;

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
                loop_var: loop_var.to_string(),
                start_var: start_name,
                stop_var: stop_name,
                step_var: step_name,
                body,
                else_body,
            },
        ])
    }

    fn handle_for_list(
        &mut self,
        node: &Bound<PyAny>,
        _line: usize,
        loop_var: &str,
        list_expr: TirExpr,
        elem_ty: ValueType,
    ) -> Result<Vec<TirStmt>> {
        let list_var = self.fresh_internal("for_list");
        let idx_var = self.fresh_internal("for_idx");
        let len_var = self.fresh_internal("for_len");

        self.declare(list_var.clone(), list_expr.ty.to_type());
        self.declare(idx_var.clone(), Type::Int);
        self.declare(len_var.clone(), Type::Int);
        self.declare(loop_var.to_string(), elem_ty.to_type());

        let body = self.lower_block_in_current_scope(&ast_get_list!(node, "body"))?;
        let else_body = self.lower_block_in_current_scope(&ast_get_list!(node, "orelse"))?;

        Ok(vec![
            TirStmt::Let {
                name: list_var.clone(),
                ty: list_expr.ty.clone(),
                value: list_expr,
            },
            TirStmt::ForList {
                loop_var: loop_var.to_string(),
                loop_var_ty: elem_ty,
                list_var,
                index_var: idx_var,
                len_var,
                body,
                else_body,
            },
        ])
    }

    fn handle_for_class_iter(
        &mut self,
        node: &Bound<PyAny>,
        line: usize,
        loop_var: &str,
        obj_expr: TirExpr,
        class_name: &str,
    ) -> Result<Vec<TirStmt>> {
        let class_info = self.lookup_class(line, class_name)?;

        // Check __iter__
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

        // Check the iterator class has __next__
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

        let elem_ty = self.value_type_from_type(&next_method.return_type);
        let next_mangled = next_method.mangled_name.clone();

        // Create temp vars
        let obj_var = self.fresh_internal("for_obj");
        let iter_var = self.fresh_internal("for_iter");

        self.declare(obj_var.clone(), obj_expr.ty.to_type());
        self.declare(iter_var.clone(), Type::Class(iter_class_name.clone()));
        self.declare(loop_var.to_string(), next_method.return_type.clone());

        let body = self.lower_block_in_current_scope(&ast_get_list!(node, "body"))?;
        let else_body = self.lower_block_in_current_scope(&ast_get_list!(node, "orelse"))?;

        Ok(vec![
            TirStmt::Let {
                name: obj_var.clone(),
                ty: obj_expr.ty.clone(),
                value: obj_expr.clone(),
            },
            TirStmt::Let {
                name: iter_var.clone(),
                ty: ValueType::Class(iter_class_name.clone()),
                value: TirExpr {
                    kind: TirExprKind::Call {
                        func: iter_mangled,
                        args: vec![TirExpr {
                            kind: TirExprKind::Var(obj_var),
                            ty: obj_expr.ty,
                        }],
                    },
                    ty: ValueType::Class(iter_class_name.clone()),
                },
            },
            TirStmt::ForIter {
                loop_var: loop_var.to_string(),
                loop_var_ty: elem_ty,
                iterator_var: iter_var,
                iterator_class: iter_class_name,
                next_mangled,
                body,
                else_body,
            },
        ])
    }

    fn handle_for_tuple(
        &mut self,
        node: &Bound<PyAny>,
        _line: usize,
        loop_var: &str,
        tuple_expr: TirExpr,
        elem_ty: ValueType,
        tuple_len: usize,
    ) -> Result<Vec<TirStmt>> {
        // Lower as: for __idx in range(0, len):  loop_var = tuple[__idx]
        let tuple_var = self.fresh_internal("for_tuple");
        let start_name = self.fresh_internal("range_start");
        let stop_name = self.fresh_internal("range_stop");
        let step_name = self.fresh_internal("range_step");
        let idx_var = self.fresh_internal("for_tuple_idx");

        self.declare(tuple_var.clone(), tuple_expr.ty.to_type());
        self.declare(start_name.clone(), Type::Int);
        self.declare(stop_name.clone(), Type::Int);
        self.declare(step_name.clone(), Type::Int);
        self.declare(idx_var.clone(), Type::Int);
        self.declare(loop_var.to_string(), elem_ty.to_type());

        let body = self.lower_block_in_current_scope(&ast_get_list!(node, "body"))?;
        let else_body = self.lower_block_in_current_scope(&ast_get_list!(node, "orelse"))?;

        // Prepend: loop_var = tuple[idx_var] via if-else chain of GetField
        let tuple_ty = tuple_expr.ty.clone();
        let mut full_body = self.gen_tuple_dynamic_getitem_stmts(
            loop_var, &elem_ty, &tuple_var, &tuple_ty, &idx_var, tuple_len,
        );
        full_body.extend(body);

        Ok(vec![
            TirStmt::Let {
                name: tuple_var,
                ty: tuple_expr.ty.clone(),
                value: tuple_expr,
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
                    kind: TirExprKind::IntLiteral(tuple_len as i64),
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
                else_body,
            },
        ])
    }

    fn handle_for_str(
        &mut self,
        node: &Bound<PyAny>,
        _line: usize,
        loop_var: &str,
        str_expr: TirExpr,
    ) -> Result<Vec<TirStmt>> {
        let str_var = self.fresh_internal("for_str");
        let idx_var = self.fresh_internal("for_str_idx");
        let len_var = self.fresh_internal("for_str_len");

        self.declare(str_var.clone(), str_expr.ty.to_type());
        self.declare(idx_var.clone(), Type::Int);
        self.declare(len_var.clone(), Type::Int);
        self.declare(loop_var.to_string(), Type::Str);

        let body = self.lower_block_in_current_scope(&ast_get_list!(node, "body"))?;
        let else_body = self.lower_block_in_current_scope(&ast_get_list!(node, "orelse"))?;

        Ok(vec![
            TirStmt::Let {
                name: str_var.clone(),
                ty: str_expr.ty.clone(),
                value: str_expr,
            },
            TirStmt::ForStr {
                loop_var: loop_var.to_string(),
                str_var,
                index_var: idx_var,
                len_var,
                body,
                else_body,
            },
        ])
    }

    fn handle_for_bytes(
        &mut self,
        node: &Bound<PyAny>,
        _line: usize,
        loop_var: &str,
        bytes_expr: TirExpr,
    ) -> Result<Vec<TirStmt>> {
        let bytes_var = self.fresh_internal("for_bytes");
        let idx_var = self.fresh_internal("for_bytes_idx");
        let len_var = self.fresh_internal("for_bytes_len");

        self.declare(bytes_var.clone(), bytes_expr.ty.to_type());
        self.declare(idx_var.clone(), Type::Int);
        self.declare(len_var.clone(), Type::Int);
        self.declare(loop_var.to_string(), Type::Int);

        let body = self.lower_block_in_current_scope(&ast_get_list!(node, "body"))?;
        let else_body = self.lower_block_in_current_scope(&ast_get_list!(node, "orelse"))?;

        Ok(vec![
            TirStmt::Let {
                name: bytes_var.clone(),
                ty: bytes_expr.ty.clone(),
                value: bytes_expr,
            },
            TirStmt::ForBytes {
                loop_var: loop_var.to_string(),
                bytes_var,
                index_var: idx_var,
                len_var,
                body,
                else_body,
            },
        ])
    }

    fn handle_for_bytearray(
        &mut self,
        node: &Bound<PyAny>,
        _line: usize,
        loop_var: &str,
        bytearray_expr: TirExpr,
    ) -> Result<Vec<TirStmt>> {
        let bytearray_var = self.fresh_internal("for_bytearray");
        let idx_var = self.fresh_internal("for_bytearray_idx");
        let len_var = self.fresh_internal("for_bytearray_len");

        self.declare(bytearray_var.clone(), bytearray_expr.ty.to_type());
        self.declare(idx_var.clone(), Type::Int);
        self.declare(len_var.clone(), Type::Int);
        self.declare(loop_var.to_string(), Type::Int);

        let body = self.lower_block_in_current_scope(&ast_get_list!(node, "body"))?;
        let else_body = self.lower_block_in_current_scope(&ast_get_list!(node, "orelse"))?;

        Ok(vec![
            TirStmt::Let {
                name: bytearray_var.clone(),
                ty: bytearray_expr.ty.clone(),
                value: bytearray_expr,
            },
            TirStmt::ForByteArray {
                loop_var: loop_var.to_string(),
                bytearray_var,
                index_var: idx_var,
                len_var,
                body,
                else_body,
            },
        ])
    }
}
