use inkwell::values::BasicValueEnum;
use inkwell::{AddressSpace, IntPredicate};

use crate::tir::builtin::BuiltinFn;
use crate::tir::{TirExpr, ValueType};

use super::super::runtime_fn::RuntimeFn;
use super::super::Codegen;

impl<'ctx> Codegen<'ctx> {
    pub(crate) fn codegen_construct(
        &mut self,
        class_name: &str,
        init_mangled_name: &str,
        args: &[TirExpr],
    ) -> BasicValueEnum<'ctx> {
        let struct_type = self.struct_types[class_name];

        // Allocate heap memory for the struct
        let size = struct_type.size_of().unwrap();
        let size_i64 = emit!(self.build_int_cast(size, self.i64_type(), "size_i64"));
        let malloc_fn = self.get_runtime_fn(RuntimeFn::Malloc);
        let call_site = emit!(self.build_call(malloc_fn, &[size_i64.into()], "malloc"));
        let ptr = self.extract_call_value(call_site).into_pointer_value();

        // Build full arg list: [self_ptr, ...args]
        let mut init_args: Vec<inkwell::values::BasicValueEnum> = vec![ptr.into()];
        init_args.extend(self.codegen_call_args(args));

        // Declare/get __init__ function
        let mut param_types = vec![ValueType::Class(class_name.to_string())];
        param_types.extend(args.iter().map(|a| a.ty.clone()));
        let init_fn = self.get_or_declare_function(init_mangled_name, &param_types, None);

        self.build_call_maybe_invoke(init_fn, &init_args, "init", true);

