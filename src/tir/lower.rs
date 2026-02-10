use anyhow::{bail, Context as _, Result};
use pyo3::prelude::*;
use pyo3::types::PyModule;
use std::collections::HashMap;
use std::path::Path;

use super::{BinOpKind, FunctionParam, TirExpr, TirExprKind, TirFunction, TirModule, TirStmt};
use crate::ast::Type;
use crate::symbol_table::SymbolTable;
use crate::{ast_get_list, ast_get_string, ast_getattr, ast_type_name};

pub struct Lowering {
    symbol_table: SymbolTable,
    module_path: String,
    functions: HashMap<String, Type>,
    call_resolution_map: HashMap<String, String>,
    module_import_map: HashMap<String, String>,
    variables: HashMap<String, Type>,
    current_return_type: Option<Type>,
}

impl Default for Lowering {
    fn default() -> Self {
        Self::new()
    }
}

impl Lowering {
    pub fn new() -> Self {
        Self {
            symbol_table: SymbolTable::new(),
            module_path: String::new(),
            functions: HashMap::new(),
            call_resolution_map: HashMap::new(),
            module_import_map: HashMap::new(),
            variables: HashMap::new(),
            current_return_type: None,
        }
    }

    pub fn lower_module(
        &mut self,
        canonical_path: &Path,
        module_path: &str,
        imports: &HashMap<String, Type>,
    ) -> Result<TirModule> {
        self.module_path = module_path.to_string();
        self.functions.clear();
        self.call_resolution_map.clear();
        self.module_import_map.clear();
        self.variables.clear();
        self.current_return_type = None;

        for (local_name, ty) in imports {
            if let Type::Module(mangled) = ty {
                if let Some(func_type) = self.symbol_table.get_type(mangled) {
                    self.functions.insert(local_name.clone(), func_type.clone());
                    self.call_resolution_map
                        .insert(local_name.clone(), mangled.clone());
                } else {
                    self.module_import_map
                        .insert(local_name.clone(), mangled.clone());
                }
            }
        }

        Python::attach(|py| -> Result<_> {
            let source = std::fs::read_to_string(canonical_path)?;
            let ast_module = PyModule::import(py, "ast")?;
            let py_ast = ast_module.call_method1("parse", (source.as_str(),))?;

            self.lower_py_ast(&py_ast)
        })
    }

    fn lower_py_ast(&mut self, py_ast: &Bound<PyAny>) -> Result<TirModule> {
        let body_list = ast_get_list!(py_ast, "body");

        // Phase 1: collect all function signatures
        for node in body_list.iter() {
            if ast_type_name!(node) == "FunctionDef" {
                self.collect_function_signature(&node)?;
            }
        }

        // Phase 2: lower function bodies + collect module-level statements
        let mut functions = HashMap::new();
        let mut module_level_stmts = Vec::new();

        for node in body_list.iter() {
            match ast_type_name!(node).as_str() {
                "FunctionDef" => {
                    let tir_func = self.lower_function(&node)?;
                    functions.insert(tir_func.name.clone(), tir_func);
                }
                "Import" | "ImportFrom" | "Assert" => {}
                _ => {
                    module_level_stmts.push(self.lower_stmt(&node)?);
                }
            }
        }

        if !module_level_stmts.is_empty() {
            let main_func = self.build_synthetic_main(module_level_stmts);
            functions.insert(main_func.name.clone(), main_func);
        }

        for func in functions.values() {
            let func_type = Type::Function {
                params: func.params.iter().map(|p| p.ty.clone()).collect(),
                return_type: Box::new(func.return_type.clone()),
            };
            self.symbol_table
                .register_function(func.name.clone(), func_type);
        }

        Ok(TirModule { functions })
    }

    fn collect_function_signature(&mut self, node: &Bound<PyAny>) -> Result<()> {
        let name = ast_get_string!(node, "name");

        let args_node = ast_getattr!(node, "args");
        let py_args = ast_get_list!(&args_node, "args");

        let mut param_types = Vec::new();
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
            param_types.push(Self::convert_type_annotation(&annotation)?);
        }

