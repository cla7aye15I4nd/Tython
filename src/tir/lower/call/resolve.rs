use anyhow::Result;
use pyo3::prelude::*;
use std::path::Path;

use pyo3::types::PyList;

use crate::ast::Type;
use crate::tir::{CallResult, IntrinsicOp, TirExpr, TirExprKind, ValueType};
use crate::{ast_get_list, ast_get_string, ast_getattr, ast_type_name};

use super::builtin_call::is_builtin_call;

use super::super::Lowering;
use super::{NormalizedCallArgs, ResolvedCall, ResolvedCallee};

impl Lowering {
    pub(in crate::tir::lower) fn lower_call(
        &mut self,
        node: &Bound<PyAny>,
        line: usize,
    ) -> Result<CallResult> {
        let func_node = ast_getattr!(node, "func");
        let args_list = ast_get_list!(node, "args");
        let keywords_list = ast_get_list!(node, "keywords");

        if let Some(fused) =
            self.detect_sum_generator_fast_path(&func_node, &args_list, &keywords_list, line)?
        {
            return Ok(fused);
        }

        let args = self.normalize_call_args(&args_list, &keywords_list, line)?;
        let resolved = self.resolve_call_target(&func_node, line, args)?;
        self.emit_resolved_call(line, resolved)
    }

    fn resolve_call_target(
        &mut self,
        func_node: &Bound<PyAny>,
        line: usize,
        args: NormalizedCallArgs,
    ) -> Result<ResolvedCall> {
        match ast_type_name!(func_node).as_str() {
            "Name" => self.resolve_name_call_target(func_node, line, args),
            "Attribute" => self.resolve_attribute_call_target(func_node, line, args),
            _ => Err(self.syntax_error(
                line,
                "only direct function calls and module.function calls are supported",
            )),
        }
    }

    fn resolve_name_call_target(
        &self,
        func_node: &Bound<PyAny>,
        line: usize,
        args: NormalizedCallArgs,
    ) -> Result<ResolvedCall> {
        let func_name = ast_get_string!(func_node, "id");

        if func_name == "print" || func_name == "open" || is_builtin_call(&func_name) {
            return Ok(ResolvedCall {
                callee: ResolvedCallee::GlobalName(func_name),
                args,
            });
        }

        let scope_type = self
            .lookup(&func_name)
            .cloned()
            .ok_or_else(|| self.name_error(line, format!("undefined function `{}`", func_name)))?;

        match scope_type {
            Type::Function { .. } => {
                let mangled = self
                    .function_mangled_names
                    .get(&func_name)
                    .cloned()
                    .ok_or_else(|| {
                        self.type_error(
                            line,
                            format!(
                                "`{}` is not callable (indirect calls through function pointers are not supported)",
                                func_name
                            ),
                        )
                    })?;

                Ok(ResolvedCall {
                    callee: ResolvedCallee::DirectFunction {
                        display_name: func_name,
                        mangled,
                        func_type: scope_type,
                    },
                    args,
                })
            }
            Type::Module(mangled) => {
                if let Some(class_info) = self.class_registry.get(&mangled).cloned() {
                    return Ok(ResolvedCall {
                        callee: ResolvedCallee::Constructor {
                            qualified_name: mangled,
                            class_info,
                        },
                        args,
                    });
                }

                let func_type = self
                    .symbol_table
                    .get(&mangled)
                    .ok_or_else(|| {
                        self.name_error(
                            line,
                            format!("imported symbol `{}` not found in symbol table", func_name),
                        )
                    })?
                    .clone();

                Ok(ResolvedCall {
                    callee: ResolvedCallee::DirectFunction {
                        display_name: func_name,
                        mangled,
                        func_type,
                    },
                    args,
                })
            }
            Type::Class(name) => {
                let class_info = self
                    .class_registry
                    .get(&name)
                    .ok_or_else(|| self.name_error(line, format!("unknown class `{}`", name)))?
                    .clone();
                Ok(ResolvedCall {
                    callee: ResolvedCallee::Constructor {
                        qualified_name: name,
                        class_info,
                    },
                    args,
                })
            }
            _ => Err(self.type_error(line, format!("`{}` is not callable", func_name))),
        }
    }

