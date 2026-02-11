use inkwell::{AddressSpace, IntPredicate};

use crate::tir::{CallTarget, TirStmt, ValueType};

use super::Codegen;

impl<'ctx> Codegen<'ctx> {
    pub(super) fn codegen_stmt(&mut self, stmt: &TirStmt) {
        match stmt {
            TirStmt::Let { name, ty, value } => {
                let value_llvm = self.codegen_expr(value);

                if let Some(&existing_ptr) = self.variables.get(name.as_str()) {
                    emit!(self.build_store(existing_ptr, value_llvm));
                } else {
                    let alloca = self.build_entry_block_alloca(self.get_llvm_type(ty), name);
                    emit!(self.build_store(alloca, value_llvm));
                    self.variables.insert(name.clone(), alloca);
                }
            }

            TirStmt::Return(expr_opt) => {
                if let Some(expr) = expr_opt {
                    let value = self.codegen_expr(expr);
                    emit!(self.build_return(Some(&value)));
                } else {
                    emit!(self.build_return(None));
                }
            }

            TirStmt::Expr(expr) => {
                self.codegen_expr(expr);
            }

            TirStmt::VoidCall { target, args } => match target {
                CallTarget::Named(func_name) => {
                    let arg_metadata = self.codegen_call_args(args);
                    let arg_types: Vec<ValueType> = args.iter().map(|a| a.ty.clone()).collect();
                    let function = self.get_or_declare_function(func_name, &arg_types, None);
                    self.build_call_maybe_invoke(function, &arg_metadata, "void_call", true);
                }
                CallTarget::Builtin(builtin_fn) => {
                    use crate::tir::builtin::BuiltinFn;
                    let function = self.get_or_declare_function(
                        builtin_fn.symbol(),
                        &builtin_fn.param_types(),
                        builtin_fn.return_type(),
                    );
                    if matches!(builtin_fn, BuiltinFn::ListAppend | BuiltinFn::ListRemove) {
                        // list, value — value needs bitcast to i64
                        let list_val = self.codegen_expr(&args[0]);
                        let elem_val = self.codegen_expr(&args[1]);
                        let i64_val = self.bitcast_to_i64(elem_val, &args[1].ty);
                        emit!(self.build_call(
                            function,
                            &[list_val.into(), i64_val.into()],
                            "list_method"
                        ));
                    } else if matches!(builtin_fn, BuiltinFn::ListInsert) {
                        // list, index, value — value needs bitcast to i64
                        let list_val = self.codegen_expr(&args[0]);
                        let idx_val = self.codegen_expr(&args[1]);
                        let elem_val = self.codegen_expr(&args[2]);
                        let i64_val = self.bitcast_to_i64(elem_val, &args[2].ty);
                        emit!(self.build_call(
                            function,
                            &[list_val.into(), idx_val.into(), i64_val.into()],
                            "list_insert",
                        ));
                    } else {
                        let arg_metadata = self.codegen_call_args(args);
                        emit!(self.build_call(
                            function,
                            &Self::to_meta_args(&arg_metadata),
                            "void_ext_call"
                        ));
                    }
                }
                CallTarget::MethodCall {
                    mangled_name,
                    object,
                } => {
                    let arg_metadata = self.codegen_call_args(args);
                    let self_val = self.codegen_expr(object);
                    let mut all_vals: Vec<inkwell::values::BasicValueEnum> = vec![self_val];
                    all_vals.extend(arg_metadata);

                    let mut param_types = vec![object.ty.clone()];
                    param_types.extend(args.iter().map(|a| a.ty.clone()));

                    let function = self.get_or_declare_function(mangled_name, &param_types, None);
                    self.build_call_maybe_invoke(function, &all_vals, "void_method_call", true);
                }
                CallTarget::Indirect(callee_expr) => {
                    let callee_ptr = self.codegen_expr(callee_expr).into_pointer_value();
                    let arg_metadata = self.codegen_call_args(args);

                    let (param_types_vt, _) = callee_expr.ty.unwrap_function();

                    let llvm_params: Vec<inkwell::types::BasicMetadataTypeEnum> = param_types_vt
                        .iter()
                        .map(|t| self.get_llvm_type(t).into())
                        .collect();

                    let fn_type = self.context.void_type().fn_type(&llvm_params, false);

                    emit!(self.build_indirect_call(
                        fn_type,
                        callee_ptr,
                        &Self::to_meta_args(&arg_metadata),
                        "void_indirect_call",
                    ));
                }
            },

            TirStmt::SetField {
                object,
                class_name,
                field_index,
                value,
            } => {
                let obj_ptr = self.codegen_expr(object).into_pointer_value();
                let struct_type = self.class_types[class_name.as_str()];

                let field_ptr = emit!(self.build_struct_gep(
                    struct_type,
                    obj_ptr,
                    *field_index as u32,
                    "field_ptr"
                ));

                let val = self.codegen_expr(value);
                emit!(self.build_store(field_ptr, val));
            }

            TirStmt::ListSet { list, index, value } => {
                let list_val = self.codegen_expr(list);
                let index_val = self.codegen_expr(index);
                let elem_val = self.codegen_expr(value);
                let i64_val = self.bitcast_to_i64(elem_val, &value.ty);
                let list_set_fn = self.get_or_declare_list_set();
                emit!(self.build_call(
                    list_set_fn,
                    &[list_val.into(), index_val.into(), i64_val.into()],
                    "list_set",
                ));
            }

            TirStmt::If {
                condition,
                then_body,
                else_body,
            } => {
                let cond_val = self.codegen_expr(condition);
                let cond_bool =
                    self.build_truthiness_check_for_value(cond_val, &condition.ty, "ifcond");

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

            TirStmt::While {
                condition,
                body,
                else_body,
            } => {
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
                let cond_bool =
                    self.build_truthiness_check_for_value(cond_val, &condition.ty, "whilecond");
                let false_dest = else_bb.unwrap_or(after_bb);
                emit!(self.build_conditional_branch(cond_bool, body_bb, false_dest));

                self.builder.position_at_end(body_bb);
                self.loop_stack.push((header_bb, after_bb)); // break → after (skips else)
                for s in body {
                    self.codegen_stmt(s);
                }
                self.loop_stack.pop();
                self.branch_if_unterminated(header_bb);

                if let Some(else_bb) = else_bb {
                    self.builder.position_at_end(else_bb);
                    for s in else_body {
                        self.codegen_stmt(s);
                    }
                    self.branch_if_unterminated(after_bb);
                }

                self.builder.position_at_end(after_bb);
            }

            TirStmt::ForRange {
                loop_var,
                start_var,
                stop_var,
                step_var,
                body,
                else_body,
            } => {
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

                let loop_ptr = if let Some(&existing_ptr) = self.variables.get(loop_var.as_str()) {
                    existing_ptr
                } else {
                    let alloca = self
                        .build_entry_block_alloca(self.get_llvm_type(&ValueType::Int), loop_var);
                    self.variables.insert(loop_var.clone(), alloca);
                    alloca
                };
                let start_ptr = self.variables[start_var.as_str()];
                let stop_ptr = self.variables[stop_var.as_str()];
                let step_ptr = self.variables[step_var.as_str()];
                let start_val = emit!(self.build_load(
                    self.get_llvm_type(&ValueType::Int),
                    start_ptr,
                    "for.start"
                ))
                .into_int_value();
                emit!(self.build_store(loop_ptr, start_val));
                emit!(self.build_unconditional_branch(header_bb));

                self.builder.position_at_end(header_bb);
                let i_val =
                    emit!(self.build_load(self.get_llvm_type(&ValueType::Int), loop_ptr, "for.i"))
                        .into_int_value();
                let stop_loaded = emit!(self.build_load(
                    self.get_llvm_type(&ValueType::Int),
                    stop_ptr,
                    "for.stop"
                ))
                .into_int_value();
                let step_loaded = emit!(self.build_load(
                    self.get_llvm_type(&ValueType::Int),
                    step_ptr,
                    "for.step"
                ))
                .into_int_value();
                let zero = self.i64_type().const_int(0, false);
                let step_pos = emit!(self.build_int_compare(
                    IntPredicate::SGT,
                    step_loaded,
                    zero,
                    "for.step_pos"
                ));
                let cond_pos = emit!(self.build_int_compare(
                    IntPredicate::SLT,
                    i_val,
                    stop_loaded,
                    "for.cond_pos"
                ));
                let cond_neg = emit!(self.build_int_compare(
                    IntPredicate::SGT,
                    i_val,
                    stop_loaded,
                    "for.cond_neg"
                ));
                let cond = emit!(self.build_select(step_pos, cond_pos, cond_neg, "for.cond"))
                    .into_int_value();
                let false_dest = else_bb.unwrap_or(after_bb);
                emit!(self.build_conditional_branch(cond, body_bb, false_dest));

                self.builder.position_at_end(body_bb);
                self.loop_stack.push((incr_bb, after_bb)); // break → after (skips else)
                for s in body {
                    self.codegen_stmt(s);
                }
                self.loop_stack.pop();
                self.branch_if_unterminated(incr_bb);

                self.builder.position_at_end(incr_bb);
                let i_curr =
                    emit!(self.build_load(self.get_llvm_type(&ValueType::Int), loop_ptr, "for.i"))
                        .into_int_value();
                let step_curr = emit!(self.build_load(
                    self.get_llvm_type(&ValueType::Int),
                    step_ptr,
                    "for.step"
                ))
                .into_int_value();
                let i_next = emit!(self.build_int_add(i_curr, step_curr, "for.next"));
                emit!(self.build_store(loop_ptr, i_next));
                emit!(self.build_unconditional_branch(header_bb));

                if let Some(else_bb) = else_bb {
                    self.builder.position_at_end(else_bb);
                    for s in else_body {
                        self.codegen_stmt(s);
                    }
                    self.branch_if_unterminated(after_bb);
                }

                self.builder.position_at_end(after_bb);
            }

            // ── exception handling (LLVM landingpad-based) ────────────────
            TirStmt::Raise {
                exc_type_tag,
                message,
            } => {
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
                    let raise_fn = self.get_or_declare_exc_raise();
                    // __tython_raise calls __cxa_throw (noreturn). When inside
                    // a try block we must use invoke so the landing pad catches it.
                    if self.try_depth > 0 {
                        let function = emit!(self.get_insert_block()).get_parent().unwrap();
                        let dead_bb = self.context.append_basic_block(function, "raise.dead");
                        let unwind_bb = *self.unwind_dest_stack.last().unwrap();
                        let basic_args: [inkwell::values::BasicValueEnum; 2] =
                            [tag_val.into(), msg_val];
                        emit!(self.build_invoke(
                            raise_fn,
                            &basic_args,
                            dead_bb,
                            unwind_bb,
                            "raise"
                        ));
                        self.builder.position_at_end(dead_bb);
                        emit!(self.build_unreachable());
                    } else {
                        emit!(self.build_call(
                            raise_fn,
                            &[tag_val.into(), msg_val.into()],
                            "raise"
                        ));
                        emit!(self.build_unreachable());
                    }
                } else if let Some((tag_alloca, msg_alloca)) = self.reraise_state {
                    // Bare raise inside except handler: re-raise saved exception
                    let tag_val =
                        emit!(self.build_load(self.i64_type(), tag_alloca, "reraise_tag"));
                    let msg_val = emit!(self.build_load(
                        self.context.ptr_type(AddressSpace::default()),
                        msg_alloca,
                        "reraise_msg",
                    ));
                    let raise_fn = self.get_or_declare_exc_raise();
                    if self.try_depth > 0 {
                        let function = emit!(self.get_insert_block()).get_parent().unwrap();
                        let dead_bb = self.context.append_basic_block(function, "reraise.dead");
                        let unwind_bb = *self.unwind_dest_stack.last().unwrap();
                        emit!(self.build_invoke(
                            raise_fn,
                            &[tag_val, msg_val],
                            dead_bb,
                            unwind_bb,
                            "reraise",
                        ));
                        self.builder.position_at_end(dead_bb);
                        emit!(self.build_unreachable());
                    } else {
                        emit!(self.build_call(
                            raise_fn,
                            &[tag_val.into(), msg_val.into()],
                            "reraise"
                        ));
                        emit!(self.build_unreachable());
                    }
                } else {
                    // Bare raise outside except handler: use __cxa_rethrow as fallback
                    let rethrow_fn = self.get_or_declare_cxa_rethrow();
                    emit!(self.build_call(rethrow_fn, &[], "rethrow"));
                    emit!(self.build_unreachable());
                }
                self.append_dead_block("raise.after");
            }

            TirStmt::TryCatch {
                try_body,
                except_clauses,
                else_body,
                finally_body,
                has_finally,
            } => {
                let function = emit!(self.get_insert_block()).get_parent().unwrap();

                let personality = self.get_or_declare_personality_fn();
                if !function.has_personality_function() {
                    function.set_personality_function(personality);
                }

                let landing_type = self.get_exception_landing_type();
                let ptr_type = self.context.ptr_type(AddressSpace::default());

                // Create basic blocks
                let try_body_bb = self.context.append_basic_block(function, "try.body");
                let landingpad_bb = self.context.append_basic_block(function, "try.lp");
                let except_dispatch_bb =
                    self.context.append_basic_block(function, "except.dispatch");
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
                // try body normal exit → else (if present) → target
                let try_exit_dest = try_else_bb.unwrap_or(target_bb);

                // Allocas for catch state
                let caught_alloca = self.build_entry_block_alloca(ptr_type.into(), "caught_alloca");
                let lp_alloca = self.build_entry_block_alloca(landing_type.into(), "lp_alloca");

                // Allocate flag for finally re-raise
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
                let null_ptr = ptr_type.const_null();
                let lp = emit!(self.build_landing_pad(
                    landing_type,
                    personality,
                    &[null_ptr.into()], // catch i8* null = catch all
                    false,
                    "try.lp",
                ));

                // Save full LP value for potential resume
                emit!(self.build_store(lp_alloca, lp));

                let exc_ptr = emit!(self.build_extract_value(lp.into_struct_value(), 0, "exc_ptr"));

                let begin_catch = self.get_or_declare_cxa_begin_catch();
                let caught = emit!(self.build_call(begin_catch, &[exc_ptr.into()], "caught"));
                let caught_ptr = self.extract_call_value(caught);
                emit!(self.build_store(caught_alloca, caught_ptr));

                emit!(self.build_unconditional_branch(except_dispatch_bb));

                // ── Except dispatch ───────────────────────────────────────
                self.builder.position_at_end(except_dispatch_bb);

                let caught_matches_fn = self.get_or_declare_caught_matches();
                let caught_message_fn = self.get_or_declare_caught_message();
                let end_catch_fn = self.get_or_declare_cxa_end_catch();

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
                    let caught_reload =
                        emit!(self.build_load(ptr_type, caught_alloca, "caught_reload"));
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
                                emit!(self.build_conditional_branch(
                                    matches_bool,
                                    handler_bbs[i],
                                    bb
                                ));
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
                // Allocas to save exception state for bare `raise` re-raise
                let reraise_tag_alloca =
                    self.build_entry_block_alloca(self.i64_type().into(), "reraise_tag");
                let reraise_msg_alloca =
                    self.build_entry_block_alloca(ptr_type.into(), "reraise_msg");
                let caught_type_tag_fn = self.get_or_declare_caught_type_tag();

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

                    // End the catch before handler body so the handler can throw new exceptions
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
                    emit!(self.build_call(end_catch_fn, &[], "end_catch_unhandled"));
                    let lp_val = emit!(self.build_load(landing_type, lp_alloca, "lp_for_resume"));
                    emit!(self.build_resume(lp_val));
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
                    let lp_val = emit!(self.build_load(landing_type, lp_alloca, "lp_resume"));
                    emit!(self.build_resume(lp_val));
                }

                self.builder.position_at_end(after_bb);
            }

            // ── for-in loops ─────────────────────────────────────────────
            TirStmt::ForList {
                loop_var,
                loop_var_ty,
                list_var,
                index_var,
                len_var,
                body,
                else_body,
            } => {
                let function = emit!(self.get_insert_block()).get_parent().unwrap();

                let header_bb = self.context.append_basic_block(function, "forlist.header");
                let body_bb = self.context.append_basic_block(function, "forlist.body");
                let incr_bb = self.context.append_basic_block(function, "forlist.incr");
                let else_bb = if !else_body.is_empty() {
                    Some(self.context.append_basic_block(function, "forlist.else"))
                } else {
                    None
                };
                let after_bb = self.context.append_basic_block(function, "forlist.after");

                // Get list pointer
                let list_ptr = self.variables[list_var.as_str()];

                // Create index variable (idx = 0)
                let idx_alloca =
                    self.build_entry_block_alloca(self.get_llvm_type(&ValueType::Int), index_var);
                emit!(self.build_store(idx_alloca, self.i64_type().const_zero()));
                self.variables.insert(index_var.clone(), idx_alloca);

                // Compute list length
                let list_val = emit!(self.build_load(
                    self.context.ptr_type(AddressSpace::default()),
                    list_ptr,
                    "forlist.list",
                ));
                let list_len_fn = self.get_or_declare_function(
                    "__tython_list_len",
                    &[ValueType::List(Box::new(ValueType::Int))],
                    Some(ValueType::Int),
                );
                let len_call =
                    emit!(self.build_call(list_len_fn, &[list_val.into()], "forlist.len_call"));
                let len_val = self.extract_call_value(len_call);

                // Create len variable
                let len_alloca =
                    self.build_entry_block_alloca(self.get_llvm_type(&ValueType::Int), len_var);
                emit!(self.build_store(len_alloca, len_val));
                self.variables.insert(len_var.clone(), len_alloca);

                // Ensure loop var has an alloca
                if !self.variables.contains_key(loop_var.as_str()) {
                    let alloca =
                        self.build_entry_block_alloca(self.get_llvm_type(loop_var_ty), loop_var);
                    self.variables.insert(loop_var.clone(), alloca);
                }

                emit!(self.build_unconditional_branch(header_bb));

                // Header: idx < len
                self.builder.position_at_end(header_bb);
                let idx_val = emit!(self.build_load(
                    self.get_llvm_type(&ValueType::Int),
                    idx_alloca,
                    "forlist.idx",
                ))
                .into_int_value();
                let len_loaded = emit!(self.build_load(
                    self.get_llvm_type(&ValueType::Int),
                    len_alloca,
                    "forlist.len",
                ))
                .into_int_value();
                let cond = emit!(self.build_int_compare(
                    IntPredicate::SLT,
                    idx_val,
                    len_loaded,
                    "forlist.cond"
                ));
                let forlist_false_dest = else_bb.unwrap_or(after_bb);
                emit!(self.build_conditional_branch(cond, body_bb, forlist_false_dest));

                // Body: loop_var = list_get(list, idx)
                self.builder.position_at_end(body_bb);
                let list_reload = emit!(self.build_load(
                    self.context.ptr_type(AddressSpace::default()),
                    list_ptr,
                    "forlist.list2",
                ));
                let idx_reload = emit!(self.build_load(
                    self.get_llvm_type(&ValueType::Int),
                    idx_alloca,
                    "forlist.idx2",
                ));
                let list_get_fn = self.get_or_declare_function(
                    "__tython_list_get",
                    &[ValueType::List(Box::new(ValueType::Int)), ValueType::Int],
                    Some(ValueType::Int),
                );
                let call = emit!(self.build_call(
                    list_get_fn,
                    &[list_reload.into(), idx_reload.into()],
                    "forlist.elem_i64",
                ));
                let elem_i64 = self.extract_call_value(call).into_int_value();
                let elem_val = self.bitcast_from_i64(elem_i64, loop_var_ty);
                let loop_var_ptr = self.variables[loop_var.as_str()];
                emit!(self.build_store(loop_var_ptr, elem_val));

                // Body statements
                self.loop_stack.push((incr_bb, after_bb)); // break → after (skips else)
                for s in body {
                    self.codegen_stmt(s);
                }
                self.loop_stack.pop();
                self.branch_if_unterminated(incr_bb);

                // Increment: idx++
                self.builder.position_at_end(incr_bb);
                let idx_curr = emit!(self.build_load(
                    self.get_llvm_type(&ValueType::Int),
                    idx_alloca,
                    "forlist.idx3",
                ))
                .into_int_value();
                let idx_next = emit!(self.build_int_add(
                    idx_curr,
                    self.i64_type().const_int(1, false),
                    "forlist.idx_next",
                ));
                emit!(self.build_store(idx_alloca, idx_next));
                emit!(self.build_unconditional_branch(header_bb));

                if let Some(else_bb) = else_bb {
                    self.builder.position_at_end(else_bb);
                    for s in else_body {
                        self.codegen_stmt(s);
                    }
                    self.branch_if_unterminated(after_bb);
                }

                self.builder.position_at_end(after_bb);
            }

            TirStmt::ForIter {
                loop_var,
                loop_var_ty,
                iterator_var,
                iterator_class,
                next_mangled,
                body,
                else_body,
            } => {
                let function = emit!(self.get_insert_block()).get_parent().unwrap();

                let personality = self.get_or_declare_personality_fn();
                if !function.has_personality_function() {
                    function.set_personality_function(personality);
                }

                let landing_type = self.get_exception_landing_type();
                let ptr_type = self.context.ptr_type(AddressSpace::default());

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

                // Ensure loop var has an alloca
                if !self.variables.contains_key(loop_var.as_str()) {
                    let alloca =
                        self.build_entry_block_alloca(self.get_llvm_type(loop_var_ty), loop_var);
                    self.variables.insert(loop_var.clone(), alloca);
                }

                // Alloca to save LP value for reraise
                let lp_alloca =
                    self.build_entry_block_alloca(landing_type.into(), "foriter.lp_alloca");

                emit!(self.build_unconditional_branch(call_next_bb));

                // ── Call __next__ via invoke ───────────────────────────────
                self.builder.position_at_end(call_next_bb);
                let iter_val = emit!(self.build_load(ptr_type, iter_ptr, "foriter.iter"));
                let next_fn = self.get_or_declare_function(
                    next_mangled,
                    &[ValueType::Class(iterator_class.clone())],
                    Some(loop_var_ty.clone()),
                );
                let call_site = emit!(self.build_invoke(
                    next_fn,
                    &[iter_val],
                    got_next_bb,
                    lp_bb,
                    "foriter.next"
                ));

                // ── Got next: store and enter body ────────────────────────
                self.builder.position_at_end(got_next_bb);
                let next_val = self.extract_call_value(call_site);
                let loop_var_ptr = self.variables[loop_var.as_str()];
                emit!(self.build_store(loop_var_ptr, next_val));
                emit!(self.build_unconditional_branch(body_bb));

                // ── Landing pad: check StopIteration ──────────────────────
                self.builder.position_at_end(lp_bb);
                let null_ptr = ptr_type.const_null();
                let lp = emit!(self.build_landing_pad(
                    landing_type,
                    personality,
                    &[null_ptr.into()],
                    false,
                    "foriter.lp",
                ));

                // Save LP for potential resume
                emit!(self.build_store(lp_alloca, lp));

                let exc_ptr =
                    emit!(self.build_extract_value(lp.into_struct_value(), 0, "foriter.exc_ptr"));

                let begin_catch = self.get_or_declare_cxa_begin_catch();
                let caught =
                    emit!(self.build_call(begin_catch, &[exc_ptr.into()], "foriter.caught"));
                let caught_ptr = self.extract_call_value(caught);

                let type_tag_fn = self.get_or_declare_caught_type_tag();
                let tag = emit!(self.build_call(type_tag_fn, &[caught_ptr.into()], "foriter.tag"));
                let tag_val = self.extract_call_value(tag).into_int_value();

                let stop_tag = self.i64_type().const_int(2, false); // TYTHON_EXC_STOP_ITERATION
                let is_stop = emit!(self.build_int_compare(
                    IntPredicate::EQ,
                    tag_val,
                    stop_tag,
                    "foriter.is_stop"
                ));
                emit!(self.build_conditional_branch(is_stop, stop_bb, reraise_bb));

                // ── Stop: end catch and exit loop (→ else or after) ─────
                self.builder.position_at_end(stop_bb);
                let end_catch = self.get_or_declare_cxa_end_catch();
                emit!(self.build_call(end_catch, &[], "foriter.end_catch"));
                let stop_dest = else_bb.unwrap_or(after_bb);
                emit!(self.build_unconditional_branch(stop_dest));

                // ── Re-raise: not StopIteration ───────────────────────────
                self.builder.position_at_end(reraise_bb);
                emit!(self.build_call(end_catch, &[], "foriter.end_catch2"));
                let lp_val = emit!(self.build_load(landing_type, lp_alloca, "foriter.lp_resume"));
                emit!(self.build_resume(lp_val));

                // ── Body ──────────────────────────────────────────────────
                self.builder.position_at_end(body_bb);
                self.loop_stack.push((call_next_bb, after_bb)); // break → after (skips else)
                for s in body {
                    self.codegen_stmt(s);
                }
                self.loop_stack.pop();
                self.branch_if_unterminated(call_next_bb);

                if let Some(else_bb) = else_bb {
                    self.builder.position_at_end(else_bb);
                    for s in else_body {
                        self.codegen_stmt(s);
                    }
                    self.branch_if_unterminated(after_bb);
                }

                self.builder.position_at_end(after_bb);
            }

            TirStmt::Break => {
                let (_, after_bb) = self.loop_stack.last().unwrap();
                emit!(self.build_unconditional_branch(*after_bb));
                self.append_dead_block("break.dead");
            }

            TirStmt::Continue => {
                let (header_bb, _) = self.loop_stack.last().unwrap();
                emit!(self.build_unconditional_branch(*header_bb));
                self.append_dead_block("cont.dead");
            }
        }
    }
}
