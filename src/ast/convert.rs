use anyhow::{bail, Result};
use pyo3::prelude::*;
use pyo3::types::PyAny;
use std::path::PathBuf;

use super::{BinOpKind, Expr, ExprKind, FunctionParam, Module, Span, Stmt, StmtKind, Type};
use crate::{ast_get_int, ast_get_list, ast_get_string, ast_getattr, ast_type_name};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportKind {
    /// `from . import module_a` — imports a module, calls via module_a.func()
    Module,
    /// `from .module_a import func_a` — imports a function directly
    Function,
}

#[derive(Debug, Clone)]
pub struct ImportDetail {
    pub kind: ImportKind,
    /// Name used in source code (may be alias)
    pub local_name: String,
    /// Actual name (same as local_name if no alias)
    pub original_name: String,
    /// For function imports: the source module name (e.g. "module_a")
    pub source_module: Option<String>,
    /// Relative import level (1 for `.`, 2 for `..`, etc.)
    pub level: usize,
}

pub struct AstConverter;

impl AstConverter {
    /// Convert a Python AST module to a Rust AST module
    pub fn convert_module(py_ast: &Bound<PyAny>, path: PathBuf) -> Result<Module> {
        let body_list = ast_get_list!(py_ast, "body");
        let mut body = Vec::new();

        for node in body_list.iter() {
            if let Some(stmt) = Self::convert_stmt(&node)? {
                body.push(stmt);
            }
        }

        Ok(Module::new(path, body))
    }

    /// Extract detailed import information from Python AST.
    ///
    /// Handles all import patterns:
    /// - `from . import module_a`       -> Module import
    /// - `from . import module_a as X`  -> Module import with alias
    /// - `from .module_a import func_a` -> Function import
    /// - `from .module_a import func_a as X` -> Function import with alias
    pub fn extract_import_info(py_ast: &Bound<PyAny>) -> Result<Vec<ImportDetail>> {
        let body_list = ast_get_list!(py_ast, "body");
        let mut imports = Vec::new();

        for node in body_list.iter() {
            let node_type = ast_type_name!(node);

            if node_type == "ImportFrom" {
                let level = ast_get_int!(node, "level", usize);
                let module_val = ast_getattr!(node, "module");
                let module_name: Option<String> = if module_val.is_none() {
                    None
                } else {
                    Some(module_val.extract::<String>()?)
                };

                let names_list = ast_get_list!(node, "names");

                for name_node in names_list.iter() {
                    let name = ast_get_string!(name_node, "name");
                    let asname_node = ast_getattr!(name_node, "asname");
                    let local_name = if asname_node.is_none() {
                        name.clone()
                    } else {
                        asname_node.extract::<String>()?
                    };

                    if module_name.is_some() {
                        // `from .module_a import func_a` — function import
                        imports.push(ImportDetail {
                            kind: ImportKind::Function,
                            local_name,
                            original_name: name,
                            source_module: module_name.clone(),
                            level,
                        });
                    } else {
                        // `from . import module_a` — module import
                        imports.push(ImportDetail {
                            kind: ImportKind::Module,
                            local_name,
                            original_name: name,
                            source_module: None,
                            level,
                        });
                    }
                }
            }
        }

        Ok(imports)
    }

    fn convert_stmt(node: &Bound<PyAny>) -> Result<Option<Stmt>> {
        let node_type = ast_type_name!(node);
        let span = Self::extract_span(node);

        let kind = match node_type.as_str() {
            "FunctionDef" => Self::convert_function_def(node)?,
            "AnnAssign" => Self::convert_ann_assign(node)?,
            "Assign" => Self::convert_assign(node)?,
            "Return" => Self::convert_return(node)?,
            "Expr" => Self::convert_expr_stmt(node)?,
            "Import" | "ImportFrom" | "Assert" => return Ok(None), // Skip imports and asserts
            _ => bail!(
                "Unsupported statement type: {} at line {}",
                node_type,
                span.line
            ),
        };

        Ok(Some(Stmt::new(kind, span)))
    }

    fn convert_function_def(node: &Bound<PyAny>) -> Result<StmtKind> {
        let name = ast_get_string!(node, "name");

        // Extract parameters with type annotations
        let args_node = ast_getattr!(node, "args");
        let py_args = ast_get_list!(&args_node, "args");

        let mut params = Vec::new();
        for arg in py_args.iter() {
            let param_name = ast_get_string!(arg, "arg");
            let annotation = ast_getattr!(arg, "annotation");

            if annotation.is_none() {
                bail!(
                    "Parameter '{}' in function '{}' requires type annotation",
                    param_name,
                    name
                );
            }

            let ty = Self::convert_type_annotation(&annotation)?;
            params.push(FunctionParam::new(param_name, ty));
        }

        // Extract return type annotation
        let returns = ast_getattr!(node, "returns");
        let return_type = if returns.is_none() {
            Type::Unit
        } else {
            Self::convert_type_annotation(&returns)?
        };

        // Convert body
        let body_list = ast_get_list!(node, "body");
        let mut body = Vec::new();
        for stmt_node in body_list.iter() {
            if let Some(stmt) = Self::convert_stmt(&stmt_node)? {
                body.push(stmt);
            }
        }

        Ok(StmtKind::FunctionDef {
            name,
            params,
            return_type,
            body,
        })
    }

