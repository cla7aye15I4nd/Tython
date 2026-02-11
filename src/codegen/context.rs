use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::targets::{CodeModel, InitializationConfig, RelocMode, Target, TargetMachine};
use inkwell::types::{BasicMetadataTypeEnum, BasicType, FloatType, IntType, StructType};
use inkwell::values::{BasicValueEnum, FunctionValue, PointerValue, ValueKind};
use inkwell::{AddressSpace, FloatPredicate, IntPredicate, OptimizationLevel};
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

use crate::ast::ClassInfo;
use crate::tir::{
    ArithBinOp, BitwiseBinOp, CallTarget, CastKind, CmpOp, LogicalOp, TirExpr, TirExprKind,
    TirFunction, TirStmt, TypedBinOp, UnaryOpKind, ValueType,
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
    class_types: HashMap<String, StructType<'ctx>>,
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
            class_types: HashMap::new(),
        }
    }

    const RUNTIME_BC: &'static str = env!("RUNTIME_BC_PATH");

    pub fn link(&self, output_path: &Path) {
        let bc_path = output_path.with_extension("o");

        self.module.write_bitcode_to_path(&bc_path);

        Command::new("clang")
            .arg("-static")
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

    // ── class registration ────────────────────────────────────────────

    pub fn register_class(&mut self, class_info: &ClassInfo) {
        let field_types: Vec<inkwell::types::BasicTypeEnum<'ctx>> = class_info
            .fields
            .iter()
            .map(|f| {
                let vty = ValueType::from_type(&f.ty).expect("ICE: class field has non-value type");
                self.get_llvm_type(&vty)
            })
            .collect();

        let struct_type = self.context.opaque_struct_type(&class_info.name);
        struct_type.set_body(&field_types, false);

        self.class_types
            .insert(class_info.name.clone(), struct_type);
    }

    // ── type helpers ──────────────────────────────────────────────────

    fn get_llvm_type(&self, ty: &ValueType) -> inkwell::types::BasicTypeEnum<'ctx> {
        match ty {
            ValueType::Int | ValueType::Bool => self.context.i64_type().into(),
            ValueType::Float => self.context.f64_type().into(),
            ValueType::Class(_) => self.context.ptr_type(AddressSpace::default()).into(),
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
        ty: &ValueType,
        label: &str,
    ) -> inkwell::values::IntValue<'ctx> {
        match ty {
            ValueType::Float => self.build_float_truthiness_check(value.into_float_value(), label),
            _ => self.build_int_truthiness_check(value.into_int_value(), label),
        }
    }

    /// Extract the return value from a call to a function known to return non-void.
    /// This is an LLVM API contract — the function has a non-void return type in IR.
    fn extract_call_value(
        &self,
        call_site: inkwell::values::CallSiteValue<'ctx>,
    ) -> BasicValueEnum<'ctx> {
        match call_site.try_as_basic_value() {
            ValueKind::Basic(val) => val,
            ValueKind::Instruction(_) => unreachable!("call to non-void function returned void"),
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
        param_types: &[ValueType],
        return_type: Option<ValueType>,
    ) -> FunctionValue<'ctx> {
        self.module.get_function(name).unwrap_or_else(|| {
            let llvm_params: Vec<BasicMetadataTypeEnum> = param_types
                .iter()
                .map(|t| self.get_llvm_type(t).into())
                .collect();

            let fn_type = match return_type {
                None => self.context.void_type().fn_type(&llvm_params, false),
                Some(ref ty) => self.get_llvm_type(ty).fn_type(&llvm_params, false),
            };

            self.module.add_function(name, fn_type, None)
        })
    }

    fn get_or_declare_malloc(&self) -> FunctionValue<'ctx> {
        self.module
            .get_function("__tython_malloc")
            .unwrap_or_else(|| {
                let i64_type = self.context.i64_type();
                let ptr_type = self.context.ptr_type(AddressSpace::default());
                let fn_type = ptr_type.fn_type(&[i64_type.into()], false);
                self.module.add_function("__tython_malloc", fn_type, None)
            })
    }

    /// Codegen a list of TIR args and return both value and metadata forms.
    fn codegen_call_args(
        &mut self,
        args: &[TirExpr],
    ) -> Vec<inkwell::values::BasicMetadataValueEnum<'ctx>> {
        args.iter()
            .map(|arg| self.codegen_expr(arg).into())
            .collect()
    }

    /// Create a dead basic block after an unconditional branch (break/continue).
    fn append_dead_block(&self, label: &str) {
        let function = self
            .builder
            .get_insert_block()
            .unwrap()
            .get_parent()
            .unwrap();
        let dead_bb = self.context.append_basic_block(function, label);
        self.builder.position_at_end(dead_bb);
    }

    /// Map a CmpOp to LLVM float predicate.
    fn float_predicate(op: &CmpOp) -> FloatPredicate {
        match op {
            CmpOp::Eq => FloatPredicate::OEQ,
            CmpOp::NotEq => FloatPredicate::ONE,
            CmpOp::Lt => FloatPredicate::OLT,
            CmpOp::LtEq => FloatPredicate::OLE,
            CmpOp::Gt => FloatPredicate::OGT,
            CmpOp::GtEq => FloatPredicate::OGE,
        }
    }

    /// Map a CmpOp to LLVM int predicate.
    fn int_predicate(op: &CmpOp) -> IntPredicate {
        match op {
            CmpOp::Eq => IntPredicate::EQ,
            CmpOp::NotEq => IntPredicate::NE,
            CmpOp::Lt => IntPredicate::SLT,
            CmpOp::LtEq => IntPredicate::SLE,
            CmpOp::Gt => IntPredicate::SGT,
            CmpOp::GtEq => IntPredicate::SGE,
        }
    }

    pub fn generate(&mut self, func: &TirFunction) {
        let param_types: Vec<ValueType> = func.params.iter().map(|p| p.ty.clone()).collect();
        let function =
            self.get_or_declare_function(&func.name, &param_types, func.return_type.clone());

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

        if func.return_type.is_none()
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

            TirStmt::VoidCall { target, args } => {
                let arg_metadata = self.codegen_call_args(args);

                match target {
                    CallTarget::Named(func_name) => {
                        let arg_types: Vec<ValueType> = args.iter().map(|a| a.ty.clone()).collect();
                        let function = self.get_or_declare_function(func_name, &arg_types, None);
                        self.builder
                            .build_call(function, &arg_metadata, "void_call")
                            .unwrap();
                    }
                    CallTarget::Builtin(builtin_fn) => {
                        let function = self.get_or_declare_function(
                            builtin_fn.symbol(),
                            &builtin_fn.param_types(),
                            builtin_fn.return_type(),
                        );
                        self.builder
                            .build_call(function, &arg_metadata, "void_ext_call")
                            .unwrap();
                    }
                    CallTarget::MethodCall {
                        mangled_name,
                        object,
                    } => {
                        let self_val = self.codegen_expr(object);
                        let mut all_meta: Vec<inkwell::values::BasicMetadataValueEnum> =
                            vec![self_val.into()];
                        all_meta.extend(arg_metadata);

                        let mut param_types = vec![object.ty.clone()];
                        param_types.extend(args.iter().map(|a| a.ty.clone()));

                        let function =
                            self.get_or_declare_function(mangled_name, &param_types, None);
                        self.builder
                            .build_call(function, &all_meta, "void_method_call")
                            .unwrap();
                    }
                }
            }

            TirStmt::SetField {
                object,
                field_name: _,
                field_index,
                value,
            } => {
                let obj_ptr = self.codegen_expr(object).into_pointer_value();
                let class_name = match &object.ty {
                    ValueType::Class(name) => name,
                    _ => unreachable!("ICE: SetField on non-class type"),
                };
                let struct_type = self.class_types[class_name];

                let field_ptr = self
                    .builder
                    .build_struct_gep(struct_type, obj_ptr, *field_index as u32, "field_ptr")
                    .unwrap();

                let val = self.codegen_expr(value);
                self.builder.build_store(field_ptr, val).unwrap();
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
                self.append_dead_block("break.dead");
            }

            TirStmt::Continue => {
                let (header_bb, _) = self.loop_stack.last().unwrap();
                self.builder.build_unconditional_branch(*header_bb).unwrap();
                self.append_dead_block("cont.dead");
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

                match op {
                    TypedBinOp::Arith(arith_op) => {
                        if expr.ty == ValueType::Float {
                            let left_float = left_val.into_float_value();
                            let right_float = right_val.into_float_value();

                            let result = match arith_op {
                                ArithBinOp::Add => self
                                    .builder
                                    .build_float_add(left_float, right_float, "fadd")
                                    .unwrap(),
                                ArithBinOp::Sub => self
                                    .builder
                                    .build_float_sub(left_float, right_float, "fsub")
                                    .unwrap(),
                                ArithBinOp::Mul => self
                                    .builder
                                    .build_float_mul(left_float, right_float, "fmul")
                                    .unwrap(),
                                ArithBinOp::Div => self
                                    .builder
                                    .build_float_div(left_float, right_float, "fdiv")
                                    .unwrap(),
                                ArithBinOp::Mod => self
                                    .builder
                                    .build_float_rem(left_float, right_float, "fmod")
                                    .unwrap(),
                                ArithBinOp::FloorDiv => {
                                    let div = self
                                        .builder
                                        .build_float_div(left_float, right_float, "fdiv")
                                        .unwrap();
                                    let floor_fn = self
                                        .module
                                        .get_function("llvm.floor.f64")
                                        .unwrap_or_else(|| {
                                            let f64_type = self.context.f64_type();
                                            let fn_type =
                                                f64_type.fn_type(&[f64_type.into()], false);
                                            self.module.add_function(
                                                "llvm.floor.f64",
                                                fn_type,
                                                None,
                                            )
                                        });
                                    let call = self
                                        .builder
                                        .build_call(floor_fn, &[div.into()], "floordiv")
                                        .unwrap();
                                    self.extract_call_value(call).into_float_value()
                                }
                                ArithBinOp::Pow => {
                                    let pow_fn = self
                                        .module
                                        .get_function("llvm.pow.f64")
                                        .unwrap_or_else(|| {
                                            let f64_type = self.context.f64_type();
                                            let fn_type = f64_type.fn_type(
                                                &[f64_type.into(), f64_type.into()],
                                                false,
                                            );
                                            self.module.add_function("llvm.pow.f64", fn_type, None)
                                        });
                                    let call = self
                                        .builder
                                        .build_call(
                                            pow_fn,
                                            &[left_float.into(), right_float.into()],
                                            "pow",
                                        )
                                        .unwrap();
                                    self.extract_call_value(call).into_float_value()
                                }
                            };
                            result.into()
                        } else {
                            let left_int = left_val.into_int_value();
                            let right_int = right_val.into_int_value();

                            let result = match arith_op {
                                ArithBinOp::Add => self
                                    .builder
                                    .build_int_add(left_int, right_int, "add")
                                    .unwrap(),
                                ArithBinOp::Sub => self
                                    .builder
                                    .build_int_sub(left_int, right_int, "sub")
                                    .unwrap(),
                                ArithBinOp::Mul => self
                                    .builder
                                    .build_int_mul(left_int, right_int, "mul")
                                    .unwrap(),
                                ArithBinOp::Div => self
                                    .builder
                                    .build_int_signed_div(left_int, right_int, "div")
                                    .unwrap(),
                                ArithBinOp::Mod => self
                                    .builder
                                    .build_int_signed_rem(left_int, right_int, "mod")
                                    .unwrap(),
                                ArithBinOp::FloorDiv => {
                                    // Python floor division: floor toward -infinity
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
                                        .build_int_compare(
                                            IntPredicate::SLT,
                                            xor_val,
                                            zero,
                                            "signs_diff",
                                        )
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
                                ArithBinOp::Pow => {
                                    let pow_fn = self.get_or_declare_function(
                                        "__tython_pow_int",
                                        &[ValueType::Int, ValueType::Int],
                                        Some(ValueType::Int),
                                    );
                                    let call = self
                                        .builder
                                        .build_call(
                                            pow_fn,
                                            &[left_int.into(), right_int.into()],
                                            "ipow",
                                        )
                                        .unwrap();
                                    self.extract_call_value(call).into_int_value()
                                }
                            };
                            result.into()
                        }
                    }

                    TypedBinOp::Bitwise(bitwise_op) => {
                        // Bitwise operations are always on integers
                        let left_int = left_val.into_int_value();
                        let right_int = right_val.into_int_value();

                        let result = match bitwise_op {
                            BitwiseBinOp::BitAnd => self
                                .builder
                                .build_and(left_int, right_int, "bitand")
                                .unwrap(),
                            BitwiseBinOp::BitOr => {
                                self.builder.build_or(left_int, right_int, "bitor").unwrap()
                            }
                            BitwiseBinOp::BitXor => self
                                .builder
                                .build_xor(left_int, right_int, "bitxor")
                                .unwrap(),
                            BitwiseBinOp::LShift => self
                                .builder
                                .build_left_shift(left_int, right_int, "lshift")
                                .unwrap(),
                            BitwiseBinOp::RShift => self
                                .builder
                                .build_right_shift(left_int, right_int, true, "rshift")
                                .unwrap(),
                        };
                        result.into()
                    }
                }
            }

            TirExprKind::Call { func, args } => {
                let arg_types: Vec<ValueType> = args.iter().map(|a| a.ty.clone()).collect();
                let function =
                    self.get_or_declare_function(func, &arg_types, Some(expr.ty.clone()));
                let arg_metadata = self.codegen_call_args(args);
                let call_site = self
                    .builder
                    .build_call(function, &arg_metadata, "call")
                    .unwrap();
                self.extract_call_value(call_site)
            }

            TirExprKind::ExternalCall { func, args } => {
                let function = self.get_or_declare_function(
                    func.symbol(),
                    &func.param_types(),
                    func.return_type(),
                );
                let arg_metadata = self.codegen_call_args(args);
                let call_site = self
                    .builder
                    .build_call(function, &arg_metadata, "ext_call")
                    .unwrap();
                self.extract_call_value(call_site)
            }

            TirExprKind::Cast { kind, arg } => {
                let arg_val = self.codegen_expr(arg);
                match kind {
                    CastKind::FloatToInt => self
                        .builder
                        .build_float_to_signed_int(
                            arg_val.into_float_value(),
                            self.i64_type(),
                            "ftoi",
                        )
                        .unwrap()
                        .into(),

                    CastKind::IntToFloat => self
                        .builder
                        .build_signed_int_to_float(
                            arg_val.into_int_value(),
                            self.f64_type(),
                            "itof",
                        )
                        .unwrap()
                        .into(),

                    CastKind::BoolToFloat => self
                        .builder
                        .build_signed_int_to_float(
                            arg_val.into_int_value(),
                            self.f64_type(),
                            "btof",
                        )
                        .unwrap()
                        .into(),

                    CastKind::IntToBool => {
                        let cmp = self.build_int_truthiness_check(arg_val.into_int_value(), "itob");
                        self.builder
                            .build_int_z_extend(cmp, self.i64_type(), "zext_bool")
                            .unwrap()
                            .into()
                    }

                    CastKind::FloatToBool => {
                        let cmp =
                            self.build_float_truthiness_check(arg_val.into_float_value(), "ftob");
                        self.builder
                            .build_int_z_extend(cmp, self.i64_type(), "zext_bool")
                            .unwrap()
                            .into()
                    }

                    CastKind::BoolToInt => arg_val, // same representation
                }
            }

            TirExprKind::Compare { op, left, right } => {
                let left_val = self.codegen_expr(left);
                let right_val = self.codegen_expr(right);

                let cmp_result = if left.ty == ValueType::Float {
                    self.builder
                        .build_float_compare(
                            Self::float_predicate(op),
                            left_val.into_float_value(),
                            right_val.into_float_value(),
                            "fcmp",
                        )
                        .unwrap()
                } else {
                    self.builder
                        .build_int_compare(
                            Self::int_predicate(op),
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
                        if operand.ty == ValueType::Float {
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

            // ── class expressions ────────────────────────────────────
            TirExprKind::Construct {
                class_name,
                init_mangled_name,
                args,
            } => {
                let struct_type = self.class_types[class_name.as_str()];

                // Allocate heap memory for the struct
                let size = struct_type.size_of().unwrap();
                let size_i64 = self
                    .builder
                    .build_int_cast(size, self.i64_type(), "size_i64")
                    .unwrap();
                let malloc_fn = self.get_or_declare_malloc();
                let call_site = self
                    .builder
                    .build_call(malloc_fn, &[size_i64.into()], "malloc")
                    .unwrap();
                let ptr = self.extract_call_value(call_site).into_pointer_value();

                // Build full arg list: [self_ptr, ...args]
                let mut init_args: Vec<inkwell::values::BasicMetadataValueEnum> = vec![ptr.into()];
                init_args.extend(self.codegen_call_args(args));

                // Declare/get __init__ function
                let mut param_types = vec![ValueType::Class(class_name.clone())];
                param_types.extend(args.iter().map(|a| a.ty.clone()));
                let init_fn = self.get_or_declare_function(init_mangled_name, &param_types, None);

                self.builder
                    .build_call(init_fn, &init_args, "init")
                    .unwrap();

                ptr.into()
            }

            TirExprKind::GetField {
                object,
                field_name: _,
                field_index,
            } => {
                let obj_ptr = self.codegen_expr(object).into_pointer_value();
                let class_name = match &object.ty {
                    ValueType::Class(name) => name,
                    _ => unreachable!("ICE: GetField on non-class type"),
                };
                let struct_type = self.class_types[class_name.as_str()];

                let field_ptr = self
                    .builder
                    .build_struct_gep(struct_type, obj_ptr, *field_index as u32, "field_ptr")
                    .unwrap();

                let field_llvm_type = self.get_llvm_type(&expr.ty);
                self.builder
                    .build_load(field_llvm_type, field_ptr, "field_val")
                    .unwrap()
            }

            TirExprKind::MethodCall {
                object,
                method_mangled_name,
                args,
            } => {
                let self_val = self.codegen_expr(object);

                // Build full arg list: [self, ...args]
                let mut all_meta: Vec<inkwell::values::BasicMetadataValueEnum> =
                    vec![self_val.into()];
                all_meta.extend(self.codegen_call_args(args));

                // Declare/get method function
                let mut param_types = vec![object.ty.clone()];
                param_types.extend(args.iter().map(|a| a.ty.clone()));
                let method_fn = self.get_or_declare_function(
                    method_mangled_name,
                    &param_types,
                    Some(expr.ty.clone()),
                );

                let call_site = self
                    .builder
                    .build_call(method_fn, &all_meta, "method_call")
                    .unwrap();

                self.extract_call_value(call_site)
            }
        }
    }

    pub fn add_c_main_wrapper(&mut self, entry_main_name: &str) {
        let c_main_type = self.context.i32_type().fn_type(&[], false);
        let c_main = self.module.add_function("main", c_main_type, None);
        let entry = self.context.append_basic_block(c_main, "entry");
        self.builder.position_at_end(entry);

        match self.module.get_function(entry_main_name) {
            Some(entry_fn) => {
                let result = self.builder.build_call(entry_fn, &[], "call_main").unwrap();

                // Check if the entry function returns void via LLVM type
                if entry_fn.get_type().get_return_type().is_none() {
                    self.builder
                        .build_return(Some(&self.context.i32_type().const_int(0, false)))
                        .unwrap();
                } else {
                    let return_val = self.extract_call_value(result);
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
            None => {
                // No entry function — return 0
                self.builder
                    .build_return(Some(&self.context.i32_type().const_int(0, false)))
                    .unwrap();
            }
        }
    }
}
