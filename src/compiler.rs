use crate::resolver::Resolver;
use anyhow::Result;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// Compilation metadata for a single module
#[derive(Debug)]
pub struct CompiledModule {
    /// File path of this module
    pub file_path: PathBuf,

    /// Dependency file paths (absolute)
    pub dependencies: Vec<PathBuf>,
}

pub struct Compiler {
    entry_point: PathBuf,

    resolver: Resolver,

    compiled_modules: HashMap<PathBuf, CompiledModule>,
}

impl Compiler {
    pub fn new(path: PathBuf) -> Result<Self> {
        assert!(path.is_file());

        let entry_point = path.canonicalize()?;
        let base_dir = entry_point.parent().unwrap();
        let resolver = Resolver::new(base_dir.to_path_buf());

        Ok(Self {
            entry_point,
            resolver,
            compiled_modules: HashMap::new(),
        })
    }

    pub fn compile(&mut self) -> Result<()> {
        self.compile_module(&self.entry_point.clone(), &mut HashSet::new())?;

        Ok(())
    }

    fn compile_module(
        &mut self,
        canonical_path: &Path,
        in_progress: &mut HashSet<PathBuf>,
    ) -> Result<()> {
        assert!(canonical_path.is_file());

        if self.compiled_modules.contains_key(canonical_path) {
            return Ok(());
        }

        if in_progress.contains(canonical_path) {
            anyhow::bail!("Circular dependency detected: {}", canonical_path.display());
        }

        log::info!("Compiling module: {}", canonical_path.display());
        in_progress.insert(canonical_path.to_path_buf());

        let dependencies = self.resolver.resolve_dependencies(canonical_path)?;

        for dep_path in &dependencies {
            self.compile_module(dep_path, in_progress)?;
        }

        in_progress.remove(canonical_path);

        let compiled = CompiledModule {
            file_path: canonical_path.to_path_buf(),
            dependencies,
        };
        self.compiled_modules
            .insert(canonical_path.to_path_buf(), compiled);

        log::info!("Successfully compiled: {}", canonical_path.display());

        Ok(())
    }

    /// Get a compiled module by path
    pub fn get_module(&self, path: &Path) -> Option<&CompiledModule> {
        path.canonicalize()
            .ok()
            .and_then(|p| self.compiled_modules.get(&p))
    }
}
