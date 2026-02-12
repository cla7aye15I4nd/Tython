use inkwell::types::StructType;
use inkwell::values::{BasicValueEnum, CallSiteValue, FunctionValue, PointerValue};
use inkwell::{AddressSpace, IntPredicate};

use crate::tir::{TirStmt, ValueType};

use super::runtime_fn::RuntimeFn;
use super::Codegen;

impl<'ctx> Codegen<'ctx> {
    // ── exception-handling helpers ───────────────────────────────────

    /// The LLVM struct type returned by a landingpad: `{ ptr, i32 }`.
    fn get_exception_landing_type(&self) -> StructType<'ctx> {
        let ptr_type = self.context.ptr_type(AddressSpace::default());
        let i32_type = self.context.i32_type();
        self.context
            .struct_type(&[ptr_type.into(), i32_type.into()], false)
    }

    /// Emit a call/invoke to `__tython_raise` (noreturn).
    /// Uses `invoke` when inside a try block so the landing pad catches it.
    fn emit_raise(&self, tag_val: BasicValueEnum<'ctx>, msg_val: BasicValueEnum<'ctx>) {
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
    fn build_landingpad_catch_all(
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
    fn end_catch_and_resume(&self, lp_alloca: PointerValue<'ctx>) {
        let end_catch = self.get_runtime_fn(RuntimeFn::CxaEndCatch);
        emit!(self.build_call(end_catch, &[], "end_catch"));
        let landing_type = self.get_exception_landing_type();
        let lp_val = emit!(self.build_load(landing_type, lp_alloca, "lp_resume"));
        emit!(self.build_resume(lp_val));
    }

    /// Emit a function call. When inside a try block (`try_depth > 0`) and
    /// `may_throw` is true, emits an `invoke` instruction that unwinds to the
    /// current landing pad; otherwise emits a regular `call`.
    pub(super) fn build_call_maybe_invoke(
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
    pub(super) fn stmts_need_personality(stmts: &[TirStmt]) -> bool {
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

    // ── exception statement codegen ─────────────────────────────────

    pub(super) fn codegen_raise(&mut self, stmt: &TirStmt) {
        let TirStmt::Raise {
            exc_type_tag,
            message,
        } = stmt
        else {
            unreachable!()
        };

        if let Some(tag) = exc_type_tag {
            let tag_val = self.i64_type().const_int(*tag as u64, false);
            let msg_val = if let Some(msg_expr) = message {
                self.codegen_expr(msg_expr)
            } else {
                self.context
                    .ptr_type(AddressSpace::default())
                    .const_null()
                    .into()
            };
            self.emit_raise(tag_val.into(), msg_val);
        } else if let Some((tag_alloca, msg_alloca)) = self.reraise_state {
            // Bare raise inside except handler: re-raise saved exception
            let tag_val = emit!(self.build_load(self.i64_type(), tag_alloca, "reraise_tag"));
            let msg_val = emit!(self.build_load(
                self.context.ptr_type(AddressSpace::default()),
                msg_alloca,
                "reraise_msg",
            ));
            self.emit_raise(tag_val, msg_val);
        } else {
            // Bare raise outside except handler: use __cxa_rethrow as fallback
            let rethrow_fn = self.get_runtime_fn(RuntimeFn::CxaRethrow);
            emit!(self.build_call(rethrow_fn, &[], "rethrow"));
            emit!(self.build_unreachable());
        }
        self.append_dead_block("raise.after");
    }

    pub(super) fn codegen_try_catch(&mut self, stmt: &TirStmt) {
        let TirStmt::TryCatch {
            try_body,
            except_clauses,
            else_body,
            finally_body,
            has_finally,
        } = stmt
        else {
            unreachable!()
        };

        let function = emit!(self.get_insert_block()).get_parent().unwrap();
        let ptr_type = self.context.ptr_type(AddressSpace::default());
        let landing_type = self.get_exception_landing_type();

        // Create basic blocks
        let try_body_bb = self.context.append_basic_block(function, "try.body");
        let landingpad_bb = self.context.append_basic_block(function, "try.lp");
        let except_dispatch_bb = self.context.append_basic_block(function, "except.dispatch");
        let unhandled_bb = self.context.append_basic_block(function, "exc.unhandled");
        let finally_bb = if *has_finally {
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

        let exc_flag = if *has_finally {
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
        let caught_message_fn = self.get_runtime_fn(RuntimeFn::CaughtMessage);
        let end_catch_fn = self.get_runtime_fn(RuntimeFn::CxaEndCatch);

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
                if let Some(tag) = clause.exc_type_tag {
                    let tag_val = self.i64_type().const_int(tag as u64, false);
                    let matches = emit!(self.build_call(
                        caught_matches_fn,
                        &[caught_reload.into(), tag_val.into()],
                        "exc_match",
                    ));
                    let matches_val = self.extract_call_value(matches).into_int_value();
                    let matches_bool =
                        self.build_int_truthiness_check(matches_val, "exc_match_bool");

                    let next_bb = if i + 1 < except_clauses.len() {
                        let bb = self
                            .context
                            .append_basic_block(function, &format!("exc_check_{}", i + 1));
                        emit!(self.build_conditional_branch(matches_bool, handler_bbs[i], bb));
                        self.builder.position_at_end(bb);
                        bb
                    } else {
                        emit!(self.build_conditional_branch(
                            matches_bool,
                            handler_bbs[i],
                            unhandled_bb,
                        ));
                        unhandled_bb
                    };
                    let _ = next_bb;
                } else {
                    // Bare except — catch all
                    emit!(self.build_unconditional_branch(handler_bbs[i]));
                }
            }
        }

        // ── Handler bodies ────────────────────────────────────────
        let reraise_tag_alloca =
            self.build_entry_block_alloca(self.i64_type().into(), "reraise_tag");
        let reraise_msg_alloca = self.build_entry_block_alloca(ptr_type.into(), "reraise_msg");
        let caught_type_tag_fn = self.get_runtime_fn(RuntimeFn::CaughtTypeTag);

        let prev_reraise_state = self.reraise_state;

        for (i, clause) in except_clauses.iter().enumerate() {
            self.builder.position_at_end(handler_bbs[i]);

            // Save exception type_tag and message before end_catch
            // so bare `raise` in handler body can re-raise
            let caught_reload =
                emit!(self.build_load(ptr_type, caught_alloca, "caught_for_reraise"));
            let tag = emit!(self.build_call(
                caught_type_tag_fn,
                &[caught_reload.into()],
                "reraise_tag_val",
            ));
            let tag_val = self.extract_call_value(tag);
            emit!(self.build_store(reraise_tag_alloca, tag_val));

            let msg = emit!(self.build_call(
                caught_message_fn,
                &[caught_reload.into()],
                "reraise_msg_val",
            ));
            let msg_val = self.extract_call_value(msg);
            emit!(self.build_store(reraise_msg_alloca, msg_val));

            self.reraise_state = Some((reraise_tag_alloca, reraise_msg_alloca));

            if let Some(var_name) = &clause.var_name {
                let alloca = self.build_entry_block_alloca(ptr_type.into(), var_name);
                emit!(self.build_store(alloca, msg_val));
                self.variables.insert(var_name.clone(), alloca);
            }

            emit!(self.build_call(end_catch_fn, &[], "end_catch"));

            for s in &clause.body {
                self.codegen_stmt(s);
            }
            self.branch_if_unterminated(target_bb);
        }

        self.reraise_state = prev_reraise_state;

        // ── Unhandled block ───────────────────────────────────────
        self.builder.position_at_end(unhandled_bb);
        if *has_finally {
            emit!(self.build_store(exc_flag.unwrap(), self.i64_type().const_int(1, false)));
            emit!(self.build_call(end_catch_fn, &[], "end_catch_unhandled"));
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
            let need_reraise = self.build_int_truthiness_check(flag_val, "need_reraise");
            emit!(self.build_conditional_branch(need_reraise, reraise_bb, after_bb));

            self.builder.position_at_end(reraise_bb);
            self.end_catch_and_resume(lp_alloca);
        }

        self.builder.position_at_end(after_bb);
    }

    pub(super) fn codegen_for_iter(&mut self, stmt: &TirStmt) {
        let TirStmt::ForIter {
            loop_var,
            loop_var_ty,
            iterator_var,
            iterator_class,
            next_mangled,
            body,
            else_body,
        } = stmt
        else {
            unreachable!()
        };

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

        let iter_ptr = self.variables[iterator_var.as_str()];

        if !self.variables.contains_key(loop_var.as_str()) {
            let alloca = self.build_entry_block_alloca(self.get_llvm_type(loop_var_ty), loop_var);
            self.variables.insert(loop_var.clone(), alloca);
        }

        let lp_alloca = self.build_entry_block_alloca(landing_type.into(), "foriter.lp_alloca");

        emit!(self.build_unconditional_branch(call_next_bb));

        // ── Call __next__ via invoke ───────────────────────────────
        self.builder.position_at_end(call_next_bb);
        let iter_val = emit!(self.build_load(ptr_type, iter_ptr, "foriter.iter"));
        let next_fn = self.get_or_declare_function(
            next_mangled,
            &[ValueType::Class(iterator_class.clone())],
            Some(loop_var_ty.clone()),
        );
        let call_site =
            emit!(self.build_invoke(next_fn, &[iter_val], got_next_bb, lp_bb, "foriter.next"));

        // ── Got next: store and enter body ────────────────────────
        self.builder.position_at_end(got_next_bb);
        let next_val = self.extract_call_value(call_site);
        let loop_var_ptr = self.variables[loop_var.as_str()];
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

    // ── top-level exception wrapper ─────────────────────────────────

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
