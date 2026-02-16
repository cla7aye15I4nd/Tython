use super::super::Codegen;
use crate::tir::ValueType;
use inkwell::values::BasicValueEnum;

impl<'ctx> Codegen<'ctx> {
    fn codegen_byte_array_literal(&self, bytes: &[u8], name: &str) -> BasicValueEnum<'ctx> {
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
        if matches!(ty, ValueType::Function { .. }) {
            if let Some(ptr) = self.variables.get(name).copied() {
                return emit!(self.build_load(self.get_llvm_type(ty), ptr, name));
            }
            let suffix = format!("${}", name);
            let func = self.module.get_function(name).or_else(|| {
                self.module.get_functions().find(|f| {
                    f.get_name()
                        .to_str()
                        .is_ok_and(|candidate| candidate.ends_with(&suffix))
                })
            });
            return func
                .expect("ICE: function symbol not found for function value expression")
                .as_global_value()
                .as_pointer_value()
                .into();
        }
        let ptr = self.variables[name];
        emit!(self.build_load(self.get_llvm_type(ty), ptr, name))
    }
}
