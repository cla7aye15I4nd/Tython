use anyhow::Result;
use pyo3::prelude::*;

use crate::tir::{FunctionParam, TirExpr, TirExprKind, TirFunction, TirStmt, ValueType};
use crate::{ast_get_list, ast_get_string, ast_getattr, ast_type_name};

use super::Lowering;

impl Lowering {
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

        self.declare(name, func_type);
        Ok(())
    }

    pub(super) fn lower_function(&mut self, node: &Bound<PyAny>) -> Result<TirFunction> {
        let name = ast_get_string!(node, "name");
        let mangled_name = self.mangle_name(&name);

        let args_node = ast_getattr!(node, "args");
        let py_args = ast_get_list!(&args_node, "args");
        let mut params = Vec::new();
        for arg in py_args.iter() {
            let param_name = ast_get_string!(arg, "arg");
            let annotation = ast_getattr!(arg, "annotation");
            let ty = self.convert_type_annotation(&annotation)?;
            let vty = Self::to_value_type(&ty);
            params.push(FunctionParam::new(param_name, vty));
        }

        let return_type_ast = self.convert_return_type(node)?;
        let return_type = Self::to_opt_value_type(&return_type_ast);

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
                continue;
            }
            tir_body.extend(self.lower_stmt(&stmt_node)?);
        }

        self.pop_scope();
        self.current_return_type = None;
        self.current_function_name = None;

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
