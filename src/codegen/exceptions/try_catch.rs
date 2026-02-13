use inkwell::AddressSpace;

use crate::tir::{ExceptClause, TirStmt};

use super::super::runtime_fn::RuntimeFn;
use super::super::Codegen;

impl<'ctx> Codegen<'ctx> {
    pub(crate) fn codegen_try_catch(
        &mut self,
        try_body: &[TirStmt],
        except_clauses: &[ExceptClause],
        else_body: &[TirStmt],
        finally_body: &[TirStmt],
        has_finally: bool,
    ) {
        let function = emit!(self.get_insert_block()).get_parent().unwrap();
        let ptr_type = self.context.ptr_type(AddressSpace::default());
        let landing_type = self.get_exception_landing_type();

        // Create basic blocks
        let try_body_bb = self.context.append_basic_block(function, "try.body");
        let landingpad_bb = self.context.append_basic_block(function, "try.lp");
        let except_dispatch_bb = self.context.append_basic_block(function, "except.dispatch");
        let unhandled_bb = self.context.append_basic_block(function, "exc.unhandled");
        let finally_bb = if has_finally {
            Some(self.context.append_basic_block(function, "finally"))
        } else {
            None
        };
        let after_bb = self.context.append_basic_block(function, "try.after");
        let target_bb = finally_bb.unwrap_or(after_bb);

        let try_else_bb = if !else_body.is_empty() {
            Some(self.context.append_basic_block(function, "try.else"))
        } else {
            None
        };
        let try_exit_dest = try_else_bb.unwrap_or(target_bb);

        // Allocas for catch state
        let caught_alloca = self.build_entry_block_alloca(ptr_type.into(), "caught_alloca");
        let lp_alloca = self.build_entry_block_alloca(landing_type.into(), "lp_alloca");

        let exc_flag = if has_finally {
            let alloca = self.build_entry_block_alloca(self.i64_type().into(), "exc_flag");
            emit!(self.build_store(alloca, self.i64_type().const_zero()));
            Some(alloca)
        } else {
            None
        };

        emit!(self.build_unconditional_branch(try_body_bb));

        // ── Try body ──────────────────────────────────────────────
        self.builder.position_at_end(try_body_bb);
        self.unwind_dest_stack.push(landingpad_bb);
        self.try_depth += 1;

        for s in try_body {
            self.codegen_stmt(s);
        }

        self.try_depth -= 1;
        self.unwind_dest_stack.pop();
        self.branch_if_unterminated(try_exit_dest);

        // ── Try else block ──────────────────────────────────────
        if let Some(try_else_bb) = try_else_bb {
            self.builder.position_at_end(try_else_bb);
            for s in else_body {
                self.codegen_stmt(s);
            }
            self.branch_if_unterminated(target_bb);
        }

        // ── Landing pad ───────────────────────────────────────────
        self.builder.position_at_end(landingpad_bb);
        let caught_ptr = self.build_landingpad_catch_all(lp_alloca, "try.lp");
        emit!(self.build_store(caught_alloca, caught_ptr));
        emit!(self.build_unconditional_branch(except_dispatch_bb));

        // ── Except dispatch ───────────────────────────────────────
        self.builder.position_at_end(except_dispatch_bb);

        let caught_matches_fn = self.get_runtime_fn(RuntimeFn::CaughtMatches);

        let handler_bbs: Vec<_> = except_clauses
            .iter()
            .enumerate()
            .map(|(i, _)| {
                self.context
                    .append_basic_block(function, &format!("handler_{}", i))
            })
            .collect();

        if except_clauses.is_empty() {
            emit!(self.build_unconditional_branch(unhandled_bb));
        } else {
            let caught_reload = emit!(self.build_load(ptr_type, caught_alloca, "caught_reload"));
            for (i, clause) in except_clauses.iter().enumerate() {
                let Some(tag) = clause.exc_type_tag else {
                    // Bare except — catch all
                    emit!(self.build_unconditional_branch(handler_bbs[i]));
                    break;
                };

                let tag_val = self.i64_type().const_int(tag as u64, false);
                let matches = emit!(self.build_call(
                    caught_matches_fn,
                    &[caught_reload.into(), tag_val.into()],
                    "exc_match",
                ));
                let matches_val = self.extract_call_value(matches).into_int_value();
                let matches_bool = emit!(self.build_int_compare(
                    inkwell::IntPredicate::NE,
                    matches_val,
                    self.i64_type().const_int(0, false),
                    "exc_match_bool",
                ));

                if i + 1 < except_clauses.len() {
                    let next_check_bb = self
                        .context
                        .append_basic_block(function, &format!("exc_check_{}", i + 1));
                    emit!(self.build_conditional_branch(
                        matches_bool,
                        handler_bbs[i],
                        next_check_bb,
                    ));
                    self.builder.position_at_end(next_check_bb);
                } else {
                    emit!(self.build_conditional_branch(
                        matches_bool,
                        handler_bbs[i],
                        unhandled_bb
                    ));
                }
            }
        }

        // ── Handler bodies ────────────────────────────────────────
        let reraise_tag_alloca =
            self.build_entry_block_alloca(self.i64_type().into(), "reraise_tag");
        let reraise_msg_alloca = self.build_entry_block_alloca(ptr_type.into(), "reraise_msg");
        let prev_reraise_state = self.reraise_state;

        for (i, clause) in except_clauses.iter().enumerate() {
            self.builder.position_at_end(handler_bbs[i]);

            // Save exception type_tag and message before end_catch
            // so bare `raise` in handler body can re-raise
            let caught_reload =
                emit!(self.build_load(ptr_type, caught_alloca, "caught_for_reraise"));
            let (tag_val, msg_val) = self.get_caught_tag_and_message(
                caught_reload,
                "reraise_tag_val",
                "reraise_msg_val",
            );
            emit!(self.build_store(reraise_tag_alloca, tag_val));
            emit!(self.build_store(reraise_msg_alloca, msg_val));

            self.reraise_state = Some((reraise_tag_alloca, reraise_msg_alloca));

            if let Some(var_name) = &clause.var_name {
                let alloca = self.build_entry_block_alloca(ptr_type.into(), var_name);
                emit!(self.build_store(alloca, msg_val));
                self.variables.insert(var_name.clone(), alloca);
            }

            self.emit_end_catch("end_catch");

            for s in &clause.body {
                self.codegen_stmt(s);
            }
            self.branch_if_unterminated(target_bb);
        }

        self.reraise_state = prev_reraise_state;

        // ── Unhandled block ───────────────────────────────────────
        self.builder.position_at_end(unhandled_bb);
        if has_finally {
            emit!(self.build_store(exc_flag.unwrap(), self.i64_type().const_int(1, false)));
            self.emit_end_catch("end_catch_unhandled");
            emit!(self.build_unconditional_branch(finally_bb.unwrap()));
        } else {
            self.end_catch_and_resume(lp_alloca);
        }

        // ── Finally block ─────────────────────────────────────────
        if let Some(finally_bb) = finally_bb {
            self.builder.position_at_end(finally_bb);
            for s in finally_body {
                self.codegen_stmt(s);
            }

            let reraise_bb = self.context.append_basic_block(function, "finally.reraise");
            let flag_val =
                emit!(self.build_load(self.i64_type(), exc_flag.unwrap(), "exc_flag_val"))
                    .into_int_value();
            let need_reraise = emit!(self.build_int_compare(
                inkwell::IntPredicate::NE,
                flag_val,
                self.i64_type().const_int(0, false),
                "need_reraise",
            ));
            emit!(self.build_conditional_branch(need_reraise, reraise_bb, after_bb));

            self.builder.position_at_end(reraise_bb);
            self.end_catch_and_resume(lp_alloca);
        }

        self.builder.position_at_end(after_bb);
    }
}
