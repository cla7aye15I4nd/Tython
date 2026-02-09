use crate::ast::Type;
use anyhow::{bail, Result};
use std::collections::HashMap;

/// Type inference context for a single module
pub struct TypeContext {
    /// Variable name → Type
    variables: HashMap<String, Type>,

    /// Function name → Type
    functions: HashMap<String, Type>,

    /// Current function return type (for checking returns)
    current_return_type: Option<Type>,
}

impl TypeContext {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            functions: HashMap::new(),
            current_return_type: None,
        }
    }

    pub fn define_var(&mut self, name: String, ty: Type) -> Result<()> {
        if self.variables.contains_key(&name) {
            bail!("Variable '{}' already defined", name);
        }
        self.variables.insert(name, ty);
        Ok(())
    }

    pub fn lookup_var(&self, name: &str) -> Result<Type> {
        self.variables
            .get(name)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Undefined variable: {}", name))
    }

    pub fn define_function(&mut self, name: String, ty: Type) -> Result<()> {
        self.functions.insert(name, ty);
        Ok(())
    }

    pub fn lookup_function(&self, name: &str) -> Result<Type> {
        log::debug!(
            "Looking up function '{}' (len={}), have {} functions: {:?}",
            name,
            name.len(),
            self.functions.len(),
            self.functions.keys().collect::<Vec<_>>()
        );
        let result = self.functions.get(name);
        log::debug!("Lookup result: {:?}", result.is_some());
        result
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Undefined function: {}", name))
    }

    pub fn enter_function(&mut self, return_type: Type) {
        self.current_return_type = Some(return_type);
    }

    pub fn exit_function(&mut self) {
        self.current_return_type = None;
    }

    pub fn get_return_type(&self) -> Option<&Type> {
        self.current_return_type.as_ref()
    }

    pub fn get_all_functions(&self) -> HashMap<String, Type> {
        self.functions.clone()
    }
}

impl Default for TypeContext {
    fn default() -> Self {
        Self::new()
    }
}
