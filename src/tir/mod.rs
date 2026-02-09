pub mod builder;

use crate::ast::{BinOpKind, FunctionParam, Type};
use std::collections::HashMap;
use std::path::PathBuf;

/// Typed IR module - fully type-checked representation
#[derive(Debug, Clone)]
pub struct TirModule {
    pub path: PathBuf,
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
    /// Local variable binding
    Let {
        name: String,
        ty: Type,
        value: TirExpr,
    },

    /// Return statement
    Return(Option<TirExpr>),

    /// Expression statement (for side effects like print)
    Expr(TirExpr),
}

#[derive(Debug, Clone)]
pub struct TirExpr {
    pub kind: TirExprKind,
    pub ty: Type, // Always present in TIR
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
