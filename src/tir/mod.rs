pub mod builtin;
pub mod lower;
#[path = "type_rules/mod.rs"]
pub mod type_rules;

use crate::ast::{ClassInfo, Type};
use std::collections::HashMap;

// ── Value types (types with LLVM register representations) ──────────

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ValueType {
    Int,
    Float,
    Bool,
    Str,
    Bytes,
    ByteArray,
    List(Box<ValueType>),
    Class(String),
}

impl ValueType {
    pub fn from_type(ty: &Type) -> Option<Self> {
        match ty {
            Type::Int => Some(ValueType::Int),
            Type::Float => Some(ValueType::Float),
            Type::Bool => Some(ValueType::Bool),
            Type::Str => Some(ValueType::Str),
            Type::Bytes => Some(ValueType::Bytes),
            Type::ByteArray => Some(ValueType::ByteArray),
            Type::List(inner) => Some(ValueType::List(Box::new(ValueType::from_type(inner)?))),
            Type::Class(name) => Some(ValueType::Class(name.clone())),
            _ => None,
        }
    }

    pub fn to_type(&self) -> Type {
        match self {
            ValueType::Int => Type::Int,
            ValueType::Float => Type::Float,
            ValueType::Bool => Type::Bool,
            ValueType::Str => Type::Str,
            ValueType::Bytes => Type::Bytes,
            ValueType::ByteArray => Type::ByteArray,
            ValueType::List(inner) => Type::List(Box::new(inner.to_type())),
            ValueType::Class(name) => Type::Class(name.clone()),
        }
    }

    pub fn is_primitive(&self) -> bool {
        matches!(self, ValueType::Int | ValueType::Float | ValueType::Bool)
    }
}

impl std::fmt::Display for ValueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValueType::Int => write!(f, "int"),
            ValueType::Float => write!(f, "float"),
            ValueType::Bool => write!(f, "bool"),
            ValueType::Str => write!(f, "str"),
            ValueType::Bytes => write!(f, "bytes"),
            ValueType::ByteArray => write!(f, "bytearray"),
            ValueType::List(inner) => write!(f, "list[{}]", inner),
            ValueType::Class(name) => write!(f, "{}", name),
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

/// Raw (untyped) binary operation — used during parsing and type-rule lookup.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RawBinOp {
    Arith(ArithBinOp),
    Bitwise(BitwiseBinOp),
}

impl std::fmt::Display for RawBinOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RawBinOp::Arith(op) => write!(f, "{}", op),
            RawBinOp::Bitwise(op) => write!(f, "{}", op),
        }
    }
}

/// Integer arithmetic operations. `Div` is absent — Python `/` always returns float.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntArithOp {
    Add,
    Sub,
    Mul,
    FloorDiv,
    Mod,
    Pow,
}

impl std::fmt::Display for IntArithOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IntArithOp::Add => write!(f, "+"),
            IntArithOp::Sub => write!(f, "-"),
            IntArithOp::Mul => write!(f, "*"),
            IntArithOp::FloorDiv => write!(f, "//"),
            IntArithOp::Mod => write!(f, "%"),
            IntArithOp::Pow => write!(f, "**"),
        }
    }
}

/// Float arithmetic operations. Includes `Div` (true division).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FloatArithOp {
    Add,
    Sub,
    Mul,
    Div,
    FloorDiv,
    Mod,
    Pow,
}

impl std::fmt::Display for FloatArithOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FloatArithOp::Add => write!(f, "+"),
            FloatArithOp::Sub => write!(f, "-"),
            FloatArithOp::Mul => write!(f, "*"),
            FloatArithOp::Div => write!(f, "/"),
            FloatArithOp::FloorDiv => write!(f, "//"),
            FloatArithOp::Mod => write!(f, "%"),
            FloatArithOp::Pow => write!(f, "**"),
        }
    }
}

/// Fully-typed binary operation stored in TIR nodes.
/// Codegen can match on this directly without checking `expr.ty`.
/// Sequence operations (concat, repeat) are lowered to `ExternalCall` instead.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypedBinOp {
    IntArith(IntArithOp),
    FloatArith(FloatArithOp),
    Bitwise(BitwiseBinOp),
}

impl std::fmt::Display for TypedBinOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypedBinOp::IntArith(op) => write!(f, "{}", op),
            TypedBinOp::FloatArith(op) => write!(f, "{}", op),
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
    MethodCall {
        mangled_name: String,
        object: TirExpr,
    },
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
    pub classes: HashMap<String, ClassInfo>,
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
    SetField {
        object: TirExpr,
        class_name: String,
        field_index: usize,
        value: TirExpr,
    },
    ListSet {
        list: TirExpr,
        index: TirExpr,
        value: TirExpr,
    },
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
    StrLiteral(String),
    BytesLiteral(Vec<u8>),
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
    GetField {
        object: Box<TirExpr>,
        class_name: String,
        field_index: usize,
    },
    Construct {
        class_name: String,
        init_mangled_name: String,
        args: Vec<TirExpr>,
    },
    MethodCall {
        object: Box<TirExpr>,
        method_mangled_name: String,
        args: Vec<TirExpr>,
    },
    ListLiteral {
        element_type: ValueType,
        elements: Vec<TirExpr>,
    },
}

/// Result of lowering a call expression — either a valued expression or a void statement.
pub enum CallResult {
    Expr(TirExpr),
    VoidStmt(TirStmt),
}
