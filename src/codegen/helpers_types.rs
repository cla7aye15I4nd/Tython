use inkwell::types::{FloatType, IntType, StructType};
use inkwell::AddressSpace;

use crate::ast::ClassInfo;
use crate::tir::ValueType;

use super::Codegen;

impl<'ctx> Codegen<'ctx> {
    pub fn register_class(&mut self, class_info: &ClassInfo) {
        let field_types: Vec<inkwell::types::BasicTypeEnum<'ctx>> = class_info
            .fields
            .iter()
            .map(|f| {
                let vty = ValueType::from_type(&f.ty).expect("ICE: class field has non-value type");
                self.get_llvm_type(&vty)
            })
            .collect();

        let struct_type = self.context.opaque_struct_type(&class_info.name);
        struct_type.set_body(&field_types, false);

        self.struct_types
            .insert(class_info.name.clone(), struct_type);
    }

    pub(crate) fn tuple_signature_key(elem_types: &[ValueType]) -> String {
        elem_types
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("|")
    }

    pub(crate) fn get_or_create_tuple_struct(
        &mut self,
        elem_types: &[ValueType],
    ) -> StructType<'ctx> {
        let key = Self::tuple_signature_key(elem_types);
        if let Some(existing) = self.struct_types.get(&key) {
            return *existing;
        }

        let struct_name = format!("__tython_tuple${}", self.struct_types.len());
        let field_types: Vec<inkwell::types::BasicTypeEnum<'ctx>> =
            elem_types.iter().map(|ty| self.get_llvm_type(ty)).collect();

        let struct_type = self.context.opaque_struct_type(&struct_name);
        struct_type.set_body(&field_types, false);
        self.struct_types.insert(key, struct_type);
        struct_type
    }

    pub(crate) fn get_llvm_type(&self, ty: &ValueType) -> inkwell::types::BasicTypeEnum<'ctx> {
        match ty {
            ValueType::Int => self.context.i64_type().into(),
            ValueType::Bool => self.context.bool_type().into(),
            ValueType::Float => self.context.f64_type().into(),
            ValueType::Str
            | ValueType::Bytes
            | ValueType::ByteArray
            | ValueType::List(_)
            | ValueType::Dict(_, _)
            | ValueType::Set(_)
            | ValueType::Tuple(_)
            | ValueType::Class(_)
            | ValueType::Function { .. } => self.context.ptr_type(AddressSpace::default()).into(),
        }
    }

    pub(crate) fn i64_type(&self) -> IntType<'ctx> {
        self.context.i64_type()
    }

    pub(crate) fn f64_type(&self) -> FloatType<'ctx> {
        self.context.f64_type()
    }

    // Note: float_predicate and int_predicate removed - we now use TypedCompare
    // which encodes both the comparison operator and operand type, eliminating
    // the need for runtime type dispatch in codegen.
}
