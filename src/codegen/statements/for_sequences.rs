use inkwell::values::{BasicValueEnum, CallSiteValue};
use inkwell::AddressSpace;

use crate::tir::builtin::BuiltinFn;
use crate::tir::{TirStmt, ValueType};

use super::super::Codegen;

enum IndexedSequenceKind<'a> {
    List { loop_var_ty: &'a ValueType },
    Str,
    Bytes,
    ByteArray,
}

impl<'a> IndexedSequenceKind<'a> {
    fn label_prefix(&self) -> &'static str {
        match self {
            Self::List { .. } => "forlist",
            Self::Str => "forstr",
            Self::Bytes => "forbytes",
            Self::ByteArray => "forbytearray",
        }
    }

    fn len_builtin(&self) -> BuiltinFn {
        match self {
            Self::List { .. } => BuiltinFn::ListLen,
            Self::Str => BuiltinFn::StrLen,
            Self::Bytes => BuiltinFn::BytesLen,
            Self::ByteArray => BuiltinFn::ByteArrayLen,
        }
    }

    fn get_builtin(&self) -> BuiltinFn {
        match self {
            Self::List { .. } => BuiltinFn::ListGet,
            Self::Str => BuiltinFn::StrGetChar,
            Self::Bytes => BuiltinFn::BytesGet,
            Self::ByteArray => BuiltinFn::ByteArrayGet,
        }
    }

    fn loop_var_type(&self) -> ValueType {
        match self {
            Self::List { loop_var_ty } => (*loop_var_ty).clone(),
            Self::Str => ValueType::Str,
            Self::Bytes | Self::ByteArray => ValueType::Int,
        }
    }

    fn decode_element<'ctx>(
        &self,
        codegen: &mut Codegen<'ctx>,
        call: CallSiteValue<'ctx>,
    ) -> BasicValueEnum<'ctx> {
        match self {
            Self::List { loop_var_ty } => {
                let elem_i64 = codegen.extract_call_value(call).into_int_value();
                codegen.bitcast_from_i64(elem_i64, loop_var_ty)
            }
            Self::Str | Self::Bytes | Self::ByteArray => codegen.extract_call_value(call),
        }
    }
}

struct IndexedSequenceLoop<'a> {
    kind: IndexedSequenceKind<'a>,
    loop_var: &'a str,
    sequence_var: &'a str,
    index_var: &'a str,
    len_var: &'a str,
    body: &'a [TirStmt],
    else_body: &'a [TirStmt],
}

impl<'ctx> Codegen<'ctx> {
    pub(crate) fn codegen_for_list_stmt(
        &mut self,
        loop_var: &str,
        loop_var_ty: &ValueType,
        list_var: &str,
        index_var: &str,
        len_var: &str,
        body: &[TirStmt],
        else_body: &[TirStmt],
    ) {
        self.codegen_indexed_sequence_loop(IndexedSequenceLoop {
            kind: IndexedSequenceKind::List { loop_var_ty },
            loop_var,
            sequence_var: list_var,
            index_var,
            len_var,
            body,
            else_body,
        });
    }

    pub(crate) fn codegen_for_str_stmt(
        &mut self,
        loop_var: &str,
        str_var: &str,
        index_var: &str,
        len_var: &str,
        body: &[TirStmt],
        else_body: &[TirStmt],
    ) {
        self.codegen_indexed_sequence_loop(IndexedSequenceLoop {
            kind: IndexedSequenceKind::Str,
            loop_var,
            sequence_var: str_var,
            index_var,
            len_var,
            body,
            else_body,
        });
    }

    pub(crate) fn codegen_for_bytes_stmt(
        &mut self,
        loop_var: &str,
        bytes_var: &str,
        index_var: &str,
        len_var: &str,
        body: &[TirStmt],
        else_body: &[TirStmt],
    ) {
        self.codegen_indexed_sequence_loop(IndexedSequenceLoop {
            kind: IndexedSequenceKind::Bytes,
            loop_var,
            sequence_var: bytes_var,
            index_var,
            len_var,
            body,
            else_body,
        });
    }

    pub(crate) fn codegen_for_bytearray_stmt(
        &mut self,
        loop_var: &str,
        bytearray_var: &str,
        index_var: &str,
        len_var: &str,
        body: &[TirStmt],
        else_body: &[TirStmt],
    ) {
        self.codegen_indexed_sequence_loop(IndexedSequenceLoop {
            kind: IndexedSequenceKind::ByteArray,
            loop_var,
            sequence_var: bytearray_var,
            index_var,
            len_var,
            body,
            else_body,
        });
    }

