use anyhow::Result;
use pyo3::prelude::*;

use crate::ast_get_list;
use crate::tir::{builtin, CallTarget, TirExpr, TirExprKind, TirStmt, ValueType};

use crate::tir::lower::Lowering;

impl Lowering {
    pub(in crate::tir::lower) fn lower_print_stmt(
        &mut self,
        call_node: &Bound<PyAny>,
    ) -> Result<Vec<TirStmt>> {
        let line = Self::get_line(call_node);
        let args_list = ast_get_list!(call_node, "args");

        let mut tir_args = Vec::new();
        for arg in args_list.iter() {
            tir_args.push(self.lower_expr(&arg)?);
        }

        let mut stmts = Vec::new();
        for (i, arg) in tir_args.into_iter().enumerate() {
            if i > 0 {
                stmts.push(TirStmt::VoidCall {
                    target: CallTarget::Builtin(builtin::BuiltinFn::PrintSpace),
                    args: vec![],
                });
            }
            self.lower_print_value_stmts(line, arg, &mut stmts)?;
        }
        stmts.push(TirStmt::VoidCall {
            target: CallTarget::Builtin(builtin::BuiltinFn::PrintNewline),
            args: vec![],
        });

        Ok(stmts)
    }

    fn lower_print_value_stmts(
        &mut self,
        line: usize,
        arg: TirExpr,
        stmts: &mut Vec<TirStmt>,
    ) -> Result<()> {
        let str_expr = match &arg.ty {
            ValueType::Str => arg,
            ValueType::Int => TirExpr {
                kind: TirExprKind::ExternalCall {
                    func: builtin::BuiltinFn::StrFromInt,
                    args: vec![arg],
                },
                ty: ValueType::Str,
            },
            ValueType::Float => TirExpr {
                kind: TirExprKind::ExternalCall {
                    func: builtin::BuiltinFn::StrFromFloat,
                    args: vec![arg],
                },
                ty: ValueType::Str,
            },
            ValueType::Bool => TirExpr {
                kind: TirExprKind::ExternalCall {
                    func: builtin::BuiltinFn::StrFromBool,
                    args: vec![arg],
                },
                ty: ValueType::Str,
            },
            ValueType::Class(_) => self.lower_class_magic_method(
                line,
                arg,
                &["__str__", "__repr__"],
                Some(ValueType::Str),
                "str",
            )?,
            ValueType::Function { .. } => {
                return Err(
                    self.type_error(line, format!("cannot print value of type `{}`", arg.ty))
                );
            }
            _ => {
                let call_result = self.lower_method_call(line, arg, "__str__", vec![])?;
                self.require_value_call_result(line, call_result, "`__str__` must return a value")?
            }
        };
        stmts.push(TirStmt::VoidCall {
            target: CallTarget::Builtin(builtin::BuiltinFn::PrintStr),
            args: vec![str_expr],
        });
        Ok(())
    }
}
