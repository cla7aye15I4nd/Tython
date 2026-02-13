use inkwell::values::PointerValue;

use crate::tir::{TirFunction, ValueType};

use super::runtime_fn::RuntimeFn;
use super::Codegen;

impl<'ctx> Codegen<'ctx> {
    pub(crate) fn branch_if_unterminated(
        &self,
        target: inkwell::basic_block::BasicBlock<'ctx>,
    ) -> bool {
        let terminated = emit!(self.get_insert_block()).get_terminator().is_some();
        if !terminated {
            emit!(self.build_unconditional_branch(target));
        }
        terminated
    }

    /// Create an alloca in the entry basic block of the current function.
    /// Entry-block allocas are promoted to registers by LLVM's mem2reg pass
    /// and ensure stable stack offsets.
    pub(crate) fn build_entry_block_alloca(
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
    pub(crate) fn append_dead_block(&self, label: &str) {
        let function = emit!(self.get_insert_block()).get_parent().unwrap();
        let dead_bb = self.context.append_basic_block(function, label);
        self.builder.position_at_end(dead_bb);
    }

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