    fn codegen_indexed_sequence_loop(&mut self, cfg: IndexedSequenceLoop<'_>) {
        let function = emit!(self.get_insert_block()).get_parent().unwrap();
        let prefix = cfg.kind.label_prefix();

        let header_bb = self
            .context
            .append_basic_block(function, &format!("{}.header", prefix));
        let body_bb = self
            .context
            .append_basic_block(function, &format!("{}.body", prefix));
        let incr_bb = self
            .context
            .append_basic_block(function, &format!("{}.incr", prefix));
        let else_bb = if !cfg.else_body.is_empty() {
            Some(
                self.context
                    .append_basic_block(function, &format!("{}.else", prefix)),
            )
        } else {
            None
        };
        let after_bb = self
            .context
            .append_basic_block(function, &format!("{}.after", prefix));

        let sequence_ptr = self.variables[cfg.sequence_var];

        let idx_alloca =
            self.build_entry_block_alloca(self.get_llvm_type(&ValueType::Int), cfg.index_var);
        emit!(self.build_store(idx_alloca, self.i64_type().const_zero()));
        self.variables.insert(cfg.index_var.to_string(), idx_alloca);

        let sequence_val = emit!(self.build_load(
            self.context.ptr_type(AddressSpace::default()),
            sequence_ptr,
            &format!("{}.seq", prefix),
        ));
        let len_fn = self.get_builtin(cfg.kind.len_builtin());
        let len_call = emit!(self.build_call(
            len_fn,
            &[sequence_val.into()],
            &format!("{}.len_call", prefix),
        ));
        let len_val = self.extract_call_value(len_call);

        let len_alloca =
            self.build_entry_block_alloca(self.get_llvm_type(&ValueType::Int), cfg.len_var);
        emit!(self.build_store(len_alloca, len_val));
        self.variables.insert(cfg.len_var.to_string(), len_alloca);

        if !self.variables.contains_key(cfg.loop_var) {
            let alloca = self.build_entry_block_alloca(
                self.get_llvm_type(&cfg.kind.loop_var_type()),
                cfg.loop_var,
            );
            self.variables.insert(cfg.loop_var.to_string(), alloca);
        }

        emit!(self.build_unconditional_branch(header_bb));

        self.builder.position_at_end(header_bb);
        let idx_val = emit!(self.build_load(
            self.get_llvm_type(&ValueType::Int),
            idx_alloca,
            &format!("{}.idx", prefix),
        ))
        .into_int_value();
        let len_loaded = emit!(self.build_load(
            self.get_llvm_type(&ValueType::Int),
            len_alloca,
            &format!("{}.len", prefix),
        ))
        .into_int_value();
        let cond = emit!(self.build_int_compare(
            inkwell::IntPredicate::SLT,
            idx_val,
            len_loaded,
            &format!("{}.cond", prefix)
        ));
        let false_dest = else_bb.unwrap_or(after_bb);
        emit!(self.build_conditional_branch(cond, body_bb, false_dest));

        self.builder.position_at_end(body_bb);
        let sequence_reload = emit!(self.build_load(
            self.context.ptr_type(AddressSpace::default()),
            sequence_ptr,
            &format!("{}.seq2", prefix),
        ));
        let idx_reload = emit!(self.build_load(
            self.get_llvm_type(&ValueType::Int),
            idx_alloca,
            &format!("{}.idx2", prefix),
        ));
        let get_fn = self.get_builtin(cfg.kind.get_builtin());
        let call = emit!(self.build_call(
            get_fn,
            &[sequence_reload.into(), idx_reload.into()],
            &format!("{}.elem", prefix),
        ));
        let elem_val = cfg.kind.decode_element(self, call);
        let loop_var_ptr = self.variables[cfg.loop_var];
        emit!(self.build_store(loop_var_ptr, elem_val));

        loop_body!(self, incr_bb, after_bb, cfg.body);

        self.builder.position_at_end(incr_bb);
        let idx_curr = emit!(self.build_load(
            self.get_llvm_type(&ValueType::Int),
            idx_alloca,
            &format!("{}.idx3", prefix),
        ))
        .into_int_value();
        let idx_next = emit!(self.build_int_add(
            idx_curr,
            self.i64_type().const_int(1, false),
            &format!("{}.idx_next", prefix),
        ));
        emit!(self.build_store(idx_alloca, idx_next));
        emit!(self.build_unconditional_branch(header_bb));

        else_body!(self, else_bb, cfg.else_body, after_bb);
        self.builder.position_at_end(after_bb);
    }
}
