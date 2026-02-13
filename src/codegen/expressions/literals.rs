use inkwell::values::BasicValueEnum;

use crate::tir::ValueType;

use super::super::runtime_fn::RuntimeFn;
use super::super::Codegen;

impl<'ctx> Codegen<'ctx> {
    pub(crate) fn codegen_str_literal(&self, s: &str) -> BasicValueEnum<'ctx> {
        let global = emit!(self.build_global_string_ptr(s, "str_data"));
        let data_ptr = global.as_pointer_value();
        let len = self.i64_type().const_int(s.len() as u64, false);
        let str_new_fn = self.get_runtime_fn(RuntimeFn::StrNew);
        let call = emit!(self.build_call(str_new_fn, &[data_ptr.into(), len.into()], "str_new"));
        self.extract_call_value(call)
    }

    pub(crate) fn codegen_bytes_literal(&self, bytes: &[u8]) -> BasicValueEnum<'ctx> {
        let byte_values: Vec<_> = bytes
            .iter()
            .map(|b| self.context.i8_type().const_int(*b as u64, false))
            .collect();
        let array_val = self.context.i8_type().const_array(&byte_values);
        let array_type = self.context.i8_type().array_type(bytes.len() as u32);
        let global = self.module.add_global(array_type, None, "bytes_data");
        global.set_initializer(&array_val);
        global.set_constant(true);
        let data_ptr = global.as_pointer_value();
        let len = self.i64_type().const_int(bytes.len() as u64, false);
        let bytes_new_fn = self.get_runtime_fn(RuntimeFn::BytesNew);
        let call =
            emit!(self.build_call(bytes_new_fn, &[data_ptr.into(), len.into()], "bytes_new"));
        self.extract_call_value(call)
    }

    pub(crate) fn codegen_var_load(&mut self, name: &str, ty: &ValueType) -> BasicValueEnum<'ctx> {
        let ptr = if let Some(ptr) = self.variables.get(name).copied() {
            ptr
        } else if let Some(ptr) = self.global_variables.get(name).copied() {
            ptr
        } else {
            let g = self.module.add_global(self.get_llvm_type(ty), None, name);
            g.set_initializer(&self.get_llvm_type(ty).const_zero());
            let p = g.as_pointer_value();
            self.global_variables.insert(name.to_string(), p);
            p
        };
        emit!(self.build_load(self.get_llvm_type(ty), ptr, name))
    }
}
