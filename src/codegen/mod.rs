use inkwell::basic_block::BasicBlock;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::{Linkage, Module};
use inkwell::targets::{CodeModel, InitializationConfig, RelocMode, Target, TargetMachine};
use inkwell::types::{BasicMetadataTypeEnum, BasicType, FloatType, IntType, StructType};
use inkwell::values::{BasicMetadataValueEnum, BasicValueEnum, FunctionValue, PointerValue};
use inkwell::{AddressSpace, FloatPredicate, IntPredicate, OptimizationLevel};
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

use crate::ast::ClassInfo;
use crate::tir::builtin::BuiltinFn;
use crate::tir::{OrderedCmpOp, TirExpr, TirFunction, ValueType};

use runtime_fn::{LlvmTy, RuntimeFn};

/// Shorthand for `self.builder.method(...).unwrap()`.
/// Usage: `emit!(self.build_int_add(l, r, "add"))`
macro_rules! emit {
    ($s:ident . $method:ident ( $($arg:expr),* $(,)? )) => {
        $s.builder.$method( $($arg),* ).unwrap()
    };
}

/// Generate a function mapping `OrderedCmpOp` to an LLVM predicate type.
macro_rules! predicate_map {
    ($name:ident -> $pred_ty:ty { $($variant:ident => $pred:expr),+ $(,)? }) => {
        fn $name(op: &OrderedCmpOp) -> $pred_ty {
            match op { $(OrderedCmpOp::$variant => $pred,)+ }
        }
    };
}

/// Push loop context, codegen body statements, pop, branch to continue target.
macro_rules! loop_body {
    ($self:ident, $continue_bb:expr, $break_bb:expr, $body:expr) => {{
        $self.loop_stack.push(($continue_bb, $break_bb));
        for s in $body {
            $self.codegen_stmt(s);
        }
        $self.loop_stack.pop();
        $self.branch_if_unterminated($continue_bb);
    }};
}

/// Codegen an optional else block, then branch to after_bb.
macro_rules! else_body {
    ($self:ident, $else_bb:expr, $stmts:expr, $after_bb:expr) => {
        if let Some(else_bb) = $else_bb {
            $self.builder.position_at_end(else_bb);
            for s in $stmts {
                $self.codegen_stmt(s);
            }
            $self.branch_if_unterminated($after_bb);
        }
    };
}

/// Generate a match dispatching enum variants to builder methods.
/// Each arm specifies its own arguments: `Variant => method(arg1, arg2, ..., "label")`.
macro_rules! dispatch {
    ($self:ident, $op:expr, $($Variant:path => $method:ident($($arg:expr),+ $(,)?)),+ $(,)?) => {
        match $op { $($Variant => emit!($self.$method($($arg),+)),)+ }
    };
}

mod exceptions;
mod expressions;
mod runtime_fn;
mod statements;

