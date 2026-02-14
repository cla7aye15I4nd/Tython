use anyhow::Result;
use pyo3::prelude::*;

use crate::ast::Type;
use crate::tir::{FunctionParam, TirExpr, TirExprKind, TirFunction, TirStmt, ValueType};
use crate::{ast_get_list, ast_get_string, ast_getattr, ast_type_name};

use super::Lowering;

impl Lowering {
    pub(super) fn lower_defaults_for_params(
        &mut self,
        args_node: &Bound<PyAny>,
        line: usize,
        fn_name: &str,
    ) -> Result<Vec<Option<TirExpr>>> {
        let py_args = ast_get_list!(args_node, "args");
        let defaults = ast_get_list!(args_node, "defaults");
        let kwonlyargs = ast_get_list!(args_node, "kwonlyargs");
        if !kwonlyargs.is_empty() {
            return Err(self.syntax_error(
                line,
                format!(
                    "function `{}`: keyword-only parameters are not supported",
                    fn_name
                ),
            ));
        }

        let posonlyargs = ast_get_list!(args_node, "posonlyargs");
        if !posonlyargs.is_empty() {
            return Err(self.syntax_error(
                line,
                format!(
                    "function `{}`: positional-only parameters are not supported",
                    fn_name
                ),
            ));
        }

        if !ast_getattr!(args_node, "vararg").is_none() {
            return Err(self.syntax_error(
                line,
                format!("function `{}`: `*args` is not supported", fn_name),
            ));
        }
        if !ast_getattr!(args_node, "kwarg").is_none() {
            return Err(self.syntax_error(
                line,
                format!("function `{}`: `**kwargs` is not supported", fn_name),
            ));
        }

        let n_params = py_args.len();
        let n_defaults = defaults.len();
        if n_defaults > n_params {
            return Err(self.syntax_error(
                line,
                format!(
                    "function `{}`: invalid defaults ({} defaults for {} parameters)",
                    fn_name, n_defaults, n_params
                ),
            ));
        }

        let mut out = vec![None; n_params];
        let start = n_params - n_defaults;
        for i in 0..n_defaults {
            let idx = start + i;
            let def_node = defaults.get_item(i)?;
            let default_expr = if ast_type_name!(def_node) == "List"
                && ast_get_list!(&def_node, "elts").is_empty()
            {
                let param_node = py_args.get_item(idx)?;
                let annotation = ast_getattr!(param_node, "annotation");
                let param_ty = self.convert_type_annotation(&annotation)?;
                match param_ty {
                    Type::List(inner) => {
                        let elem_ty = ValueType::from_type(&inner).expect(
                            "ICE: list default annotation should contain a value element type",
                        );
                        TirExpr {
                            kind: TirExprKind::ListLiteral {
                                element_type: elem_ty.clone(),
                                elements: vec![],
                            },
                            ty: ValueType::List(Box::new(elem_ty)),
                        }
                    }
                    other => {
                        return Err(self.syntax_error(
                            line,
                            format!(
                                "empty list default for `{}` requires list annotation, got `{}`",
                                fn_name, other
                            ),
                        ))
                    }
                }
            } else {
                self.lower_expr(&def_node)?
            };
            self.ensure_supported_default_expr(line, &default_expr)?;
            out[idx] = Some(default_expr);
        }

        Ok(out)
    }

    pub(super) fn collect_function_signature(&mut self, node: &Bound<PyAny>) -> Result<()> {
        let name = ast_get_string!(node, "name");
        let line = Self::get_line(node);

        let args_node = ast_getattr!(node, "args");
        let py_args = ast_get_list!(&args_node, "args");

        let mut param_types = Vec::new();
        for arg in py_args.iter() {
            let param_name = ast_get_string!(arg, "arg");
            let annotation = ast_getattr!(arg, "annotation");
            if annotation.is_none() {
                return Err(self.syntax_error(
                    line,
                    format!(
                        "parameter `{}` in function `{}` requires a type annotation",
                        param_name, name
                    ),
                ));
            }
            param_types.push(self.convert_type_annotation(&annotation)?);
        }

        let return_type = self.convert_return_type(node)?;
        let func_type = crate::ast::Type::Function {
            params: param_types,
            return_type: Box::new(return_type),
        };

        let mangled = self.mangle_name(&name);
        self.function_mangled_names.insert(name.clone(), mangled);
        self.declare(name, func_type);
        Ok(())
    }

    pub(super) fn lower_function(&mut self, node: &Bound<PyAny>) -> Result<TirFunction> {
        let name = ast_get_string!(node, "name");
        let mangled_name = self.mangle_name(&name);

        let args_node = ast_getattr!(node, "args");
        let py_args = ast_get_list!(&args_node, "args");
        let default_values =
            self.lower_defaults_for_params(&args_node, Self::get_line(node), &name)?;
        let mut params = Vec::new();
        let mut param_names = Vec::new();
        for arg in py_args.iter() {
            let param_name = ast_get_string!(arg, "arg");
            let annotation = ast_getattr!(arg, "annotation");
            let ty = self.convert_type_annotation(&annotation)?;
            let vty = self.value_type_from_type(&ty);
            params.push(FunctionParam::new(param_name, vty));
            param_names.push(ast_get_string!(arg, "arg"));
        }

        let return_type_ast = self.convert_return_type(node)?;
        let return_type = self.opt_value_type_from_type(&return_type_ast);

        self.push_scope();
        for param in &params {
            self.declare(param.name.clone(), param.ty.to_type());
        }
        self.current_return_type = Some(return_type_ast);
        self.current_function_name = Some(name.clone());

        let body_list = ast_get_list!(node, "body");
        let mut tir_body = Vec::new();
        for stmt_node in body_list.iter() {
            let node_type = ast_type_name!(stmt_node);
            if node_type == "Import" || node_type == "ImportFrom" {
                return Err(self.syntax_error(
                    Self::get_line(&stmt_node),
                    "imports are only allowed at module top-level",
                ));
            }
            tir_body.extend(self.lower_stmt(&stmt_node)?);
        }

        self.pop_scope();
        self.current_return_type = None;
        self.current_function_name = None;

        self.register_function_signature(mangled_name.clone(), param_names, default_values);

        Ok(TirFunction {
            name: mangled_name,
            params,
            return_type,
            body: tir_body,
        })
    }

    pub(super) fn build_synthetic_main(&self, mut stmts: Vec<TirStmt>) -> TirFunction {
        stmts.push(TirStmt::Return(Some(TirExpr {
            kind: TirExprKind::IntLiteral(0),
            ty: ValueType::Int,
        })));

        TirFunction {
            name: self.mangle_name("$main$"),
            params: Vec::new(),
            return_type: Some(ValueType::Int),
            body: stmts,
        }
    }
}
