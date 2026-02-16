use crate::ast::{ClassInfo, Type};
use crate::tir::TirExpr;

mod bind;
pub mod builtin_call;
mod emit;
mod resolve;

#[derive(Clone)]
pub(super) struct NormalizedCallArgs {
    pub positional: Vec<TirExpr>,
    pub keyword: Vec<(String, TirExpr)>,
}

#[derive(Clone)]
pub(super) enum ResolvedCallee {
    GlobalName(String),
    DirectFunction {
        display_name: String,
        mangled: String,
        func_type: Type,
    },
    Constructor {
        qualified_name: String,
        class_info: ClassInfo,
    },
    ClassMethod {
        object: TirExpr,
        class_name: String,
        method_name: String,
    },
    BuiltinMethod {
        object: TirExpr,
        method_name: String,
    },
}

#[derive(Clone)]
pub(super) struct ResolvedCall {
    pub callee: ResolvedCallee,
    pub args: NormalizedCallArgs,
}
