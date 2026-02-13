use inkwell::values::BasicValueEnum;

use crate::tir::ValueType;

use super::super::Codegen;

impl<'ctx> Codegen<'ctx> {
    fn codegen_byte_array_literal(&self, bytes: &[u8], name: &str) -> BasicValueEnum<'ctx> {
        // Create struct: { i64 len, [i8 x N] data }
        let len_field = self.i64_type().const_int(bytes.len() as u64, false);
        let byte_values: Vec<_> = bytes
            .iter()
            .map(|b| self.context.i8_type().const_int(*b as u64, false))
            .collect();
        let data_array = self.context.i8_type().const_array(&byte_values);

        let struct_type = self.context.struct_type(
            &[
                self.i64_type().into(),
                self.context.i8_type().array_type(bytes.len() as u32).into(),
            ],
            false,
        );
        let struct_value = struct_type.const_named_struct(&[len_field.into(), data_array.into()]);

        let global = self.module.add_global(struct_type, None, name);
        global.set_initializer(&struct_value);
        global.set_constant(true);

        global.as_pointer_value().into()
    }

    pub(crate) fn codegen_str_literal(&self, s: &str) -> BasicValueEnum<'ctx> {
        self.codegen_byte_array_literal(s.as_bytes(), "str_literal")
    }

    pub(crate) fn codegen_bytes_literal(&self, bytes: &[u8]) -> BasicValueEnum<'ctx> {
        self.codegen_byte_array_literal(bytes, "bytes_literal")
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
