use crate::tir::ValueType;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuiltinClassMagicRule {
    pub method_names: &'static [&'static str],
    /// `Some(ty)` = validate the method returns exactly `ty`.
    /// `None` = infer the return type from the method declaration.
    pub return_type: Option<ValueType>,
}

pub fn lookup_builtin_class_magic(name: &str) -> Option<BuiltinClassMagicRule> {
    match name {
        "str" => Some(BuiltinClassMagicRule {
            method_names: &["__str__", "__repr__"],
            return_type: Some(ValueType::Str),
        }),
        "repr" => Some(BuiltinClassMagicRule {
            method_names: &["__repr__"],
            return_type: Some(ValueType::Str),
        }),
        "len" => Some(BuiltinClassMagicRule {
            method_names: &["__len__"],
            return_type: Some(ValueType::Int),
        }),
        "iter" => Some(BuiltinClassMagicRule {
            method_names: &["__iter__"],
            return_type: None,
        }),
        "next" => Some(BuiltinClassMagicRule {
            method_names: &["__next__"],
            return_type: None,
        }),
        _ => None,
    }
}
