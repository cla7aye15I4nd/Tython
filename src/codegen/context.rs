use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::IntType;
use inkwell::values::{FunctionValue, PointerValue};
use std::collections::HashMap;

use crate::ast::Type;

pub struct CodegenContext<'ctx> {
    pub context: &'ctx Context,
    pub module: Module<'ctx>,
    pub builder: Builder<'ctx>,

    /// Variable name → LLVM value (alloca pointer)
    variables: HashMap<String, PointerValue<'ctx>>,

    /// Function name → LLVM function
    functions: HashMap<String, FunctionValue<'ctx>>,
}

impl<'ctx> CodegenContext<'ctx> {
    pub fn new(context: &'ctx Context, module_name: &str) -> Self {
        let module = context.create_module(module_name);
        let builder = context.create_builder();

        Self {
            context,
            module,
            builder,
            variables: HashMap::new(),
            functions: HashMap::new(),
        }
    }

    /// Get LLVM type from Tython type
    pub fn get_llvm_type(&self, ty: &Type) -> inkwell::types::BasicTypeEnum<'ctx> {
        match ty {
            Type::Int => self.context.i64_type().into(),
            _ => panic!("Unsupported type for LLVM conversion: {:?}", ty),
        }
    }

    pub fn i64_type(&self) -> IntType<'ctx> {
        self.context.i64_type()
    }

    pub fn register_variable(&mut self, name: String, ptr: PointerValue<'ctx>) {
        self.variables.insert(name, ptr);
    }

    pub fn get_variable(&self, name: &str) -> Option<PointerValue<'ctx>> {
        self.variables.get(name).copied()
    }

    pub fn register_function(&mut self, name: String, func: FunctionValue<'ctx>) {
        self.functions.insert(name, func);
    }

    pub fn get_function(&self, name: &str) -> Option<FunctionValue<'ctx>> {
        self.functions.get(name).copied()
    }

    pub fn clear_variables(&mut self) {
        self.variables.clear();
    }
}
