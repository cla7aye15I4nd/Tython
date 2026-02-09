use anyhow::{Context as _, Result};
use inkwell::types::BasicMetadataTypeEnum;

use super::context::CodegenContext;
use super::expr::ExprCodegen;
use crate::ast::Type;
use crate::tir::{TirFunction, TirStmt};

pub struct FunctionCodegen<'a, 'ctx> {
    context: &'a mut CodegenContext<'ctx>,
}

impl<'a, 'ctx> FunctionCodegen<'a, 'ctx> {
    pub fn new(context: &'a mut CodegenContext<'ctx>) -> Self {
        Self { context }
    }

    /// Declare function signature without body.
    /// func.name is already mangled (e.g. "imports.module_a$func_a").
    pub fn declare_function(&mut self, func: &TirFunction) -> Result<()> {
        let param_types: Vec<BasicMetadataTypeEnum> = func
            .params
            .iter()
            .map(|p| self.context.get_llvm_type(&p.ty).into())
            .collect();

        let fn_type = if func.return_type == Type::Int {
            self.context.i64_type().fn_type(&param_types, false)
        } else {
            // void function
            self.context
                .context
                .void_type()
                .fn_type(&param_types, false)
        };

        let function = self.context.module.add_function(&func.name, fn_type, None);
        self.context.register_function(func.name.clone(), function);

        Ok(())
    }

    /// Generate function body (assumes signature already declared)
    pub fn generate_function_body(&mut self, func: &TirFunction) -> Result<()> {
        // Get the function we already declared
        let function = self
            .context
            .get_function(&func.name)
            .ok_or_else(|| anyhow::anyhow!("Function {} not declared", func.name))?;

        // Create entry basic block
        let entry_bb = self.context.context.append_basic_block(function, "entry");
        self.context.builder.position_at_end(entry_bb);

        // Allocate space for parameters
        self.context.clear_variables();
        for (i, param) in func.params.iter().enumerate() {
            let param_value = function.get_nth_param(i as u32).unwrap();
            let alloca = self
                .context
                .builder
                .build_alloca(self.context.get_llvm_type(&param.ty), &param.name)
                .map_err(|e| anyhow::anyhow!("Failed to build alloca: {:?}", e))?;
            self.context
                .builder
                .build_store(alloca, param_value)
                .map_err(|e| anyhow::anyhow!("Failed to build store: {:?}", e))?;
            self.context.register_variable(param.name.clone(), alloca);
        }

        // Generate code for body
        for stmt in &func.body {
            self.codegen_stmt(stmt)
                .with_context(|| format!("In function '{}'", func.name))?;
        }

        // Add implicit return if needed
        if func.return_type == Type::Unit {
            self.context
                .builder
                .build_return(None)
                .map_err(|e| anyhow::anyhow!("Failed to build return: {:?}", e))?;
        }

        Ok(())
    }

    fn codegen_stmt(&mut self, stmt: &TirStmt) -> Result<()> {
        match stmt {
            TirStmt::Let { name, ty, value } => {
                let mut expr_codegen = ExprCodegen::new(self.context);
                let value_llvm = expr_codegen.codegen_expr(value)?;

                // Allocate space for variable
                let alloca = self
                    .context
                    .builder
                    .build_alloca(self.context.get_llvm_type(ty), name)
                    .map_err(|e| anyhow::anyhow!("Failed to build alloca: {:?}", e))?;

                self.context
                    .builder
                    .build_store(alloca, value_llvm)
                    .map_err(|e| anyhow::anyhow!("Failed to build store: {:?}", e))?;
                self.context.register_variable(name.clone(), alloca);
            }

            TirStmt::Return(expr_opt) => {
                if let Some(expr) = expr_opt {
                    let mut expr_codegen = ExprCodegen::new(self.context);
                    let value = expr_codegen.codegen_expr(expr)?;
                    self.context
                        .builder
                        .build_return(Some(&value))
                        .map_err(|e| anyhow::anyhow!("Failed to build return: {:?}", e))?;
                } else {
                    self.context
                        .builder
                        .build_return(None)
                        .map_err(|e| anyhow::anyhow!("Failed to build return: {:?}", e))?;
                }
            }

            TirStmt::Expr(expr) => {
                let mut expr_codegen = ExprCodegen::new(self.context);
                expr_codegen.codegen_expr(expr)?;
            }
        }

        Ok(())
    }
}
