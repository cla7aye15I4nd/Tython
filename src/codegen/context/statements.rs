use inkwell::IntPredicate;

use crate::tir::{CallTarget, TirStmt, ValueType};

use super::Codegen;

impl<'ctx> Codegen<'ctx> {
    pub(super) fn codegen_stmt(&mut self, stmt: &TirStmt) {
        match stmt {
            TirStmt::Let { name, ty, value } => {
                let value_llvm = self.codegen_expr(value);

                if let Some(&existing_ptr) = self.variables.get(name.as_str()) {
                    self.builder.build_store(existing_ptr, value_llvm).unwrap();
                } else {
                    let alloca = self
                        .builder
                        .build_alloca(self.get_llvm_type(ty), name)
                        .unwrap();
                    self.builder.build_store(alloca, value_llvm).unwrap();
                    self.variables.insert(name.clone(), alloca);
                }
            }

            TirStmt::Return(expr_opt) => {
                if let Some(expr) = expr_opt {
                    let value = self.codegen_expr(expr);
                    self.builder.build_return(Some(&value)).unwrap();
                } else {
                    self.builder.build_return(None).unwrap();
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
                    self.builder
                        .build_call(function, &arg_metadata, "void_call")
                        .unwrap();
                }
                CallTarget::Builtin(builtin_fn) => {
                    use crate::tir::builtin::BuiltinFn;
                    let function = self.get_or_declare_function(
                        builtin_fn.symbol(),
                        &builtin_fn.param_types(),
                        builtin_fn.return_type(),
                    );
                    if matches!(builtin_fn, BuiltinFn::ListAppend) {
                        let list_val = self.codegen_expr(&args[0]);
                        let elem_val = self.codegen_expr(&args[1]);
                        let i64_val = self.bitcast_to_i64(elem_val, &args[1].ty);
                        self.builder
                            .build_call(function, &[list_val.into(), i64_val.into()], "list_append")
                            .unwrap();
                    } else {
                        let arg_metadata = self.codegen_call_args(args);
                        self.builder
                            .build_call(function, &arg_metadata, "void_ext_call")
                            .unwrap();
                    }
                }
                CallTarget::MethodCall {
                    mangled_name,
                    object,
                } => {
                    let arg_metadata = self.codegen_call_args(args);
                    let self_val = self.codegen_expr(object);
                    let mut all_meta: Vec<inkwell::values::BasicMetadataValueEnum> =
                        vec![self_val.into()];
                    all_meta.extend(arg_metadata);

                    let mut param_types = vec![object.ty.clone()];
                    param_types.extend(args.iter().map(|a| a.ty.clone()));

                    let function = self.get_or_declare_function(mangled_name, &param_types, None);
                    self.builder
                        .build_call(function, &all_meta, "void_method_call")
                        .unwrap();
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

                let field_ptr = self
                    .builder
                    .build_struct_gep(struct_type, obj_ptr, *field_index as u32, "field_ptr")
                    .unwrap();

                let val = self.codegen_expr(value);
                self.builder.build_store(field_ptr, val).unwrap();
            }

            TirStmt::ListSet { list, index, value } => {
                let list_val = self.codegen_expr(list);
                let index_val = self.codegen_expr(index);
                let elem_val = self.codegen_expr(value);
                let i64_val = self.bitcast_to_i64(elem_val, &value.ty);
                let list_set_fn = self.get_or_declare_list_set();
                self.builder
                    .build_call(
                        list_set_fn,
                        &[list_val.into(), index_val.into(), i64_val.into()],
                        "list_set",
                    )
                    .unwrap();
            }

            TirStmt::If {
                condition,
                then_body,
                else_body,
            } => {
                let cond_val = self.codegen_expr(condition);
                let cond_bool =
                    self.build_truthiness_check_for_value(cond_val, &condition.ty, "ifcond");

                let function = self
                    .builder
                    .get_insert_block()
                    .unwrap()
                    .get_parent()
                    .unwrap();

                let then_bb = self.context.append_basic_block(function, "then");
                let else_bb = self.context.append_basic_block(function, "else");
                let merge_bb = self.context.append_basic_block(function, "ifcont");

                self.builder
                    .build_conditional_branch(cond_bool, then_bb, else_bb)
                    .unwrap();

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
                    self.builder.build_unreachable().unwrap();
                }
            }

            TirStmt::While { condition, body } => {
                let function = self
                    .builder
                    .get_insert_block()
                    .unwrap()
                    .get_parent()
                    .unwrap();

                let header_bb = self.context.append_basic_block(function, "while.header");
                let body_bb = self.context.append_basic_block(function, "while.body");
                let after_bb = self.context.append_basic_block(function, "while.after");

                self.builder.build_unconditional_branch(header_bb).unwrap();

                self.builder.position_at_end(header_bb);
                let cond_val = self.codegen_expr(condition);
                let cond_bool =
                    self.build_truthiness_check_for_value(cond_val, &condition.ty, "whilecond");
                self.builder
                    .build_conditional_branch(cond_bool, body_bb, after_bb)
                    .unwrap();

                self.builder.position_at_end(body_bb);
                self.loop_stack.push((header_bb, after_bb));
                for s in body {
                    self.codegen_stmt(s);
                }
                self.loop_stack.pop();
                self.branch_if_unterminated(header_bb);

                self.builder.position_at_end(after_bb);
            }

            TirStmt::ForRange {
                loop_var,
                start_var,
                stop_var,
                step_var,
                body,
            } => {
                let function = self
                    .builder
                    .get_insert_block()
                    .unwrap()
                    .get_parent()
                    .unwrap();

                let header_bb = self.context.append_basic_block(function, "for.header");
                let body_bb = self.context.append_basic_block(function, "for.body");
                let incr_bb = self.context.append_basic_block(function, "for.incr");
                let after_bb = self.context.append_basic_block(function, "for.after");

                let loop_ptr = if let Some(&existing_ptr) = self.variables.get(loop_var.as_str()) {
                    existing_ptr
                } else {
                    let alloca = self
                        .builder
                        .build_alloca(self.get_llvm_type(&ValueType::Int), loop_var)
                        .unwrap();
                    self.variables.insert(loop_var.clone(), alloca);
                    alloca
                };
                let start_ptr = self.variables[start_var.as_str()];
                let stop_ptr = self.variables[stop_var.as_str()];
                let step_ptr = self.variables[step_var.as_str()];
                let start_val = self
                    .builder
                    .build_load(self.get_llvm_type(&ValueType::Int), start_ptr, "for.start")
                    .unwrap()
                    .into_int_value();
                self.builder.build_store(loop_ptr, start_val).unwrap();
                self.builder.build_unconditional_branch(header_bb).unwrap();

                self.builder.position_at_end(header_bb);
                let i_val = self
                    .builder
                    .build_load(self.get_llvm_type(&ValueType::Int), loop_ptr, "for.i")
                    .unwrap()
                    .into_int_value();
                let stop_loaded = self
                    .builder
                    .build_load(self.get_llvm_type(&ValueType::Int), stop_ptr, "for.stop")
                    .unwrap()
                    .into_int_value();
                let step_loaded = self
                    .builder
                    .build_load(self.get_llvm_type(&ValueType::Int), step_ptr, "for.step")
                    .unwrap()
                    .into_int_value();
                let zero = self.i64_type().const_int(0, false);
                let step_pos = self
                    .builder
                    .build_int_compare(IntPredicate::SGT, step_loaded, zero, "for.step_pos")
                    .unwrap();
                let cond_pos = self
                    .builder
                    .build_int_compare(IntPredicate::SLT, i_val, stop_loaded, "for.cond_pos")
                    .unwrap();
                let cond_neg = self
                    .builder
                    .build_int_compare(IntPredicate::SGT, i_val, stop_loaded, "for.cond_neg")
                    .unwrap();
                let cond = self
                    .builder
                    .build_select(step_pos, cond_pos, cond_neg, "for.cond")
                    .unwrap()
                    .into_int_value();
                self.builder
                    .build_conditional_branch(cond, body_bb, after_bb)
                    .unwrap();

                self.builder.position_at_end(body_bb);
                self.loop_stack.push((incr_bb, after_bb));
                for s in body {
                    self.codegen_stmt(s);
                }
                self.loop_stack.pop();
                self.branch_if_unterminated(incr_bb);

                self.builder.position_at_end(incr_bb);
                let i_curr = self
                    .builder
                    .build_load(self.get_llvm_type(&ValueType::Int), loop_ptr, "for.i")
                    .unwrap()
                    .into_int_value();
                let step_curr = self
                    .builder
                    .build_load(self.get_llvm_type(&ValueType::Int), step_ptr, "for.step")
                    .unwrap()
                    .into_int_value();
                let i_next = self
                    .builder
                    .build_int_add(i_curr, step_curr, "for.next")
                    .unwrap();
                self.builder.build_store(loop_ptr, i_next).unwrap();
                self.builder.build_unconditional_branch(header_bb).unwrap();

                self.builder.position_at_end(after_bb);
            }

            TirStmt::Break => {
                let (_, after_bb) = self.loop_stack.last().unwrap();
                self.builder.build_unconditional_branch(*after_bb).unwrap();
                self.append_dead_block("break.dead");
            }

            TirStmt::Continue => {
                let (header_bb, _) = self.loop_stack.last().unwrap();
                self.builder.build_unconditional_branch(*header_bb).unwrap();
                self.append_dead_block("cont.dead");
            }
        }
    }
}
