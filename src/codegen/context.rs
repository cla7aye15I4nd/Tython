use anyhow::{Context as _, Result};
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::{BasicMetadataTypeEnum, IntType};
use inkwell::values::{BasicValueEnum, FunctionValue, PointerValue, ValueKind};
use std::collections::HashMap;

use crate::ast::Type;
use crate::tir::{BinOpKind, TirExpr, TirExprKind, TirFunction, TirStmt};

pub struct Codegen<'ctx> {
    context: &'ctx Context,
    pub module: Module<'ctx>,
    builder: Builder<'ctx>,
    variables: HashMap<String, PointerValue<'ctx>>,
    functions: HashMap<String, FunctionValue<'ctx>>,
}

impl<'ctx> Codegen<'ctx> {
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

    // ── Type helpers ─────────────────────────────────────────────

    fn get_llvm_type(&self, ty: &Type) -> inkwell::types::BasicTypeEnum<'ctx> {
        match ty {
            Type::Int => self.context.i64_type().into(),
            _ => panic!("Unsupported type for LLVM conversion: {:?}", ty),
        }
    }

    fn i64_type(&self) -> IntType<'ctx> {
        self.context.i64_type()
    }

    // ── Function codegen ─────────────────────────────────────────

    pub fn declare_function(&mut self, func: &TirFunction) -> Result<()> {
        let param_types: Vec<BasicMetadataTypeEnum> = func
            .params
            .iter()
            .map(|p| self.get_llvm_type(&p.ty).into())
            .collect();

        let fn_type = if func.return_type == Type::Int {
            self.i64_type().fn_type(&param_types, false)
        } else {
            self.context.void_type().fn_type(&param_types, false)
        };

        let function = self.module.add_function(&func.name, fn_type, None);
        self.functions.insert(func.name.clone(), function);

        Ok(())
    }

    pub fn generate_function_body(&mut self, func: &TirFunction) -> Result<()> {
        let function = *self
            .functions
            .get(&func.name)
            .ok_or_else(|| anyhow::anyhow!("Function {} not declared", func.name))?;

        let entry_bb = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(entry_bb);

        self.variables.clear();
        for (i, param) in func.params.iter().enumerate() {
            let param_value = function.get_nth_param(i as u32).unwrap();
            let alloca = self
                .builder
                .build_alloca(self.get_llvm_type(&param.ty), &param.name)
                .map_err(|e| anyhow::anyhow!("Failed to build alloca: {:?}", e))?;
            self.builder
                .build_store(alloca, param_value)
                .map_err(|e| anyhow::anyhow!("Failed to build store: {:?}", e))?;
            self.variables.insert(param.name.clone(), alloca);
        }

        for stmt in &func.body {
            self.codegen_stmt(stmt)
                .with_context(|| format!("In function '{}'", func.name))?;
        }

        if func.return_type == Type::Unit {
            self.builder
                .build_return(None)
                .map_err(|e| anyhow::anyhow!("Failed to build return: {:?}", e))?;
        }

        Ok(())
    }

    fn codegen_stmt(&mut self, stmt: &TirStmt) -> Result<()> {
        match stmt {
            TirStmt::Let { name, ty, value } => {
                let value_llvm = self.codegen_expr(value)?;

                let alloca = self
                    .builder
                    .build_alloca(self.get_llvm_type(ty), name)
                    .map_err(|e| anyhow::anyhow!("Failed to build alloca: {:?}", e))?;

                self.builder
                    .build_store(alloca, value_llvm)
                    .map_err(|e| anyhow::anyhow!("Failed to build store: {:?}", e))?;
                self.variables.insert(name.clone(), alloca);
            }

            TirStmt::Return(expr_opt) => {
                if let Some(expr) = expr_opt {
                    let value = self.codegen_expr(expr)?;
                    self.builder
                        .build_return(Some(&value))
                        .map_err(|e| anyhow::anyhow!("Failed to build return: {:?}", e))?;
                } else {
                    self.builder
                        .build_return(None)
                        .map_err(|e| anyhow::anyhow!("Failed to build return: {:?}", e))?;
                }
            }

            TirStmt::Expr(expr) => {
                self.codegen_expr(expr)?;
            }
        }

        Ok(())
    }

    // ── Expression codegen ───────────────────────────────────────

    fn codegen_expr(&mut self, expr: &TirExpr) -> Result<BasicValueEnum<'ctx>> {
        match &expr.kind {
            TirExprKind::IntLiteral(val) => {
                Ok(self.i64_type().const_int(*val as u64, false).into())
            }

            TirExprKind::Var(name) => {
                let ptr = self
                    .variables
                    .get(name.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Undefined variable: {}", name))?;
                let value = self
                    .builder
                    .build_load(self.get_llvm_type(&expr.ty), *ptr, name)
                    .map_err(|e| anyhow::anyhow!("Failed to build load: {:?}", e))?;
                Ok(value)
            }

            TirExprKind::BinOp { op, left, right } => {
                let left_val = self.codegen_expr(left)?;
                let right_val = self.codegen_expr(right)?;

                let left_int = left_val.into_int_value();
                let right_int = right_val.into_int_value();

                let result = match op {
                    BinOpKind::Add => self
                        .builder
                        .build_int_add(left_int, right_int, "add")
                        .map_err(|e| anyhow::anyhow!("Failed to build add: {:?}", e))?,
                    BinOpKind::Sub => self
                        .builder
                        .build_int_sub(left_int, right_int, "sub")
                        .map_err(|e| anyhow::anyhow!("Failed to build sub: {:?}", e))?,
                    BinOpKind::Mul => self
                        .builder
                        .build_int_mul(left_int, right_int, "mul")
                        .map_err(|e| anyhow::anyhow!("Failed to build mul: {:?}", e))?,
                    BinOpKind::Div => self
                        .builder
                        .build_int_signed_div(left_int, right_int, "div")
                        .map_err(|e| anyhow::anyhow!("Failed to build div: {:?}", e))?,
                    BinOpKind::Mod => self
                        .builder
                        .build_int_signed_rem(left_int, right_int, "mod")
                        .map_err(|e| anyhow::anyhow!("Failed to build mod: {:?}", e))?,
                };

                Ok(result.into())
            }

            TirExprKind::Call { func, args } => {
                if func == "print" {
                    return self.codegen_print_call(&args[0]);
                }

                let function = *self
                    .functions
                    .get(func.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Undefined function: {}", func))?;

                let arg_values: Vec<BasicValueEnum> = args
                    .iter()
                    .map(|arg| self.codegen_expr(arg))
                    .collect::<Result<_>>()?;

                let arg_metadata: Vec<_> = arg_values.iter().map(|v| (*v).into()).collect();

                let call_site = self
                    .builder
                    .build_call(function, &arg_metadata, "call")
                    .map_err(|e| anyhow::anyhow!("Failed to build call: {:?}", e))?;

                match call_site.try_as_basic_value() {
                    ValueKind::Basic(return_val) => Ok(return_val),
                    ValueKind::Instruction(_) => Ok(self.i64_type().const_int(0, false).into()),
                }
            }
        }
    }

    fn codegen_print_call(&mut self, arg: &TirExpr) -> Result<BasicValueEnum<'ctx>> {
        let arg_val = self.codegen_expr(arg)?;

        let printf_type = self.context.i32_type().fn_type(
            &[self
                .context
                .ptr_type(inkwell::AddressSpace::default())
                .into()],
            true,
        );

        let printf = self
            .module
            .get_function("printf")
            .unwrap_or_else(|| self.module.add_function("printf", printf_type, None));

        let format_str = self
            .builder
            .build_global_string_ptr("%lld\n", "printf_fmt")
            .map_err(|e| anyhow::anyhow!("Failed to build format string: {:?}", e))?;

        self.builder
            .build_call(
                printf,
                &[format_str.as_pointer_value().into(), arg_val.into()],
                "printf_call",
            )
            .map_err(|e| anyhow::anyhow!("Failed to build printf call: {:?}", e))?;

        Ok(self.i64_type().const_int(0, false).into())
    }

    // ── Entry-point wrapper ──────────────────────────────────────

    pub fn add_c_main_wrapper(&mut self, entry_main_name: &str) -> Result<()> {
        let entry_fn = self
            .module
            .get_function(entry_main_name)
            .ok_or_else(|| anyhow::anyhow!("{} function not found", entry_main_name))?;

        let c_main_type = self.context.i32_type().fn_type(&[], false);
        let c_main = self.module.add_function("main", c_main_type, None);

        let entry = self.context.append_basic_block(c_main, "entry");
        self.builder.position_at_end(entry);

        let result = self
            .builder
            .build_call(entry_fn, &[], "call_main")
            .map_err(|e| anyhow::anyhow!("Failed to build call: {:?}", e))?;
        let return_val = match result.try_as_basic_value() {
            ValueKind::Basic(val) => val,
            ValueKind::Instruction(_) => anyhow::bail!("Main function must return a value"),
        };

        let i32_result = self
            .builder
            .build_int_cast(
                return_val.into_int_value(),
                self.context.i32_type(),
                "cast_to_i32",
            )
            .map_err(|e| anyhow::anyhow!("Failed to build cast: {:?}", e))?;

        self.builder
            .build_return(Some(&i32_result))
            .map_err(|e| anyhow::anyhow!("Failed to build return: {:?}", e))?;

        Ok(())
    }
}
