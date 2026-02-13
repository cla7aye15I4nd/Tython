use inkwell::AddressSpace;

use crate::tir::{TirExpr, TirStmt};

use super::super::runtime_fn::RuntimeFn;
use super::super::Codegen;

impl<'ctx> Codegen<'ctx> {
    /// Recursively check whether any statement contains TryCatch or ForIter,
    /// which means the enclosing function needs a personality function.
    pub(crate) fn stmts_need_personality(stmts: &[TirStmt]) -> bool {
        for stmt in stmts {
            match stmt {
                TirStmt::TryCatch { .. } | TirStmt::ForIter { .. } => return true,
                TirStmt::If {
                    then_body,
                    else_body,
                    ..
                } => {
                    if Self::stmts_need_personality(then_body)
                        || Self::stmts_need_personality(else_body)
                    {
                        return true;
                    }
                }
                TirStmt::While { body, .. }
                | TirStmt::ForRange { body, .. }
                | TirStmt::ForList { body, .. } => {
                    if Self::stmts_need_personality(body) {
                        return true;
                    }
                }
                TirStmt::Let { .. }
                | TirStmt::Return(_)
                | TirStmt::Expr(_)
                | TirStmt::VoidCall { .. }
                | TirStmt::Break
                | TirStmt::Continue
                | TirStmt::SetField { .. }
                | TirStmt::ListSet { .. }
                | TirStmt::Raise { .. } => {}
            }
        }
        false
    }

    pub(crate) fn codegen_raise(&mut self, exc_type_tag: Option<i64>, message: Option<&TirExpr>) {
        if let Some(tag) = exc_type_tag {
            let tag_val = self.i64_type().const_int(tag as u64, false);
            let msg_val = if let Some(msg_expr) = message.as_ref() {
                self.codegen_expr(msg_expr)
            } else {
                self.context
                    .ptr_type(AddressSpace::default())
                    .const_null()
                    .into()
            };
            self.emit_raise(tag_val.into(), msg_val);
        } else if let Some((tag_alloca, msg_alloca)) = self.reraise_state {
            // Bare raise inside except handler: re-raise saved exception
            let tag_val = emit!(self.build_load(self.i64_type(), tag_alloca, "reraise_tag"));
            let msg_val = emit!(self.build_load(
                self.context.ptr_type(AddressSpace::default()),
                msg_alloca,
                "reraise_msg",
            ));
            self.emit_raise(tag_val, msg_val);
        } else {
            // Bare raise outside except handler: use __cxa_rethrow as fallback
            let rethrow_fn = self.get_runtime_fn(RuntimeFn::CxaRethrow);
            emit!(self.build_call(rethrow_fn, &[], "rethrow"));
            emit!(self.build_unreachable());
        }
        self.append_dead_block("raise.after");
    }
}
