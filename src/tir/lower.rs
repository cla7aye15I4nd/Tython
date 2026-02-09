use anyhow::{bail, Context as _, Result};
use pyo3::prelude::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::{BinOpKind, FunctionParam, TirExpr, TirExprKind, TirFunction, TirModule, TirStmt};
use crate::ast::Type;
use crate::resolver::Resolver;
use crate::symbol_table::SymbolTable;
use crate::{ast_get_int, ast_get_list, ast_get_string, ast_getattr, ast_type_name};

// ---------------------------------------------------------------------------
// Import types (moved from ast/convert.rs)
// ---------------------------------------------------------------------------

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
    pub local_name: String,
    pub original_name: String,
    pub source_module: Option<String>,
    pub level: usize,
}

// ---------------------------------------------------------------------------
// Lowering context
// ---------------------------------------------------------------------------

pub struct LoweringContext {
    module_path: String,
    /// Module-level function signatures (source name → Type::Function)
    functions: HashMap<String, Type>,
    /// Direct call names → mangled LLVM names
    call_resolution_map: HashMap<String, String>,
    /// Module aliases → dotted module paths
    module_import_map: HashMap<String, String>,
    /// Function-scoped variables (reset per function)
    variables: HashMap<String, Type>,
    /// Expected return type of current function
    current_return_type: Option<Type>,
}

impl LoweringContext {
    fn new(module_path: String) -> Self {
        Self {
            module_path,
            functions: HashMap::new(),
            call_resolution_map: HashMap::new(),
            module_import_map: HashMap::new(),
            variables: HashMap::new(),
            current_return_type: None,
        }
    }

    // -----------------------------------------------------------------------
    // Public entry point
    // -----------------------------------------------------------------------

    pub fn lower_module(
        py_ast: &Bound<PyAny>,
        canonical_path: &Path,
        module_path: &str,
        dependencies: &[PathBuf],
        symbol_table: &SymbolTable,
        resolver: &Resolver,
    ) -> Result<TirModule> {
        let mut ctx = Self::new(module_path.to_string());

        let body_list = ast_get_list!(py_ast, "body");

        // Extract import details
        let import_details = Self::extract_import_info(py_ast)?;

        // Register imported symbols + build resolution maps
        ctx.register_imports(
            canonical_path,
            &import_details,
            dependencies,
            symbol_table,
            resolver,
        )?;

        // Phase 1: collect all function signatures (lightweight scan)
        for node in body_list.iter() {
            let node_type = ast_type_name!(node);
            if node_type == "FunctionDef" {
                ctx.collect_function_signature(&node)?;
            }
        }

        // Phase 2: lower function bodies + collect module-level statements
        let mut functions = HashMap::new();
        let mut module_level_stmts = Vec::new();

        for node in body_list.iter() {
            let node_type = ast_type_name!(node);
            match node_type.as_str() {
                "FunctionDef" => {
                    let tir_func = ctx.lower_function(&node)?;
                    functions.insert(tir_func.name.clone(), tir_func);
                }
                "Import" | "ImportFrom" | "Assert" => {
                    // Skip imports and asserts
                }
                _ => {
                    // Module-level statement → collect for synthetic main
                    let tir_stmt = ctx.lower_stmt(&node)?;
                    module_level_stmts.push(tir_stmt);
                }
            }
        }

        // Wrap module-level statements in synthetic main if needed
        if !module_level_stmts.is_empty() {
            let main_func = ctx.build_synthetic_main(module_level_stmts);
            functions.insert(main_func.name.clone(), main_func);
        }

        Ok(TirModule {
            path: canonical_path.to_path_buf(),
            functions,
        })
    }

    // -----------------------------------------------------------------------
    // Import extraction (from ast/convert.rs)
    // -----------------------------------------------------------------------