    fn resolve_attribute_call_target(
        &mut self,
        func_node: &Bound<PyAny>,
        line: usize,
        args: NormalizedCallArgs,
    ) -> Result<ResolvedCall> {
        let value_node = ast_getattr!(func_node, "value");
        let attr = ast_get_string!(func_node, "attr");

        if ast_type_name!(value_node) == "Name" {
            let name = ast_get_string!(value_node, "id");
            if let Some(Type::Module(mod_path)) = self.lookup(&name).cloned() {
                if mod_path == "math" || mod_path == "random" {
                    return Ok(ResolvedCall {
                        callee: ResolvedCallee::NativeModuleFunction {
                            module: mod_path,
                            attr,
                        },
                        args,
                    });
                }

                let resolved = format!("{}${}", mod_path, attr);
                if let Some(class_info) = self.class_registry.get(&resolved).cloned() {
                    return Ok(ResolvedCall {
                        callee: ResolvedCallee::Constructor {
                            qualified_name: resolved,
                            class_info,
                        },
                        args,
                    });
                }

                let func_type = self
                    .symbol_table
                    .get(&resolved)
                    .ok_or_else(|| self.name_error(line, format!("undefined function `{}`", attr)))?
                    .clone();

                return Ok(ResolvedCall {
                    callee: ResolvedCallee::DirectFunction {
                        display_name: attr,
                        mangled: resolved,
                        func_type,
                    },
                    args,
                });
            }
        }

        if let Some(qualified) = self.try_resolve_class_path(func_node) {
            if let Some(class_info) = self.class_registry.get(&qualified).cloned() {
                return Ok(ResolvedCall {
                    callee: ResolvedCallee::Constructor {
                        qualified_name: qualified,
                        class_info,
                    },
                    args,
                });
            }
        }

        let obj_expr = self.lower_expr(&value_node)?;
        if let ValueType::Class(class_name) = obj_expr.ty.clone() {
            return Ok(ResolvedCall {
                callee: ResolvedCallee::ClassMethod {
                    object: obj_expr,
                    class_name,
                    method_name: attr,
                },
                args,
            });
        }

        Ok(ResolvedCall {
            callee: ResolvedCallee::BuiltinMethod {
                object: obj_expr,
                method_name: attr,
            },
            args,
        })
    }

    fn emit_resolved_call(&mut self, line: usize, resolved: ResolvedCall) -> Result<CallResult> {
        match resolved.callee {
            ResolvedCallee::GlobalName(name) => {
                self.emit_global_name_call(line, &name, resolved.args)
            }
            ResolvedCallee::DirectFunction {
                display_name,
                mangled,
                func_type,
            } => self.lower_callable_symbol_call(
                line,
                &display_name,
                &mangled,
                &func_type,
                resolved.args,
            ),
            ResolvedCallee::Constructor {
                qualified_name,
                class_info,
            } => {
                if !resolved.args.keyword.is_empty() {
                    return Err(
                        self.syntax_error(line, "constructor keyword arguments are not supported")
                    );
                }
                self.lower_constructor_call(
                    line,
                    &qualified_name,
                    &class_info,
                    resolved.args.positional,
                )
            }
            ResolvedCallee::NativeModuleFunction { module, attr } => self.lower_native_module_call(
                line,
                &module,
                &attr,
                resolved.args.positional,
                resolved.args.keyword,
            ),
            ResolvedCallee::ClassMethod {
                object,
                class_name,
                method_name,
            } => self.lower_class_instance_method_call(
                line,
                object,
                &class_name,
                &method_name,
                resolved.args,
            ),
            ResolvedCallee::BuiltinMethod {
                object,
                method_name,
            } => {
                if !resolved.args.keyword.is_empty() {
                    return Err(
                        self.syntax_error(line, "method keyword arguments are not supported")
                    );
                }
                self.lower_method_call(line, object, &method_name, resolved.args.positional)
            }
        }
    }

