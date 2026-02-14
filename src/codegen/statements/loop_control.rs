use super::super::Codegen;

impl<'ctx> Codegen<'ctx> {
    pub(crate) fn codegen_break_stmt(&self) {
        let (_, after_bb) = self.loop_stack.last().unwrap();
        emit!(self.build_unconditional_branch(*after_bb));
        self.append_dead_block("break.dead");
    }

    pub(crate) fn codegen_continue_stmt(&self) {
        let (header_bb, _) = self.loop_stack.last().unwrap();
        emit!(self.build_unconditional_branch(*header_bb));
        self.append_dead_block("cont.dead");
    }
}
