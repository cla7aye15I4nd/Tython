use super::super::Codegen;

impl<'ctx> Codegen<'ctx> {
    pub fn create_runtime_entry_point(&mut self, entry_main_name: &str) {
        // Create the wrapper function that runtime expects: void __tython_user_main(void)
        let wrapper_type = self.context.void_type().fn_type(&[], false);
        let wrapper_fn = self
            .module
            .add_function("__tython_user_main", wrapper_type, None);

        // Get the actual synthetic main function (returns i64)
        let synthetic_main = self
            .module
            .get_function(entry_main_name)
            .expect("Entry point function not found");

        // Build wrapper that calls synthetic main and discards return value:
        // void __tython_user_main() {
        //     synthetic_main();  // ignore i64 return
        // }
        let entry_bb = self.context.append_basic_block(wrapper_fn, "entry");
        self.builder.position_at_end(entry_bb);
        emit!(self.build_call(synthetic_main, &[], "call_main"));
        emit!(self.build_return(None));
    }
}
