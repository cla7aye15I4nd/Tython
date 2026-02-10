pub mod builtin;
pub mod lower;
pub mod type_rules;

use crate::ast::Type;
use std::collections::HashMap;

// ── Value types (types with LLVM register representations) ──────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ValueType {
    Int,
    Float,
    Bool,
}

impl ValueType {
    pub fn from_type(ty: &Type) -> Option<Self> {
        match ty {
            Type::Int => Some(ValueType::Int),
            Type::Float => Some(ValueType::Float),
            Type::Bool => Some(ValueType::Bool),
            _ => None,
        }
    }

    pub fn to_type(self) -> Type {
        match self {
            ValueType::Int => Type::Int,
            ValueType::Float => Type::Float,
            ValueType::Bool => Type::Bool,
        }
    }
}

impl std::fmt::Display for ValueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValueType::Int => write!(f, "int"),
            ValueType::Float => write!(f, "float"),
            ValueType::Bool => write!(f, "bool"),
        }
    }
}

// ── Binary operations ───────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArithBinOp {
    Add,
    Sub,
    Mul,
    Div,
    FloorDiv,
    Mod,
    Pow,
}

impl std::fmt::Display for ArithBinOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArithBinOp::Add => write!(f, "+"),
            ArithBinOp::Sub => write!(f, "-"),
            ArithBinOp::Mul => write!(f, "*"),
            ArithBinOp::Div => write!(f, "/"),
            ArithBinOp::FloorDiv => write!(f, "//"),
            ArithBinOp::Mod => write!(f, "%"),
            ArithBinOp::Pow => write!(f, "**"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BitwiseBinOp {
    BitAnd,
    BitOr,
    BitXor,
    LShift,
    RShift,
}

impl std::fmt::Display for BitwiseBinOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BitwiseBinOp::BitAnd => write!(f, "&"),
            BitwiseBinOp::BitOr => write!(f, "|"),
            BitwiseBinOp::BitXor => write!(f, "^"),
            BitwiseBinOp::LShift => write!(f, "<<"),
            BitwiseBinOp::RShift => write!(f, ">>"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypedBinOp {
    Arith(ArithBinOp),
    Bitwise(BitwiseBinOp),
}

impl std::fmt::Display for TypedBinOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypedBinOp::Arith(op) => write!(f, "{}", op),
            TypedBinOp::Bitwise(op) => write!(f, "{}", op),
        }
    }
}

// ── Cast kinds ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CastKind {
    IntToFloat,
    FloatToInt,
    BoolToFloat,
    IntToBool,
    FloatToBool,
    BoolToInt,
}

// ── Call target (for void calls) ────────────────────────────────────

#[derive(Debug, Clone)]
pub enum CallTarget {
    Named(String),
    Builtin(builtin::BuiltinFn),
}

// ── Comparison / unary / logical ops ────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CmpOp {
    Eq,
    NotEq,
    Lt,
    LtEq,
    Gt,
    GtEq,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOpKind {
    Neg,
    Pos,
    Not,
    BitNot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogicalOp {
    And,
    Or,
}

// ── TIR nodes ───────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct FunctionParam {
    pub name: String,
    pub ty: ValueType,
}

impl FunctionParam {
    pub fn new(name: String, ty: ValueType) -> Self {
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
    pub return_type: Option<ValueType>,
    pub body: Vec<TirStmt>,
}

#[derive(Debug, Clone)]
pub enum TirStmt {
    Let {
        name: String,
        ty: ValueType,
        value: TirExpr,
    },
    Return(Option<TirExpr>),
    Expr(TirExpr),
    VoidCall {
        target: CallTarget,
        args: Vec<TirExpr>,
    },
    If {
        condition: TirExpr,
        then_body: Vec<TirStmt>,
        else_body: Vec<TirStmt>,
    },
    While {
        condition: TirExpr,
        body: Vec<TirStmt>,
    },
    Break,
    Continue,
}

#[derive(Debug, Clone)]
pub struct TirExpr {
    pub kind: TirExprKind,
    pub ty: ValueType,
}

#[derive(Debug, Clone)]
pub enum TirExprKind {
    IntLiteral(i64),
    FloatLiteral(f64),
    Var(String),
    BinOp {
        op: TypedBinOp,
        left: Box<TirExpr>,
        right: Box<TirExpr>,
    },
    Call {
        func: String,
        args: Vec<TirExpr>,
    },
    ExternalCall {
        func: builtin::BuiltinFn,
        args: Vec<TirExpr>,
    },
    Cast {
        kind: CastKind,
        arg: Box<TirExpr>,
    },
    Compare {
        op: CmpOp,
        left: Box<TirExpr>,
        right: Box<TirExpr>,
    },
    UnaryOp {
        op: UnaryOpKind,
        operand: Box<TirExpr>,
    },
    LogicalOp {
        op: LogicalOp,
        left: Box<TirExpr>,
        right: Box<TirExpr>,
    },
}

/// Result of lowering a call expression — either a valued expression or a void statement.
pub enum CallResult {
    Expr(TirExpr),
    VoidStmt(TirStmt),
}
