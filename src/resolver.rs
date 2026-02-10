use anyhow::{Context, Result};
use pyo3::prelude::*;
use pyo3::types::PyModule;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::ast::Type;
use crate::{ast_extract, ast_get_int, ast_get_list, ast_get_string, ast_getattr, ast_type_name};

pub struct ResolvedImports {
    pub dependencies: Vec<PathBuf>,
    pub symbols: HashMap<String, Type>,
}

pub struct Resolver {
    base_dir: PathBuf,
}

impl Resolver {
    pub fn new(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }

    pub fn resolve_imports(&self, file_path: &Path) -> Result<ResolvedImports> {
        Python::attach(|py| {
            let source = std::fs::read_to_string(file_path).unwrap();
            let ast_module = PyModule::import(py, "ast")?;
            let ast = ast_module.call_method1("parse", (source.as_str(),))?;
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

            let asname_node = ast_getattr!(alias, "asname");
            let local_name = if asname_node.is_none() {
                name
            } else {
                ast_extract!(asname_node, String)
            };

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
                    let asname_node = ast_getattr!(alias, "asname");
                    let local_name = if asname_node.is_none() {
                        name.clone()
                    } else {
                        ast_extract!(asname_node, String)
                    };
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

            let asname_node = ast_getattr!(alias, "asname");
            let local_name = if asname_node.is_none() {
                name
            } else {
                ast_extract!(asname_node, String)
            };

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

    pub fn mangle_synthetic_main(&self, file_path: &Path) -> String {
        format!("{}$$main$", self.compute_module_path(file_path))
    }
}
