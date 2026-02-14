use anyhow::Result;
use pyo3::prelude::*;

use crate::tir::{builtin, type_rules, CallResult, TirExpr, TirExprKind, TirStmt, ValueType};
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
            let part_expr = match ast_type_name!(part).as_str() {
                "Constant" => {
                    let value = ast_getattr!(part, "value");
                    let s = value.extract::<String>().map_err(|_| {
                        self.syntax_error(line, "f-string constants must be string literals")
                    })?;
                    TirExpr {
                        kind: TirExprKind::StrLiteral(s),
                        ty: ValueType::Str,
                    }
                }
                "FormattedValue" => self.lower_formatted_value(&part, line)?,
                other => {
                    return Err(self
                        .syntax_error(line, format!("unsupported f-string segment `{}`", other)))
                }
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
            let spec_expr = match ast_type_name!(format_spec).as_str() {
                "JoinedStr" => self.lower_joined_str(&format_spec, line)?,
                "Constant" => self.lower_expr(&format_spec)?,
                other => {
                    return Err(self.syntax_error(
                        line,
                        format!("unsupported f-string format spec `{}`", other),
                    ))
                }
            };
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

        match conversion {
            -1 | 115 => self.lower_fstring_convert(line, "str", value_expr),
            114 | 97 => self.lower_fstring_convert(line, "repr", value_expr),
            other => Err(self.syntax_error(
                line,
                format!("unsupported f-string conversion code `{}`", other),
            )),
        }
    }

    fn lower_fstring_convert(&mut self, line: usize, name: &str, arg: TirExpr) -> Result<TirExpr> {
        let arg_types: Vec<&ValueType> = vec![&arg.ty];
        let rule = type_rules::lookup_builtin_call(name, &arg_types).ok_or_else(|| {
            self.type_error(
                line,
                format!(
                    "f-string conversion `{}` is not defined for type `{}`",
                    name, arg.ty
                ),
            )
        })?;

        if let type_rules::BuiltinCallRule::ClassMagic {
            method_names,
            return_type,
        } = rule
        {
            return self.lower_class_magic_method(line, arg, method_names, return_type, name);
        }

        if matches!(rule, type_rules::BuiltinCallRule::StrAuto) {
            return Ok(self.lower_str_auto(arg));
        }
        if matches!(rule, type_rules::BuiltinCallRule::ReprAuto) {
            return Ok(self.lower_repr_str_expr(arg));
        }

        match Self::lower_builtin_rule(rule, vec![arg]) {
            CallResult::Expr(expr) => Ok(expr),
            CallResult::VoidStmt(_) => Err(self.type_error(
                line,
                format!("f-string conversion `{}` produced no value", name),
            )),
        }
    }
}
