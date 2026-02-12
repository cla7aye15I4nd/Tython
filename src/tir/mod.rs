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
    Tuple(Vec<ValueType>),
    Class(String),
    Function {
        params: Vec<ValueType>,
        return_type: Option<Box<ValueType>>,
    },
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
            Type::Tuple(elements) => Some(ValueType::Tuple(
                elements
                    .iter()
                    .map(ValueType::from_type)
                    .collect::<Option<Vec<_>>>()?,
            )),
            Type::Class(name) => Some(ValueType::Class(name.clone())),
            Type::Function {
                params,
                return_type,
            } => {
                let vt_params: Vec<ValueType> = params
                    .iter()
                    .map(ValueType::from_type)
                    .collect::<Option<Vec<_>>>()?;
                let vt_ret = match return_type.as_ref() {
                    Type::Unit => None,
                    other => Some(Box::new(ValueType::from_type(other)?)),
                };
                Some(ValueType::Function {
                    params: vt_params,
                    return_type: vt_ret,
                })
            }
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
            ValueType::Tuple(elements) => {
                Type::Tuple(elements.iter().map(ValueType::to_type).collect())
            }
            ValueType::Class(name) => Type::Class(name.clone()),
            ValueType::Function {
                params,
                return_type,
            } => Type::Function {
                params: params.iter().map(ValueType::to_type).collect(),
                return_type: Box::new(
                    return_type
                        .as_ref()
                        .map(|vt| vt.to_type())
                        .unwrap_or(Type::Unit),
                ),
            },
        }
    }

    pub fn is_primitive(&self) -> bool {
        matches!(self, ValueType::Int | ValueType::Float | ValueType::Bool)
    }

    /// Returns `true` if the type supports ordering comparisons (`<`, `>`, `<=`, `>=`),
    /// i.e. the type conceptually has a `__lt__` method.
    pub fn supports_ordering(&self) -> bool {
        matches!(
            self,
            ValueType::Int
                | ValueType::Float
                | ValueType::Bool
                | ValueType::Str
                | ValueType::Bytes
                | ValueType::ByteArray
        )
    }

    pub fn unwrap_function(&self) -> (&Vec<ValueType>, &Option<Box<ValueType>>) {
        match self {
            ValueType::Function {
                params,
                return_type,
            } => (params, return_type),
            _ => panic!("ICE: expected Function type, got {self}"),
        }
    }

    pub fn is_ref_type(&self) -> bool {
        matches!(
            self,
            ValueType::Str
                | ValueType::Bytes
                | ValueType::ByteArray
                | ValueType::List(_)
                | ValueType::Tuple(_)
                | ValueType::Class(_)
                | ValueType::Function { .. }
        )
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
            ValueType::Tuple(elements) => {
                write!(f, "tuple[")?;
                for (i, elt) in elements.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", elt)?;
                }
                write!(f, "]")
            }
            ValueType::Class(name) => write!(f, "{}", name),
            ValueType::Function {
                params,
                return_type,
            } => {
                write!(f, "callable[[")?;
                for (i, p) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", p)?;
                }
                write!(f, "], ")?;
                match return_type {
                    Some(rt) => write!(f, "{}]", rt),
                    None => write!(f, "None]"),
                }
            }
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

/// Fully-typed binary operation stored in TIR nodes.
/// Codegen can match on this directly without checking `expr.ty`.
/// Sequence operations (concat, repeat) are lowered to `ExternalCall` instead.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypedBinOp {
    IntArith(IntArithOp),
    FloatArith(FloatArithOp),
    Bitwise(BitwiseBinOp),
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
    Indirect(TirExpr),
}

// ── Comparison / unary / logical ops ────────────────────────────────

/// Raw comparison operator — used during parsing / lowering.
/// `In`, `NotIn`, `Is`, `IsNot` are desugared by the lowerer and never
/// appear in the final TIR `Compare` node.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CmpOp {
    Eq,
    NotEq,
    Lt,
    LtEq,
    Gt,
    GtEq,
    In,
    NotIn,
    Is,
    IsNot,
}

/// Ordered comparison operator — the only variants that survive into TIR
/// `Compare` nodes and map directly to LLVM int/float predicates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderedCmpOp {
    Eq,
    NotEq,
    Lt,
    LtEq,
    Gt,
    GtEq,
}

impl OrderedCmpOp {
    /// Convert a raw `CmpOp` that is known to be an ordered comparison.
    /// Panics on `In`/`NotIn`/`Is`/`IsNot` — those must be desugared first.
    pub fn from_cmp_op(op: CmpOp) -> Self {
        match op {
            CmpOp::Eq => OrderedCmpOp::Eq,
            CmpOp::NotEq => OrderedCmpOp::NotEq,
            CmpOp::Lt => OrderedCmpOp::Lt,
            CmpOp::LtEq => OrderedCmpOp::LtEq,
            CmpOp::Gt => OrderedCmpOp::Gt,
            CmpOp::GtEq => OrderedCmpOp::GtEq,
            other => panic!(
                "ICE: cannot convert {:?} to OrderedCmpOp — must be desugared first",
                other
            ),
        }
    }
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

// ── Exception handling ──────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ExceptClause {
    pub exc_type_tag: Option<i64>, // None = bare except (catch-all)
    pub var_name: Option<String>,  // `as e` variable (bound as str)
    pub body: Vec<TirStmt>,
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
        else_body: Vec<TirStmt>,
    },
    ForRange {
        loop_var: String,
        start_var: String,
        stop_var: String,
        step_var: String,
        body: Vec<TirStmt>,
        else_body: Vec<TirStmt>,
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
    TryCatch {
        try_body: Vec<TirStmt>,
        except_clauses: Vec<ExceptClause>,
        else_body: Vec<TirStmt>,
        finally_body: Vec<TirStmt>,
        has_finally: bool,
    },
    Raise {
        exc_type_tag: Option<i64>,
        message: Option<TirExpr>,
    },
    ForList {
        loop_var: String,
        loop_var_ty: ValueType,
        list_var: String,
        index_var: String,
        len_var: String,
        body: Vec<TirStmt>,
        else_body: Vec<TirStmt>,
    },
    ForIter {
        loop_var: String,
        loop_var_ty: ValueType,
        iterator_var: String,
        iterator_class: String,
        next_mangled: String,
        body: Vec<TirStmt>,
        else_body: Vec<TirStmt>,
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
        op: OrderedCmpOp,
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
    TupleLiteral {
        elements: Vec<TirExpr>,
        element_types: Vec<ValueType>,
    },
    TupleGet {
        tuple: Box<TirExpr>,
        index: usize,
        element_types: Vec<ValueType>,
    },
    TupleGetDynamic {
        tuple: Box<TirExpr>,
        index: Box<TirExpr>,
        len: usize,
        element_types: Vec<ValueType>,
    },
    ListLiteral {
        element_type: ValueType,
        elements: Vec<TirExpr>,
    },
    FuncRef {
        mangled_name: String,
    },
    IndirectCall {
        callee: Box<TirExpr>,
        args: Vec<TirExpr>,
    },
}

/// Result of lowering a call expression — either a valued expression or a void statement.
pub enum CallResult {
    Expr(TirExpr),
    VoidStmt(Box<TirStmt>),
}