pub struct Codegen<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    variables: HashMap<String, PointerValue<'ctx>>,
    global_variables: HashMap<String, PointerValue<'ctx>>,
    loop_stack: Vec<(BasicBlock<'ctx>, BasicBlock<'ctx>)>,
    struct_types: HashMap<String, StructType<'ctx>>,
    /// > 0 when inside a try/except or ForIter — calls use `invoke` instead of `call`.
    try_depth: usize,
    /// Stack of unwind destinations for nested try/ForIter blocks.
    unwind_dest_stack: Vec<BasicBlock<'ctx>>,
    /// Saved exception state for bare `raise` inside except handlers.
    /// (type_tag_alloca, message_ptr_alloca)
    reraise_state: Option<(PointerValue<'ctx>, PointerValue<'ctx>)>,
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
            global_variables: HashMap::new(),
            loop_stack: Vec::new(),
            struct_types: HashMap::new(),
            try_depth: 0,
            unwind_dest_stack: Vec::new(),
            reraise_state: None,
        }
    }

    const RUNTIME_BC: &'static str = env!("RUNTIME_BC_PATH");

    /// Path to the tcmalloc static archive (.a or .lo), if built with TCMALLOC_LIB.
    const TCMALLOC_LIB: Option<&'static str> = option_env!("TCMALLOC_LIB");

    pub fn link(&self, output_path: &Path) {
        if let Err(e) = self.module.verify() {
            panic!("module verification failed:\n{}", e.to_string());
        }
        let bc_path = output_path.with_extension("o");

        self.module.write_bitcode_to_path(&bc_path);

        let mut cmd = Command::new("clang++");
        cmd.arg("-static")
            .arg("-flto")
            .arg("-O2")
            .arg("-o")
            .arg(output_path)
            .arg(&bc_path)
            .arg(Self::RUNTIME_BC);

        if let Some(tcmalloc_lib) = Self::TCMALLOC_LIB {
            cmd.arg("-Wl,--whole-archive")
                .arg(tcmalloc_lib)
                .arg("-Wl,--no-whole-archive")
                .arg("-lpthread")
                .arg("-lstdc++");
        }

        cmd.arg("-lm");

        let output = cmd.output().expect("failed to invoke clang++");
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            panic!("link failed (exit {}):\n{}", output.status, stderr);
        }
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

        self.struct_types
            .insert(class_info.name.clone(), struct_type);
    }

    fn tuple_signature_key(elem_types: &[ValueType]) -> String {
        elem_types
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("|")
    }

    fn get_or_create_tuple_struct(&mut self, elem_types: &[ValueType]) -> StructType<'ctx> {
        let key = Self::tuple_signature_key(elem_types);
        if let Some(existing) = self.struct_types.get(&key) {
            return *existing;
        }

        let struct_name = format!("__tython_tuple${}", self.struct_types.len());
        let field_types: Vec<inkwell::types::BasicTypeEnum<'ctx>> =
            elem_types.iter().map(|ty| self.get_llvm_type(ty)).collect();

        let struct_type = self.context.opaque_struct_type(&struct_name);
        struct_type.set_body(&field_types, false);
        self.struct_types.insert(key, struct_type);
        struct_type
    }

    // ── type helpers ──────────────────────────────────────────────────

    fn get_llvm_type(&self, ty: &ValueType) -> inkwell::types::BasicTypeEnum<'ctx> {
        match ty {
            ValueType::Int | ValueType::Bool => self.context.i64_type().into(),
            ValueType::Float => self.context.f64_type().into(),
            ValueType::Str
            | ValueType::Bytes
            | ValueType::ByteArray
            | ValueType::List(_)
            | ValueType::Dict(_, _)
            | ValueType::Set(_)
            | ValueType::Tuple(_)
            | ValueType::Class(_)
            | ValueType::Function { .. } => self.context.ptr_type(AddressSpace::default()).into(),
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
        emit!(self.build_int_compare(
            IntPredicate::NE,
            value,
            self.i64_type().const_int(0, false),
            label,
        ))
    }

    fn build_float_truthiness_check(
        &self,
        value: inkwell::values::FloatValue<'ctx>,
        label: &str,
    ) -> inkwell::values::IntValue<'ctx> {
        emit!(self.build_float_compare(
            FloatPredicate::ONE,
            value,
            self.f64_type().const_float(0.0),
            label,
        ))
    }

    fn build_truthiness_check_for_value(
        &self,
        value: BasicValueEnum<'ctx>,
        ty: &ValueType,
        label: &str,
    ) -> inkwell::values::IntValue<'ctx> {
        macro_rules! seq_truthiness {
            ($($variant:ident => $builtin:ident),+ $(,)?) => {
                match ty {
                    ValueType::Float => self.build_float_truthiness_check(value.into_float_value(), label),
                    $(
                        ValueType::$variant => {
                            let func = self.get_builtin(BuiltinFn::$builtin);
                            let call = emit!(self.build_call(func, &[value.into()], "len_truth"));
                            let len_val = self.extract_call_value(call).into_int_value();
                            self.build_int_truthiness_check(len_val, label)
                        }
                    )+
                    ValueType::List(_) => {
                        let func = self.get_builtin(BuiltinFn::ListLen);
                        let call = emit!(self.build_call(func, &[value.into()], "len_truth"));
                        let len_val = self.extract_call_value(call).into_int_value();
                        self.build_int_truthiness_check(len_val, label)
                    }
                    ValueType::Dict(_, _) => {
                        let func = self.get_builtin(BuiltinFn::DictLen);
                        let call = emit!(self.build_call(func, &[value.into()], "len_truth"));
                        let len_val = self.extract_call_value(call).into_int_value();
                        self.build_int_truthiness_check(len_val, label)
                    }
                    ValueType::Set(_) => {
                        let func = self.get_builtin(BuiltinFn::SetLen);
                        let call = emit!(self.build_call(func, &[value.into()], "len_truth"));
                        let len_val = self.extract_call_value(call).into_int_value();
                        self.build_int_truthiness_check(len_val, label)
                    }
                    ValueType::Tuple(elements) => self
                        .context
                        .bool_type()
                        .const_int((!elements.is_empty()) as u64, false),
                    ValueType::Class(_) => self.i64_type().const_int(1, false),
                    _ => self.build_int_truthiness_check(value.into_int_value(), label),
                }
            };
        }
        seq_truthiness! {
            Str => StrLen,
            Bytes => BytesLen,
            ByteArray => ByteArrayLen,
        }
    }

    /// Extract the return value from a call to a function known to return non-void.
    /// This is an LLVM API contract — the function has a non-void return type in IR.
    fn extract_call_value(
        &self,
        call_site: inkwell::values::CallSiteValue<'ctx>,
    ) -> BasicValueEnum<'ctx> {
        call_site.try_as_basic_value().basic().unwrap()
    }

    fn branch_if_unterminated(&self, target: inkwell::basic_block::BasicBlock<'ctx>) -> bool {
        let terminated = emit!(self.get_insert_block()).get_terminator().is_some();
        if !terminated {
            emit!(self.build_unconditional_branch(target));
        }
        terminated
    }

    fn is_current_function_module_main(&self) -> bool {
        let function = emit!(self.get_insert_block()).get_parent().unwrap();
        function
            .get_name()
            .to_str()
            .map(|n| n.contains("$$main$"))
            .unwrap_or(false)
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

    fn get_builtin(&self, builtin: BuiltinFn) -> FunctionValue<'ctx> {
        self.get_or_declare_function(
            builtin.symbol(),
            &builtin.param_types(),
            builtin.return_type(),
        )
    }

    fn resolve_llvm_ty(&self, ty: &LlvmTy) -> inkwell::types::BasicTypeEnum<'ctx> {
        match ty {
            LlvmTy::I64 => self.context.i64_type().into(),
            LlvmTy::I32 => self.context.i32_type().into(),
            LlvmTy::Ptr => self.context.ptr_type(AddressSpace::default()).into(),
        }
    }

    fn get_runtime_fn(&self, rt: RuntimeFn) -> FunctionValue<'ctx> {
        let name = rt.symbol();
        if let Some(f) = self.module.get_function(name) {
            return f;
        }

        let params: Vec<BasicMetadataTypeEnum> = rt
            .params()
            .iter()
            .map(|ty| self.resolve_llvm_ty(ty).into())
            .collect();
        let is_variadic = matches!(rt, RuntimeFn::Personality);

        let fn_type = match rt.ret() {
            None => self.context.void_type().fn_type(&params, is_variadic),
            Some(ret) => self.resolve_llvm_ty(&ret).fn_type(&params, is_variadic),
        };

        let linkage = if matches!(rt, RuntimeFn::Personality) {
            Some(Linkage::External)
        } else {
            None
        };

        let func = self.module.add_function(name, fn_type, linkage);

        if matches!(rt, RuntimeFn::CxaRethrow) {
            func.add_attribute(
                inkwell::attributes::AttributeLoc::Function,
                self.context.create_enum_attribute(
                    inkwell::attributes::Attribute::get_named_enum_kind_id("noreturn"),
                    0,
                ),
            );
        }

        func
    }

    /// Convert `BasicValueEnum` args to `BasicMetadataValueEnum` for `build_call`.
    fn to_meta_args(args: &[BasicValueEnum<'ctx>]) -> Vec<BasicMetadataValueEnum<'ctx>> {
        args.iter().copied().map(Into::into).collect()
    }

    /// Get or declare an LLVM intrinsic function by name.
    fn get_llvm_intrinsic(
        &self,
        name: &str,
        fn_type: inkwell::types::FunctionType<'ctx>,
    ) -> FunctionValue<'ctx> {
        self.module
            .get_function(name)
            .unwrap_or_else(|| self.module.add_function(name, fn_type, None))
    }

    // ── bitcast helpers ──────────────────────────────────────────────

    fn bitcast_to_i64(
        &self,
        val: BasicValueEnum<'ctx>,
        elem_ty: &ValueType,
    ) -> inkwell::values::IntValue<'ctx> {
        match elem_ty {
            ValueType::Int | ValueType::Bool => val.into_int_value(),
            ValueType::Float => {
                emit!(self.build_bit_cast(val, self.i64_type(), "f2i")).into_int_value()
            }
            _ => emit!(self.build_ptr_to_int(val.into_pointer_value(), self.i64_type(), "p2i")),
        }
    }

    fn bitcast_from_i64(
        &self,
        val: inkwell::values::IntValue<'ctx>,
        elem_ty: &ValueType,
    ) -> BasicValueEnum<'ctx> {
        match elem_ty {
            ValueType::Int | ValueType::Bool => val.into(),
            ValueType::Float => emit!(self.build_bit_cast(val, self.f64_type(), "i2f")),
            _ => emit!(self.build_int_to_ptr(
                val,
                self.context.ptr_type(AddressSpace::default()),
                "i2p"
            ))
            .into(),
        }
    }

    /// Codegen a list of TIR args into basic values.
    fn codegen_call_args(&mut self, args: &[TirExpr]) -> Vec<BasicValueEnum<'ctx>> {
        args.iter().map(|arg| self.codegen_expr(arg)).collect()
    }

    /// Codegen a call to a user-defined function, returning its value if non-void.
    fn codegen_named_call(
        &mut self,
        func: &str,
        args: &[TirExpr],
        return_type: Option<&ValueType>,
    ) -> Option<BasicValueEnum<'ctx>> {
        let arg_types: Vec<ValueType> = args.iter().map(|a| a.ty.clone()).collect();
        let function = self.get_or_declare_function(func, &arg_types, return_type.cloned());
        let arg_values = self.codegen_call_args(args);
        let call_site = self.build_call_maybe_invoke(function, &arg_values, "call", true);
        return_type.map(|_| self.extract_call_value(call_site))
    }

    /// Codegen a call to a builtin (runtime) function.
    ///
    /// Handles container-element bitcasting conventions automatically:
    /// - `ListPop`/`ListGet` return an i64 slot that is bitcast to the element type.
    /// - `DictGet`/`DictPop`/`SetPop` return an i64 slot that is bitcast.
    /// - `ListAppend`/`ListRemove`/`ListInsert`/`ListContains`/`ListIndex`/`ListCount`
    ///   take an element as the **last** argument which is bitcast *to* i64.
    fn codegen_builtin_call(
        &mut self,
        func: BuiltinFn,
        args: &[TirExpr],
        result_ty: Option<&ValueType>,
    ) -> Option<BasicValueEnum<'ctx>> {
        let function = self.get_builtin(func);

        // DictGet/DictPop need both:
        // - key (arg1) bitcasted to i64
        // - returned slot bitcasted from i64 to the value type
        if matches!(func, BuiltinFn::DictGet | BuiltinFn::DictPop) {
            let mut call_args: Vec<BasicMetadataValueEnum> = Vec::with_capacity(args.len());
            for (i, arg) in args.iter().enumerate() {
                let val = self.codegen_expr(arg);
                if i == 1 {
                    call_args.push(self.bitcast_to_i64(val, &arg.ty).into());
                } else {
                    call_args.push(val.into());
                }
            }
            let call = emit!(self.build_call(function, &call_args, "builtin_call"));
            let i64_val = self.extract_call_value(call).into_int_value();
            return Some(self.bitcast_from_i64(i64_val, result_ty.unwrap()));
        }

        // List ops returning an element stored as i64 — bitcast result
        if matches!(
            func,
            BuiltinFn::ListPop | BuiltinFn::ListGet | BuiltinFn::SetPop
        ) {
            let arg_values = self.codegen_call_args(args);
            let call =
                emit!(self.build_call(function, &Self::to_meta_args(&arg_values), "builtin_call"));
            let i64_val = self.extract_call_value(call).into_int_value();
            return Some(self.bitcast_from_i64(i64_val, result_ty.unwrap()));
        }

        // List ops where the last arg is an element — bitcast it to i64
        if matches!(
            func,
            BuiltinFn::ListContains
                | BuiltinFn::ListIndex
                | BuiltinFn::ListCount
                | BuiltinFn::ListAppend
                | BuiltinFn::ListRemove
                | BuiltinFn::ListInsert
        ) {
            let last = args.len() - 1;
            let mut call_args: Vec<BasicMetadataValueEnum> = Vec::with_capacity(args.len());
            for (i, arg) in args.iter().enumerate() {
                let val = self.codegen_expr(arg);
                if i == last {
                    call_args.push(self.bitcast_to_i64(val, &arg.ty).into());
                } else {
                    call_args.push(val.into());
                }
            }
            let call = emit!(self.build_call(function, &call_args, "builtin_call"));
            return result_ty.map(|_| self.extract_call_value(call));
        }

        // Dict ops with key in position 1; set/get/pop bitcast that key.
        if matches!(
            func,
            BuiltinFn::DictContains | BuiltinFn::DictGet | BuiltinFn::DictPop
        ) {
            let mut call_args: Vec<BasicMetadataValueEnum> = Vec::with_capacity(args.len());
            for (i, arg) in args.iter().enumerate() {
                let val = self.codegen_expr(arg);
                if i == 1 {
                    call_args.push(self.bitcast_to_i64(val, &arg.ty).into());
                } else {
                    call_args.push(val.into());
                }
            }
            let call = emit!(self.build_call(function, &call_args, "builtin_call"));
            return result_ty.map(|_| self.extract_call_value(call));
        }

        // DictSet bitcasts key (arg1) and value (arg2).
        if matches!(func, BuiltinFn::DictSet) {
            let mut call_args: Vec<BasicMetadataValueEnum> = Vec::with_capacity(args.len());
            for (i, arg) in args.iter().enumerate() {
                let val = self.codegen_expr(arg);
                if i == 1 || i == 2 {
                    call_args.push(self.bitcast_to_i64(val, &arg.ty).into());
                } else {
                    call_args.push(val.into());
                }
            }
            emit!(self.build_call(function, &call_args, "builtin_call"));
            return None;
        }

        // Set ops with element arg in position 1.
        if matches!(
            func,
            BuiltinFn::SetContains
                | BuiltinFn::SetAdd
                | BuiltinFn::SetRemove
                | BuiltinFn::SetDiscard
        ) {
            let mut call_args: Vec<BasicMetadataValueEnum> = Vec::with_capacity(args.len());
            for (i, arg) in args.iter().enumerate() {
                let val = self.codegen_expr(arg);
                if i == 1 {
                    call_args.push(self.bitcast_to_i64(val, &arg.ty).into());
                } else {
                    call_args.push(val.into());
                }
            }
            let call = emit!(self.build_call(function, &call_args, "builtin_call"));
            return result_ty.map(|_| self.extract_call_value(call));
        }

        // General case — no bitcasting
        let arg_values = self.codegen_call_args(args);
        let call =
            emit!(self.build_call(function, &Self::to_meta_args(&arg_values), "builtin_call"));
        result_ty.map(|_| self.extract_call_value(call))
    }

    /// Create an alloca in the entry basic block of the current function.
    /// Entry-block allocas are promoted to registers by LLVM's mem2reg pass
    /// and ensure stable stack offsets.
    fn build_entry_block_alloca(
        &self,
        ty: inkwell::types::BasicTypeEnum<'ctx>,
        name: &str,
    ) -> PointerValue<'ctx> {
        let function = emit!(self.get_insert_block()).get_parent().unwrap();
        let entry_bb = function.get_first_basic_block().unwrap();

        // Create a temporary builder positioned at the start of the entry block
        let entry_builder = self.context.create_builder();
        if let Some(first_instr) = entry_bb.get_first_instruction() {
            entry_builder.position_before(&first_instr);
        } else {
            entry_builder.position_at_end(entry_bb);
        }
        entry_builder.build_alloca(ty, name).unwrap()
    }

    /// Create a dead basic block after an unconditional branch (break/continue).
    fn append_dead_block(&self, label: &str) {
        let function = emit!(self.get_insert_block()).get_parent().unwrap();
        let dead_bb = self.context.append_basic_block(function, label);
        self.builder.position_at_end(dead_bb);
    }

    predicate_map!(float_predicate -> FloatPredicate {
        Eq => FloatPredicate::OEQ, NotEq => FloatPredicate::ONE,
        Lt => FloatPredicate::OLT, LtEq => FloatPredicate::OLE,
        Gt => FloatPredicate::OGT, GtEq => FloatPredicate::OGE,
    });

    predicate_map!(int_predicate -> IntPredicate {
        Eq => IntPredicate::EQ,  NotEq => IntPredicate::NE,
        Lt => IntPredicate::SLT, LtEq => IntPredicate::SLE,
        Gt => IntPredicate::SGT, GtEq => IntPredicate::SGE,
    });

    pub fn generate(&mut self, func: &TirFunction) {
        let param_types: Vec<ValueType> = func.params.iter().map(|p| p.ty.clone()).collect();
        let function =
            self.get_or_declare_function(&func.name, &param_types, func.return_type.clone());

        // Set personality function if this function contains try/except or for-iter.
        if Self::stmts_need_personality(&func.body) {
            let personality = self.get_runtime_fn(RuntimeFn::Personality);
            function.set_personality_function(personality);
        }

        let entry_bb = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(entry_bb);

        self.variables.clear();
        self.try_depth = 0;
        self.unwind_dest_stack.clear();
        for (i, param) in func.params.iter().enumerate() {
            let param_value = function.get_nth_param(i as u32).unwrap();
            let alloca = emit!(self.build_alloca(self.get_llvm_type(&param.ty), &param.name));
            emit!(self.build_store(alloca, param_value));
            self.variables.insert(param.name.clone(), alloca);
        }

        for stmt in &func.body {
            self.codegen_stmt(stmt);
        }

        let current_bb = emit!(self.get_insert_block());
        if current_bb.get_terminator().is_none() {
            if func.return_type.is_none() {
                emit!(self.build_return(None));
            } else {
                // All reachable paths already returned a value; this block
                // is dead (e.g. the merge point after a try/catch where
                // every branch returns).  Add `unreachable` so the block
                // is well-formed LLVM IR.
                emit!(self.build_unreachable());
            }
        }
    }
}
