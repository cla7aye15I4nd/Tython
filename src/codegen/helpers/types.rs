use inkwell::types::{FloatType, IntType};
use inkwell::AddressSpace;

use crate::tir::{TirClassInfo, ValueType};

use super::super::Codegen;

impl<'ctx> Codegen<'ctx> {
    pub fn register_class(&mut self, class_info: &TirClassInfo) {
        let field_types: Vec<inkwell::types::BasicTypeEnum<'ctx>> = class_info
            .fields
            .iter()
            .map(|f| self.get_llvm_type(&f.ty))
            .collect();

        let struct_type = self.context.opaque_struct_type(&class_info.name);
        struct_type.set_body(&field_types, false);

        self.struct_types
            .insert(class_info.name.clone(), struct_type);
    }

    pub(crate) fn get_llvm_type(&self, ty: &ValueType) -> inkwell::types::BasicTypeEnum<'ctx> {
        match ty {
            ValueType::Int => self.context.i64_type().into(),
            ValueType::Bool => self.context.bool_type().into(),
            ValueType::Float => self.context.f64_type().into(),
            ValueType::Str
            | ValueType::File
            | ValueType::Bytes
            | ValueType::ByteArray
            | ValueType::List(_)
            | ValueType::Dict(_, _)
            | ValueType::Set(_)
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

    pub(crate) fn bool_type(&self) -> IntType<'ctx> {
        self.context.bool_type()
    }

    // Note: float_predicate and int_predicate removed - we now use TypedCompare
    // which encodes both the comparison operator and operand type, eliminating
    // the need for runtime type dispatch in codegen.
}
