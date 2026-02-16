use crate::ast::Type;
use crate::codegen::Codegen;
use crate::resolver::Resolver;
use crate::tir::lower::Lowering;

use anyhow::{bail, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, PartialEq, Eq)]
enum ModuleColor {
    Gray,
    Black,
}

enum CompileAction {
    Enter(PathBuf),
    Compile(PathBuf, HashMap<String, Type>),
}

pub struct Compiler {
    entry_point: PathBuf,
    resolver: Resolver,
}

impl Compiler {
    pub fn new(input_path: PathBuf) -> Result<Self> {
        let entry_point = input_path.canonicalize()?;
        let base_dir = entry_point.parent().unwrap().to_path_buf();
        let resolver = Resolver::new(base_dir);

        Ok(Self {
            entry_point,
            resolver,
        })
    }

    pub fn check(&mut self) -> Result<()> {
        let mut lowering = Lowering::new();
        self.lower_modules(&mut lowering)?;
        Ok(())
    }

    pub fn compile(&mut self, output_path: PathBuf) -> Result<()> {
        let context = inkwell::context::Context::create();
        let mut codegen = Codegen::new(&context);
        let mut lowering = Lowering::new();

        self.compile_modules(&self.entry_point.clone(), &mut codegen, &mut lowering)?;
        codegen.emit_intrinsic_dispatchers();

        let entry_main_mangled = self.resolver.mangle_synthetic_main(&self.entry_point);

        codegen.create_runtime_entry_point(&entry_main_mangled);

        codegen.link(&output_path);

        Ok(())
    }

    fn compile_modules(
        &mut self,
        entry: &Path,
        codegen: &mut Codegen,
        lowering: &mut Lowering,
    ) -> Result<()> {
        let mut colors: HashMap<PathBuf, ModuleColor> = HashMap::new();
        let mut stack = vec![CompileAction::Enter(entry.to_path_buf())];

        while let Some(action) = stack.pop() {
            match action {
                CompileAction::Enter(path) => {
                    match colors.get(&path) {
                        Some(ModuleColor::Black) => continue,
                        Some(ModuleColor::Gray) => {
                            bail!("circular dependency detected: {}", path.display());
                        }
                        None => {}
                    }

                    colors.insert(path.clone(), ModuleColor::Gray);

                    let resolved = self.resolver.resolve_imports(&path)?;
                    stack.push(CompileAction::Compile(path, resolved.symbols));
                    for dep in resolved.dependencies.into_iter().rev() {
                        stack.push(CompileAction::Enter(dep));
                    }
                }
                CompileAction::Compile(path, imports) => {
                    let module_path = self.resolver.compute_module_path(&path);
                    let tir = lowering.lower_module(&path, &module_path, &imports)?;
                    codegen.register_intrinsic_instances(&tir.intrinsic_instances);

                    for class_info in tir.classes.values() {
                        codegen.register_class(class_info);
                    }

                    // Predeclare all function signatures in this module so
                    // intrinsic kernel emission can resolve method symbols
                    // (e.g. tuple/class __eq__) regardless of generation order.
                    for func in tir.functions.values() {
                        let param_types =
                            func.params.iter().map(|p| p.ty.clone()).collect::<Vec<_>>();
                        codegen.get_or_declare_function(
                            &func.name,
                            &param_types,
                            func.return_type.clone(),
                        );
                    }

                    for func in tir.functions.values() {
                        codegen.generate(func);
                    }

                    colors.insert(path.to_path_buf(), ModuleColor::Black);
                }
            }
        }

        Ok(())
    }

    fn lower_modules(&mut self, lowering: &mut Lowering) -> Result<()> {
        let mut colors: HashMap<PathBuf, ModuleColor> = HashMap::new();
        let mut stack = vec![CompileAction::Enter(self.entry_point.clone())];

        while let Some(action) = stack.pop() {
            match action {
                CompileAction::Enter(path) => {
                    match colors.get(&path) {
                        Some(ModuleColor::Black) => continue,
                        Some(ModuleColor::Gray) => {
                            bail!("circular dependency detected: {}", path.display());
                        }
                        None => {}
                    }

                    colors.insert(path.clone(), ModuleColor::Gray);

                    let resolved = self.resolver.resolve_imports(&path)?;
                    stack.push(CompileAction::Compile(path, resolved.symbols));
                    for dep in resolved.dependencies.into_iter().rev() {
                        stack.push(CompileAction::Enter(dep));
                    }
                }
                CompileAction::Compile(path, imports) => {
                    let module_path = self.resolver.compute_module_path(&path);
                    lowering.lower_module(&path, &module_path, &imports)?;
                    colors.insert(path.to_path_buf(), ModuleColor::Black);
                }
            }
        }

        Ok(())
    }
}
