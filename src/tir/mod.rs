pub mod lower;

use crate::ast::Type;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOpKind {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
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

#[derive(Debug, Clone)]
pub struct TirModule {
    pub functions: HashMap<String, TirFunction>,
}

#[derive(Debug, Clone)]
pub struct TirFunction {
    pub name: String,
    pub params: Vec<FunctionParam>,
    pub return_type: Type,
    pub body: Vec<TirStmt>,
}

#[derive(Debug, Clone)]
pub enum TirStmt {
    Let {
        name: String,
        ty: Type,
        value: TirExpr,
    },
    Return(Option<TirExpr>),
    Expr(TirExpr),
}

#[derive(Debug, Clone)]
pub struct TirExpr {
    pub kind: TirExprKind,
    pub ty: Type,
}

#[derive(Debug, Clone)]
pub enum TirExprKind {
    IntLiteral(i64),
    Var(String),
    BinOp {
        op: BinOpKind,
        left: Box<TirExpr>,
        right: Box<TirExpr>,
    },
    Call {
        func: String,
        args: Vec<TirExpr>,
    },
}
