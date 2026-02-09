use anyhow::Result;
use inkwell::values::BasicValueEnum;

use super::context::CodegenContext;
use crate::ast::BinOpKind;
use crate::tir::{TirExpr, TirExprKind};

pub struct ExprCodegen<'a, 'ctx> {
    context: &'a mut CodegenContext<'ctx>,
}

impl<'a, 'ctx> ExprCodegen<'a, 'ctx> {
    pub fn new(context: &'a mut CodegenContext<'ctx>) -> Self {
        Self { context }
    }

    pub fn codegen_expr(&mut self, expr: &TirExpr) -> Result<BasicValueEnum<'ctx>> {
        match &expr.kind {
            TirExprKind::IntLiteral(val) => {
                Ok(self.context.i64_type().const_int(*val as u64, false).into())
            }

            TirExprKind::Var(name) => {
                let ptr = self
                    .context
                    .get_variable(name)
                    .ok_or_else(|| anyhow::anyhow!("Undefined variable: {}", name))?;
                let value = self
                    .context
                    .builder
                    .build_load(self.context.get_llvm_type(&expr.ty), ptr, name)
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
                        .context
                        .builder
                        .build_int_add(left_int, right_int, "add")
                        .map_err(|e| anyhow::anyhow!("Failed to build add: {:?}", e))?,
                    BinOpKind::Sub => self
                        .context
                        .builder
                        .build_int_sub(left_int, right_int, "sub")
                        .map_err(|e| anyhow::anyhow!("Failed to build sub: {:?}", e))?,
                    BinOpKind::Mul => self
                        .context
                        .builder
                        .build_int_mul(left_int, right_int, "mul")
                        .map_err(|e| anyhow::anyhow!("Failed to build mul: {:?}", e))?,
                    BinOpKind::Div => self
                        .context
                        .builder
                        .build_int_signed_div(left_int, right_int, "div")
                        .map_err(|e| anyhow::anyhow!("Failed to build div: {:?}", e))?,
                    BinOpKind::Mod => self
                        .context
                        .builder
                        .build_int_signed_rem(left_int, right_int, "mod")
                        .map_err(|e| anyhow::anyhow!("Failed to build mod: {:?}", e))?,
                };

                Ok(result.into())
            }

            TirExprKind::Call { func, args } => {
                // Special case for print
                if func == "print" {
                    return self.codegen_print_call(&args[0]);
                }

                let function = self
                    .context
                    .get_function(func)
                    .ok_or_else(|| anyhow::anyhow!("Undefined function: {}", func))?;

                let arg_values: Vec<BasicValueEnum> = args
                    .iter()
                    .map(|arg| self.codegen_expr(arg))
                    .collect::<Result<_>>()?;

                let arg_metadata: Vec<_> = arg_values.iter().map(|v| (*v).into()).collect();

                let call_site = self
                    .context
                    .builder
                    .build_call(function, &arg_metadata, "call")
                    .map_err(|e| anyhow::anyhow!("Failed to build call: {:?}", e))?;

                // Try to get return value, or return dummy for void functions
                use inkwell::values::ValueKind;
                match call_site.try_as_basic_value() {
                    ValueKind::Basic(return_val) => Ok(return_val),
                    ValueKind::Instruction(_) => {
                        // void return - return a dummy value
                        Ok(self.context.i64_type().const_int(0, false).into())
                    }
                }
            }
        }
    }

    fn codegen_print_call(&mut self, arg: &TirExpr) -> Result<BasicValueEnum<'ctx>> {
        // Generate printf call for int
        let arg_val = self.codegen_expr(arg)?;

        // Declare printf if not already declared
        let printf_type = self.context.context.i32_type().fn_type(
            &[self
                .context
                .context
                .ptr_type(inkwell::AddressSpace::default())
                .into()],
            true, // variadic
        );

        let printf = self
            .context
            .module
            .get_function("printf")
            .unwrap_or_else(|| {
                self.context
                    .module
                    .add_function("printf", printf_type, None)
            });

        // Create format string "%lld\n"
        let format_str = self
            .context
            .builder
            .build_global_string_ptr("%lld\n", "printf_fmt")
            .map_err(|e| anyhow::anyhow!("Failed to build format string: {:?}", e))?;

        // Call printf
        self.context
            .builder
            .build_call(
                printf,
                &[format_str.as_pointer_value().into(), arg_val.into()],
                "printf_call",
            )
            .map_err(|e| anyhow::anyhow!("Failed to build printf call: {:?}", e))?;

        Ok(self.context.i64_type().const_int(0, false).into())
    }
}
