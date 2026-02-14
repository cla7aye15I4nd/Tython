use crate::tir::{TirExpr, TirStmt, ValueType};

use super::super::Codegen;

impl<'ctx> Codegen<'ctx> {
    pub(crate) fn codegen_if_stmt(
        &mut self,
        condition: &TirExpr,
        then_body: &[TirStmt],
        else_body: &[TirStmt],
    ) {
        let cond_val = self.codegen_expr(condition);
        let cond_bool = cond_val.into_int_value();

        let function = emit!(self.get_insert_block()).get_parent().unwrap();

        let then_bb = self.context.append_basic_block(function, "then");
        let else_bb = self.context.append_basic_block(function, "else");
        let merge_bb = self.context.append_basic_block(function, "ifcont");

        emit!(self.build_conditional_branch(cond_bool, then_bb, else_bb));

        self.builder.position_at_end(then_bb);
        for s in then_body {
            self.codegen_stmt(s);
        }
        let then_terminated = self.branch_if_unterminated(merge_bb);

        self.builder.position_at_end(else_bb);
        for s in else_body {
            self.codegen_stmt(s);
        }
        let else_terminated = self.branch_if_unterminated(merge_bb);

        self.builder.position_at_end(merge_bb);
        if then_terminated && else_terminated {
            emit!(self.build_unreachable());
        }
    }

    pub(crate) fn codegen_while_stmt(
        &mut self,
        condition: &TirExpr,
        body: &[TirStmt],
        else_body: &[TirStmt],
    ) {
        let function = emit!(self.get_insert_block()).get_parent().unwrap();

        let header_bb = self.context.append_basic_block(function, "while.header");
        let body_bb = self.context.append_basic_block(function, "while.body");
        let else_bb = if !else_body.is_empty() {
            Some(self.context.append_basic_block(function, "while.else"))
        } else {
            None
        };
        let after_bb = self.context.append_basic_block(function, "while.after");

        emit!(self.build_unconditional_branch(header_bb));

        self.builder.position_at_end(header_bb);
        let cond_val = self.codegen_expr(condition);
        let cond_bool = cond_val.into_int_value();
        let false_dest = else_bb.unwrap_or(after_bb);
        emit!(self.build_conditional_branch(cond_bool, body_bb, false_dest));

        self.builder.position_at_end(body_bb);
        loop_body!(self, header_bb, after_bb, body);
        else_body!(self, else_bb, else_body, after_bb);
        self.builder.position_at_end(after_bb);
    }

    pub(crate) fn codegen_for_range_stmt(
        &mut self,
        loop_var: &str,
        start_var: &str,
        stop_var: &str,
        step_var: &str,
        body: &[TirStmt],
        else_body: &[TirStmt],
    ) {
        // Lower `for range` through `if + while` TIR:
        // if step > 0:
        //   while start < stop: ...
        // else:
        //   while start > stop: ...
        //
        // Increment happens before user body so `continue` still advances.
        let int_lit = |value: i64| TirExpr {
            kind: crate::tir::TirExprKind::IntLiteral(value),
            ty: ValueType::Int,
        };
        let int_var = |name: &str| TirExpr {
            kind: crate::tir::TirExprKind::Var(name.to_string()),
            ty: ValueType::Int,
        };
        let bool_expr = |kind| TirExpr {
            kind,
            ty: ValueType::Bool,
        };

        let step_pos = bool_expr(crate::tir::TirExprKind::IntGt(
            Box::new(int_var(step_var)),
            Box::new(int_lit(0)),
        ));
        let cond_pos = bool_expr(crate::tir::TirExprKind::IntLt(
            Box::new(int_var(start_var)),
            Box::new(int_var(stop_var)),
        ));
        let cond_neg = bool_expr(crate::tir::TirExprKind::IntGt(
            Box::new(int_var(start_var)),
            Box::new(int_var(stop_var)),
        ));
        let build_while_body = || {
            let mut while_body = Vec::with_capacity(body.len() + 2);
            while_body.push(TirStmt::Let {
                name: loop_var.to_string(),
                ty: ValueType::Int,
                value: int_var(start_var),
            });
            while_body.push(TirStmt::Let {
                name: start_var.to_string(),
                ty: ValueType::Int,
                value: TirExpr {
                    kind: crate::tir::TirExprKind::IntAdd(
                        Box::new(int_var(start_var)),
                        Box::new(int_var(step_var)),
                    ),
                    ty: ValueType::Int,
                },
            });
            while_body.extend_from_slice(body);
            while_body
        };

        let then_body = vec![TirStmt::While {
            condition: cond_pos,
            body: build_while_body(),
            else_body: else_body.to_vec(),
        }];
        let else_body = vec![TirStmt::While {
            condition: cond_neg,
            body: build_while_body(),
            else_body: else_body.to_vec(),
        }];

        self.codegen_if_stmt(&step_pos, &then_body, &else_body);
    }
}