    fn extract_import_info(py_ast: &Bound<PyAny>) -> Result<Vec<ImportDetail>> {
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
                        imports.push(ImportDetail {
                            kind: ImportKind::Function,
                            local_name,
                            original_name: name,
                            source_module: module_name.clone(),
                            level,
                        });
                    } else {
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

    // -----------------------------------------------------------------------
    // Import registration + resolution maps
    // (from compiler.rs register_dependency_types + build_resolution_maps)
    // -----------------------------------------------------------------------

    fn register_imports(
        &mut self,
        canonical_path: &Path,
        import_details: &[ImportDetail],
        dependencies: &[PathBuf],
        symbol_table: &SymbolTable,
        resolver: &Resolver,
    ) -> Result<()> {
        let file_dir = canonical_path.parent().unwrap();

        // Register dependency function types (unmangled names)
        for dep_path in dependencies {
            if let Some(mangled_names) = symbol_table.get_functions_for_module(dep_path) {
                for mangled_name in mangled_names {
                    if mangled_name.contains("$$main$") {
                        continue;
                    }
                    let func_type = symbol_table.get_type(mangled_name).unwrap().clone();
                    let name = Self::original_name_from_mangled(mangled_name);
                    self.functions.insert(name, func_type);
                }
            }
        }

        // Handle aliased imports
        for detail in import_details {
            if detail.local_name != detail.original_name {
                for dep_path in dependencies {
                    let mangled = resolver.mangle_function_name(dep_path, &detail.original_name);
                    if let Some(func_type) = symbol_table.get_type(&mangled) {
                        self.functions
                            .insert(detail.local_name.clone(), func_type.clone());
                        break;
                    }
                }
            }
        }

        // Build resolution maps from import details
        for detail in import_details {
            match detail.kind {
                ImportKind::Module => {
                    let dep_path =
                        resolver.resolve_module(file_dir, detail.level, &detail.original_name)?;
                    let dep_module_path = resolver.compute_module_path(&dep_path);
                    self.module_import_map
                        .insert(detail.local_name.clone(), dep_module_path);
                }
                ImportKind::Function => {
                    let source = detail.source_module.as_ref().unwrap();
                    let source_file = resolver.resolve_module(file_dir, detail.level, source);

                    if let Ok(source_path) = source_file {
                        let mangled =
                            resolver.mangle_function_name(&source_path, &detail.original_name);
                        self.call_resolution_map
                            .insert(detail.local_name.clone(), mangled);
                    } else {
                        let full_module = format!("{}.{}", source, detail.original_name);
                        let dep_path =
                            resolver.resolve_module(file_dir, detail.level, &full_module)?;
                        let dep_module_path = resolver.compute_module_path(&dep_path);
                        self.module_import_map
                            .insert(detail.local_name.clone(), dep_module_path);
                    }
                }
            }
        }

        Ok(())
    }

    fn original_name_from_mangled(mangled: &str) -> String {
        mangled.rsplit('$').next().unwrap_or(mangled).to_string()
    }

    // -----------------------------------------------------------------------
    // Phase 1: Collect function signatures
    // -----------------------------------------------------------------------

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

        let returns = ast_getattr!(node, "returns");
        let return_type = if returns.is_none() {
            Type::Unit
        } else {
            Self::convert_type_annotation(&returns)?
        };

        let func_type = Type::Function {
            params: param_types,
            return_type: Box::new(return_type.clone()),
        };

        log::debug!("Registering function '{}' in module", name);
        self.functions.insert(name.clone(), func_type);

        // Also add to call_resolution_map
        let mangled = self.mangle_name(&name);
        self.call_resolution_map.insert(name, mangled);

        Ok(())
    }

    // -----------------------------------------------------------------------
    // Phase 2: Lower functions
    // -----------------------------------------------------------------------

