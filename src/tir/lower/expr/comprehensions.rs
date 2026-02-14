use anyhow::Result;
use pyo3::prelude::*;

use crate::tir::{
    builtin, ArithBinOp, CallTarget, RawBinOp, TirExpr, TirExprKind, TirStmt, Type, ValueType,
};
use crate::{ast_get_list, ast_get_string, ast_getattr, ast_type_name};

use crate::tir::lower::Lowering;

impl Lowering {
    pub(in crate::tir::lower) fn lower_list_comprehension(
        &mut self,
        node: &Bound<PyAny>,
        line: usize,
    ) -> Result<TirExpr> {
        let elt_node = ast_getattr!(node, "elt");
        let generators = ast_get_list!(node, "generators");
        self.lower_comp_impl(&elt_node, &generators, line)
    }

    /// Shared implementation for ListComp and GeneratorExp lowering.
    /// Returns a Var expression pointing to the result list.
    /// Emits setup+loop stmts into self.pre_stmts.
    pub(in crate::tir::lower) fn lower_comp_impl(
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

    /// Fused sum(GeneratorExp, start) lowering.
    /// Instead of materializing the generator as a list and then summing,
    /// this directly accumulates: `acc = start; for x in iter: acc = acc + elt`.
    pub(in crate::tir::lower) fn lower_sum_generator(
        &mut self,
        gen_node: &Bound<PyAny>,
        start_expr: TirExpr,
        line: usize,
    ) -> Result<TirExpr> {
        let elt_node = ast_getattr!(gen_node, "elt");
        let generators = ast_get_list!(gen_node, "generators");

        self.push_scope();

        // Phase 1: Parse generators (same as lower_comp_impl)
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

        // Phase 2: Lower the element expression
        let elt_expr = self.lower_expr(&elt_node)?;
        let elt_pre = std::mem::take(&mut self.pre_stmts);
        let elem_ty = elt_expr.ty.clone();

        self.pop_scope();

        // Phase 3: Build accumulation loop instead of list construction
        let acc_var = self.fresh_internal("sum_acc");
        self.declare(acc_var.clone(), elem_ty.to_type());

        // Build the add expression: acc = acc + elt
        let lhs = TirExpr {
            kind: TirExprKind::Var(acc_var.clone()),
            ty: elem_ty.clone(),
        };
        let add_expr = self.resolve_binop(line, RawBinOp::Arith(ArithBinOp::Add), lhs, elt_expr)?;

        // Innermost body: pre_stmts from elt + acc = acc + elt
        let mut body: Vec<TirStmt> = elt_pre;
        body.push(TirStmt::Let {
            name: acc_var.clone(),
            ty: elem_ty.clone(),
            value: add_expr,
        });

        // Build from inside out (same as lower_comp_impl)
        for gen_info in gen_infos.iter().rev() {
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
            body = self.build_comp_for_loop(&gen_info.var_name, &gen_info.kind, body);
        }

        // Emit: initialize accumulator + loop stmts
        let mut stmts = vec![TirStmt::Let {
            name: acc_var.clone(),
            ty: elem_ty.clone(),
            value: start_expr,
        }];
        stmts.extend(body);

        self.pre_stmts.extend(stmts);

        Ok(TirExpr {
            kind: TirExprKind::Var(acc_var),
            ty: elem_ty,
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
                let mut full_body = vec![TirStmt::Let {
                    name: var_name.to_string(),
                    ty: elem_ty.clone(),
                    value: TirExpr {
                        kind: TirExprKind::ExternalCall {
                            func: builtin::BuiltinFn::TupleGetItem,
                            args: vec![
                                TirExpr {
                                    kind: TirExprKind::Var(tuple_var.clone()),
                                    ty: tuple_expr.ty.clone(),
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
