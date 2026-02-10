use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::targets::{CodeModel, InitializationConfig, RelocMode, Target, TargetMachine};
use inkwell::types::{BasicMetadataTypeEnum, BasicType, FloatType, IntType};
use inkwell::values::{BasicValueEnum, FunctionValue, PointerValue, ValueKind};
use inkwell::{FloatPredicate, IntPredicate, OptimizationLevel};
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

use crate::ast::Type;
use crate::tir::{
    BinOpKind, CmpOp, LogicalOp, TirExpr, TirExprKind, TirFunction, TirStmt, UnaryOpKind,
};

pub struct Codegen<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    variables: HashMap<String, PointerValue<'ctx>>,
    loop_stack: Vec<(
        inkwell::basic_block::BasicBlock<'ctx>,
        inkwell::basic_block::BasicBlock<'ctx>,
    )>,
}

impl<'ctx> Codegen<'ctx> {
    pub fn new(context: &'ctx Context) -> Self {
        Target::initialize_native(&InitializationConfig::default())
            .expect("Failed to initialize native target");

        let triple = TargetMachine::get_default_triple();
        let target = Target::from_triple(&triple).unwrap();
        let target_machine = target
            .create_target_machine(
                &triple,
                "",
                "",
                OptimizationLevel::Default,
                RelocMode::Default,
                CodeModel::Default,
            )
            .expect("Failed to create target machine");

        let module = context.create_module("__main__");
        module.set_triple(&triple);
        module.set_data_layout(&target_machine.get_target_data().get_data_layout());

        let builder = context.create_builder();
        Self {
            context,
            module,
            builder,
            variables: HashMap::new(),
            loop_stack: Vec::new(),
        }
    }

    const RUNTIME_BC: &'static str = env!("RUNTIME_BC_PATH");

    pub fn link(&self, output_path: &Path) {
        let bc_path = output_path.with_extension("o");

        self.module.write_bitcode_to_path(&bc_path);

        Command::new("clang")
            .arg("-flto")
            .arg("-O2")
            .arg("-o")
            .arg(output_path)
            .arg(&bc_path)
            .arg(Self::RUNTIME_BC)
            .arg("-lm")
            .output()
            .unwrap();
    }

