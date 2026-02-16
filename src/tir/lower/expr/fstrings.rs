use anyhow::Result;
use pyo3::prelude::*;

use crate::tir::{builtin, TirExpr, TirExprKind, TirStmt, ValueType};
use crate::{ast_get_int, ast_get_list, ast_get_string, ast_getattr, ast_type_name};

use crate::tir::lower::Lowering;

impl Lowering {
    pub(in crate::tir::lower) fn lower_joined_str(
        &mut self,
        node: &Bound<PyAny>,
        line: usize,
    ) -> Result<TirExpr> {
        let values = ast_get_list!(node, "values");
        let mut result = TirExpr {
            kind: TirExprKind::StrLiteral(String::new()),
            ty: ValueType::Str,
        };

        for part in values.iter() {
            let part_kind = ast_type_name!(part);
            let part_expr = if part_kind == "Constant" {
                let s = ast_get_string!(part, "value");
                TirExpr {
                    kind: TirExprKind::StrLiteral(s),
                    ty: ValueType::Str,
                }
            } else {
                self.lower_formatted_value(&part, line)?
            };

            result = TirExpr {
                kind: TirExprKind::ExternalCall {
                    func: builtin::BuiltinFn::StrConcat,
                    args: vec![result, part_expr],
                },
                ty: ValueType::Str,
            };
        }

        Ok(result)
    }

    pub(in crate::tir::lower) fn lower_formatted_value(
        &mut self,
        node: &Bound<PyAny>,
        line: usize,
    ) -> Result<TirExpr> {
        let value_expr = self.lower_expr(&ast_getattr!(node, "value"))?;
        let conversion = ast_get_int!(node, "conversion", i64);

        let format_spec = ast_getattr!(node, "format_spec");
        let format_spec_expr = if !format_spec.is_none() {
            let spec_expr = self.lower_expr(&format_spec)?;
            debug_assert!(
                spec_expr.ty == ValueType::Str,
                "f-string format spec should lower to str"
            );
            let tmp = self.fresh_internal("fstr_spec");
            self.pre_stmts.push(TirStmt::Let {
                name: tmp.clone(),
                ty: ValueType::Str,
                value: spec_expr,
            });
            Some(TirExpr {
                kind: TirExprKind::Var(tmp),
                ty: ValueType::Str,
            })
        } else {
            None
        };

        if conversion == -1 {
            if let Some(spec_expr) = format_spec_expr {
                return self.lower_fstring_apply_format_spec(line, value_expr, spec_expr);
            }
            return self.lower_fstring_convert(line, "str", value_expr);
        }

        let conv = if conversion == 115 { "str" } else { "repr" };
        // For explicit conversions (!s, !r, !a), conversion happens before formatting.
        // We currently preserve conversion behavior and ignore format details in that path.
        self.lower_fstring_convert(line, conv, value_expr)
    }

    fn lower_fstring_convert(&mut self, line: usize, name: &str, arg: TirExpr) -> Result<TirExpr> {
        self.lower_builtin_single_arg_expr(line, name, arg)
    }

    fn lower_fstring_apply_format_spec(
        &mut self,
        line: usize,
        value_expr: TirExpr,
        spec_expr: TirExpr,
    ) -> Result<TirExpr> {
        match value_expr.ty {
            ValueType::Int => Ok(TirExpr {
                kind: TirExprKind::ExternalCall {
                    func: builtin::BuiltinFn::StrFormatInt,
                    args: vec![value_expr, spec_expr],
                },
                ty: ValueType::Str,
            }),
            ValueType::Float => Ok(TirExpr {
                kind: TirExprKind::ExternalCall {
                    func: builtin::BuiltinFn::StrFormatFloat,
                    args: vec![value_expr, spec_expr],
                },
                ty: ValueType::Str,
            }),
            _ => {
                // Keep side effects from evaluating format spec, but fall back to str(value)
                // until richer `__format__` support is implemented.
                self.lower_builtin_single_arg_expr(line, "str", value_expr)
            }
        }
    }
}
