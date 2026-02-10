use anyhow::{Context as _, Result};
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::{BasicMetadataTypeEnum, IntType};
use inkwell::values::{BasicValueEnum, FunctionValue, PointerValue, ValueKind};
use inkwell::IntPredicate;
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

use crate::ast::Type;
use crate::tir::{BinOpKind, CmpOp, TirExpr, TirExprKind, TirFunction, TirStmt};

pub struct Codegen<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    variables: HashMap<String, PointerValue<'ctx>>,
}

impl<'ctx> Codegen<'ctx> {
    pub fn new(context: &'ctx Context) -> Self {
        let module = context.create_module("__main__");
        let builder = context.create_builder();
        Self {
            context,
            module,
            builder,
            variables: HashMap::new(),
        }
    }

    pub fn link(&self, output_path: &Path) -> Result<()> {
        let bc_path = output_path.with_extension("bc");

        self.module.write_bitcode_to_path(&bc_path);

        let output = Command::new("clang")
            .arg("-O2")
            .arg("-o")
            .arg(output_path)
            .arg(&bc_path)
            .output()
            .context("Failed to run clang")?;

        assert!(output.status.success());

        Ok(())
    }

    // ── Type helpers ─────────────────────────────────────────────

    fn get_llvm_type(&self, ty: &Type) -> inkwell::types::BasicTypeEnum<'ctx> {
        match ty {
            Type::Int | Type::Bool => self.context.i64_type().into(),
            _ => panic!("Unsupported type for LLVM conversion: {:?}", ty),
        }
    }

    fn i64_type(&self) -> IntType<'ctx> {
        self.context.i64_type()
    }

    // ── Function helpers ─────────────────────────────────────────

    fn get_or_declare_function(
        &self,
        name: &str,
        param_types: &[Type],
        return_type: &Type,
    ) -> FunctionValue<'ctx> {
        self.module.get_function(name).unwrap_or_else(|| {
            let llvm_params: Vec<BasicMetadataTypeEnum> = param_types
                .iter()
                .map(|t| self.get_llvm_type(t).into())
                .collect();

            let fn_type = if *return_type == Type::Int {
                self.i64_type().fn_type(&llvm_params, false)
            } else {
                self.context.void_type().fn_type(&llvm_params, false)
            };

            self.module.add_function(name, fn_type, None)
        })
    }

    // ── Function codegen ─────────────────────────────────────────

    pub fn generate(&mut self, func: &TirFunction) {
        let param_types: Vec<Type> = func.params.iter().map(|p| p.ty.clone()).collect();
        let function = self.get_or_declare_function(&func.name, &param_types, &func.return_type);

        let entry_bb = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(entry_bb);

        self.variables.clear();
        for (i, param) in func.params.iter().enumerate() {
            let param_value = function.get_nth_param(i as u32).unwrap();
            let alloca = self
                .builder
                .build_alloca(self.get_llvm_type(&param.ty), &param.name)
                .unwrap();
            self.builder.build_store(alloca, param_value).unwrap();
            self.variables.insert(param.name.clone(), alloca);
        }

        for stmt in &func.body {
            self.codegen_stmt(stmt);
        }

        if func.return_type == Type::Unit
            && self
                .builder
                .get_insert_block()
                .unwrap()
                .get_terminator()
                .is_none()
        {
            self.builder.build_return(None).unwrap();
        }
    }

    fn codegen_stmt(&mut self, stmt: &TirStmt) {
        match stmt {
            TirStmt::Let { name, ty, value } => {
                let value_llvm = self.codegen_expr(value);

                if let Some(&existing_ptr) = self.variables.get(name.as_str()) {
                    // Reassignment: store to existing alloca
                    self.builder.build_store(existing_ptr, value_llvm).unwrap();
                } else {
                    // New variable: create alloca and store
                    let alloca = self
                        .builder
                        .build_alloca(self.get_llvm_type(ty), name)
                        .unwrap();
                    self.builder.build_store(alloca, value_llvm).unwrap();
                    self.variables.insert(name.clone(), alloca);
                }
            }

            TirStmt::Return(expr_opt) => {
                if let Some(expr) = expr_opt {
                    let value = self.codegen_expr(expr);
                    self.builder.build_return(Some(&value)).unwrap();
                } else {
                    self.builder.build_return(None).unwrap();
                }
            }

            TirStmt::Expr(expr) => {
                self.codegen_expr(expr);
            }

            TirStmt::If {
                condition,
                then_body,
                else_body,
            } => {
                let cond_val = self.codegen_expr(condition).into_int_value();
                let cond_bool = self
                    .builder
                    .build_int_compare(
                        IntPredicate::NE,
                        cond_val,
                        self.i64_type().const_int(0, false),
                        "ifcond",
                    )
                    .unwrap();

                let function = self
                    .builder
                    .get_insert_block()
                    .unwrap()
                    .get_parent()
                    .unwrap();

                let then_bb = self.context.append_basic_block(function, "then");
                let else_bb = self.context.append_basic_block(function, "else");
                let merge_bb = self.context.append_basic_block(function, "ifcont");

                self.builder
                    .build_conditional_branch(cond_bool, then_bb, else_bb)
                    .unwrap();

                // Then block
                self.builder.position_at_end(then_bb);
                for s in then_body {
                    self.codegen_stmt(s);
                }
                let then_terminated = self
                    .builder
                    .get_insert_block()
                    .unwrap()
                    .get_terminator()
                    .is_some();
                if !then_terminated {
                    self.builder.build_unconditional_branch(merge_bb).unwrap();
                }

                // Else block
                self.builder.position_at_end(else_bb);
                for s in else_body {
                    self.codegen_stmt(s);
                }
                let else_terminated = self
                    .builder
                    .get_insert_block()
                    .unwrap()
                    .get_terminator()
                    .is_some();
                if !else_terminated {
                    self.builder.build_unconditional_branch(merge_bb).unwrap();
                }

                self.builder.position_at_end(merge_bb);
                // If both branches terminated (e.g. both return), merge is dead
                if then_terminated && else_terminated {
                    self.builder.build_unreachable().unwrap();
                }
            }

            TirStmt::While { condition, body } => {
                let function = self
                    .builder
                    .get_insert_block()
                    .unwrap()
                    .get_parent()
                    .unwrap();

                let header_bb = self.context.append_basic_block(function, "while.header");
                let body_bb = self.context.append_basic_block(function, "while.body");
                let after_bb = self.context.append_basic_block(function, "while.after");

                self.builder.build_unconditional_branch(header_bb).unwrap();

                // Header: evaluate condition
                self.builder.position_at_end(header_bb);
                let cond_val = self.codegen_expr(condition).into_int_value();
                let cond_bool = self
                    .builder
                    .build_int_compare(
                        IntPredicate::NE,
                        cond_val,
                        self.i64_type().const_int(0, false),
                        "whilecond",
                    )
                    .unwrap();
                self.builder
                    .build_conditional_branch(cond_bool, body_bb, after_bb)
                    .unwrap();

                // Body
                self.builder.position_at_end(body_bb);
                for s in body {
                    self.codegen_stmt(s);
                }
                if self
                    .builder
                    .get_insert_block()
                    .unwrap()
                    .get_terminator()
                    .is_none()
                {
                    self.builder.build_unconditional_branch(header_bb).unwrap();
                }

                self.builder.position_at_end(after_bb);
            }

            TirStmt::Assert(condition) => {
                let cond_val = self.codegen_expr(condition).into_int_value();
                let cond_bool = self
                    .builder
                    .build_int_compare(
                        IntPredicate::NE,
                        cond_val,
                        self.i64_type().const_int(0, false),
                        "assertcond",
                    )
                    .unwrap();

                let function = self
                    .builder
                    .get_insert_block()
                    .unwrap()
                    .get_parent()
                    .unwrap();

                let fail_bb = self.context.append_basic_block(function, "assert.fail");
                let pass_bb = self.context.append_basic_block(function, "assert.pass");

                self.builder
                    .build_conditional_branch(cond_bool, pass_bb, fail_bb)
                    .unwrap();

                // Fail block: call abort()
                self.builder.position_at_end(fail_bb);
                let abort_fn = self.module.get_function("abort").unwrap_or_else(|| {
                    let abort_type = self.context.void_type().fn_type(&[], false);
                    self.module.add_function("abort", abort_type, None)
                });
                self.builder.build_call(abort_fn, &[], "").unwrap();
                self.builder.build_unreachable().unwrap();

                self.builder.position_at_end(pass_bb);
            }
        }
    }

    // ── Expression codegen ───────────────────────────────────────

    fn codegen_expr(&mut self, expr: &TirExpr) -> BasicValueEnum<'ctx> {
        match &expr.kind {
            TirExprKind::IntLiteral(val) => self.i64_type().const_int(*val as u64, false).into(),

            TirExprKind::Var(name) => {
                let ptr = self.variables[name.as_str()];
                self.builder
                    .build_load(self.get_llvm_type(&expr.ty), ptr, name)
                    .unwrap()
            }

            TirExprKind::BinOp { op, left, right } => {
                let left_val = self.codegen_expr(left);
                let right_val = self.codegen_expr(right);

                let left_int = left_val.into_int_value();
                let right_int = right_val.into_int_value();

                let result = match op {
                    BinOpKind::Add => self
                        .builder
                        .build_int_add(left_int, right_int, "add")
                        .unwrap(),
                    BinOpKind::Sub => self
                        .builder
                        .build_int_sub(left_int, right_int, "sub")
                        .unwrap(),
                    BinOpKind::Mul => self
                        .builder
                        .build_int_mul(left_int, right_int, "mul")
                        .unwrap(),
                    BinOpKind::Div => self
                        .builder
                        .build_int_signed_div(left_int, right_int, "div")
                        .unwrap(),
                    BinOpKind::Mod => self
                        .builder
                        .build_int_signed_rem(left_int, right_int, "mod")
                        .unwrap(),
                };

                result.into()
            }

            TirExprKind::Call { func, args } => {
                if func == "print" {
                    return self.codegen_print_call(&args[0]);
                }

                let arg_types: Vec<Type> = args.iter().map(|a| a.ty.clone()).collect();
                let function = self.get_or_declare_function(func, &arg_types, &expr.ty);

                let arg_values: Vec<BasicValueEnum> =
                    args.iter().map(|arg| self.codegen_expr(arg)).collect();

                let arg_metadata: Vec<_> = arg_values.iter().map(|v| (*v).into()).collect();

                let call_site = self
                    .builder
                    .build_call(function, &arg_metadata, "call")
                    .unwrap();

                match call_site.try_as_basic_value() {
                    ValueKind::Basic(return_val) => return_val,
                    ValueKind::Instruction(_) => self.i64_type().const_int(0, false).into(),
                }
            }

            TirExprKind::Compare { op, left, right } => {
                let left_val = self.codegen_expr(left).into_int_value();
                let right_val = self.codegen_expr(right).into_int_value();

                let predicate = match op {
                    CmpOp::Eq => IntPredicate::EQ,
                    CmpOp::NotEq => IntPredicate::NE,
                    CmpOp::Lt => IntPredicate::SLT,
                    CmpOp::LtEq => IntPredicate::SLE,
                    CmpOp::Gt => IntPredicate::SGT,
                    CmpOp::GtEq => IntPredicate::SGE,
                };

                let cmp_result = self
                    .builder
                    .build_int_compare(predicate, left_val, right_val, "cmp")
                    .unwrap();

                // zext i1 -> i64 to match our Bool representation
                self.builder
                    .build_int_z_extend(cmp_result, self.i64_type(), "zext_bool")
                    .unwrap()
                    .into()
            }
        }
    }

    fn codegen_print_call(&mut self, arg: &TirExpr) -> BasicValueEnum<'ctx> {
        let arg_val = self.codegen_expr(arg);

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
            .unwrap();

        self.builder
            .build_call(
                printf,
                &[format_str.as_pointer_value().into(), arg_val.into()],
                "printf_call",
            )
            .unwrap();

        self.i64_type().const_int(0, false).into()
    }

    // ── Entry-point wrapper ──────────────────────────────────────

    pub fn add_c_main_wrapper(&mut self, entry_main_name: &str) {
        let entry_fn = self.module.get_function(entry_main_name).unwrap();

        let c_main_type = self.context.i32_type().fn_type(&[], false);
        let c_main = self.module.add_function("main", c_main_type, None);

        let entry = self.context.append_basic_block(c_main, "entry");
        self.builder.position_at_end(entry);

        let result = self.builder.build_call(entry_fn, &[], "call_main").unwrap();
        let return_val = match result.try_as_basic_value() {
            ValueKind::Basic(val) => val,
            ValueKind::Instruction(_) => panic!("Main function must return a value"),
        };

        let i32_result = self
            .builder
            .build_int_cast(
                return_val.into_int_value(),
                self.context.i32_type(),
                "cast_to_i32",
            )
            .unwrap();

        self.builder.build_return(Some(&i32_result)).unwrap();
    }
}