    fn get_llvm_type(&self, ty: &Type) -> inkwell::types::BasicTypeEnum<'ctx> {
        match ty {
            Type::Int | Type::Bool => self.context.i64_type().into(),
            Type::Float => self.context.f64_type().into(),
            _ => panic!("Unsupported type for LLVM conversion: {:?}", ty),
        }
    }

    fn i64_type(&self) -> IntType<'ctx> {
        self.context.i64_type()
    }

    fn f64_type(&self) -> FloatType<'ctx> {
        self.context.f64_type()
    }

    fn build_int_truthiness_check(
        &self,
        value: inkwell::values::IntValue<'ctx>,
        label: &str,
    ) -> inkwell::values::IntValue<'ctx> {
        self.builder
            .build_int_compare(
                IntPredicate::NE,
                value,
                self.i64_type().const_int(0, false),
                label,
            )
            .unwrap()
    }

    fn build_float_truthiness_check(
        &self,
        value: inkwell::values::FloatValue<'ctx>,
        label: &str,
    ) -> inkwell::values::IntValue<'ctx> {
        self.builder
            .build_float_compare(
                FloatPredicate::ONE,
                value,
                self.f64_type().const_float(0.0),
                label,
            )
            .unwrap()
    }

    fn build_truthiness_check_for_value(
        &self,
        value: BasicValueEnum<'ctx>,
        ty: &Type,
        label: &str,
    ) -> inkwell::values::IntValue<'ctx> {
        match ty {
            Type::Float => self.build_float_truthiness_check(value.into_float_value(), label),
            _ => self.build_int_truthiness_check(value.into_int_value(), label),
        }
    }

    fn branch_if_unterminated(&self, target: inkwell::basic_block::BasicBlock<'ctx>) -> bool {
        let terminated = self
            .builder
            .get_insert_block()
            .unwrap()
            .get_terminator()
            .is_some();
        if !terminated {
            self.builder.build_unconditional_branch(target).unwrap();
        }
        terminated
    }

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

            let fn_type = match return_type {
                Type::Unit => self.context.void_type().fn_type(&llvm_params, false),
                other => self.get_llvm_type(other).fn_type(&llvm_params, false),
            };

            self.module.add_function(name, fn_type, None)
        })
    }

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
                    self.builder.build_store(existing_ptr, value_llvm).unwrap();
                } else {
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
                let cond_val = self.codegen_expr(condition);
                let cond_bool =
                    self.build_truthiness_check_for_value(cond_val, &condition.ty, "ifcond");

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

                self.builder.position_at_end(then_bb);
                for s in then_body {
                    self.codegen_stmt(s);
                }
                let then_terminated = self.branch_if_unterminated(merge_bb);

                self.builder.position_at_end(else_bb);
                for s in else_body {
                    self.codegen_stmt(s);
                }
                let else_terminated = self.branch_if_unterminated(merge_bb);

                self.builder.position_at_end(merge_bb);
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

                self.builder.position_at_end(header_bb);
                let cond_val = self.codegen_expr(condition);
                let cond_bool =
                    self.build_truthiness_check_for_value(cond_val, &condition.ty, "whilecond");
                self.builder
                    .build_conditional_branch(cond_bool, body_bb, after_bb)
                    .unwrap();

                self.builder.position_at_end(body_bb);
                self.loop_stack.push((header_bb, after_bb));
                for s in body {
                    self.codegen_stmt(s);
                }
                self.loop_stack.pop();
                self.branch_if_unterminated(header_bb);

                self.builder.position_at_end(after_bb);
            }

            TirStmt::Break => {
                let (_, after_bb) = self.loop_stack.last().unwrap();
                self.builder.build_unconditional_branch(*after_bb).unwrap();
                // Create dead block for any unreachable code after break
                let function = self
                    .builder
                    .get_insert_block()
                    .unwrap()
                    .get_parent()
                    .unwrap();
                let dead_bb = self.context.append_basic_block(function, "break.dead");
                self.builder.position_at_end(dead_bb);
            }

            TirStmt::Continue => {
                let (header_bb, _) = self.loop_stack.last().unwrap();
                self.builder.build_unconditional_branch(*header_bb).unwrap();
                // Create dead block for any unreachable code after continue
                let function = self
                    .builder
                    .get_insert_block()
                    .unwrap()
                    .get_parent()
                    .unwrap();
                let dead_bb = self.context.append_basic_block(function, "cont.dead");
                self.builder.position_at_end(dead_bb);
            }
        }
    }

    fn codegen_expr(&mut self, expr: &TirExpr) -> BasicValueEnum<'ctx> {
        match &expr.kind {
            TirExprKind::IntLiteral(val) => self.i64_type().const_int(*val as u64, false).into(),

            TirExprKind::FloatLiteral(val) => self.f64_type().const_float(*val).into(),

            TirExprKind::Var(name) => {
                let ptr = self.variables[name.as_str()];
                self.builder
                    .build_load(self.get_llvm_type(&expr.ty), ptr, name)
                    .unwrap()
            }

            TirExprKind::BinOp { op, left, right } => {
                let left_val = self.codegen_expr(left);
                let right_val = self.codegen_expr(right);

                if expr.ty == Type::Float {
                    let left_float = left_val.into_float_value();
                    let right_float = right_val.into_float_value();

                    let result = match op {
                        BinOpKind::Add => self
                            .builder
                            .build_float_add(left_float, right_float, "fadd")
                            .unwrap(),
                        BinOpKind::Sub => self
                            .builder
                            .build_float_sub(left_float, right_float, "fsub")
                            .unwrap(),
                        BinOpKind::Mul => self
                            .builder
                            .build_float_mul(left_float, right_float, "fmul")
                            .unwrap(),
                        BinOpKind::Div => self
                            .builder
                            .build_float_div(left_float, right_float, "fdiv")
                            .unwrap(),
                        BinOpKind::Mod => self
                            .builder
                            .build_float_rem(left_float, right_float, "fmod")
                            .unwrap(),
                        BinOpKind::FloorDiv => {
                            let div = self
                                .builder
                                .build_float_div(left_float, right_float, "fdiv")
                                .unwrap();
                            let floor_fn = self
                                .module
                                .get_function("llvm.floor.f64")
                                .unwrap_or_else(|| {
                                    let f64_type = self.context.f64_type();
                                    let fn_type = f64_type.fn_type(&[f64_type.into()], false);
                                    self.module.add_function("llvm.floor.f64", fn_type, None)
                                });
                            match self
                                .builder
                                .build_call(floor_fn, &[div.into()], "floordiv")
                                .unwrap()
                                .try_as_basic_value()
                            {
                                ValueKind::Basic(val) => val.into_float_value(),
                                _ => panic!("llvm.floor.f64 must return a value"),
                            }
                        }
                        BinOpKind::Pow => {
                            let pow_fn =
                                self.module.get_function("llvm.pow.f64").unwrap_or_else(|| {
                                    let f64_type = self.context.f64_type();
                                    let fn_type = f64_type
                                        .fn_type(&[f64_type.into(), f64_type.into()], false);
                                    self.module.add_function("llvm.pow.f64", fn_type, None)
                                });
                            match self
                                .builder
                                .build_call(pow_fn, &[left_float.into(), right_float.into()], "pow")
                                .unwrap()
                                .try_as_basic_value()
                            {
                                ValueKind::Basic(val) => val.into_float_value(),
                                _ => panic!("llvm.pow.f64 must return a value"),
                            }
                        }
                        BinOpKind::BitAnd
                        | BinOpKind::BitOr
                        | BinOpKind::BitXor
                        | BinOpKind::LShift
                        | BinOpKind::RShift => {
                            panic!("Bitwise operations not supported on floats")
                        }
                    };
                    result.into()
                } else {
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
                        BinOpKind::FloorDiv => {
                            // Python floor division: floor toward -infinity
                            // floor_div(a,b) = a/b - ((a%b != 0) & ((a^b) < 0))
                            let div = self
                                .builder
                                .build_int_signed_div(left_int, right_int, "div_tmp")
                                .unwrap();
                            let rem = self
                                .builder
                                .build_int_signed_rem(left_int, right_int, "rem_tmp")
                                .unwrap();
                            let zero = self.i64_type().const_int(0, false);
                            let rem_nonzero = self
                                .builder
                                .build_int_compare(IntPredicate::NE, rem, zero, "rem_nz")
                                .unwrap();
                            let xor_val = self
                                .builder
                                .build_xor(left_int, right_int, "xor_signs")
                                .unwrap();
                            let signs_differ = self
                                .builder
                                .build_int_compare(IntPredicate::SLT, xor_val, zero, "signs_diff")
                                .unwrap();
                            let need_adjust = self
                                .builder
                                .build_and(rem_nonzero, signs_differ, "need_adj")
                                .unwrap();
                            let adjust = self
                                .builder
                                .build_int_z_extend(need_adjust, self.i64_type(), "adj_ext")
                                .unwrap();
                            self.builder.build_int_sub(div, adjust, "floordiv").unwrap()
                        }
                        BinOpKind::Pow => {
                            // Call __tython_pow_int runtime function
                            let pow_fn = self.get_or_declare_function(
                                "__tython_pow_int",
                                &[Type::Int, Type::Int],
                                &Type::Int,
                            );
                            match self
                                .builder
                                .build_call(pow_fn, &[left_int.into(), right_int.into()], "ipow")
                                .unwrap()
                                .try_as_basic_value()
                            {
                                ValueKind::Basic(val) => val.into_int_value(),
                                _ => panic!("__tython_pow_int must return a value"),
                            }
                        }
                        BinOpKind::BitAnd => self
                            .builder
                            .build_and(left_int, right_int, "bitand")
                            .unwrap(),
                        BinOpKind::BitOr => {
                            self.builder.build_or(left_int, right_int, "bitor").unwrap()
                        }
                        BinOpKind::BitXor => self
                            .builder
                            .build_xor(left_int, right_int, "bitxor")
                            .unwrap(),
                        BinOpKind::LShift => self
                            .builder
                            .build_left_shift(left_int, right_int, "lshift")
                            .unwrap(),
                        BinOpKind::RShift => self
                            .builder
                            .build_right_shift(left_int, right_int, true, "rshift")
                            .unwrap(),
                    };
                    result.into()
                }
            }

            TirExprKind::Call { func, args } => {
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

            TirExprKind::ExternalCall { func, args } => {
                let function = self.get_or_declare_function(
                    func.symbol(),
                    &func.param_types(),
                    &func.return_type(),
                );

                let arg_values: Vec<BasicValueEnum> =
                    args.iter().map(|arg| self.codegen_expr(arg)).collect();
                let arg_metadata: Vec<_> = arg_values.iter().map(|v| (*v).into()).collect();

                let call_site = self
                    .builder
                    .build_call(function, &arg_metadata, "ext_call")
                    .unwrap();

                match func.return_type() {
                    Type::Unit => self.i64_type().const_int(0, false).into(),
                    _ => match call_site.try_as_basic_value() {
                        ValueKind::Basic(val) => val,
                        ValueKind::Instruction(_) => {
                            panic!("Expected return value from {}", func.symbol())
                        }
                    },
                }
            }

            TirExprKind::Cast { target, arg } => {
                let arg_val = self.codegen_expr(arg);
                match (&arg.ty, target) {
                    (Type::Int, Type::Int)
                    | (Type::Float, Type::Float)
                    | (Type::Bool, Type::Bool)
                    | (Type::Bool, Type::Int) => arg_val,

                    (Type::Float, Type::Int) => self
                        .builder
                        .build_float_to_signed_int(
                            arg_val.into_float_value(),
                            self.i64_type(),
                            "ftoi",
                        )
                        .unwrap()
                        .into(),

                    (Type::Int, Type::Float) => self
                        .builder
                        .build_signed_int_to_float(
                            arg_val.into_int_value(),
                            self.f64_type(),
                            "itof",
                        )
                        .unwrap()
                        .into(),

                    (Type::Bool, Type::Float) => self
                        .builder
                        .build_signed_int_to_float(
                            arg_val.into_int_value(),
                            self.f64_type(),
                            "btof",
                        )
                        .unwrap()
                        .into(),

                    (Type::Int, Type::Bool) => {
                        let cmp = self.build_int_truthiness_check(arg_val.into_int_value(), "itob");
                        self.builder
                            .build_int_z_extend(cmp, self.i64_type(), "zext_bool")
                            .unwrap()
                            .into()
                    }

                    (Type::Float, Type::Bool) => {
                        let cmp =
                            self.build_float_truthiness_check(arg_val.into_float_value(), "ftob");
                        self.builder
                            .build_int_z_extend(cmp, self.i64_type(), "zext_bool")
                            .unwrap()
                            .into()
                    }

                    _ => panic!("Unsupported cast: {:?} -> {:?}", arg.ty, target),
                }
            }

            TirExprKind::Compare { op, left, right } => {
                let left_val = self.codegen_expr(left);
                let right_val = self.codegen_expr(right);

                let cmp_result = if left.ty == Type::Float {
                    let predicate = match op {
                        CmpOp::Eq => FloatPredicate::OEQ,
                        CmpOp::NotEq => FloatPredicate::ONE,
                        CmpOp::Lt => FloatPredicate::OLT,
                        CmpOp::LtEq => FloatPredicate::OLE,
                        CmpOp::Gt => FloatPredicate::OGT,
                        CmpOp::GtEq => FloatPredicate::OGE,
                    };
                    self.builder
                        .build_float_compare(
                            predicate,
                            left_val.into_float_value(),
                            right_val.into_float_value(),
                            "fcmp",
                        )
                        .unwrap()
                } else {
                    let predicate = match op {
                        CmpOp::Eq => IntPredicate::EQ,
                        CmpOp::NotEq => IntPredicate::NE,
                        CmpOp::Lt => IntPredicate::SLT,
                        CmpOp::LtEq => IntPredicate::SLE,
                        CmpOp::Gt => IntPredicate::SGT,
                        CmpOp::GtEq => IntPredicate::SGE,
                    };
                    self.builder
                        .build_int_compare(
                            predicate,
                            left_val.into_int_value(),
                            right_val.into_int_value(),
                            "cmp",
                        )
                        .unwrap()
                };

                self.builder
                    .build_int_z_extend(cmp_result, self.i64_type(), "zext_bool")
                    .unwrap()
                    .into()
            }

            TirExprKind::UnaryOp { op, operand } => {
                let operand_val = self.codegen_expr(operand);
                match op {
                    UnaryOpKind::Neg => {
                        if operand.ty == Type::Float {
                            let zero = self.f64_type().const_float(0.0);
                            self.builder
                                .build_float_sub(zero, operand_val.into_float_value(), "fneg")
                                .unwrap()
                                .into()
                        } else {
                            let zero = self.i64_type().const_int(0, false);
                            self.builder
                                .build_int_sub(zero, operand_val.into_int_value(), "neg")
                                .unwrap()
                                .into()
                        }
                    }
                    UnaryOpKind::Pos => operand_val,
                    UnaryOpKind::Not => {
                        let truth = self.build_truthiness_check_for_value(
                            operand_val,
                            &operand.ty,
                            "not_truth",
                        );
                        let inverted = self.builder.build_not(truth, "not").unwrap();
                        self.builder
                            .build_int_z_extend(inverted, self.i64_type(), "not_zext")
                            .unwrap()
                            .into()
                    }
                    UnaryOpKind::BitNot => {
                        let val = operand_val.into_int_value();
                        let all_ones = self.i64_type().const_all_ones();
                        self.builder
                            .build_xor(val, all_ones, "bitnot")
                            .unwrap()
                            .into()
                    }
                }
            }

            TirExprKind::LogicalOp { op, left, right } => {
                let function = self
                    .builder
                    .get_insert_block()
                    .unwrap()
                    .get_parent()
                    .unwrap();

                // Evaluate left side
                let left_val = self.codegen_expr(left);
                let left_truth =
                    self.build_truthiness_check_for_value(left_val, &left.ty, "log_left");
                let left_bb = self.builder.get_insert_block().unwrap();

                let right_bb = self.context.append_basic_block(function, "log_right");
                let merge_bb = self.context.append_basic_block(function, "log_merge");

                match op {
                    LogicalOp::And => {
                        // If left is falsy, short-circuit; else evaluate right
                        self.builder
                            .build_conditional_branch(left_truth, right_bb, merge_bb)
                            .unwrap();
                    }
                    LogicalOp::Or => {
                        // If left is truthy, short-circuit; else evaluate right
                        self.builder
                            .build_conditional_branch(left_truth, merge_bb, right_bb)
                            .unwrap();
                    }
                }

                // Evaluate right in right_bb
                self.builder.position_at_end(right_bb);
                let right_val = self.codegen_expr(right);
                let right_end_bb = self.builder.get_insert_block().unwrap();
                self.builder.build_unconditional_branch(merge_bb).unwrap();

                // Merge: phi node selects left_val or right_val
                self.builder.position_at_end(merge_bb);
                let llvm_type = self.get_llvm_type(&expr.ty);
                let phi = self.builder.build_phi(llvm_type, "log_result").unwrap();
                phi.add_incoming(&[(&left_val, left_bb), (&right_val, right_end_bb)]);

                phi.as_basic_value()
            }
        }
    }

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
