use crate::codegen::Codegen;
use crate::resolver::Resolver;
use crate::symbol_table::SymbolTable;
use crate::tir::lower::LoweringContext;

use anyhow::{bail, Result};
use pyo3::prelude::*;
use pyo3::types::PyModule;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, PartialEq, Eq)]
enum ModuleColor {
    Gray,  // in progress (dependencies being compiled)
    Black, // fully compiled
}

enum CompileAction {
    Enter(PathBuf),
    Compile(PathBuf, Vec<PathBuf>),
}

pub struct Compiler {
    entry_point: PathBuf,
    resolver: Resolver,
    symbol_table: SymbolTable,
    module_exports: HashMap<PathBuf, Vec<String>>,
}

impl Compiler {
    pub fn new(input_path: PathBuf) -> Result<Self> {
        assert!(input_path.is_file());

        let entry_point = input_path.canonicalize()?;
        let base_dir = entry_point.parent().unwrap().to_path_buf();
        let resolver = Resolver::new(base_dir);

        Ok(Self {
            entry_point,
            resolver,
            symbol_table: SymbolTable::new(),
            module_exports: HashMap::new(),
        })
    }

    pub fn compile(&mut self, output_path: PathBuf) -> Result<()> {
        let context = inkwell::context::Context::create();
        let mut codegen = Codegen::new(&context);

        self.compile_modules(&self.entry_point.clone(), &mut codegen)?;

        let entry_main_mangled = self.resolver.mangle_synthetic_main(&self.entry_point);

        codegen.add_c_main_wrapper(&entry_main_mangled)?;

        codegen.link(&output_path)?;

        Ok(())
    }

    fn compile_modules(&mut self, entry: &Path, codegen: &mut Codegen) -> Result<()> {
        let mut colors: HashMap<PathBuf, ModuleColor> = HashMap::new();
        let mut stack = vec![CompileAction::Enter(entry.to_path_buf())];

        while let Some(action) = stack.pop() {
            match action {
                CompileAction::Enter(path) => {
                    assert!(path.is_file());

                    match colors.get(&path) {
                        Some(ModuleColor::Black) => continue,
                        Some(ModuleColor::Gray) => {
                            bail!("Circular dependency detected: {}", path.display());
                        }
                        None => {}
                    }

                    log::info!("Compiling module: {}", path.display());
                    colors.insert(path.clone(), ModuleColor::Gray);

                    let dependencies = self.resolver.resolve_dependencies(&path)?;

                    // Push compile action first (processed after all deps)
                    stack.push(CompileAction::Compile(path, dependencies.clone()));

                    // Push dependencies in reverse so they're processed left-to-right
                    for dep in dependencies.into_iter().rev() {
                        stack.push(CompileAction::Enter(dep));
                    }
                }
                CompileAction::Compile(path, dependencies) => {
                    // Single-pass lowering: Python AST → TIR
                    let module_path = self.resolver.compute_module_path(&path);
                    let tir = Python::attach(|py| -> Result<_> {
                        let source = std::fs::read_to_string(&path)?;
                        let ast_module = PyModule::import(py, "ast")?;
                        let py_ast = ast_module.call_method1("parse", (source.as_str(),))?;
                        LoweringContext::lower_module(
                            &py_ast,
                            &path,
                            &module_path,
                            &dependencies,
                            &self.symbol_table,
                            &self.module_exports,
                            &self.resolver,
                        )
                    })?;

                    // Register functions in symbol table and track module exports
                    let mut export_names = Vec::new();
                    for func in tir.functions.values() {
                        let func_type = crate::ast::Type::Function {
                            params: func.params.iter().map(|p| p.ty.clone()).collect(),
                            return_type: Box::new(func.return_type.clone()),
                        };
                        self.symbol_table
                            .register_function(func.name.clone(), func_type);
                        export_names.push(func.name.clone());
                    }
                    self.module_exports.insert(path.to_path_buf(), export_names);

                    for func in tir.functions.values() {
                        codegen.generate(func)?;
                    }

                    colors.insert(path.to_path_buf(), ModuleColor::Black);
                    log::info!("Successfully compiled: {}", path.display());
                }
            }
        }

        Ok(())
    }
}
