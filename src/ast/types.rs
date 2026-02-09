/// Core type system for Tython
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    /// 64-bit signed integer (Python int)
    Int,

    /// Function type: parameter types → return type
    Function {
        params: Vec<Type>,
        return_type: Box<Type>,
    },

    /// Unit type (similar to void, for statements)
    Unit,

    /// Unknown type (for inference)
    Unknown,
}

impl Type {
    pub fn is_unit(&self) -> bool {
        matches!(self, Type::Unit)
    }
}
