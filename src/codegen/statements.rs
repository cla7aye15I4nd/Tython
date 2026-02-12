use inkwell::{AddressSpace, IntPredicate};

use crate::tir::builtin::BuiltinFn;
use crate::tir::{CallTarget, TirStmt, ValueType};

use super::runtime_fn::RuntimeFn;
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
                    self.codegen_named_call(func_name, args, None);
                }
                CallTarget::Builtin(builtin_fn) => {
                    self.codegen_builtin_call(*builtin_fn, args, None);
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
                let struct_type = self.struct_types[class_name.as_str()];

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
                let list_set_fn = self.get_runtime_fn(RuntimeFn::ListSet);
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
                loop_body!(self, header_bb, after_bb, body);
                else_body!(self, else_bb, else_body, after_bb);
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
                loop_body!(self, incr_bb, after_bb, body);

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

                else_body!(self, else_bb, else_body, after_bb);
                self.builder.position_at_end(after_bb);
            }

            TirStmt::Raise { .. } => self.codegen_raise(stmt),
            TirStmt::TryCatch { .. } => self.codegen_try_catch(stmt),

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
                let list_len_fn = self.get_builtin(BuiltinFn::ListLen);
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
                let list_get_fn = self.get_builtin(BuiltinFn::ListGet);
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
                loop_body!(self, incr_bb, after_bb, body);

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

                else_body!(self, else_bb, else_body, after_bb);
                self.builder.position_at_end(after_bb);
            }

            TirStmt::ForIter { .. } => self.codegen_for_iter(stmt),

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
