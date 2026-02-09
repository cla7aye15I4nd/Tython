use super::expr::{Expr, Span};
use super::types::Type;

#[derive(Debug, Clone)]
pub struct Stmt {
    pub kind: StmtKind,
    pub span: Span,
}

impl Stmt {
    pub fn new(kind: StmtKind, span: Span) -> Self {
        Self { kind, span }
    }
}

#[derive(Debug, Clone)]
pub enum StmtKind {
    /// Function definition
    FunctionDef {
        name: String,
        params: Vec<FunctionParam>,
        return_type: Type,
        body: Vec<Stmt>,
    },

    /// Variable assignment with type annotation
    Assign {
        target: String,
        ty: Option<Type>,
        value: Expr,
    },

    /// Return statement
    Return(Option<Expr>),

    /// Expression statement (for print, calls, etc.)
    Expr(Expr),
}

#[derive(Debug, Clone)]
pub struct FunctionParam {
    pub name: String,
    pub ty: Type,
}

impl FunctionParam {
    pub fn new(name: String, ty: Type) -> Self {
        Self { name, ty }
    }
}
