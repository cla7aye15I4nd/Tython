use crate::tir::TirStmt;

use super::super::Codegen;

impl<'ctx> Codegen<'ctx> {
    pub(crate) fn codegen_stmt(&mut self, stmt: &TirStmt) {
        match stmt {
            TirStmt::Let { name, ty, value } => self.codegen_let_stmt(name, ty, value),
            TirStmt::Return(expr_opt) => self.codegen_return_stmt(expr_opt),
            TirStmt::Expr(expr) => {
                self.codegen_expr(expr);
            }
            TirStmt::VoidCall { target, args } => self.codegen_void_call_stmt(target, args),
            TirStmt::SetField {
                object,
                class_name,
                field_index,
                value,
            } => self.codegen_set_field_stmt(object, class_name, *field_index, value),
            TirStmt::ListSet { list, index, value } => {
                self.codegen_list_set_stmt(list, index, value)
            }
            TirStmt::If {
                condition,
                then_body,
                else_body,
            } => self.codegen_if_stmt(condition, then_body, else_body),
            TirStmt::While {
                condition,
                body,
                else_body,
            } => self.codegen_while_stmt(condition, body, else_body),
            TirStmt::ForRange {
                loop_var,
                start_var,
                stop_var,
                step_var,
                body,
                else_body,
            } => self
                .codegen_for_range_stmt(loop_var, start_var, stop_var, step_var, body, else_body),
            TirStmt::Raise {
                exc_type_tag,
                message,
            } => self.codegen_raise(*exc_type_tag, message.as_ref()),
            TirStmt::TryCatch {
                try_body,
                except_clauses,
                else_body,
                finally_body,
                has_finally,
            } => self.codegen_try_catch(
                try_body,
                except_clauses,
                else_body,
                finally_body,
                *has_finally,
            ),
            TirStmt::ForList {
                loop_var,
                loop_var_ty,
                list_var,
                index_var,
                len_var,
                body,
                else_body,
            } => self.codegen_for_list_stmt(
                loop_var,
                loop_var_ty,
                list_var,
                index_var,
                len_var,
                body,
                else_body,
            ),
            TirStmt::ForIter {
                loop_var,
                loop_var_ty,
                iterator_var,
                iterator_class,
                next_mangled,
                body,
                else_body,
            } => self.codegen_for_iter(
                loop_var,
                loop_var_ty,
                iterator_var,
                iterator_class,
                next_mangled,
                body,
                else_body,
            ),
            TirStmt::Break => self.codegen_break_stmt(),
            TirStmt::Continue => self.codegen_continue_stmt(),
        }
    }
}
