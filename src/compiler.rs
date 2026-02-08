use crate::resolver::Resolver;
use anyhow::{Context, Result};
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

    in_progress: HashSet<PathBuf>,
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
            in_progress: HashSet::new(),
        })
    }

    pub fn compile(&mut self) -> Result<()> {
        let entry_point = self.entry_point.clone();
        self.compile_module(&entry_point)?;

        Ok(())
    }

    fn compile_module(&mut self, canonical_path: &Path) -> Result<()> {
        assert!(canonical_path.is_file());

        if self.compiled_modules.contains_key(canonical_path) {
            return Ok(());
        }

        if self.in_progress.contains(canonical_path) {
            return Ok(());
        }

        log::info!("Compiling module: {}", canonical_path.display());
        self.in_progress.insert(canonical_path.to_path_buf());

        // Step 1: Resolve all dependencies for this file
        log::debug!("  [1/2] Resolving dependencies...");
        let base_dir = canonical_path
            .parent()
            .context("Failed to get parent directory")?;

        let dependencies = self.resolver.resolve_dependencies(canonical_path)?;
        log::debug!("  Found {} dependencies", dependencies.len());

        // Step 2: Recursively compile all dependencies (depth-first)
        log::debug!("  [2/2] Compiling dependencies...");
        for dep_path in &dependencies {
            log::debug!("    Compiling dependency: {}", dep_path.display());
            self.compile_module(dep_path)?;
        }

        // Store metadata for dependency tracking
        let compiled = CompiledModule {
            file_path: canonical_path.to_path_buf(),
            dependencies,
        };
        self.compiled_modules
            .insert(canonical_path.to_path_buf(), compiled);

        // Remove from in-progress
        self.in_progress.remove(canonical_path);

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
