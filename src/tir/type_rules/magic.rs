use crate::tir::ValueType;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuiltinClassMagicRule {
    pub method_names: &'static [&'static str],
    pub return_type: ValueType,
}

pub fn lookup_builtin_class_magic(name: &str) -> Option<BuiltinClassMagicRule> {
    match name {
        "str" => Some(BuiltinClassMagicRule {
            method_names: &["__str__", "__repr__"],
            return_type: ValueType::Str,
        }),
        "repr" => Some(BuiltinClassMagicRule {
            method_names: &["__repr__"],
            return_type: ValueType::Str,
        }),
        "len" => Some(BuiltinClassMagicRule {
            method_names: &["__len__"],
            return_type: ValueType::Int,
        }),
        _ => None,
    }
}
