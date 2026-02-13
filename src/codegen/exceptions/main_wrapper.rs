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
        let lp_alloca = self
            .build_entry_block_alloca(self.get_exception_landing_type().into(), "main.lp_alloca");
        let caught_ptr = self.build_landingpad_catch_all(lp_alloca, "main.lp");
        let (tag_val, msg_val) = self.get_caught_tag_and_message(caught_ptr, "tag", "msg");
        self.emit_end_catch("end_catch");

        let print_fn = self.get_runtime_fn(RuntimeFn::PrintUnhandled);
        emit!(self.build_call(print_fn, &[tag_val.into(), msg_val.into()], "print_exc"));

        emit!(self.build_return(Some(&self.context.i32_type().const_int(1, false))));
    }
}
