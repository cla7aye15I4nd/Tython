use inkwell::values::BasicValueEnum;

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

        self.build_call_maybe_invoke(init_fn, &init_args, "init");

        ptr.into()
    }

    pub(crate) fn codegen_get_field(
        &mut self,
        object: &TirExpr,
        field_index: usize,
        field_ty: &ValueType,
    ) -> BasicValueEnum<'ctx> {
        let obj_ptr = self.codegen_expr(object).into_pointer_value();

        let struct_type = match &object.ty {
            ValueType::Class(name) => self.struct_types[name.as_str()],
            other => panic!("ICE: GetField on unsupported type `{}`", other),
        };

        let field_ptr =
            emit!(self.build_struct_gep(struct_type, obj_ptr, field_index as u32, "field_ptr"));

        let field_llvm_type = self.get_llvm_type(field_ty);
        emit!(self.build_load(field_llvm_type, field_ptr, "field_val"))
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
