use anyhow::Result;
use pyo3::prelude::*;

use crate::tir::{builtin, TirExpr, TirExprKind, TirStmt, ValueType};
use crate::{ast_get_int, ast_get_list, ast_getattr, ast_type_name};

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
                let value = ast_getattr!(part, "value");
                let s = value.extract::<String>()?;
                TirExpr {
                    kind: TirExprKind::StrLiteral(s),
                    ty: ValueType::Str,
                }
            } else {
                debug_assert_eq!(
                    part_kind, "FormattedValue",
                    "unexpected f-string segment kind"
                );
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

        // Parse and evaluate format spec for compatibility, but ignore formatting details for now.
        let format_spec = ast_getattr!(node, "format_spec");
        if !format_spec.is_none() {
            let spec_expr = self.lower_expr(&format_spec)?;
            if spec_expr.ty != ValueType::Str {
                return Err(self.type_error(
                    line,
                    format!("f-string format spec must be `str`, got `{}`", spec_expr.ty),
                ));
            }
            let tmp = self.fresh_internal("fstr_spec");
            self.pre_stmts.push(TirStmt::Let {
                name: tmp,
                ty: ValueType::Str,
                value: spec_expr,
            });
        }

        debug_assert!(
            matches!(conversion, -1 | 115 | 114 | 97),
            "unexpected f-string conversion code"
        );
        let conv = if matches!(conversion, -1 | 115) {
            "str"
        } else {
            "repr"
        };
        self.lower_fstring_convert(line, conv, value_expr)
    }

    fn lower_fstring_convert(&mut self, line: usize, name: &str, arg: TirExpr) -> Result<TirExpr> {
        self.lower_builtin_single_arg_expr(line, name, arg)
    }
}
