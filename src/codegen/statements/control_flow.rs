use inkwell::IntPredicate;

use crate::tir::{TirExpr, TirStmt, ValueType};

use super::super::Codegen;

impl<'ctx> Codegen<'ctx> {
    pub(crate) fn codegen_if_stmt(
        &mut self,
        condition: &TirExpr,
        then_body: &[TirStmt],
        else_body: &[TirStmt],
    ) {
        let cond_val = self.codegen_expr(condition);
        let cond_bool = cond_val.into_int_value();

        let function = emit!(self.get_insert_block()).get_parent().unwrap();

        let then_bb = self.context.append_basic_block(function, "then");
        let else_bb = self.context.append_basic_block(function, "else");
        let merge_bb = self.context.append_basic_block(function, "ifcont");

        emit!(self.build_conditional_branch(cond_bool, then_bb, else_bb));

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
            emit!(self.build_unreachable());
        }
    }

    pub(crate) fn codegen_while_stmt(
        &mut self,
        condition: &TirExpr,
        body: &[TirStmt],
        else_body: &[TirStmt],
    ) {
        let function = emit!(self.get_insert_block()).get_parent().unwrap();

        let header_bb = self.context.append_basic_block(function, "while.header");
        let body_bb = self.context.append_basic_block(function, "while.body");
        let else_bb = if !else_body.is_empty() {
            Some(self.context.append_basic_block(function, "while.else"))
        } else {
            None
        };
        let after_bb = self.context.append_basic_block(function, "while.after");

        emit!(self.build_unconditional_branch(header_bb));

        self.builder.position_at_end(header_bb);
        let cond_val = self.codegen_expr(condition);
        let cond_bool = cond_val.into_int_value();
        let false_dest = else_bb.unwrap_or(after_bb);
        emit!(self.build_conditional_branch(cond_bool, body_bb, false_dest));

        self.builder.position_at_end(body_bb);
        loop_body!(self, header_bb, after_bb, body);
        else_body!(self, else_bb, else_body, after_bb);
        self.builder.position_at_end(after_bb);
    }

    pub(crate) fn codegen_for_range_stmt(
        &mut self,
        loop_var: &str,
        start_var: &str,
        stop_var: &str,
        step_var: &str,
        body: &[TirStmt],
        else_body: &[TirStmt],
    ) {
        let function = emit!(self.get_insert_block()).get_parent().unwrap();

        let header_bb = self.context.append_basic_block(function, "for.header");
        let body_bb = self.context.append_basic_block(function, "for.body");
        let incr_bb = self.context.append_basic_block(function, "for.incr");
        let else_bb = if !else_body.is_empty() {
            Some(self.context.append_basic_block(function, "for.else"))
        } else {
            None
        };
        let after_bb = self.context.append_basic_block(function, "for.after");

        let loop_ptr = if let Some(&existing_ptr) = self.variables.get(loop_var) {
            existing_ptr
        } else {
            let alloca =
                self.build_entry_block_alloca(self.get_llvm_type(&ValueType::Int), loop_var);
            self.variables.insert(loop_var.to_string(), alloca);
            alloca
        };
        // Keep loop progress in a dedicated induction slot, separate from the
        // user-visible loop variable. This matches Python semantics where
        // rebinding/mutating the loop variable does not control iteration.
        let induction_ptr =
            self.build_entry_block_alloca(self.get_llvm_type(&ValueType::Int), "for.induction");
        let start_ptr = self.variables[start_var];
        let stop_ptr = self.variables[stop_var];
        let step_ptr = self.variables[step_var];
        let start_val =
            emit!(self.build_load(self.get_llvm_type(&ValueType::Int), start_ptr, "for.start"))
                .into_int_value();
        emit!(self.build_store(induction_ptr, start_val));
        emit!(self.build_unconditional_branch(header_bb));

        self.builder.position_at_end(header_bb);
        let i_val =
            emit!(self.build_load(self.get_llvm_type(&ValueType::Int), induction_ptr, "for.i",))
                .into_int_value();
        let stop_loaded =
            emit!(self.build_load(self.get_llvm_type(&ValueType::Int), stop_ptr, "for.stop"))
                .into_int_value();
        let step_loaded =
            emit!(self.build_load(self.get_llvm_type(&ValueType::Int), step_ptr, "for.step"))
                .into_int_value();
        let zero = self.i64_type().const_int(0, false);
        let step_pos =
            emit!(self.build_int_compare(IntPredicate::SGT, step_loaded, zero, "for.step_pos"));
        let cond_pos =
            emit!(self.build_int_compare(IntPredicate::SLT, i_val, stop_loaded, "for.cond_pos"));
        let cond_neg =
            emit!(self.build_int_compare(IntPredicate::SGT, i_val, stop_loaded, "for.cond_neg"));
        let cond =
            emit!(self.build_select(step_pos, cond_pos, cond_neg, "for.cond")).into_int_value();
        let false_dest = else_bb.unwrap_or(after_bb);
        emit!(self.build_conditional_branch(cond, body_bb, false_dest));

        self.builder.position_at_end(body_bb);
        emit!(self.build_store(loop_ptr, i_val));
        loop_body!(self, incr_bb, after_bb, body);

        self.builder.position_at_end(incr_bb);
        let i_curr =
            emit!(self.build_load(self.get_llvm_type(&ValueType::Int), induction_ptr, "for.i",))
                .into_int_value();
        let step_curr =
            emit!(self.build_load(self.get_llvm_type(&ValueType::Int), step_ptr, "for.step"))
                .into_int_value();
        let i_next = emit!(self.build_int_add(i_curr, step_curr, "for.next"));
        emit!(self.build_store(induction_ptr, i_next));
        emit!(self.build_unconditional_branch(header_bb));

        else_body!(self, else_bb, else_body, after_bb);
        self.builder.position_at_end(after_bb);
    }
}
