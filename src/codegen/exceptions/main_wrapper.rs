use inkwell::AddressSpace;

use super::super::runtime_fn::RuntimeFn;
use super::super::Codegen;

impl<'ctx> Codegen<'ctx> {
    pub fn add_c_main_wrapper(&mut self, entry_main_name: &str) {
        let c_main_type = self.context.i32_type().fn_type(&[], false);
        let c_main = self.module.add_function("main", c_main_type, None);

        let personality = self.get_runtime_fn(RuntimeFn::Personality);
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

        let begin_catch = self.get_runtime_fn(RuntimeFn::CxaBeginCatch);
        let caught = emit!(self.build_call(begin_catch, &[exc_ptr.into()], "caught"));
        let caught_ptr = self.extract_call_value(caught);

        let type_tag_fn = self.get_runtime_fn(RuntimeFn::CaughtTypeTag);
        let tag = emit!(self.build_call(type_tag_fn, &[caught_ptr.into()], "tag"));
        let tag_val = self.extract_call_value(tag);

        let message_fn = self.get_runtime_fn(RuntimeFn::CaughtMessage);
        let msg = emit!(self.build_call(message_fn, &[caught_ptr.into()], "msg"));
        let msg_val = self.extract_call_value(msg);

        let end_catch = self.get_runtime_fn(RuntimeFn::CxaEndCatch);
        emit!(self.build_call(end_catch, &[], "end_catch"));

        let print_fn = self.get_runtime_fn(RuntimeFn::PrintUnhandled);
        emit!(self.build_call(print_fn, &[tag_val.into(), msg_val.into()], "print_exc"));

        emit!(self.build_return(Some(&self.context.i32_type().const_int(1, false))));
    }
}
