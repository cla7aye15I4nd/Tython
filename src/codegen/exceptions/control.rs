use inkwell::AddressSpace;

use crate::tir::{TirExpr, TirStmt};

use super::super::runtime_fn::RuntimeFn;
use super::super::Codegen;

impl<'ctx> Codegen<'ctx> {
    /// Recursively check whether any statement contains TryCatch or ForIter,
    /// which means the enclosing function needs a personality function.
    pub(crate) fn stmts_need_personality(stmts: &[TirStmt]) -> bool {
        stmts.iter().any(|stmt| match stmt {
            TirStmt::TryCatch { .. } | TirStmt::ForIter { .. } => true,
            TirStmt::If {
                then_body,
                else_body,
                ..
            } => Self::stmts_need_personality(then_body) || Self::stmts_need_personality(else_body),
            TirStmt::While { body, .. }
            | TirStmt::ForRange { body, .. }
            | TirStmt::ForList { body, .. } => Self::stmts_need_personality(body),
            _ => false,
        })
    }

    pub(crate) fn codegen_raise(&mut self, exc_type_tag: Option<i64>, message: Option<&TirExpr>) {
        let null_msg = || {
            self.context
                .ptr_type(AddressSpace::default())
                .const_null()
                .into()
        };

        match (exc_type_tag, self.reraise_state) {
            (Some(tag), _) => {
                let tag_val = self.i64_type().const_int(tag as u64, false).into();
                let msg_val = message
                    .map(|msg_expr| self.codegen_expr(msg_expr))
                    .unwrap_or_else(null_msg);
                self.emit_raise(tag_val, msg_val);
            }
            (None, Some((tag_alloca, msg_alloca))) => {
                // Bare raise inside except handler: re-raise saved exception
                let tag_val = emit!(self.build_load(self.i64_type(), tag_alloca, "reraise_tag"));
                let msg_val = emit!(self.build_load(
                    self.context.ptr_type(AddressSpace::default()),
                    msg_alloca,
                    "reraise_msg",
                ));
                self.emit_raise(tag_val, msg_val);
            }
            (None, None) => {
                // Bare raise outside except handler: use __cxa_rethrow as fallback
                let rethrow_fn = self.get_runtime_fn(RuntimeFn::CxaRethrow);
                emit!(self.build_call(rethrow_fn, &[], "rethrow"));
                emit!(self.build_unreachable());
            }
        }
        self.append_dead_block("raise.after");
    }
}