    fn emit_global_name_call(
        &mut self,
        line: usize,
        name: &str,
        mut args: NormalizedCallArgs,
    ) -> Result<CallResult> {
        if name == "print" {
            return Err(self.syntax_error(line, "print() can only be used as a statement"));
        }

        if name == "open" {
            if !args.keyword.is_empty() {
                return Err(self.syntax_error(line, "open() does not accept keywords"));
            }
            if args.positional.len() != 1 {
                return Err(self.type_error(
                    line,
                    format!("open() expects 1 argument, got {}", args.positional.len()),
                ));
            }
            let mut path_arg = args.positional.remove(0);
            if path_arg.ty != ValueType::Str {
                return Err(self.type_error(
                    line,
                    format!("open() path must be `str`, got `{}`", path_arg.ty),
                ));
            }

            if let TirExprKind::StrLiteral(path) = &path_arg.kind {
                let p = Path::new(path);
                if p.is_relative() {
                    let base = Path::new(&self.current_file)
                        .parent()
                        .expect("source file should have a parent directory");
                    let abs = base.join(p);
                    path_arg = TirExpr {
                        kind: TirExprKind::StrLiteral(abs.to_string_lossy().into_owned()),
                        ty: ValueType::Str,
                    };
                }
            }

            return Ok(CallResult::Expr(TirExpr {
                kind: TirExprKind::ExternalCall {
                    func: crate::tir::builtin::BuiltinFn::OpenReadAll,
                    args: vec![path_arg],
                },
                ty: ValueType::Str,
            }));
        }

        if !is_builtin_call(name) {
            return Err(self.name_error(line, format!("undefined function `{}`", name)));
        }

        if !args.keyword.is_empty() {
            return Err(self.syntax_error(
                line,
                format!("builtin `{}` does not support keyword arguments", name),
            ));
        }

        let mut positional_args = args.positional;

        if name == "sorted" {
            if positional_args.len() != 1 {
                return Err(self.type_error(
                    line,
                    format!(
                        "sorted() expects exactly 1 argument, got {}",
                        positional_args.len()
                    ),
                ));
            }
            let list_arg = positional_args.remove(0);
            let ValueType::List(inner) = &list_arg.ty else {
                return Err(self.type_error(
                    line,
                    format!(
                        "sorted() requires a list whose elements support ordering (`__lt__`), got `{}`",
                        list_arg.ty
                    ),
                ));
            };
            self.require_list_leaf_lt_support(line, inner)?;
            let sorted_ty = ValueType::List(inner.clone());
            let lt_tag = self.register_intrinsic_instance(IntrinsicOp::Lt, inner);
            return Ok(CallResult::Expr(TirExpr {
                kind: TirExprKind::ExternalCall {
                    func: crate::tir::builtin::BuiltinFn::SortedByTag,
                    args: vec![
                        list_arg,
                        TirExpr {
                            kind: TirExprKind::IntLiteral(lt_tag),
                            ty: ValueType::Int,
                        },
                    ],
                },
                ty: sorted_ty,
            }));
        }

        if (name == "min" || name == "max")
            && positional_args
                .iter()
                .all(|a| matches!(a.ty, ValueType::Int | ValueType::Float))
            && positional_args.iter().any(|a| a.ty == ValueType::Float)
        {
            positional_args = positional_args
                .into_iter()
                .map(|arg| self.cast_to_float_if_needed(arg))
                .collect();
        }

        if name == "sum" && positional_args.len() == 2 {
            let list_expr = positional_args[0].clone();
            let start_expr = positional_args[1].clone();
            if let ValueType::List(inner) = &list_expr.ty {
                let elem_ty = (**inner).clone();
                if elem_ty == start_expr.ty && matches!(elem_ty, ValueType::Class(_)) {
                    return Ok(CallResult::Expr(
                        self.lower_sum_class_list(line, list_expr, start_expr, elem_ty)?,
                    ));
                }
            }
        }

        self.lower_builtin_call(line, name, positional_args)
    }