    fn lower_function(&mut self, node: &Bound<PyAny>) -> Result<TirFunction> {
        let name = ast_get_string!(node, "name");
        let mangled_name = self.mangle_name(&name);

        // Extract parameters
        let args_node = ast_getattr!(node, "args");
        let py_args = ast_get_list!(&args_node, "args");
        let mut params = Vec::new();
        for arg in py_args.iter() {
            let param_name = ast_get_string!(arg, "arg");
            let annotation = ast_getattr!(arg, "annotation");
            let ty = Self::convert_type_annotation(&annotation)?;
            params.push(FunctionParam::new(param_name, ty));
        }

        let returns = ast_getattr!(node, "returns");
        let return_type = if returns.is_none() {
            Type::Unit
        } else {
            Self::convert_type_annotation(&returns)?
        };

        // Set up function scope
        self.variables.clear();
        for param in &params {
            self.variables.insert(param.name.clone(), param.ty.clone());
        }
        self.current_return_type = Some(return_type.clone());

        // Lower body
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

        // Clean up scope
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

    // -----------------------------------------------------------------------
    // Statement lowering (from infer.rs + builder.rs)
    // -----------------------------------------------------------------------

    fn lower_stmt(&mut self, node: &Bound<PyAny>) -> Result<TirStmt> {
        let node_type = ast_type_name!(node);
        let line = Self::get_line(node);

        match node_type.as_str() {
            "FunctionDef" => {
                bail!("Nested functions not supported at line {}", line)
            }

            "AnnAssign" => {
                // target : annotation = value
                let target_node = ast_getattr!(node, "target");
                if ast_type_name!(target_node) != "Name" {
                    bail!(
                        "Only simple variable assignments are supported at line {}",
                        line
                    );
                }
                let target = ast_get_string!(target_node, "id");

                let annotation = ast_getattr!(node, "annotation");
                let annotated_ty = if !annotation.is_none() {
                    Some(Self::convert_type_annotation(&annotation)?)
                } else {
                    None
                };

                let value_node = ast_getattr!(node, "value");
                let tir_value = self.lower_expr(&value_node)?;

                // Type check: annotation must match inferred
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
                // target = value (no type annotation)
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
                    // Bare return
                    if let Some(ref expected) = self.current_return_type {
                        if !expected.is_unit() {
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
                let tir_expr = self.lower_expr(&value_node)?;
                Ok(TirStmt::Expr(tir_expr))
            }

            _ => bail!("Unsupported statement type: {} at line {}", node_type, line),
        }
    }

    // -----------------------------------------------------------------------
    // Expression lowering (from infer.rs + convert.rs + builder.rs)
    // -----------------------------------------------------------------------

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

            "Call" => {
                let func_node = ast_getattr!(node, "func");
                let args_list = ast_get_list!(node, "args");

                // Lower arguments
                let mut tir_args = Vec::new();
                for arg in args_list.iter() {
                    tir_args.push(self.lower_expr(&arg)?);
                }

                // Resolve function name + get type info
                let func_node_type = ast_type_name!(func_node);
                match func_node_type.as_str() {
                    "Name" => {
                        let func_name = ast_get_string!(func_node, "id");

                        // Special case: print
                        if func_name == "print" {
                            let resolved = self
                                .call_resolution_map
                                .get(&func_name)
                                .cloned()
                                .unwrap_or_else(|| func_name.clone());
                            return Ok(TirExpr {
                                kind: TirExprKind::Call {
                                    func: resolved,
                                    args: tir_args,
                                },
                                ty: Type::Unit,
                            });
                        }

                        // Look up function type for type checking
                        let func_type =
                            self.functions.get(&func_name).cloned().ok_or_else(|| {
                                anyhow::anyhow!(
                                    "Undefined function: {} at line {}, column {}",
                                    func_name,
                                    line,
                                    col
                                )
                            })?;

                        let return_type = match func_type {
                            Type::Function {
                                ref params,
                                ref return_type,
                            } => {
                                if tir_args.len() != params.len() {
                                    bail!(
                                        "Function '{}' at line {} expects {} arguments, got {}",
                                        func_name,
                                        line,
                                        params.len(),
                                        tir_args.len()
                                    );
                                }
                                for (i, (arg, expected)) in
                                    tir_args.iter().zip(params.iter()).enumerate()
                                {
                                    if &arg.ty != expected {
                                        bail!(
                                            "Argument {} type mismatch in call to '{}' at line {}: expected {:?}, got {:?}",
                                            i,
                                            func_name,
                                            line,
                                            expected,
                                            arg.ty
                                        );
                                    }
                                }
                                *return_type.clone()
                            }
                            _ => bail!("Cannot call non-function type at line {}", line),
                        };

                        let resolved = self
                            .call_resolution_map
                            .get(&func_name)
                            .cloned()
                            .unwrap_or_else(|| func_name.clone());

                        Ok(TirExpr {
                            kind: TirExprKind::Call {
                                func: resolved,
                                args: tir_args,
                            },
                            ty: return_type,
                        })
                    }

                    "Attribute" => {
                        // module.function() style call
                        let value_node = ast_getattr!(func_node, "value");
                        let attr = ast_get_string!(func_node, "attr");

                        if ast_type_name!(value_node) != "Name" {
                            bail!("Complex attribute access not supported at line {}", line);
                        }
                        let mod_name = ast_get_string!(value_node, "id");

                        // Resolve the mangled function name
                        let resolved = if let Some(mod_path) = self.module_import_map.get(&mod_name)
                        {
                            format!("{}${}", mod_path, attr)
                        } else {
                            bail!("Unknown module: {} at line {}", mod_name, line);
                        };

                        // Type check using the attr name (which should be in our functions context)
                        let func_type = self.functions.get(&attr).cloned();

                        let return_type = if let Some(Type::Function {
                            ref params,
                            ref return_type,
                        }) = func_type
                        {
                            if tir_args.len() != params.len() {
                                bail!(
                                    "Function '{}.{}' at line {} expects {} arguments, got {}",
                                    mod_name,
                                    attr,
                                    line,
                                    params.len(),
                                    tir_args.len()
                                );
                            }
                            for (i, (arg, expected)) in
                                tir_args.iter().zip(params.iter()).enumerate()
                            {
                                if &arg.ty != expected {
                                    bail!(
                                        "Argument {} type mismatch in call to '{}.{}' at line {}: expected {:?}, got {:?}",
                                        i,
                                        mod_name,
                                        attr,
                                        line,
                                        expected,
                                        arg.ty
                                    );
                                }
                            }
                            *return_type.clone()
                        } else {
                            // If we can't find the type, treat as unknown
                            // (attribute calls on imported modules may not have their type registered)
                            Type::Unknown
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

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    fn mangle_name(&self, name: &str) -> String {
        if name == "main" {
            format!("{}$$main$", self.module_path)
        } else {
            format!("{}${}", self.module_path, name)
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
