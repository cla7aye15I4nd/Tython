use crate::codegen::context::CodegenContext;
use crate::codegen::function::FunctionCodegen;
use crate::resolver::Resolver;
use crate::symbol_table::SymbolTable;
use crate::tir::lower::LoweringContext;

use anyhow::{bail, Context, Result};
use inkwell::context::Context as LlvmContext;
use pyo3::prelude::*;
use pyo3::types::PyModule as PyPyModule;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct Compiler {
    entry_point: PathBuf,
    resolver: Resolver,
    symbol_table: SymbolTable,
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
        })
    }

    pub fn compile(&mut self, output_path: PathBuf) -> Result<()> {
        let llvm_context = LlvmContext::create();
        let mut codegen_ctx = CodegenContext::new(&llvm_context, "__main__");

        self.compile_module(
            &self.entry_point.clone(),
            &mut HashSet::new(),
            &mut HashSet::new(),
            &mut codegen_ctx,
        )?;

        let entry_main_mangled = self.resolver.mangle_synthetic_main(&self.entry_point);

        assert!(codegen_ctx
            .module
            .get_function(&entry_main_mangled)
            .is_some());

        crate::codegen::add_c_main_wrapper(&mut codegen_ctx, &entry_main_mangled)?;

        assert!(codegen_ctx.module.verify().is_ok());

        let bc_path = self.entry_point.with_extension("bc");
        let ll_path = self.entry_point.with_extension("ll");
        let _ = codegen_ctx.module.print_to_file(&ll_path);
        codegen_ctx.module.write_bitcode_to_path(&bc_path);

        Self::link_with_clang(&bc_path, &output_path)?;

        Ok(())
    }

    fn compile_module(
        &mut self,
        canonical_path: &Path,
        in_progress: &mut HashSet<PathBuf>,
        compiled: &mut HashSet<PathBuf>,
        codegen_ctx: &mut CodegenContext,
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
            self.compile_module(dep_path, in_progress, compiled, codegen_ctx)?;
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
                &self.resolver,
            )
        })?;

        // Register functions in symbol table
        for func in tir.functions.values() {
            let func_type = crate::ast::Type::Function {
                params: func.params.iter().map(|p| p.ty.clone()).collect(),
                return_type: Box::new(func.return_type.clone()),
            };
            self.symbol_table
                .register_function(func.name.clone(), canonical_path, func_type);
        }

        // Declare all signatures before generating bodies (forward references)
        for func in tir.functions.values() {
            let mut func_codegen = FunctionCodegen::new(codegen_ctx);
            func_codegen.declare_function(func)?;
        }
        for func in tir.functions.values() {
            let mut func_codegen = FunctionCodegen::new(codegen_ctx);
            func_codegen.generate_function_body(func)?;
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