        ptr.into()
    }

    pub(crate) fn codegen_get_field(
        &mut self,
        object: &TirExpr,
        class_name: &str,
        field_index: usize,
        field_ty: &ValueType,
    ) -> BasicValueEnum<'ctx> {
        let obj_ptr = self.codegen_expr(object).into_pointer_value();
        let struct_type = self.struct_types[class_name];

        let field_ptr =
            emit!(self.build_struct_gep(struct_type, obj_ptr, field_index as u32, "field_ptr"));

        let field_llvm_type = self.get_llvm_type(field_ty);
        emit!(self.build_load(field_llvm_type, field_ptr, "field_val"))
    }

    pub(crate) fn codegen_tuple_literal(
        &mut self,
        elements: &[TirExpr],
        element_types: &[ValueType],
    ) -> BasicValueEnum<'ctx> {
        let struct_type = self.get_or_create_tuple_struct(element_types);
        let size = struct_type.size_of().unwrap();
        let size_i64 = emit!(self.build_int_cast(size, self.i64_type(), "tuple_size_i64"));
        let malloc_fn = self.get_runtime_fn(RuntimeFn::Malloc);
        let call_site = emit!(self.build_call(malloc_fn, &[size_i64.into()], "tuple_malloc"));
        let tuple_ptr = self.extract_call_value(call_site).into_pointer_value();

        for (i, elem) in elements.iter().enumerate() {
            let field_ptr =
                emit!(self.build_struct_gep(struct_type, tuple_ptr, i as u32, "tuple_field_ptr"));
            let elem_val = self.codegen_expr(elem);
            emit!(self.build_store(field_ptr, elem_val));
        }
        tuple_ptr.into()
    }

    pub(crate) fn codegen_tuple_get(
        &mut self,
        tuple: &TirExpr,
        index: usize,
        element_types: &[ValueType],
        result_ty: &ValueType,
    ) -> BasicValueEnum<'ctx> {
        let tuple_ptr = self.codegen_expr(tuple).into_pointer_value();
        let struct_type = self.get_or_create_tuple_struct(element_types);
        let field_ptr =
            emit!(self.build_struct_gep(struct_type, tuple_ptr, index as u32, "tuple_get_ptr"));
        emit!(self.build_load(self.get_llvm_type(result_ty), field_ptr, "tuple_get"))
    }

    pub(crate) fn codegen_tuple_get_dynamic(
        &mut self,
        tuple: &TirExpr,
        index: &TirExpr,
        len: usize,
        element_types: &[ValueType],
        result_ty: &ValueType,
    ) -> BasicValueEnum<'ctx> {
        let tuple_ptr = self.codegen_expr(tuple).into_pointer_value();
        let idx_val = self.codegen_expr(index).into_int_value();
        let struct_type = self.get_or_create_tuple_struct(element_types);

        let result_alloca =
            self.build_entry_block_alloca(self.get_llvm_type(result_ty), "tuple_dyn_get_tmp");

        let default_val: BasicValueEnum<'ctx> = match result_ty {
            ValueType::Int => self.i64_type().const_zero().into(),
            ValueType::Bool => self.context.bool_type().const_zero().into(),
            ValueType::Float => self.f64_type().const_float(0.0).into(),
            _ => self
                .context
                .ptr_type(AddressSpace::default())
                .const_null()
                .into(),
        };
        emit!(self.build_store(result_alloca, default_val));

        let len_i64 = self.i64_type().const_int(len as u64, false);
        let is_neg = emit!(self.build_int_compare(
            IntPredicate::SLT,
            idx_val,
            self.i64_type().const_zero(),
            "tuple_idx_neg",
        ));
        let neg_adjusted = emit!(self.build_int_add(idx_val, len_i64, "tuple_idx_norm_neg"));
        let norm_idx = emit!(self.build_select(is_neg, neg_adjusted, idx_val, "tuple_idx_norm"))
            .into_int_value();

        let function = emit!(self.get_insert_block()).get_parent().unwrap();
        let default_bb = self
            .context
            .append_basic_block(function, "tuple_idx_default");
        let merge_bb = self.context.append_basic_block(function, "tuple_idx_merge");

        let mut case_bbs = Vec::with_capacity(len);
        for i in 0..len {
            case_bbs.push(
                self.context
                    .append_basic_block(function, &format!("tuple_idx_case_{}", i)),
            );
        }
        let switch_cases: Vec<_> = case_bbs
            .iter()
            .enumerate()
            .map(|(i, bb)| (self.i64_type().const_int(i as u64, false), *bb))
            .collect();
        emit!(self.build_switch(norm_idx, default_bb, &switch_cases));

        for (i, case_bb) in case_bbs.iter().enumerate() {
            self.builder.position_at_end(*case_bb);
            let field_ptr =
                emit!(self.build_struct_gep(struct_type, tuple_ptr, i as u32, "tuple_dyn_get_ptr"));
            let field_val =
                emit!(self.build_load(self.get_llvm_type(result_ty), field_ptr, "tuple_dyn_get"));
            emit!(self.build_store(result_alloca, field_val));
            emit!(self.build_unconditional_branch(merge_bb));
        }

        self.builder.position_at_end(default_bb);
        emit!(self.build_unconditional_branch(merge_bb));

        self.builder.position_at_end(merge_bb);
        emit!(self.build_load(
            self.get_llvm_type(result_ty),
            result_alloca,
            "tuple_dyn_get_out",
        ))
    }

    pub(crate) fn codegen_list_literal(
        &mut self,
        element_type: &ValueType,
        elements: &[TirExpr],
    ) -> BasicValueEnum<'ctx> {
        if elements.is_empty() {
            let empty_fn = self.get_builtin(BuiltinFn::ListEmpty);
            let call = emit!(self.build_call(empty_fn, &[], "list_empty"));
            self.extract_call_value(call)
        } else {
            let len = elements.len();
            let i64_ty = self.i64_type();
            let array_ty = i64_ty.array_type(len as u32);
            let array_alloca = self.build_entry_block_alloca(array_ty.into(), "list_data");

            for (i, elem) in elements.iter().enumerate() {
                let val = self.codegen_expr(elem);
                let i64_val = self.bitcast_to_i64(val, element_type);
                let zero = self.context.i32_type().const_int(0, false);
                let idx = self.context.i32_type().const_int(i as u64, false);
                let elem_ptr = unsafe {
                    emit!(self.build_in_bounds_gep(
                        array_ty,
                        array_alloca,
                        &[zero, idx],
                        "elem_ptr",
                    ))
                };
                emit!(self.build_store(elem_ptr, i64_val));
            }

            let len_val = i64_ty.const_int(len as u64, false);
            let list_new_fn = self.get_runtime_fn(RuntimeFn::ListNew);
            let call = emit!(self.build_call(
                list_new_fn,
                &[array_alloca.into(), len_val.into()],
                "list_new",
            ));
            self.extract_call_value(call)
        }
    }
}
