use crate::ast::Type;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Global symbol table: tracks all functions across all compiled modules.
#[derive(Debug)]
pub struct SymbolTable {
    /// mangled_name -> Type::Function
    symbols: HashMap<String, Type>,
    /// module file path -> list of mangled function names in that module
    modules: HashMap<PathBuf, Vec<String>>,
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            symbols: HashMap::new(),
            modules: HashMap::new(),
        }
    }

    pub fn register_function(&mut self, mangled_name: String, module_path: &Path, func_type: Type) {
        self.modules
            .entry(module_path.to_path_buf())
            .or_default()
            .push(mangled_name.clone());
        self.symbols.insert(mangled_name, func_type);
    }

    pub fn get_type(&self, mangled_name: &str) -> Option<&Type> {
        self.symbols.get(mangled_name)
    }

    pub fn get_functions_for_module(&self, module_path: &Path) -> Option<&[String]> {
        self.modules.get(module_path).map(|v| v.as_slice())
    }
}
