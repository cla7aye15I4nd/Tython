use crate::codegen::Codegen;
use crate::resolver::Resolver;
use crate::symbol_table::SymbolTable;
use crate::tir::lower::LoweringContext;

use anyhow::{bail, Context, Result};
use pyo3::prelude::*;
use pyo3::types::PyModule as PyPyModule;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::process::Command;

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

        self.compile_module(
            &self.entry_point.clone(),
            &mut codegen,
            &mut HashSet::new(),
            &mut HashSet::new(),
        )?;

        let entry_main_mangled = self.resolver.mangle_synthetic_main(&self.entry_point);

        codegen.add_c_main_wrapper(&entry_main_mangled)?;
        assert!(codegen.verify());

        let bc_path = self.entry_point.with_extension("bc");
        let ll_path = self.entry_point.with_extension("ll");
        codegen.emit_ir(&ll_path);
        codegen.emit_bitcode(&bc_path);

        Self::link_with_clang(&bc_path, &output_path)?;

        Ok(())
    }

    fn compile_module(
        &mut self,
        canonical_path: &Path,
        codegen: &mut Codegen,
        in_progress: &mut HashSet<PathBuf>,
        compiled: &mut HashSet<PathBuf>,
    ) -> Result<()> {
        assert!(canonical_path.is_file());

        if compiled.contains(canonical_path) {
            return Ok(());
        }

        if in_progress.contains(canonical_path) {
            bail!("Circular dependency detected: {}", canonical_path.display());
        }

        log::info!("Compiling module: {}", canonical_path.display());
        in_progress.insert(canonical_path.to_path_buf());

        let dependencies = self.resolver.resolve_dependencies(canonical_path)?;

        for dep_path in &dependencies {
            self.compile_module(dep_path, codegen, in_progress, compiled)?;
        }

        in_progress.remove(canonical_path);

        // Single-pass lowering: Python AST → TIR
        let module_path = self.resolver.compute_module_path(canonical_path);
        let tir = Python::attach(|py| -> Result<_> {
            let source = std::fs::read_to_string(canonical_path)?;
            let ast_module = PyPyModule::import(py, "ast")?;
            let py_ast = ast_module.call_method1("parse", (source.as_str(),))?;
            LoweringContext::lower_module(
                &py_ast,
                canonical_path,
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
        self.module_exports
            .insert(canonical_path.to_path_buf(), export_names);

        for func in tir.functions.values() {
            codegen.generate(func)?;
        }

        compiled.insert(canonical_path.to_path_buf());
        log::info!("Successfully compiled: {}", canonical_path.display());

        Ok(())
    }

    fn link_with_clang(bc_path: &Path, output_path: &Path) -> Result<()> {
        let output = Command::new("clang")
            .arg("-O2")
            .arg("-o")
            .arg(output_path)
            .arg(bc_path)
            .output()
            .context("Failed to run clang")?;

        assert!(output.status.success());

        Ok(())
    }
}
