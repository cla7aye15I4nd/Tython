use inkwell::{AddressSpace, IntPredicate};

use crate::tir::{TirStmt, ValueType};

use super::super::runtime_fn::RuntimeFn;
use super::super::Codegen;

impl<'ctx> Codegen<'ctx> {
    pub(crate) fn codegen_for_iter(
        &mut self,
        loop_var: &str,
        loop_var_ty: &ValueType,
        iterator_var: &str,
        iterator_class: &str,
        next_mangled: &str,
        body: &[TirStmt],
        else_body: &[TirStmt],
    ) {
        let function = emit!(self.get_insert_block()).get_parent().unwrap();
        let ptr_type = self.context.ptr_type(AddressSpace::default());
        let landing_type = self.get_exception_landing_type();

        let call_next_bb = self
            .context
            .append_basic_block(function, "foriter.call_next");
        let got_next_bb = self
            .context
            .append_basic_block(function, "foriter.got_next");
        let lp_bb = self.context.append_basic_block(function, "foriter.lp");
        let stop_bb = self.context.append_basic_block(function, "foriter.stop");
        let reraise_bb = self.context.append_basic_block(function, "foriter.reraise");
        let body_bb = self.context.append_basic_block(function, "foriter.body");
        let else_bb = if !else_body.is_empty() {
            Some(self.context.append_basic_block(function, "foriter.else"))
        } else {
            None
        };
        let after_bb = self.context.append_basic_block(function, "foriter.after");

        let iter_ptr = self.variables[iterator_var];

        if !self.variables.contains_key(loop_var) {
            let alloca = self.build_entry_block_alloca(self.get_llvm_type(loop_var_ty), loop_var);
            self.variables.insert(loop_var.to_string(), alloca);
        }

        let lp_alloca = self.build_entry_block_alloca(landing_type.into(), "foriter.lp_alloca");

        emit!(self.build_unconditional_branch(call_next_bb));

        // ── Call __next__ via invoke ───────────────────────────────
        self.builder.position_at_end(call_next_bb);
        let iter_val = emit!(self.build_load(ptr_type, iter_ptr, "foriter.iter"));
        let next_fn = self.get_or_declare_function(
            next_mangled,
            &[ValueType::Class(iterator_class.to_string())],
            Some(loop_var_ty.clone()),
        );
        let call_site =
            emit!(self.build_invoke(next_fn, &[iter_val], got_next_bb, lp_bb, "foriter.next"));

        // ── Got next: store and enter body ────────────────────────
        self.builder.position_at_end(got_next_bb);
        let next_val = self.extract_call_value(call_site);
        let loop_var_ptr = self.variables[loop_var];
        emit!(self.build_store(loop_var_ptr, next_val));
        emit!(self.build_unconditional_branch(body_bb));

        // ── Landing pad: check StopIteration ──────────────────────
        self.builder.position_at_end(lp_bb);
        let caught_ptr = self.build_landingpad_catch_all(lp_alloca, "foriter.lp");

        let type_tag_fn = self.get_runtime_fn(RuntimeFn::CaughtTypeTag);
        let tag = emit!(self.build_call(type_tag_fn, &[caught_ptr.into()], "foriter.tag"));
        let tag_val = self.extract_call_value(tag).into_int_value();

        let stop_tag = self.i64_type().const_int(2, false); // TYTHON_EXC_STOP_ITERATION
        let is_stop =
            emit!(self.build_int_compare(IntPredicate::EQ, tag_val, stop_tag, "foriter.is_stop"));
        emit!(self.build_conditional_branch(is_stop, stop_bb, reraise_bb));

        // ── Stop: end catch and exit loop (→ else or after) ─────
        self.builder.position_at_end(stop_bb);
        let end_catch = self.get_runtime_fn(RuntimeFn::CxaEndCatch);
        emit!(self.build_call(end_catch, &[], "foriter.end_catch"));
        let stop_dest = else_bb.unwrap_or(after_bb);
        emit!(self.build_unconditional_branch(stop_dest));

        // ── Re-raise: not StopIteration ───────────────────────────
        self.builder.position_at_end(reraise_bb);
        self.end_catch_and_resume(lp_alloca);

        // ── Body ──────────────────────────────────────────────────
        self.builder.position_at_end(body_bb);
        loop_body!(self, call_next_bb, after_bb, body);
        else_body!(self, else_bb, else_body, after_bb);
        self.builder.position_at_end(after_bb);
    }
}
