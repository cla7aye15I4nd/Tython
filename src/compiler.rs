use crate::ast::Type;
use crate::codegen::Codegen;
use crate::resolver::Resolver;
use crate::tir::lower::Lowering;

use anyhow::{bail, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, PartialEq, Eq)]
enum ModuleColor {
    Gray,  // in progress (dependencies being compiled)
    Black, // fully compiled
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
        assert!(input_path.is_file());

        let entry_point = input_path.canonicalize()?;
        let base_dir = entry_point.parent().unwrap().to_path_buf();
        let resolver = Resolver::new(base_dir);

        Ok(Self {
            entry_point,
            resolver,
        })
    }

    pub fn compile(&mut self, output_path: PathBuf) -> Result<()> {
        let context = inkwell::context::Context::create();
        let mut codegen = Codegen::new(&context);
        let mut lowering = Lowering::new();

        self.compile_modules(&self.entry_point.clone(), &mut codegen, &mut lowering)?;

        let entry_main_mangled = self.resolver.mangle_synthetic_main(&self.entry_point);

        codegen.add_c_main_wrapper(&entry_main_mangled);

        codegen.link(&output_path)?;

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

                    let resolved = self.resolver.resolve_imports(&path)?;
                    stack.push(CompileAction::Compile(path, resolved.symbols));
                    for dep in resolved.dependencies.into_iter().rev() {
                        stack.push(CompileAction::Enter(dep));
                    }
                }
                CompileAction::Compile(path, imports) => {
                    let module_path = self.resolver.compute_module_path(&path);
                    let tir = lowering.lower_module(&path, &module_path, &imports)?;

                    for func in tir.functions.values() {
                        codegen.generate(func);
                    }

                    colors.insert(path.to_path_buf(), ModuleColor::Black);
                    log::info!("Successfully compiled: {}", path.display());
                }
            }
        }

        Ok(())
    }
}
