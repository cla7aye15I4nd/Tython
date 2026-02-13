use inkwell::types::{FloatType, IntType, StructType};
use inkwell::values::BasicValueEnum;
use inkwell::{AddressSpace, FloatPredicate, IntPredicate};

use crate::ast::ClassInfo;
use crate::tir::builtin::BuiltinFn;
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
            ValueType::Int | ValueType::Bool => self.context.i64_type().into(),
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

    pub(crate) fn build_int_truthiness_check(
        &self,
        value: inkwell::values::IntValue<'ctx>,
        label: &str,
    ) -> inkwell::values::IntValue<'ctx> {
        emit!(self.build_int_compare(
            IntPredicate::NE,
            value,
            self.i64_type().const_int(0, false),
            label,
        ))
    }

    pub(crate) fn build_float_truthiness_check(
        &self,
        value: inkwell::values::FloatValue<'ctx>,
        label: &str,
    ) -> inkwell::values::IntValue<'ctx> {
        emit!(self.build_float_compare(
            FloatPredicate::ONE,
            value,
            self.f64_type().const_float(0.0),
            label,
        ))
    }

    pub(crate) fn build_truthiness_check_for_value(
        &self,
        value: BasicValueEnum<'ctx>,
        ty: &ValueType,
        label: &str,
    ) -> inkwell::values::IntValue<'ctx> {
        macro_rules! seq_truthiness {
            ($($variant:ident => $builtin:ident),+ $(,)?) => {
                match ty {
                    ValueType::Float => self.build_float_truthiness_check(value.into_float_value(), label),
                    $(
                        ValueType::$variant => {
                            let func = self.get_builtin(BuiltinFn::$builtin);
                            let call = emit!(self.build_call(func, &[value.into()], "len_truth"));
                            let len_val = self.extract_call_value(call).into_int_value();
                            self.build_int_truthiness_check(len_val, label)
                        }
                    )+
                    ValueType::List(_) => {
                        let func = self.get_builtin(BuiltinFn::ListLen);
                        let call = emit!(self.build_call(func, &[value.into()], "len_truth"));
                        let len_val = self.extract_call_value(call).into_int_value();
                        self.build_int_truthiness_check(len_val, label)
                    }
                    ValueType::Dict(_, _) => {
                        let func = self.get_builtin(BuiltinFn::DictLen);
                        let call = emit!(self.build_call(func, &[value.into()], "len_truth"));
                        let len_val = self.extract_call_value(call).into_int_value();
                        self.build_int_truthiness_check(len_val, label)
                    }
                    ValueType::Set(_) => {
                        let func = self.get_builtin(BuiltinFn::SetLen);
                        let call = emit!(self.build_call(func, &[value.into()], "len_truth"));
                        let len_val = self.extract_call_value(call).into_int_value();
                        self.build_int_truthiness_check(len_val, label)
                    }
                    ValueType::Tuple(elements) => self
                        .context
                        .bool_type()
                        .const_int((!elements.is_empty()) as u64, false),
                    ValueType::Class(_) => self.i64_type().const_int(1, false),
                    _ => self.build_int_truthiness_check(value.into_int_value(), label),
                }
            };
        }
        seq_truthiness! {
            Str => StrLen,
            Bytes => BytesLen,
            ByteArray => ByteArrayLen,
        }
    }

    predicate_map!(float_predicate -> FloatPredicate {
        Eq => FloatPredicate::OEQ, NotEq => FloatPredicate::ONE,
        Lt => FloatPredicate::OLT, LtEq => FloatPredicate::OLE,
        Gt => FloatPredicate::OGT, GtEq => FloatPredicate::OGE,
    });

    predicate_map!(int_predicate -> IntPredicate {
        Eq => IntPredicate::EQ,  NotEq => IntPredicate::NE,
        Lt => IntPredicate::SLT, LtEq => IntPredicate::SLE,
        Gt => IntPredicate::SGT, GtEq => IntPredicate::SGE,
    });
}
