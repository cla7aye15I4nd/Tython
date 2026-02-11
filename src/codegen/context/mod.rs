use inkwell::basic_block::BasicBlock;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::{Linkage, Module};
use inkwell::targets::{CodeModel, InitializationConfig, RelocMode, Target, TargetMachine};
use inkwell::types::{BasicMetadataTypeEnum, BasicType, FloatType, IntType, StructType};
use inkwell::values::{
    BasicMetadataValueEnum, BasicValueEnum, CallSiteValue, FunctionValue, PointerValue,
};
use inkwell::{AddressSpace, FloatPredicate, IntPredicate, OptimizationLevel};
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

use crate::ast::ClassInfo;
use crate::tir::{OrderedCmpOp, TirExpr, TirFunction, TirStmt, ValueType};

/// Shorthand for `self.builder.method(...).unwrap()`.
/// Usage: `emit!(self.build_int_add(l, r, "add"))`
macro_rules! emit {
    ($s:ident . $method:ident ( $($arg:expr),* $(,)? )) => {
        $s.builder.$method( $($arg),* ).unwrap()
    };
}

mod expressions;
mod statements;

pub struct Codegen<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    variables: HashMap<String, PointerValue<'ctx>>,
    loop_stack: Vec<(BasicBlock<'ctx>, BasicBlock<'ctx>)>,
    class_types: HashMap<String, StructType<'ctx>>,
    tuple_types: HashMap<String, StructType<'ctx>>,
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
            loop_stack: Vec::new(),
            class_types: HashMap::new(),
            tuple_types: HashMap::new(),
            try_depth: 0,
            unwind_dest_stack: Vec::new(),
            reraise_state: None,
        }
    }

    const RUNTIME_BC: &'static str = env!("RUNTIME_BC_PATH");

    pub fn link(&self, output_path: &Path) {
        let bc_path = output_path.with_extension("o");

        self.module.write_bitcode_to_path(&bc_path);

        Command::new("clang++")
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

    fn tuple_signature_key(elem_types: &[ValueType]) -> String {
        elem_types
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("|")
    }

    fn get_or_create_tuple_struct(&mut self, elem_types: &[ValueType]) -> StructType<'ctx> {
        let key = Self::tuple_signature_key(elem_types);
        if let Some(existing) = self.tuple_types.get(&key) {
            return *existing;
        }

        let struct_name = format!("__tython_tuple${}", self.tuple_types.len());
        let field_types: Vec<inkwell::types::BasicTypeEnum<'ctx>> =
            elem_types.iter().map(|ty| self.get_llvm_type(ty)).collect();

        let struct_type = self.context.opaque_struct_type(&struct_name);
        struct_type.set_body(&field_types, false);
        self.tuple_types.insert(key, struct_type);
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
                            use crate::tir::builtin::BuiltinFn;
                            let len_fn = BuiltinFn::$builtin;
                            let func = self.get_or_declare_function(
                                len_fn.symbol(),
                                &len_fn.param_types(),
                                len_fn.return_type(),
                            );
                            let call = emit!(self.build_call(func, &[value.into()], "len_truth"));
                            let len_val = self.extract_call_value(call).into_int_value();
                            self.build_int_truthiness_check(len_val, label)
                        }
                    )+
                    ValueType::List(_) => {
                        use crate::tir::builtin::BuiltinFn;
                        let len_fn = BuiltinFn::ListLen;
                        let func = self.get_or_declare_function(
                            len_fn.symbol(),
                            &len_fn.param_types(),
                            len_fn.return_type(),
                        );
                        let call = emit!(self.build_call(func, &[value.into()], "len_truth"));
                        let len_val = self.extract_call_value(call).into_int_value();
                        self.build_int_truthiness_check(len_val, label)
                    }
                    ValueType::Tuple(elements) => self
                        .context
                        .bool_type()
                        .const_int((!elements.is_empty()) as u64, false),
                    ValueType::Class(_) | ValueType::Function { .. } => self.i64_type().const_int(1, false),
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

    fn get_or_declare_str_new(&self) -> FunctionValue<'ctx> {
        self.module
            .get_function("__tython_str_new")
            .unwrap_or_else(|| {
                let ptr_type = self.context.ptr_type(AddressSpace::default());
                let i64_type = self.context.i64_type();
                let fn_type = ptr_type.fn_type(&[ptr_type.into(), i64_type.into()], false);
                self.module.add_function("__tython_str_new", fn_type, None)
            })
    }

    fn get_or_declare_bytes_new(&self) -> FunctionValue<'ctx> {
        self.module
            .get_function("__tython_bytes_new")
            .unwrap_or_else(|| {
                let ptr_type = self.context.ptr_type(AddressSpace::default());
                let i64_type = self.context.i64_type();
                let fn_type = ptr_type.fn_type(&[ptr_type.into(), i64_type.into()], false);
                self.module
                    .add_function("__tython_bytes_new", fn_type, None)
            })
    }

    fn get_or_declare_list_new(&self) -> FunctionValue<'ctx> {
        self.module
            .get_function("__tython_list_new")
            .unwrap_or_else(|| {
                let ptr_type = self.context.ptr_type(AddressSpace::default());
                let i64_type = self.context.i64_type();
                let fn_type = ptr_type.fn_type(&[ptr_type.into(), i64_type.into()], false);
                self.module.add_function("__tython_list_new", fn_type, None)
            })
    }

    fn get_or_declare_list_set(&self) -> FunctionValue<'ctx> {
        self.module
            .get_function("__tython_list_set")
            .unwrap_or_else(|| {
                let ptr_type = self.context.ptr_type(AddressSpace::default());
                let i64_type = self.context.i64_type();
                let fn_type = self
                    .context
                    .void_type()
                    .fn_type(&[ptr_type.into(), i64_type.into(), i64_type.into()], false);
                self.module.add_function("__tython_list_set", fn_type, None)
            })
    }

    // ── exception helpers (LLVM landingpad-based) ────────────────────

    fn get_or_declare_personality_fn(&self) -> FunctionValue<'ctx> {
        self.module
            .get_function("__gxx_personality_v0")
            .unwrap_or_else(|| {
                let i32_type = self.context.i32_type();
                let fn_type = i32_type.fn_type(&[], true); // variadic
                self.module
                    .add_function("__gxx_personality_v0", fn_type, Some(Linkage::External))
            })
    }

    fn get_or_declare_exc_raise(&self) -> FunctionValue<'ctx> {
        self.module
            .get_function("__tython_raise")
            .unwrap_or_else(|| {
                let i64_type = self.context.i64_type();
                let ptr_type = self.context.ptr_type(AddressSpace::default());
                let fn_type = self
                    .context
                    .void_type()
                    .fn_type(&[i64_type.into(), ptr_type.into()], false);
                self.module.add_function("__tython_raise", fn_type, None)
            })
    }

    fn get_or_declare_cxa_begin_catch(&self) -> FunctionValue<'ctx> {
        self.module
            .get_function("__cxa_begin_catch")
            .unwrap_or_else(|| {
                let ptr_type = self.context.ptr_type(AddressSpace::default());
                let fn_type = ptr_type.fn_type(&[ptr_type.into()], false);
                self.module.add_function("__cxa_begin_catch", fn_type, None)
            })
    }

    fn get_or_declare_cxa_end_catch(&self) -> FunctionValue<'ctx> {
        self.module
            .get_function("__cxa_end_catch")
            .unwrap_or_else(|| {
                let fn_type = self.context.void_type().fn_type(&[], false);
                self.module.add_function("__cxa_end_catch", fn_type, None)
            })
    }

    fn get_or_declare_cxa_rethrow(&self) -> FunctionValue<'ctx> {
        self.module
            .get_function("__cxa_rethrow")
            .unwrap_or_else(|| {
                let fn_type = self.context.void_type().fn_type(&[], false);
                let func = self.module.add_function("__cxa_rethrow", fn_type, None);
                func.add_attribute(
                    inkwell::attributes::AttributeLoc::Function,
                    self.context.create_enum_attribute(
                        inkwell::attributes::Attribute::get_named_enum_kind_id("noreturn"),
                        0,
                    ),
                );
                func
            })
    }

    fn get_or_declare_caught_type_tag(&self) -> FunctionValue<'ctx> {
        self.module
            .get_function("__tython_caught_type_tag")
            .unwrap_or_else(|| {
                let i64_type = self.context.i64_type();
                let ptr_type = self.context.ptr_type(AddressSpace::default());
                let fn_type = i64_type.fn_type(&[ptr_type.into()], false);
                self.module
                    .add_function("__tython_caught_type_tag", fn_type, None)
            })
    }

    fn get_or_declare_caught_message(&self) -> FunctionValue<'ctx> {
        self.module
            .get_function("__tython_caught_message")
            .unwrap_or_else(|| {
                let ptr_type = self.context.ptr_type(AddressSpace::default());
                let fn_type = ptr_type.fn_type(&[ptr_type.into()], false);
                self.module
                    .add_function("__tython_caught_message", fn_type, None)
            })
    }

    fn get_or_declare_caught_matches(&self) -> FunctionValue<'ctx> {
        self.module
            .get_function("__tython_caught_matches")
            .unwrap_or_else(|| {
                let i64_type = self.context.i64_type();
                let ptr_type = self.context.ptr_type(AddressSpace::default());
                let fn_type = i64_type.fn_type(&[ptr_type.into(), i64_type.into()], false);
                self.module
                    .add_function("__tython_caught_matches", fn_type, None)
            })
    }

    fn get_or_declare_print_unhandled(&self) -> FunctionValue<'ctx> {
        self.module
            .get_function("__tython_print_unhandled")
            .unwrap_or_else(|| {
                let i64_type = self.context.i64_type();
                let ptr_type = self.context.ptr_type(AddressSpace::default());
                let fn_type = self
                    .context
                    .void_type()
                    .fn_type(&[i64_type.into(), ptr_type.into()], false);
                self.module
                    .add_function("__tython_print_unhandled", fn_type, None)
            })
    }

    /// The LLVM struct type returned by a landingpad: `{ ptr, i32 }`.
    fn get_exception_landing_type(&self) -> StructType<'ctx> {
        let ptr_type = self.context.ptr_type(AddressSpace::default());
        let i32_type = self.context.i32_type();
        self.context
            .struct_type(&[ptr_type.into(), i32_type.into()], false)
    }

    /// Convert `BasicValueEnum` args to `BasicMetadataValueEnum` for `build_call`.
    fn to_meta_args(args: &[BasicValueEnum<'ctx>]) -> Vec<BasicMetadataValueEnum<'ctx>> {
        args.iter().copied().map(Into::into).collect()
    }

    /// Emit a function call. When inside a try block (`try_depth > 0`) and
    /// `may_throw` is true, emits an `invoke` instruction that unwinds to the
    /// current landing pad; otherwise emits a regular `call`.
    fn build_call_maybe_invoke(
        &self,
        function: FunctionValue<'ctx>,
        args: &[BasicValueEnum<'ctx>],
        name: &str,
        may_throw: bool,
    ) -> CallSiteValue<'ctx> {
        if may_throw && self.try_depth > 0 {
            let current_fn = emit!(self.get_insert_block()).get_parent().unwrap();
            let cont_bb = self
                .context
                .append_basic_block(current_fn, &format!("{}.cont", name));
            let unwind_bb = *self
                .unwind_dest_stack
                .last()
                .expect("ICE: try_depth > 0 but no unwind destination");

            let call_site = emit!(self.build_invoke(function, args, cont_bb, unwind_bb, name));

            self.builder.position_at_end(cont_bb);
            call_site
        } else {
            let meta_args = Self::to_meta_args(args);
            emit!(self.build_call(function, &meta_args, name))
        }
    }

    /// Recursively check whether any statement contains TryCatch or ForIter,
    /// which means the enclosing function needs a personality function.
    fn stmts_need_personality(stmts: &[TirStmt]) -> bool {
        for stmt in stmts {
            match stmt {
                TirStmt::TryCatch { .. } | TirStmt::ForIter { .. } => return true,
                TirStmt::If {
                    then_body,
                    else_body,
                    ..
                } => {
                    if Self::stmts_need_personality(then_body)
                        || Self::stmts_need_personality(else_body)
                    {
                        return true;
                    }
                }
                TirStmt::While { body, .. }
                | TirStmt::ForRange { body, .. }
                | TirStmt::ForList { body, .. } => {
                    if Self::stmts_need_personality(body) {
                        return true;
                    }
                }
                TirStmt::Let { .. }
                | TirStmt::Return(_)
                | TirStmt::Expr(_)
                | TirStmt::VoidCall { .. }
                | TirStmt::Break
                | TirStmt::Continue
                | TirStmt::SetField { .. }
                | TirStmt::ListSet { .. }
                | TirStmt::Raise { .. } => {}
            }
        }
        false
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

    /// Map an OrderedCmpOp to LLVM float predicate.
    fn float_predicate(op: &OrderedCmpOp) -> FloatPredicate {
        match op {
            OrderedCmpOp::Eq => FloatPredicate::OEQ,
            OrderedCmpOp::NotEq => FloatPredicate::ONE,
            OrderedCmpOp::Lt => FloatPredicate::OLT,
            OrderedCmpOp::LtEq => FloatPredicate::OLE,
            OrderedCmpOp::Gt => FloatPredicate::OGT,
            OrderedCmpOp::GtEq => FloatPredicate::OGE,
        }
    }

    /// Map an OrderedCmpOp to LLVM int predicate.
    fn int_predicate(op: &OrderedCmpOp) -> IntPredicate {
        match op {
            OrderedCmpOp::Eq => IntPredicate::EQ,
            OrderedCmpOp::NotEq => IntPredicate::NE,
            OrderedCmpOp::Lt => IntPredicate::SLT,
            OrderedCmpOp::LtEq => IntPredicate::SLE,
            OrderedCmpOp::Gt => IntPredicate::SGT,
            OrderedCmpOp::GtEq => IntPredicate::SGE,
        }
    }

    pub fn generate(&mut self, func: &TirFunction) {
        let param_types: Vec<ValueType> = func.params.iter().map(|p| p.ty.clone()).collect();
        let function =
            self.get_or_declare_function(&func.name, &param_types, func.return_type.clone());

        // Set personality function if this function contains try/except or for-iter.
        if Self::stmts_need_personality(&func.body) {
            let personality = self.get_or_declare_personality_fn();
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

    pub fn add_c_main_wrapper(&mut self, entry_main_name: &str) {
        let c_main_type = self.context.i32_type().fn_type(&[], false);
        let c_main = self.module.add_function("main", c_main_type, None);

        // Set personality so we can catch unhandled exceptions.
        let personality = self.get_or_declare_personality_fn();
        c_main.set_personality_function(personality);

        let entry = self.context.append_basic_block(c_main, "entry");
        let normal_bb = self.context.append_basic_block(c_main, "normal");
        let unwind_bb = self.context.append_basic_block(c_main, "unwind");

        // entry: invoke the user's __main__ function
        self.builder.position_at_end(entry);
        let entry_fn = self.module.get_function(entry_main_name).unwrap();
        emit!(self.build_invoke(entry_fn, &[], normal_bb, unwind_bb, "call_main"));

        // normal: return 0
        self.builder.position_at_end(normal_bb);
        emit!(self.build_return(Some(&self.context.i32_type().const_int(0, false))));

        // unwind: catch all, print error, return 1
        self.builder.position_at_end(unwind_bb);
        let landing_type = self.get_exception_landing_type();
        let null_ptr = self.context.ptr_type(AddressSpace::default()).const_null();
        let lp = emit!(self.build_landing_pad(
            landing_type,
            personality,
            &[null_ptr.into()],
            false,
            "lp"
        ));

        let exc_ptr = emit!(self.build_extract_value(lp.into_struct_value(), 0, "exc_ptr"));

        let begin_catch = self.get_or_declare_cxa_begin_catch();
        let caught = emit!(self.build_call(begin_catch, &[exc_ptr.into()], "caught"));
        let caught_ptr = self.extract_call_value(caught);

        let type_tag_fn = self.get_or_declare_caught_type_tag();
        let tag = emit!(self.build_call(type_tag_fn, &[caught_ptr.into()], "tag"));
        let tag_val = self.extract_call_value(tag);

        let message_fn = self.get_or_declare_caught_message();
        let msg = emit!(self.build_call(message_fn, &[caught_ptr.into()], "msg"));
        let msg_val = self.extract_call_value(msg);

        let end_catch = self.get_or_declare_cxa_end_catch();
        emit!(self.build_call(end_catch, &[], "end_catch"));

        let print_fn = self.get_or_declare_print_unhandled();
        emit!(self.build_call(print_fn, &[tag_val.into(), msg_val.into()], "print_exc"));

        emit!(self.build_return(Some(&self.context.i32_type().const_int(1, false))));
    }
}
