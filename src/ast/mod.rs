#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    Int,
    Bool,
    Function {
        params: Vec<Type>,
        return_type: Box<Type>,
    },
    Module(String),
    Unit,
}
