use anyhow::{Context, Result};
use pyo3::prelude::*;
use pyo3::types::PyModule;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::ast::Type;
use crate::{
    ast_extract, ast_get_int, ast_get_list, ast_get_string, ast_get_string_or, ast_getattr,
    ast_type_name,
};

pub struct ResolvedImports {
    pub dependencies: Vec<PathBuf>,
    pub symbols: HashMap<String, Type>,
}

pub struct Resolver {
    base_dir: PathBuf,
    stdlib_dir: PathBuf,
}

impl Resolver {
    pub fn new(base_dir: PathBuf) -> Self {
        let stdlib_dir = PathBuf::from(env!("TYTHON_STDLIB_DIR"));
        Self {
            base_dir,
            stdlib_dir,
        }
    }

    pub fn resolve_imports(&self, file_path: &Path) -> Result<ResolvedImports> {
        Python::attach(|py| {
            let source = std::fs::read_to_string(file_path).unwrap();
            let ast_module = PyModule::import(py, "ast").unwrap();
            let ast = ast_module
                .call_method1("parse", (source.as_str(),))
                .unwrap();
            let file_dir = file_path.parent().unwrap();

            let mut dependencies = Vec::new();
            let mut symbols = HashMap::new();
            let body_list = ast_get_list!(&ast, "body");

            for node in body_list.iter() {
                match ast_type_name!(node).as_str() {
                    "Import" => self.handle_import(&node, &mut dependencies, &mut symbols)?,
                    "ImportFrom" => {
                        self.handle_import_from(&node, file_dir, &mut dependencies, &mut symbols)?
                    }
                    _ => {}
                }
            }

            Ok(ResolvedImports {
                dependencies,
                symbols,
            })
        })
    }

    fn handle_import(
        &self,
        node: &Bound<'_, PyAny>,
        dependencies: &mut Vec<PathBuf>,
        symbols: &mut HashMap<String, Type>,
    ) -> Result<()> {
        for alias in ast_get_list!(node, "names").iter() {
            let name = ast_get_string!(alias, "name");
            let path = self.resolve_absolute_import(&name)?;
            let mod_path = self.compute_module_path(&path);

            let local_name = ast_get_string_or!(alias, "asname", name);

            symbols.insert(local_name, Type::Module(mod_path));
            dependencies.push(path);
        }
        Ok(())
    }

    fn handle_import_from(
        &self,
        node: &Bound<'_, PyAny>,
        file_dir: &Path,
        dependencies: &mut Vec<PathBuf>,
        symbols: &mut HashMap<String, Type>,
    ) -> Result<()> {
        let level = ast_get_int!(node, "level", usize);
        let module_val = ast_getattr!(node, "module");
        let module_name = (!module_val.is_none()).then(|| ast_extract!(module_val, String));

        if let Some(ref mod_name) = module_name {
            if let Ok(module_file) = self.resolve_module(file_dir, level, mod_name) {
                let mod_path = self.compute_module_path(&module_file);
                dependencies.push(module_file);

                for alias in ast_get_list!(node, "names").iter() {
                    let name = ast_get_string!(alias, "name");
                    let local_name = ast_get_string_or!(alias, "asname", name.clone());
                    let mangled = format!("{}${}", mod_path, name);
                    symbols.insert(local_name, Type::Module(mangled));
                }

                return Ok(());
            }
        }

        for alias in ast_get_list!(node, "names").iter() {
            let name = ast_get_string!(alias, "name");
            let module = match &module_name {
                None => name.clone(),
                Some(base) => format!("{}.{}", base, name),
            };
            let path = self.resolve_module(file_dir, level, &module)?;
            let mod_path = self.compute_module_path(&path);

            let local_name = ast_get_string_or!(alias, "asname", name);

            symbols.insert(local_name, Type::Module(mod_path));
            dependencies.push(path);
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

    /// Convert a dotted module name to a file path relative to a directory.
    fn module_to_file_path(dir: &Path, module: &str) -> PathBuf {
        dir.join(format!("{}.py", module.replace('.', "/")))
    }

    fn resolve_absolute_import(&self, import: &str) -> Result<PathBuf> {
        // 1. Local project directory
        let local_file = Self::module_to_file_path(&self.base_dir, import);
        if local_file.exists() && local_file.is_file() {
            return Ok(local_file);
        }

        // 2. stdlib/ directory
        let stdlib_file = Self::module_to_file_path(&self.stdlib_dir, import);
        if stdlib_file.exists() && stdlib_file.is_file() {
            return Ok(stdlib_file);
        }

        anyhow::bail!("failed to resolve import `{}`", import)
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

        let module_file = Self::module_to_file_path(current, module);
        if module_file.exists() && module_file.is_file() {
            module_file
                .canonicalize()
                .context("Failed to canonicalize path")
        } else {
            anyhow::bail!("module file not found: {}", module)
        }
    }

    pub fn compute_module_path(&self, file_path: &Path) -> String {
        if let Some(s) = file_path.to_str() {
            if let Some(stripped) = s.strip_prefix("__native__/") {
                return stripped.strip_suffix(".py").unwrap_or(stripped).to_string();
            }
        }
        let relative = file_path
            .strip_prefix(&self.stdlib_dir)
            .or_else(|_| file_path.strip_prefix(&self.base_dir))
            .unwrap();
        let without_ext = relative.with_extension("");
        without_ext.to_string_lossy().replace('/', ".")
    }

    pub fn mangle_synthetic_main(&self, file_path: &Path) -> String {
        format!("{}$$main$", self.compute_module_path(file_path))
    }
}
