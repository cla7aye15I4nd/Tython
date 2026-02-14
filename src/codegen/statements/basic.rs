use inkwell::AddressSpace;

use crate::tir::builtin::BuiltinFn;
use crate::tir::{CallTarget, TirExpr, ValueType};

use super::super::runtime_fn::RuntimeFn;
use super::super::Codegen;

impl<'ctx> Codegen<'ctx> {
    pub(crate) fn codegen_let_stmt(&mut self, name: &str, ty: &ValueType, value: &TirExpr) {
        let value_llvm = self.codegen_expr(value);

        if let Some(&existing_ptr) = self.variables.get(name) {
            emit!(self.build_store(existing_ptr, value_llvm));
        } else {
            let alloca = self.build_entry_block_alloca(self.get_llvm_type(ty), name);
            emit!(self.build_store(alloca, value_llvm));
            self.variables.insert(name.to_string(), alloca);
        }
    }

    pub(crate) fn codegen_return_stmt(&mut self, expr_opt: &Option<TirExpr>) {
        if let Some(expr) = expr_opt {
            let value = self.codegen_expr(expr);
            emit!(self.build_return(Some(&value)));
        } else {
            emit!(self.build_return(None));
        }
    }

    pub(crate) fn codegen_void_call_stmt(&mut self, target: &CallTarget, args: &[TirExpr]) {
        match target {
            CallTarget::Named(func_name) => {
                self.codegen_named_call(func_name, args, None);
            }
            CallTarget::Builtin(builtin_fn) => {
                self.codegen_builtin_call(*builtin_fn, args, None);
            }
        }
    }

    pub(crate) fn codegen_set_field_stmt(
        &mut self,
        object: &TirExpr,
        class_name: &str,
        field_index: usize,
        value: &TirExpr,
    ) {
        let obj_ptr = self.codegen_expr(object).into_pointer_value();
        let struct_type = self.struct_types[class_name];

        let field_ptr =
            emit!(self.build_struct_gep(struct_type, obj_ptr, field_index as u32, "field_ptr"));

        let val = self.codegen_expr(value);
        emit!(self.build_store(field_ptr, val));
    }

