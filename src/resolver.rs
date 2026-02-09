use anyhow::{Context, Result};
use pyo3::prelude::*;
use pyo3::types::PyModule;
use std::path::{Path, PathBuf};

use crate::{ast_extract, ast_get_int, ast_get_list, ast_get_string, ast_getattr, ast_type_name};

pub struct Resolver {
    base_dir: PathBuf,
}

impl Resolver {
    pub fn new(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }

    pub fn resolve_dependencies(&self, file_path: &Path) -> Result<Vec<PathBuf>> {
        assert!(file_path.is_file());

        Python::attach(|py| {
            let source = std::fs::read_to_string(file_path).unwrap();
            let ast_module = PyModule::import(py, "ast")?;
            let ast = ast_module.call_method1("parse", (source.as_str(),))?;
            let file_dir = file_path.parent().unwrap();

            let mut dependencies = Vec::new();
            let body_list = ast_get_list!(&ast, "body");

            for node in body_list.iter() {
                match ast_type_name!(node).as_str() {
                    "Import" => self.handle_import(&node, &mut dependencies)?,
                    "ImportFrom" => self.handle_import_from(&node, file_dir, &mut dependencies)?,
                    _ => {}
                }
            }

            Ok(dependencies)
        })
    }

    fn handle_import(
        &self,
        node: &Bound<'_, PyAny>,
        dependencies: &mut Vec<PathBuf>,
    ) -> Result<()> {
        for alias in ast_get_list!(node, "names").iter() {
            let name = ast_get_string!(alias, "name");
            dependencies.push(self.resolve_absolute_import(&name)?);
        }
        Ok(())
    }

    fn handle_import_from(
        &self,
        node: &Bound<'_, PyAny>,
        file_dir: &Path,
        dependencies: &mut Vec<PathBuf>,
    ) -> Result<()> {
        let level = ast_get_int!(node, "level", usize);
        let module_val = ast_getattr!(node, "module");
        let module_name = (!module_val.is_none()).then(|| ast_extract!(module_val, String));

        if let Some(ref mod_name) = module_name {
            if let Ok(module_file) = self.resolve_module(file_dir, level, mod_name) {
                dependencies.push(module_file);
                return Ok(());
            }
        }

        for alias in ast_get_list!(node, "names").iter() {
            let name = ast_get_string!(alias, "name");
            let module = match &module_name {
                None => name,
                Some(base) => format!("{}.{}", base, name),
            };
            dependencies.push(self.resolve_module(file_dir, level, &module)?);
        }

        Ok(())
    }

    pub fn resolve_module(&self, file_dir: &Path, level: usize, module: &str) -> Result<PathBuf> {
        if level > 0 {
            self.resolve_relative_import(file_dir, level, module)
        } else {
            self.resolve_absolute_import(module)
        }
    }

    fn resolve_absolute_import(&self, import: &str) -> Result<PathBuf> {
        let module_path = import.replace('.', "/");
        let module_file = self.base_dir.join(format!("{}.py", module_path));

        if module_file.exists() && module_file.is_file() {
            Ok(module_file)
        } else {
            anyhow::bail!("Failed to resolve import: {}", import)
        }
    }

    fn resolve_relative_import(
        &self,
        base_dir: &Path,
        level: usize,
        module: &str,
    ) -> Result<PathBuf> {
        let mut current = base_dir;
        for _ in 1..level {
            current = current
                .parent()
                .context("Cannot resolve relative import: not enough parent directories")?;
        }

        let module_path = module.replace('.', "/");
        let module_file = current.join(format!("{}.py", module_path));

        if module_file.exists() && module_file.is_file() {
            module_file
                .canonicalize()
                .context("Failed to canonicalize path")
        } else {
            anyhow::bail!("Module file not found: {}", module)
        }
    }

    pub fn compute_module_path(&self, file_path: &Path) -> String {
        let relative = file_path.strip_prefix(&self.base_dir).unwrap();
        let without_ext = relative.with_extension("");
        without_ext.to_string_lossy().replace('/', ".")
    }

    pub fn mangle_function_name(&self, file_path: &Path, func_name: &str) -> String {
        format!("{}${}", self.compute_module_path(file_path), func_name)
    }

    pub fn mangle_synthetic_main(&self, file_path: &Path) -> String {
        format!("{}$$main$", self.compute_module_path(file_path))
    }
}