    fn convert_ann_assign(node: &Bound<PyAny>) -> Result<StmtKind> {
        // AnnAssign: target : annotation = value
        let target_node = ast_getattr!(node, "target");
        let target = if ast_type_name!(target_node) == "Name" {
            ast_get_string!(target_node, "id")
        } else {
            bail!("Only simple variable assignments are supported");
        };

        let annotation = ast_getattr!(node, "annotation");
        let ty = if !annotation.is_none() {
            Some(Self::convert_type_annotation(&annotation)?)
        } else {
            None
        };

        let value_node = ast_getattr!(node, "value");
        let value = Self::convert_expr(&value_node)?;

        Ok(StmtKind::Assign { target, ty, value })
    }

    fn convert_assign(node: &Bound<PyAny>) -> Result<StmtKind> {
        // Assign: target = value (no type annotation)
        let targets_list = ast_get_list!(node, "targets");
        if targets_list.len() != 1 {
            bail!("Multiple assignment targets not supported");
        }

        let target_node = targets_list.get_item(0)?;
        let target = if ast_type_name!(target_node) == "Name" {
            ast_get_string!(target_node, "id")
        } else {
            bail!("Only simple variable assignments are supported");
        };

        let value_node = ast_getattr!(node, "value");
        let value = Self::convert_expr(&value_node)?;

        // No type annotation - will be inferred
        Ok(StmtKind::Assign {
            target,
            ty: None,
            value,
        })
    }

    fn convert_return(node: &Bound<PyAny>) -> Result<StmtKind> {
        let value_node = ast_getattr!(node, "value");
        let expr = if value_node.is_none() {
            None
        } else {
            Some(Self::convert_expr(&value_node)?)
        };

        Ok(StmtKind::Return(expr))
    }

    fn convert_expr_stmt(node: &Bound<PyAny>) -> Result<StmtKind> {
        let value_node = ast_getattr!(node, "value");
        let expr = Self::convert_expr(&value_node)?;
        Ok(StmtKind::Expr(expr))
    }

    fn convert_type_annotation(node: &Bound<PyAny>) -> Result<Type> {
        let node_type = ast_type_name!(node);

        match node_type.as_str() {
            "Name" => {
                let id = ast_get_string!(node, "id");
                match id.as_str() {
                    "int" => Ok(Type::Int),
                    _ => bail!("Unsupported type: {}", id),
                }
            }
            "Constant" => {
                // In Python 3.8+, None is represented as a Constant
                let value = ast_getattr!(node, "value");
                if value.is_none() {
                    Ok(Type::Unit)
                } else {
                    bail!("Unsupported constant type annotation")
                }
            }
            _ => bail!("Unsupported type annotation: {}", node_type),
        }
    }

    fn convert_expr(node: &Bound<PyAny>) -> Result<Expr> {
        let node_type = ast_type_name!(node);
        let span = Self::extract_span(node);

        let kind = match node_type.as_str() {
            "Constant" => {
                let value = ast_getattr!(node, "value");
                if let Ok(int_val) = value.extract::<i64>() {
                    ExprKind::IntLiteral(int_val)
                } else {
                    bail!("Unsupported constant type at line {}", span.line);
                }
            }
            "Name" => {
                let id = ast_get_string!(node, "id");
                ExprKind::Var(id)
            }
            "BinOp" => {
                let left = Self::convert_expr(&ast_getattr!(node, "left"))?;
                let right = Self::convert_expr(&ast_getattr!(node, "right"))?;
                let op_node = ast_getattr!(node, "op");
                let op = Self::convert_binop(&op_node)?;
                ExprKind::BinOp {
                    op,
                    left: Box::new(left),
                    right: Box::new(right),
                }
            }
            "Call" => {
                let func = Self::convert_expr(&ast_getattr!(node, "func"))?;
                let args_list = ast_get_list!(node, "args");
                let mut args = Vec::new();
                for arg in args_list.iter() {
                    args.push(Self::convert_expr(&arg)?);
                }
                ExprKind::Call {
                    func: Box::new(func),
                    args,
                }
            }
            "Attribute" => {
                let value = Self::convert_expr(&ast_getattr!(node, "value"))?;
                let attr = ast_get_string!(node, "attr");
                ExprKind::Attribute {
                    value: Box::new(value),
                    attr,
                }
            }
            _ => bail!(
                "Unsupported expression type: {} at line {}",
                node_type,
                span.line
            ),
        };

        Ok(Expr::new(kind, span))
    }

    fn convert_binop(node: &Bound<PyAny>) -> Result<BinOpKind> {
        let op_type = ast_type_name!(node);
        match op_type.as_str() {
            "Add" => Ok(BinOpKind::Add),
            "Sub" => Ok(BinOpKind::Sub),
            "Mult" => Ok(BinOpKind::Mul),
            "Div" => Ok(BinOpKind::Div),
            "Mod" => Ok(BinOpKind::Mod),
            _ => bail!("Unsupported binary operator: {}", op_type),
        }
    }

    fn extract_span(node: &Bound<PyAny>) -> Span {
        let line: usize = ast_getattr!(node, "lineno")
            .extract::<usize>()
            .unwrap_or_default();
        let column: usize = ast_getattr!(node, "col_offset")
            .extract::<usize>()
            .unwrap_or_default();
        Span::new(line, column)
    }
}