    pub(crate) fn codegen_list_set_stmt(
        &mut self,
        list: &TirExpr,
        index: &TirExpr,
        value: &TirExpr,
    ) {
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

    pub(crate) fn codegen_for_list_stmt(
        &mut self,
        loop_var: &str,
        loop_var_ty: &ValueType,
        list_var: &str,
        index_var: &str,
        len_var: &str,
        body: &[crate::tir::TirStmt],
        else_body: &[crate::tir::TirStmt],
    ) {
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
        let list_ptr = self.variables[list_var];

        // Create index variable (idx = 0)
        let idx_alloca =
            self.build_entry_block_alloca(self.get_llvm_type(&ValueType::Int), index_var);
        emit!(self.build_store(idx_alloca, self.i64_type().const_zero()));
        self.variables.insert(index_var.to_string(), idx_alloca);

        // Compute list length
        let list_val = emit!(self.build_load(
            self.context.ptr_type(AddressSpace::default()),
            list_ptr,
            "forlist.list",
        ));
        let list_len_fn = self.get_builtin(BuiltinFn::ListLen);
        let len_call = emit!(self.build_call(list_len_fn, &[list_val.into()], "forlist.len_call"));
        let len_val = self.extract_call_value(len_call);

        // Create len variable
        let len_alloca =
            self.build_entry_block_alloca(self.get_llvm_type(&ValueType::Int), len_var);
        emit!(self.build_store(len_alloca, len_val));
        self.variables.insert(len_var.to_string(), len_alloca);

        // Ensure loop var has an alloca
        if !self.variables.contains_key(loop_var) {
            let alloca = self.build_entry_block_alloca(self.get_llvm_type(loop_var_ty), loop_var);
            self.variables.insert(loop_var.to_string(), alloca);
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
            inkwell::IntPredicate::SLT,
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
        let loop_var_ptr = self.variables[loop_var];
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

    pub(crate) fn codegen_for_str_stmt(
        &mut self,
        loop_var: &str,
        str_var: &str,
        index_var: &str,
        len_var: &str,
        body: &[crate::tir::TirStmt],
        else_body: &[crate::tir::TirStmt],
    ) {
        let function = emit!(self.get_insert_block()).get_parent().unwrap();

        let header_bb = self.context.append_basic_block(function, "forstr.header");
        let body_bb = self.context.append_basic_block(function, "forstr.body");
        let incr_bb = self.context.append_basic_block(function, "forstr.incr");
        let else_bb = if !else_body.is_empty() {
            Some(self.context.append_basic_block(function, "forstr.else"))
        } else {
            None
        };
        let after_bb = self.context.append_basic_block(function, "forstr.after");

        let str_ptr = self.variables[str_var];

        let idx_alloca =
            self.build_entry_block_alloca(self.get_llvm_type(&ValueType::Int), index_var);
        emit!(self.build_store(idx_alloca, self.i64_type().const_zero()));
        self.variables.insert(index_var.to_string(), idx_alloca);

        let str_val = emit!(self.build_load(
            self.context.ptr_type(AddressSpace::default()),
            str_ptr,
            "forstr.str",
        ));
        let str_len_fn = self.get_builtin(BuiltinFn::StrLen);
        let len_call = emit!(self.build_call(str_len_fn, &[str_val.into()], "forstr.len_call"));
        let len_val = self.extract_call_value(len_call);

        let len_alloca =
            self.build_entry_block_alloca(self.get_llvm_type(&ValueType::Int), len_var);
        emit!(self.build_store(len_alloca, len_val));
        self.variables.insert(len_var.to_string(), len_alloca);

        if !self.variables.contains_key(loop_var) {
            let alloca =
                self.build_entry_block_alloca(self.get_llvm_type(&ValueType::Str), loop_var);
            self.variables.insert(loop_var.to_string(), alloca);
        }

        emit!(self.build_unconditional_branch(header_bb));

        self.builder.position_at_end(header_bb);
        let idx_val = emit!(self.build_load(
            self.get_llvm_type(&ValueType::Int),
            idx_alloca,
            "forstr.idx",
        ))
        .into_int_value();
        let len_loaded = emit!(self.build_load(
            self.get_llvm_type(&ValueType::Int),
            len_alloca,
            "forstr.len",
        ))
        .into_int_value();
        let cond = emit!(self.build_int_compare(
            inkwell::IntPredicate::SLT,
            idx_val,
            len_loaded,
            "forstr.cond"
        ));
        let forstr_false_dest = else_bb.unwrap_or(after_bb);
        emit!(self.build_conditional_branch(cond, body_bb, forstr_false_dest));

        self.builder.position_at_end(body_bb);
        let str_reload = emit!(self.build_load(
            self.context.ptr_type(AddressSpace::default()),
            str_ptr,
            "forstr.str2",
        ));
        let idx_reload = emit!(self.build_load(
            self.get_llvm_type(&ValueType::Int),
            idx_alloca,
            "forstr.idx2",
        ));
        let str_get_fn = self.get_builtin(BuiltinFn::StrGetChar);
        let call = emit!(self.build_call(
            str_get_fn,
            &[str_reload.into(), idx_reload.into()],
            "forstr.char",
        ));
        let char_val = self.extract_call_value(call);
        let loop_var_ptr = self.variables[loop_var];
        emit!(self.build_store(loop_var_ptr, char_val));

        loop_body!(self, incr_bb, after_bb, body);

        self.builder.position_at_end(incr_bb);
        let idx_curr = emit!(self.build_load(
            self.get_llvm_type(&ValueType::Int),
            idx_alloca,
            "forstr.idx3",
        ))
        .into_int_value();
        let idx_next = emit!(self.build_int_add(
            idx_curr,
            self.i64_type().const_int(1, false),
            "forstr.idx_next",
        ));
        emit!(self.build_store(idx_alloca, idx_next));
        emit!(self.build_unconditional_branch(header_bb));

        else_body!(self, else_bb, else_body, after_bb);
        self.builder.position_at_end(after_bb);
    }

    pub(crate) fn codegen_for_bytes_stmt(
        &mut self,
        loop_var: &str,
        bytes_var: &str,
        index_var: &str,
        len_var: &str,
        body: &[crate::tir::TirStmt],
        else_body: &[crate::tir::TirStmt],
    ) {
        let function = emit!(self.get_insert_block()).get_parent().unwrap();

        let header_bb = self.context.append_basic_block(function, "forbytes.header");
        let body_bb = self.context.append_basic_block(function, "forbytes.body");
        let incr_bb = self.context.append_basic_block(function, "forbytes.incr");
        let else_bb = if !else_body.is_empty() {
            Some(self.context.append_basic_block(function, "forbytes.else"))
        } else {
            None
        };
        let after_bb = self.context.append_basic_block(function, "forbytes.after");

        let bytes_ptr = self.variables[bytes_var];

        let idx_alloca =
            self.build_entry_block_alloca(self.get_llvm_type(&ValueType::Int), index_var);
        emit!(self.build_store(idx_alloca, self.i64_type().const_zero()));
        self.variables.insert(index_var.to_string(), idx_alloca);

        let bytes_val = emit!(self.build_load(
            self.context.ptr_type(AddressSpace::default()),
            bytes_ptr,
            "forbytes.bytes",
        ));
        let bytes_len_fn = self.get_builtin(BuiltinFn::BytesLen);
        let len_call =
            emit!(self.build_call(bytes_len_fn, &[bytes_val.into()], "forbytes.len_call"));
        let len_val = self.extract_call_value(len_call);

        let len_alloca =
            self.build_entry_block_alloca(self.get_llvm_type(&ValueType::Int), len_var);
        emit!(self.build_store(len_alloca, len_val));
        self.variables.insert(len_var.to_string(), len_alloca);

        if !self.variables.contains_key(loop_var) {
            let alloca =
                self.build_entry_block_alloca(self.get_llvm_type(&ValueType::Int), loop_var);
            self.variables.insert(loop_var.to_string(), alloca);
        }

        emit!(self.build_unconditional_branch(header_bb));

        self.builder.position_at_end(header_bb);
        let idx_val = emit!(self.build_load(
            self.get_llvm_type(&ValueType::Int),
            idx_alloca,
            "forbytes.idx",
        ))
        .into_int_value();
        let len_loaded = emit!(self.build_load(
            self.get_llvm_type(&ValueType::Int),
            len_alloca,
            "forbytes.len",
        ))
        .into_int_value();
        let cond = emit!(self.build_int_compare(
            inkwell::IntPredicate::SLT,
            idx_val,
            len_loaded,
            "forbytes.cond"
        ));
        let forbytes_false_dest = else_bb.unwrap_or(after_bb);
        emit!(self.build_conditional_branch(cond, body_bb, forbytes_false_dest));

        self.builder.position_at_end(body_bb);
        let bytes_reload = emit!(self.build_load(
            self.context.ptr_type(AddressSpace::default()),
            bytes_ptr,
            "forbytes.bytes2",
        ));
        let idx_reload = emit!(self.build_load(
            self.get_llvm_type(&ValueType::Int),
            idx_alloca,
            "forbytes.idx2",
        ));
        let bytes_get_fn = self.get_builtin(BuiltinFn::BytesGet);
        let call = emit!(self.build_call(
            bytes_get_fn,
            &[bytes_reload.into(), idx_reload.into()],
            "forbytes.byte",
        ));
        let byte_val = self.extract_call_value(call);
        let loop_var_ptr = self.variables[loop_var];
        emit!(self.build_store(loop_var_ptr, byte_val));

        loop_body!(self, incr_bb, after_bb, body);

        self.builder.position_at_end(incr_bb);
        let idx_curr = emit!(self.build_load(
            self.get_llvm_type(&ValueType::Int),
            idx_alloca,
            "forbytes.idx3",
        ))
        .into_int_value();
        let idx_next = emit!(self.build_int_add(
            idx_curr,
            self.i64_type().const_int(1, false),
            "forbytes.idx_next",
        ));
        emit!(self.build_store(idx_alloca, idx_next));
        emit!(self.build_unconditional_branch(header_bb));

        else_body!(self, else_bb, else_body, after_bb);
        self.builder.position_at_end(after_bb);
    }

    pub(crate) fn codegen_for_bytearray_stmt(
        &mut self,
        loop_var: &str,
        bytearray_var: &str,
        index_var: &str,
        len_var: &str,
        body: &[crate::tir::TirStmt],
        else_body: &[crate::tir::TirStmt],
    ) {
        let function = emit!(self.get_insert_block()).get_parent().unwrap();

        let header_bb = self
            .context
            .append_basic_block(function, "forbytearray.header");
        let body_bb = self
            .context
            .append_basic_block(function, "forbytearray.body");
        let incr_bb = self
            .context
            .append_basic_block(function, "forbytearray.incr");
        let else_bb = if !else_body.is_empty() {
            Some(
                self.context
                    .append_basic_block(function, "forbytearray.else"),
            )
        } else {
            None
        };
        let after_bb = self
            .context
            .append_basic_block(function, "forbytearray.after");

        let bytearray_ptr = self.variables[bytearray_var];

        let idx_alloca =
            self.build_entry_block_alloca(self.get_llvm_type(&ValueType::Int), index_var);
        emit!(self.build_store(idx_alloca, self.i64_type().const_zero()));
        self.variables.insert(index_var.to_string(), idx_alloca);

        let bytearray_val = emit!(self.build_load(
            self.context.ptr_type(AddressSpace::default()),
            bytearray_ptr,
            "forbytearray.ba",
        ));
        let bytearray_len_fn = self.get_builtin(BuiltinFn::ByteArrayLen);
        let len_call = emit!(self.build_call(
            bytearray_len_fn,
            &[bytearray_val.into()],
            "forbytearray.len_call"
        ));
        let len_val = self.extract_call_value(len_call);

        let len_alloca =
            self.build_entry_block_alloca(self.get_llvm_type(&ValueType::Int), len_var);
        emit!(self.build_store(len_alloca, len_val));
        self.variables.insert(len_var.to_string(), len_alloca);

        if !self.variables.contains_key(loop_var) {
            let alloca =
                self.build_entry_block_alloca(self.get_llvm_type(&ValueType::Int), loop_var);
            self.variables.insert(loop_var.to_string(), alloca);
        }

        emit!(self.build_unconditional_branch(header_bb));

        self.builder.position_at_end(header_bb);
        let idx_val = emit!(self.build_load(
            self.get_llvm_type(&ValueType::Int),
            idx_alloca,
            "forbytearray.idx",
        ))
        .into_int_value();
        let len_loaded = emit!(self.build_load(
            self.get_llvm_type(&ValueType::Int),
            len_alloca,
            "forbytearray.len",
        ))
        .into_int_value();
        let cond = emit!(self.build_int_compare(
            inkwell::IntPredicate::SLT,
            idx_val,
            len_loaded,
            "forbytearray.cond"
        ));
        let forbytearray_false_dest = else_bb.unwrap_or(after_bb);
        emit!(self.build_conditional_branch(cond, body_bb, forbytearray_false_dest));

        self.builder.position_at_end(body_bb);
        let bytearray_reload = emit!(self.build_load(
            self.context.ptr_type(AddressSpace::default()),
            bytearray_ptr,
            "forbytearray.ba2",
        ));
        let idx_reload = emit!(self.build_load(
            self.get_llvm_type(&ValueType::Int),
            idx_alloca,
            "forbytearray.idx2",
        ));
        let bytearray_get_fn = self.get_builtin(BuiltinFn::ByteArrayGet);
        let call = emit!(self.build_call(
            bytearray_get_fn,
            &[bytearray_reload.into(), idx_reload.into()],
            "forbytearray.byte",
        ));
        let byte_val = self.extract_call_value(call);
        let loop_var_ptr = self.variables[loop_var];
        emit!(self.build_store(loop_var_ptr, byte_val));

        loop_body!(self, incr_bb, after_bb, body);

        self.builder.position_at_end(incr_bb);
        let idx_curr = emit!(self.build_load(
            self.get_llvm_type(&ValueType::Int),
            idx_alloca,
            "forbytearray.idx3",
        ))
        .into_int_value();
        let idx_next = emit!(self.build_int_add(
            idx_curr,
            self.i64_type().const_int(1, false),
            "forbytearray.idx_next",
        ));
        emit!(self.build_store(idx_alloca, idx_next));
        emit!(self.build_unconditional_branch(header_bb));

        else_body!(self, else_bb, else_body, after_bb);
        self.builder.position_at_end(after_bb);
    }

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
