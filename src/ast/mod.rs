use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    Int,
    Float,
    Bool,
    Str,
    Bytes,
    ByteArray,
    Function {
        params: Vec<Type>,
        return_type: Box<Type>,
    },
    Module(String),
    Unit,
    Class(String),
}

impl Type {
    pub fn is_reference_type(&self) -> bool {
        matches!(
            self,
            Type::Class(_) | Type::Str | Type::Bytes | Type::ByteArray
        )
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Int => write!(f, "int"),
            Type::Float => write!(f, "float"),
            Type::Bool => write!(f, "bool"),
            Type::Str => write!(f, "str"),
            Type::Bytes => write!(f, "bytes"),
            Type::ByteArray => write!(f, "bytearray"),
            Type::Unit => write!(f, "None"),
            Type::Module(path) => write!(f, "module '{}'", path),
            Type::Class(name) => write!(f, "{}", name),
            Type::Function {
                params,
                return_type,
            } => {
                write!(f, "(")?;
                for (i, p) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", p)?;
                }
                write!(f, ") -> {}", return_type)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct ClassField {
    pub name: String,
    pub ty: Type,
    pub index: usize,
}

#[derive(Debug, Clone)]
pub struct ClassMethod {
    pub name: String,
    pub params: Vec<Type>,
    pub return_type: Type,
    pub mangled_name: String,
}

#[derive(Debug, Clone)]
pub struct ClassInfo {
    pub name: String,
    pub fields: Vec<ClassField>,
    pub methods: HashMap<String, ClassMethod>,
    pub field_map: HashMap<String, usize>,
}
