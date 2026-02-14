use anyhow::Result;

use crate::tir::{
    builtin::BuiltinFn, type_rules, CallResult, CallTarget, TirExpr, TirExprKind, TirStmt,
    ValueType,
};

use super::Lowering;

pub mod dict;
pub mod list;
pub mod set;

// ── Helper Functions ─────────────────────────────────────────────────

/// Check that the method call has the expected number of arguments.
#[inline]
pub fn check_arity(
    ctx: &Lowering,
    line: usize,
    type_name: &str,
    method_name: &str,
    expected: usize,
    actual: usize,
) -> Result<()> {
    if actual != expected {
        return Err(ctx.type_error(
            line,
            format!(
                "{}.{}() takes {} argument{}, got {}",
                type_name,
                method_name,
                expected,
                if expected == 1 { "" } else { "s" },
                actual
            ),
        ));
    }
    Ok(())
}

/// Check that an argument has the expected type.
#[inline]
pub fn check_type(
    ctx: &Lowering,
    line: usize,
    type_name: &str,
    method_name: &str,
    arg: &TirExpr,
    expected: &ValueType,
) -> Result<()> {
    if &arg.ty != expected {
        return Err(ctx.type_error(
            line,
            format!(
                "{}.{}() expected argument of type {}, got {}",
                type_name, method_name, expected, arg.ty
            ),
        ));
    }
    Ok(())
}

/// Build a void call statement (method that returns nothing).
#[inline]
pub fn void_call(func: BuiltinFn, obj: TirExpr, mut args: Vec<TirExpr>) -> CallResult {
    args.insert(0, obj);
    CallResult::VoidStmt(Box::new(TirStmt::VoidCall {
        target: CallTarget::Builtin(func),
        args,
    }))
}

/// Build an expression call (method that returns a value).
#[inline]
pub fn expr_call(
    func: BuiltinFn,
    return_type: ValueType,
    obj: TirExpr,
    mut args: Vec<TirExpr>,
) -> CallResult {
    args.insert(0, obj);
    CallResult::Expr(TirExpr {
        kind: TirExprKind::ExternalCall { func, args },
        ty: return_type,
    })
}

// ── Dispatcher ───────────────────────────────────────────────────────

impl Lowering {
    pub(in crate::tir::lower) fn emit_method_rule_call(
        &self,
        line: usize,
        obj_expr: TirExpr,
        tir_args: Vec<TirExpr>,
        method_name: &str,
        type_name: &str,
        lookup: Option<Result<type_rules::MethodCallRule, String>>,
    ) -> Result<CallResult> {
        let rule = match lookup {
            Some(Ok(rule)) => rule,
            Some(Err(msg)) => return Err(self.type_error(line, msg)),
            None => {
                return Err(self.attribute_error(
                    line,
                    format!("{} has no method `{}`", type_name, method_name),
                ))
            }
        };

        if tir_args.len() != rule.params.len() {
            return Err(self.type_error(
                line,
                type_rules::method_call_arity_error(
                    type_name,
                    method_name,
                    rule.params.len(),
                    tir_args.len(),
                ),
            ));
        }
        for (arg, expected) in tir_args.iter().zip(rule.params.iter()) {
            if arg.ty != *expected {
                return Err(self.type_error(
                    line,
                    type_rules::method_call_type_error(type_name, method_name, expected, &arg.ty),
                ));
            }
        }

        let mut full_args = Vec::with_capacity(1 + tir_args.len());
        full_args.push(obj_expr);
        full_args.extend(tir_args);

        Ok(match rule.result {
            type_rules::MethodCallResult::Void(func) => {
                CallResult::VoidStmt(Box::new(TirStmt::VoidCall {
                    target: CallTarget::Builtin(func),
                    args: full_args,
                }))
            }
            type_rules::MethodCallResult::Expr { func, return_type } => CallResult::Expr(TirExpr {
                kind: TirExprKind::ExternalCall {
                    func,
                    args: full_args,
                },
                ty: return_type,
            }),
        })
    }

    /// Dispatch method calls for builtin types.
    pub(in crate::tir::lower) fn lower_method_call(
        &mut self,
        line: usize,
        obj_expr: TirExpr,
        method_name: &str,
        args: Vec<TirExpr>,
    ) -> Result<CallResult> {
        // Clone what we need from obj_expr before matching to avoid borrow issues
        let obj_ty = obj_expr.ty.clone();

        match obj_ty {
            ValueType::List(inner) => {
                list::lower_list_method_call(self, line, obj_expr, method_name, args, &inner)
            }
            ValueType::Dict(key, value) => {
                dict::lower_dict_method_call(self, line, obj_expr, method_name, args, &key, &value)
            }
            ValueType::Set(inner) => {
                set::lower_set_method_call(self, line, obj_expr, method_name, args, &inner)
            }
            ref ty => {
                let type_name = type_rules::builtin_type_display_name(ty);
                let lookup = type_rules::lookup_builtin_method(ty, method_name);
                self.emit_method_rule_call(line, obj_expr, args, method_name, &type_name, lookup)
            }
        }
    }
}
