use crate::ast::convert::{AstConverter, ImportDetail, ImportKind};
use crate::codegen::context::CodegenContext;
use crate::codegen::function::FunctionCodegen;
use crate::resolver::Resolver;
use crate::symbol_table::{FunctionSymbol, SymbolTable};
use crate::tir::builder::TirBuilder;
use crate::typeinfer::TypeInferencer;

use anyhow::{bail, Context, Result};
use inkwell::context::Context as LlvmContext;
use pyo3::prelude::*;
use pyo3::types::PyModule as PyPyModule;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct Compiler {
    entry_point: PathBuf,
    resolver: Resolver,
    symbol_table: SymbolTable,
    output_path: Option<PathBuf>,
}

impl Compiler {
    pub fn new(input_path: PathBuf, output_path: Option<PathBuf>) -> Result<Self> {
        assert!(input_path.is_file());

        let entry_point = input_path.canonicalize()?;
        let base_dir = entry_point.parent().unwrap().to_path_buf();
        let resolver = Resolver::new(base_dir);

        Ok(Self {
            entry_point,
            resolver,
            symbol_table: SymbolTable::new(),
            output_path,
        })
    }

    pub fn compile(&mut self) -> Result<PathBuf> {
        let llvm_context = LlvmContext::create();
        let mut codegen_ctx = CodegenContext::new(&llvm_context, "__main__");

        self.compile_module(
            &self.entry_point.clone(),
            &mut HashSet::new(),
            &mut HashSet::new(),
            &mut codegen_ctx,
        )?;

        let entry_main_mangled = self.resolver.mangle_synthetic_main(&self.entry_point);
        if self.symbol_table.get_symbol(&entry_main_mangled).is_none() {
            bail!(
                "Entry module must contain module-level code or a main() function (expected {})",
                entry_main_mangled
            );
        }

        crate::codegen::add_c_main_wrapper(&mut codegen_ctx, &entry_main_mangled)?;

        assert!(codegen_ctx.module.verify().is_ok());

        let bc_path = self.entry_point.with_extension("bc");
        let ll_path = self.entry_point.with_extension("ll");
        let _ = codegen_ctx.module.print_to_file(&ll_path);
        codegen_ctx.module.write_bitcode_to_path(&bc_path);

        let output_path = self
            .output_path
            .clone()
            .unwrap_or_else(|| self.entry_point.with_extension(""));

        Self::link_with_clang(&bc_path, &output_path)?;

        Ok(output_path)
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

        // Resolve dependencies
        let dependencies = self.resolver.resolve_dependencies(canonical_path)?;

        // Recursively compile dependencies first
        for dep_path in &dependencies {
            self.compile_module(dep_path, in_progress, compiled, codegen_ctx)?;
        }

        in_progress.remove(canonical_path);

        // Parse Python AST
        let py_ast = Python::attach(|py| -> Result<_> {
            let source = std::fs::read_to_string(canonical_path)?;
            let ast_module = PyPyModule::import(py, "ast")?;
            let ast = ast_module.call_method1("parse", (source.as_str(),))?;
            Ok(ast.unbind())
        })?;

        // Convert to Rust AST
        let mut rust_ast = Python::attach(|py| {
            AstConverter::convert_module(py_ast.bind(py), canonical_path.to_path_buf())
        })?;

        // Extract detailed import info
        let import_details =
            Python::attach(|py| AstConverter::extract_import_info(py_ast.bind(py)))?;
        log::debug!(
            "Extracted {} imports: {:?}",
            import_details.len(),
            import_details
        );

        // Ensure module has a main() function (create synthetic one if needed)
        rust_ast.ensure_main();

        // Type inference with cross-module function definitions from symbol table
        let mut type_inferencer = TypeInferencer::new();
        self.register_dependency_types(&dependencies, &import_details, &mut type_inferencer)?;

        type_inferencer.infer_module(&mut rust_ast)?;

        // Build resolution maps for name mangling
        let module_path = self.resolver.compute_module_path(canonical_path);
        let (call_resolution_map, module_import_map) =
            self.build_resolution_maps(canonical_path, &rust_ast, &import_details)?;

        log::debug!("Module path: {}", module_path);
        log::debug!("Call resolution map: {:?}", call_resolution_map);
        log::debug!("Module import map: {:?}", module_import_map);

        // Build TIR with mangled names
        let tir = TirBuilder::build_module(
            rust_ast,
            &module_path,
            call_resolution_map,
            module_import_map,
        )?;

        // Register all functions in the global symbol table
        for func in tir.functions.values() {
            // Extract original name from mangled: "mod.path$name" or "mod.path$$main$"
            let original_name = if func.name.ends_with("$$main$") {
                "main".to_string()
            } else {
                func.name
                    .rsplit_once('$')
                    .map(|(_, name)| name.to_string())
                    .unwrap_or_else(|| func.name.clone())
            };

            self.symbol_table.register_function(FunctionSymbol {
                mangled_name: func.name.clone(),
                original_name,
                module_path: canonical_path.to_path_buf(),
                param_types: func.params.iter().map(|p| p.ty.clone()).collect(),
                return_type: func.return_type.clone(),
            });
        }

        // Emit to global LLVM module: declare all signatures, then generate bodies
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

    /// Register dependency function types in the type inferencer using the global symbol table.
    /// Type inference uses source-level (unmangled) names.
    fn register_dependency_types(
        &self,
        dependencies: &[PathBuf],
        import_details: &[ImportDetail],
        type_inferencer: &mut TypeInferencer,
    ) -> Result<()> {
        // Register all functions from all dependencies by original name
        for dep_path in dependencies {
            if let Some(mangled_names) = self.symbol_table.get_functions_for_module(dep_path) {
                for mangled_name in mangled_names {
                    let sym = self.symbol_table.get_symbol(mangled_name).unwrap();
                    // Skip synthetic main functions for type registration
                    if sym.original_name == "main" && mangled_name.contains("$$main$") {
                        continue;
                    }
                    let func_type = crate::ast::Type::Function {
                        params: sym.param_types.clone(),
                        return_type: Box::new(sym.return_type.clone()),
                    };
                    type_inferencer.add_function(sym.original_name.clone(), func_type)?;
                }
            }
        }

        // Register function aliases (where local_name != original_name)
        for detail in import_details {
            if detail.local_name != detail.original_name {
                // Find the original function in any dependency
                for dep_path in dependencies {
                    if let Some(sym) = self
                        .symbol_table
                        .find_function_in_module(dep_path, &detail.original_name)
                    {
                        let func_type = crate::ast::Type::Function {
                            params: sym.param_types.clone(),
                            return_type: Box::new(sym.return_type.clone()),
                        };
                        type_inferencer.add_function(detail.local_name.clone(), func_type)?;
                        break;
                    }
                }
            }
        }

        Ok(())
    }

    /// Build the two resolution maps for TirBuilder:
    /// - call_resolution_map: direct call name -> mangled LLVM name
    /// - module_import_map: module alias -> dotted module path
    fn build_resolution_maps(
        &self,
        canonical_path: &Path,
        rust_ast: &crate::ast::Module,
        import_details: &[ImportDetail],
    ) -> Result<(HashMap<String, String>, HashMap<String, String>)> {
        let mut call_resolution_map = HashMap::new();
        let mut module_import_map = HashMap::new();

        let file_dir = canonical_path.parent().unwrap();

        // Register all functions defined in the current module
        for stmt in &rust_ast.body {
            if let crate::ast::StmtKind::FunctionDef { name, .. } = &stmt.kind {
                let mangled = self.resolver.mangle_function_name(canonical_path, name);
                call_resolution_map.insert(name.clone(), mangled);
            }
        }

        // Process import details
        for detail in import_details {
            match detail.kind {
                ImportKind::Module => {
                    // `from . import module_a` or `from . import module_a as mod_a`
                    let dep_path = self.resolver.resolve_module(
                        file_dir,
                        detail.level,
                        &detail.original_name,
                    )?;
                    let dep_module_path = self.resolver.compute_module_path(&dep_path);
                    module_import_map.insert(detail.local_name.clone(), dep_module_path);
                }
                ImportKind::Function => {
                    // `from .module_a import func_a` or `from .X.Y import func`
                    let source = detail.source_module.as_ref().unwrap();

                    // Try to resolve source_module as a file first
                    let source_file = self.resolver.resolve_module(file_dir, detail.level, source);

                    if let Ok(source_path) = source_file {
                        // source_module is a file -> imported name is a function
                        let mangled = self
                            .resolver
                            .mangle_function_name(&source_path, &detail.original_name);
                        call_resolution_map.insert(detail.local_name.clone(), mangled);
                    } else {
                        // source_module is a package -> imported name is a submodule
                        let full_module = format!("{}.{}", source, detail.original_name);
                        let dep_path =
                            self.resolver
                                .resolve_module(file_dir, detail.level, &full_module)?;
                        let dep_module_path = self.resolver.compute_module_path(&dep_path);
                        module_import_map.insert(detail.local_name.clone(), dep_module_path);
                    }
                }
            }
        }

        Ok((call_resolution_map, module_import_map))
    }

    fn link_with_clang(bc_path: &Path, output_path: &Path) -> Result<()> {
        eprintln!(
            "Compiling {} to {}",
            bc_path.display(),
            output_path.display()
        );

        let output = Command::new("clang")
            .arg("-O2")
            .arg("-o")
            .arg(output_path)
            .arg(bc_path)
            .output()
            .context("Failed to run clang")?;

        if !output.status.success() {
            bail!(
                "Clang compilation failed:\nstdout: {}\nstderr: {}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }

        eprintln!("Generated executable: {}", output_path.display());
        Ok(())
    }
}