        let return_type = Self::convert_return_type(node)?;
        let func_type = Type::Function {
            params: param_types,
            return_type: Box::new(return_type),
        };

        self.functions.insert(name.clone(), func_type);
        let mangled = self.mangle_name(&name);
        self.call_resolution_map.insert(name, mangled);

        Ok(())
    }

    fn lower_function(&mut self, node: &Bound<PyAny>) -> Result<TirFunction> {
        let name = ast_get_string!(node, "name");
        let mangled_name = self.mangle_name(&name);

        let args_node = ast_getattr!(node, "args");
        let py_args = ast_get_list!(&args_node, "args");
        let mut params = Vec::new();
        for arg in py_args.iter() {
            let param_name = ast_get_string!(arg, "arg");
            let annotation = ast_getattr!(arg, "annotation");
            let ty = Self::convert_type_annotation(&annotation)?;
            params.push(FunctionParam::new(param_name, ty));
        }

        let return_type = Self::convert_return_type(node)?;

        self.variables.clear();
        for param in &params {
            self.variables.insert(param.name.clone(), param.ty.clone());
        }
        self.current_return_type = Some(return_type.clone());

        let body_list = ast_get_list!(node, "body");
        let mut tir_body = Vec::new();
        for stmt_node in body_list.iter() {
            let node_type = ast_type_name!(stmt_node);
            if node_type == "Import" || node_type == "ImportFrom" || node_type == "Assert" {
                continue;
            }
            tir_body.push(self.lower_stmt(&stmt_node).with_context(|| {
                format!(
                    "In function '{}' at line {}",
                    name,
                    Self::get_line(&stmt_node)
                )
            })?);
        }

        self.variables.clear();
        self.current_return_type = None;

        Ok(TirFunction {
            name: mangled_name,
            params,
            return_type,
            body: tir_body,
        })
    }

    fn build_synthetic_main(&self, mut stmts: Vec<TirStmt>) -> TirFunction {
        stmts.push(TirStmt::Return(Some(TirExpr {
            kind: TirExprKind::IntLiteral(0),
            ty: Type::Int,
        })));

        TirFunction {
            name: format!("{}$$main$", self.module_path),
            params: Vec::new(),
            return_type: Type::Int,
            body: stmts,
        }
    }

    fn lower_stmt(&mut self, node: &Bound<PyAny>) -> Result<TirStmt> {
        let node_type = ast_type_name!(node);
        let line = Self::get_line(node);

        match node_type.as_str() {
            "FunctionDef" => bail!("Nested functions not supported at line {}", line),

            "AnnAssign" => {
                let target_node = ast_getattr!(node, "target");
                if ast_type_name!(target_node) != "Name" {
                    bail!(
                        "Only simple variable assignments are supported at line {}",
                        line
                    );
                }
                let target = ast_get_string!(target_node, "id");

                let annotation = ast_getattr!(node, "annotation");
                let annotated_ty = (!annotation.is_none())
                    .then(|| Self::convert_type_annotation(&annotation))
                    .transpose()?;

                let value_node = ast_getattr!(node, "value");
                let tir_value = self.lower_expr(&value_node)?;

                if let Some(ref ann_ty) = annotated_ty {
                    if ann_ty != &tir_value.ty {
                        bail!(
                            "Type mismatch at line {}: expected {:?}, got {:?}",
                            line,
                            ann_ty,
                            tir_value.ty
                        );
                    }
                }

                let var_type = annotated_ty.unwrap_or_else(|| tir_value.ty.clone());
                self.variables.insert(target.clone(), var_type.clone());

                Ok(TirStmt::Let {
                    name: target,
                    ty: var_type,
                    value: tir_value,
                })
            }

            "Assign" => {
                let targets_list = ast_get_list!(node, "targets");
                if targets_list.len() != 1 {
                    bail!("Multiple assignment targets not supported at line {}", line);
                }

                let target_node = targets_list.get_item(0)?;
                if ast_type_name!(target_node) != "Name" {
                    bail!(
                        "Only simple variable assignments are supported at line {}",
                        line
                    );
                }
                let target = ast_get_string!(target_node, "id");

                let value_node = ast_getattr!(node, "value");
                let tir_value = self.lower_expr(&value_node)?;
                let var_type = tir_value.ty.clone();
                self.variables.insert(target.clone(), var_type.clone());

                Ok(TirStmt::Let {
                    name: target,
                    ty: var_type,
                    value: tir_value,
                })
            }

            "Return" => {
                let value_node = ast_getattr!(node, "value");
                if value_node.is_none() {
                    if let Some(ref expected) = self.current_return_type {
                        if *expected != Type::Unit {
                            bail!(
                                "Return without value at line {}, but function expects {:?}",
                                line,
                                expected
                            );
                        }
                    }
                    Ok(TirStmt::Return(None))
                } else {
                    let tir_expr = self.lower_expr(&value_node)?;
                    if let Some(ref expected) = self.current_return_type {
                        if expected != &tir_expr.ty {
                            bail!(
                                "Return type mismatch at line {}: expected {:?}, got {:?}",
                                line,
                                expected,
                                tir_expr.ty
                            );
                        }
                    }
                    Ok(TirStmt::Return(Some(tir_expr)))
                }
            }

            "Expr" => {
                let value_node = ast_getattr!(node, "value");
                Ok(TirStmt::Expr(self.lower_expr(&value_node)?))
            }

            _ => bail!("Unsupported statement type: {} at line {}", node_type, line),
        }
    }

    fn lower_expr(&mut self, node: &Bound<PyAny>) -> Result<TirExpr> {
        let node_type = ast_type_name!(node);
        let line = Self::get_line(node);
        let col = Self::get_col(node);

        match node_type.as_str() {
            "Constant" => {
                let value = ast_getattr!(node, "value");
                if let Ok(int_val) = value.extract::<i64>() {
                    Ok(TirExpr {
                        kind: TirExprKind::IntLiteral(int_val),
                        ty: Type::Int,
                    })
                } else {
                    bail!("Unsupported constant type at line {}", line)
                }
            }

            "Name" => {
                let id = ast_get_string!(node, "id");
                let ty = self.variables.get(&id).cloned().ok_or_else(|| {
                    anyhow::anyhow!(
                        "Undefined variable: {} at line {}, column {}",
                        id,
                        line,
                        col
                    )
                })?;
                Ok(TirExpr {
                    kind: TirExprKind::Var(id),
                    ty,
                })
            }

            "BinOp" => {
                let left = self.lower_expr(&ast_getattr!(node, "left"))?;
                let right = self.lower_expr(&ast_getattr!(node, "right"))?;
                let op = Self::convert_binop(&ast_getattr!(node, "op"))?;

                if left.ty != Type::Int || right.ty != Type::Int {
                    bail!(
                        "Binary operator {:?} at line {} requires int operands, got {:?} and {:?}",
                        op,
                        line,
                        left.ty,
                        right.ty
                    );
                }

                Ok(TirExpr {
                    kind: TirExprKind::BinOp {
                        op,
                        left: Box::new(left),
                        right: Box::new(right),
                    },
                    ty: Type::Int,
                })
            }

            "Call" => self.lower_call(node, line, col),

            "Attribute" => {
                bail!(
                    "Attribute access outside of function calls not yet supported at line {}",
                    line
                )
            }

            _ => bail!(
                "Unsupported expression type: {} at line {}",
                node_type,
                line
            ),
        }
    }

    fn lower_call(&mut self, node: &Bound<PyAny>, line: usize, col: usize) -> Result<TirExpr> {
        let func_node = ast_getattr!(node, "func");
        let args_list = ast_get_list!(node, "args");

        let mut tir_args = Vec::new();
        for arg in args_list.iter() {
            tir_args.push(self.lower_expr(&arg)?);
        }

        let func_node_type = ast_type_name!(func_node);
        match func_node_type.as_str() {
            "Name" => {
                let func_name = ast_get_string!(func_node, "id");

                if func_name == "print" {
                    return Ok(TirExpr {
                        kind: TirExprKind::Call {
                            func: "print".to_string(),
                            args: tir_args,
                        },
                        ty: Type::Unit,
                    });
                }

                let func_type = self.functions.get(&func_name).cloned().ok_or_else(|| {
                    anyhow::anyhow!(
                        "Undefined function: {} at line {}, column {}",
                        func_name,
                        line,
                        col
                    )
                })?;

                let return_type = self.check_call_args(&func_name, &func_type, &tir_args, line)?;

                let resolved = self
                    .call_resolution_map
                    .get(&func_name)
                    .cloned()
                    .unwrap_or(func_name);

                Ok(TirExpr {
                    kind: TirExprKind::Call {
                        func: resolved,
                        args: tir_args,
                    },
                    ty: return_type,
                })
            }

            "Attribute" => {
                let value_node = ast_getattr!(func_node, "value");
                let attr = ast_get_string!(func_node, "attr");

                if ast_type_name!(value_node) != "Name" {
                    bail!("Complex attribute access not supported at line {}", line);
                }
                let mod_name = ast_get_string!(value_node, "id");

                let mod_path = self.module_import_map.get(&mod_name).ok_or_else(|| {
                    anyhow::anyhow!("Unknown module: {} at line {}", mod_name, line)
                })?;
                let resolved = format!("{}${}", mod_path, attr);

                let func_type = self
                    .symbol_table
                    .get_type(&resolved)
                    .ok_or_else(|| {
                        anyhow::anyhow!(
                            "Undefined function: {}.{} at line {}, column {}",
                            mod_name,
                            attr,
                            line,
                            col
                        )
                    })?
                    .clone();

                let return_type = {
                    let label = format!("{}.{}", mod_name, attr);
                    self.check_call_args(&label, &func_type, &tir_args, line)?
                };

                Ok(TirExpr {
                    kind: TirExprKind::Call {
                        func: resolved,
                        args: tir_args,
                    },
                    ty: return_type,
                })
            }

            _ => bail!(
                "Only direct function calls and module.function calls supported at line {}",
                line
            ),
        }
    }

    fn check_call_args(
        &self,
        func_name: &str,
        func_type: &Type,
        args: &[TirExpr],
        line: usize,
    ) -> Result<Type> {
        match func_type {
            Type::Function {
                params,
                return_type,
            } => {
                if args.len() != params.len() {
                    bail!(
                        "Function '{}' at line {} expects {} arguments, got {}",
                        func_name,
                        line,
                        params.len(),
                        args.len()
                    );
                }
                for (i, (arg, expected)) in args.iter().zip(params.iter()).enumerate() {
                    if &arg.ty != expected {
                        bail!(
                            "Argument {} type mismatch in call to '{}' at line {}: expected {:?}, got {:?}",
                            i, func_name, line, expected, arg.ty
                        );
                    }
                }
                Ok(*return_type.clone())
            }
            _ => bail!("Cannot call non-function type at line {}", line),
        }
    }

    fn mangle_name(&self, name: &str) -> String {
        if name == "main" {
            format!("{}$$main$", self.module_path)
        } else {
            format!("{}${}", self.module_path, name)
        }
    }

    fn convert_return_type(node: &Bound<PyAny>) -> Result<Type> {
        let returns = ast_getattr!(node, "returns");
        if returns.is_none() {
            Ok(Type::Unit)
        } else {
            Self::convert_type_annotation(&returns)
        }
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

    fn get_line(node: &Bound<PyAny>) -> usize {
        ast_getattr!(node, "lineno")
            .extract::<usize>()
            .unwrap_or_default()
    }

    fn get_col(node: &Bound<PyAny>) -> usize {
        ast_getattr!(node, "col_offset")
            .extract::<usize>()
            .unwrap_or_default()
    }
}
