use crate::ast::Type;
use std::collections::HashMap;

pub struct SymbolTable {
    symbols: HashMap<String, Type>,
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
        }
    }

    pub fn register_function(&mut self, mangled_name: String, func_type: Type) {
        self.symbols.insert(mangled_name, func_type);
    }

    pub fn get_type(&self, mangled_name: &str) -> Option<&Type> {
        self.symbols.get(mangled_name)
    }
}
