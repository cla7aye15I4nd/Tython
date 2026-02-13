use inkwell::types::StructType;
use inkwell::values::{BasicValueEnum, CallSiteValue, FunctionValue, PointerValue};
use inkwell::AddressSpace;

use super::super::runtime_fn::RuntimeFn;
use super::super::Codegen;

impl<'ctx> Codegen<'ctx> {
    /// The LLVM struct type returned by a landingpad: `{ ptr, i32 }`.
    pub(crate) fn get_exception_landing_type(&self) -> StructType<'ctx> {
        let ptr_type = self.context.ptr_type(AddressSpace::default());
        let i32_type = self.context.i32_type();
        self.context
            .struct_type(&[ptr_type.into(), i32_type.into()], false)
    }

    /// Emit a call/invoke to `__tython_raise` (noreturn).
    /// Uses `invoke` when inside a try block so the landing pad catches it.
    pub(crate) fn emit_raise(&self, tag_val: BasicValueEnum<'ctx>, msg_val: BasicValueEnum<'ctx>) {
        let raise_fn = self.get_runtime_fn(RuntimeFn::Raise);
        if self.try_depth > 0 {
            let function = emit!(self.get_insert_block()).get_parent().unwrap();
            let dead_bb = self.context.append_basic_block(function, "raise.dead");
            let unwind_bb = *self.unwind_dest_stack.last().unwrap();
            emit!(self.build_invoke(raise_fn, &[tag_val, msg_val], dead_bb, unwind_bb, "raise"));
            self.builder.position_at_end(dead_bb);
        } else {
            emit!(self.build_call(raise_fn, &[tag_val.into(), msg_val.into()], "raise"));
        }
        emit!(self.build_unreachable());
    }

    /// Emit a catch-all landing pad, save the LP struct for potential resume,
    /// and call `__cxa_begin_catch`. Returns the caught object pointer.
    pub(crate) fn build_landingpad_catch_all(
        &self,
        lp_alloca: PointerValue<'ctx>,
        label: &str,
    ) -> BasicValueEnum<'ctx> {
        let landing_type = self.get_exception_landing_type();
        let ptr_type = self.context.ptr_type(AddressSpace::default());
        let personality = self.get_runtime_fn(RuntimeFn::Personality);
        let null_ptr = ptr_type.const_null();

        let lp = emit!(self.build_landing_pad(
            landing_type,
            personality,
            &[null_ptr.into()],
            false,
            label,
        ));
        emit!(self.build_store(lp_alloca, lp));

        let exc_ptr =
            emit!(self.build_extract_value(lp.into_struct_value(), 0, &format!("{}.exc", label),));
        let begin_catch = self.get_runtime_fn(RuntimeFn::CxaBeginCatch);
        let caught =
            emit!(self.build_call(begin_catch, &[exc_ptr.into()], &format!("{}.caught", label),));
        self.extract_call_value(caught)
    }

    /// Emit `__cxa_end_catch` followed by a `resume` with the saved landing-pad value.
    pub(crate) fn end_catch_and_resume(&self, lp_alloca: PointerValue<'ctx>) {
        let end_catch = self.get_runtime_fn(RuntimeFn::CxaEndCatch);
        emit!(self.build_call(end_catch, &[], "end_catch"));
        let landing_type = self.get_exception_landing_type();
        let lp_val = emit!(self.build_load(landing_type, lp_alloca, "lp_resume"));
        emit!(self.build_resume(lp_val));
    }

    /// Emit a function call. When inside a try block (`try_depth > 0`) and
    /// `may_throw` is true, emits an `invoke` instruction that unwinds to the
    /// current landing pad; otherwise emits a regular `call`.
    pub(crate) fn build_call_maybe_invoke(
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
}
