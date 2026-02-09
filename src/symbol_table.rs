use crate::ast::Type;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Information about a single function visible globally.
#[derive(Debug, Clone)]
pub struct FunctionSymbol {
    /// The mangled LLVM name, e.g. "imports.module_a$func_a"
    pub mangled_name: String,
    /// The original unmangled name, e.g. "func_a"
    pub original_name: String,
    /// The canonical file path this function belongs to
    pub module_path: PathBuf,
    /// Parameter types
    pub param_types: Vec<Type>,
    /// Return type
    pub return_type: Type,
}

/// Global symbol table: tracks all functions across all compiled modules.
#[derive(Debug)]
pub struct SymbolTable {
    /// mangled_name -> FunctionSymbol
    symbols: HashMap<String, FunctionSymbol>,
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

    pub fn register_function(&mut self, symbol: FunctionSymbol) {
        let mangled = symbol.mangled_name.clone();
        let module = symbol.module_path.clone();
        self.symbols.insert(mangled.clone(), symbol);
        self.modules.entry(module).or_default().push(mangled);
    }

    pub fn get_symbol(&self, mangled_name: &str) -> Option<&FunctionSymbol> {
        self.symbols.get(mangled_name)
    }

    pub fn get_functions_for_module(&self, module_path: &Path) -> Option<&[String]> {
        self.modules.get(module_path).map(|v| v.as_slice())
    }

    /// Look up a function by its module file path and original (unmangled) name.
    pub fn find_function_in_module(
        &self,
        module_path: &Path,
        original_name: &str,
    ) -> Option<&FunctionSymbol> {
        let mangled_names = self.modules.get(module_path)?;
        for mangled in mangled_names {
            if let Some(sym) = self.symbols.get(mangled) {
                if sym.original_name == original_name {
                    return Some(sym);
                }
            }
        }
        None
    }
}