    fn lower_callable_symbol_call(
        &mut self,
        line: usize,
        display_name: &str,
        mangled: &str,
        func_type: &Type,
        args: NormalizedCallArgs,
    ) -> Result<CallResult> {
        let bound_args = self.bind_user_function_args(
            line,
            display_name,
            mangled,
            func_type,
            args.positional,
            args.keyword,
        )?;

        let mut bound_args = match func_type {
            Type::Function { params, .. } => self.coerce_args_to_param_types(bound_args, params),
            _ => bound_args,
        };

        let return_type = self.check_call_args(line, display_name, func_type, &bound_args)?;
        self.append_nested_captures_if_needed(line, mangled, &mut bound_args)?;

        Ok(self.build_named_call_result(mangled.to_string(), return_type, bound_args))
    }

    fn lower_class_instance_method_call(
        &mut self,
        line: usize,
        object: TirExpr,
        class_name: &str,
        method_name: &str,
        args: NormalizedCallArgs,
    ) -> Result<CallResult> {
        let class_info = self
            .class_registry
            .get(class_name)
            .ok_or_else(|| self.name_error(line, format!("unknown class `{}`", class_name)))?
            .clone();

        let method = class_info.methods.get(method_name).ok_or_else(|| {
            self.attribute_error(
                line,
                format!("class `{}` has no method `{}`", class_name, method_name),
            )
        })?;

        if !args.keyword.is_empty() {
            return Err(self.syntax_error(line, "method keyword arguments are not supported"));
        }
        if args.positional.len() != method.params.len() {
            return Err(self.type_error(
                line,
                format!(
                    "{}.{}() expects {} argument{}, got {}",
                    class_name,
                    method_name,
                    method.params.len(),
                    if method.params.len() == 1 { "" } else { "s" },
                    args.positional.len()
                ),
            ));
        }
        for (i, (arg, expected)) in args.positional.iter().zip(method.params.iter()).enumerate() {
            let expected_vty = self.value_type_from_type(expected);
            if arg.ty != expected_vty {
                return Err(self.type_error(
                    line,
                    format!(
                        "argument {} type mismatch in {}.{}(): expected `{}`, got `{}`",
                        i, class_name, method_name, expected, arg.ty
                    ),
                ));
            }
        }

        let mut all_args = vec![object];
        all_args.extend(args.positional);
        Ok(self.build_named_call_result(
            method.mangled_name.clone(),
            method.return_type.clone(),
            all_args,
        ))
    }

    pub(super) fn normalize_call_args(
        &mut self,
        args_list: &Bound<PyList>,
        keywords_list: &Bound<PyList>,
        line: usize,
    ) -> Result<NormalizedCallArgs> {
        let mut positional = Vec::with_capacity(args_list.len());
        for arg in args_list.iter() {
            positional.push(self.lower_expr(&arg)?);
        }

        let mut keyword: Vec<(String, TirExpr)> = Vec::with_capacity(keywords_list.len());
        for kw in keywords_list.iter() {
            let kw_name_node = ast_getattr!(kw, "arg");
            if kw_name_node.is_none() {
                return Err(self.syntax_error(
                    line,
                    "dictionary unpacking in calls (`**kwargs`) is not supported",
                ));
            }
            let kw_name = kw_name_node.extract::<String>()?;
            let kw_value = self.lower_expr(&ast_getattr!(kw, "value"))?;
            keyword.push((kw_name, kw_value));
        }

        Ok(NormalizedCallArgs {
            positional,
            keyword,
        })
    }

    pub(super) fn detect_sum_generator_fast_path(
        &mut self,
        func_node: &Bound<PyAny>,
        args_list: &Bound<PyList>,
        keywords_list: &Bound<PyList>,
        line: usize,
    ) -> Result<Option<crate::tir::CallResult>> {
        if ast_type_name!(func_node) != "Name" {
            return Ok(None);
        }
        let func_name = crate::ast_get_string!(func_node, "id");
        if func_name != "sum" || args_list.len() != 2 || !keywords_list.is_empty() {
            return Ok(None);
        }
        let first_arg = args_list.get_item(0)?;
        if ast_type_name!(first_arg) != "GeneratorExp" {
            return Ok(None);
        }

        let start_expr = self.lower_expr(&args_list.get_item(1)?)?;
        let lowered = self.lower_sum_generator(&first_arg, start_expr, line)?;
        Ok(Some(crate::tir::CallResult::Expr(lowered)))
    }
}
