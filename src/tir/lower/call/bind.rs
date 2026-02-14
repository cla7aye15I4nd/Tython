use anyhow::Result;

use crate::ast::Type;
use crate::tir::{TirExpr, TirExprKind, ValueType};

use super::super::Lowering;

impl Lowering {
    pub(super) fn bind_user_function_args(
        &self,
        line: usize,
        func_display_name: &str,
        signature_key: &str,
        func_type: &Type,
        positional_args: Vec<TirExpr>,
        keyword_args: Vec<(String, TirExpr)>,
    ) -> Result<Vec<TirExpr>> {
        let (params, _) = match func_type {
            Type::Function {
                params,
                return_type,
            } => (params, return_type),
            _ => return Err(self.type_error(line, "cannot call non-function type")),
        };

        let param_count = params.len();
        if positional_args.len() > param_count {
            return Err(self.type_error(
                line,
                format!(
                    "function `{}` expects at most {} argument{}, got {}",
                    func_display_name,
                    param_count,
                    if param_count == 1 { "" } else { "s" },
                    positional_args.len()
                ),
            ));
        }

        let sig = self.function_signatures.get(signature_key);
        if sig.is_none() {
            if keyword_args.is_empty() && positional_args.len() == param_count {
                return Ok(positional_args);
            }
            return Err(self.syntax_error(
                line,
                format!(
                    "function `{}` is missing signature metadata for keyword/default argument binding",
                    func_display_name
                ),
            ));
        }
        let sig = sig.expect("checked is_some above");

        if sig.param_names.len() != param_count || sig.default_values.len() != param_count {
            return Err(self.syntax_error(
                line,
                format!(
                    "function `{}` has inconsistent signature metadata",
                    func_display_name
                ),
            ));
        }

        let mut bound: Vec<Option<TirExpr>> = vec![None; param_count];
        let positional_count = positional_args.len();
        for (i, arg) in positional_args.into_iter().enumerate() {
            bound[i] = Some(arg);
        }

        for (kw_name, kw_value) in keyword_args {
            let Some(idx) = sig.param_names.iter().position(|p| p == &kw_name) else {
                return Err(self.type_error(
                    line,
                    format!(
                        "function `{}` got an unexpected keyword argument `{}`",
                        func_display_name, kw_name
                    ),
                ));
            };
            if idx < positional_count || bound[idx].is_some() {
                return Err(self.type_error(
                    line,
                    format!(
                        "function `{}` got multiple values for argument `{}`",
                        func_display_name, kw_name
                    ),
                ));
            }
            bound[idx] = Some(kw_value);
        }

        for (i, slot) in bound.iter_mut().enumerate() {
            if slot.is_none() {
                *slot = sig.default_values[i].clone();
            }
        }

        let missing: Vec<String> = bound
            .iter()
            .enumerate()
            .filter_map(|(i, arg)| {
                if arg.is_none() {
                    Some(format!("`{}`", sig.param_names[i]))
                } else {
                    None
                }
            })
            .collect();
        if !missing.is_empty() {
            return Err(self.type_error(
                line,
                format!(
                    "function `{}` missing required argument{}: {}",
                    func_display_name,
                    if missing.len() == 1 { "" } else { "s" },
                    missing.join(", ")
                ),
            ));
        }

        Ok(bound
            .into_iter()
            .map(|arg| arg.expect("checked missing arguments above"))
            .collect())
    }

    pub(super) fn coerce_args_to_param_types(
        &self,
        mut args: Vec<TirExpr>,
        params: &[Type],
    ) -> Vec<TirExpr> {
        for (arg, expected) in args.iter_mut().zip(params.iter()) {
            if arg.ty.to_type() == *expected {
                continue;
            }
            let target = match expected {
                Type::Float => Some(ValueType::Float),
                Type::Int => Some(ValueType::Int),
                Type::Bool => Some(ValueType::Bool),
                _ => None,
            };
            let Some(target_ty) = target else {
                continue;
            };
            let from = arg.ty.clone();
            if matches!(
                (&from, &target_ty),
                (ValueType::Int, ValueType::Float)
                    | (ValueType::Bool, ValueType::Float)
                    | (ValueType::Bool, ValueType::Int)
                    | (ValueType::Int, ValueType::Bool)
                    | (ValueType::Float, ValueType::Bool)
            ) {
                let old = std::mem::replace(
                    arg,
                    TirExpr {
                        kind: TirExprKind::IntLiteral(0),
                        ty: ValueType::Int,
                    },
                );
                *arg = TirExpr {
                    kind: TirExprKind::Cast {
                        kind: Self::compute_cast_kind(&from, &target_ty),
                        arg: Box::new(old),
                    },
                    ty: target_ty,
                };
            }
        }
        args
    }

    pub(super) fn append_nested_captures_if_needed(
        &mut self,
        line: usize,
        mangled: &str,
        args: &mut Vec<TirExpr>,
    ) -> Result<()> {
        let Some(captures) = self.nested_function_captures.get(mangled) else {
            return Ok(());
        };
        let captures = captures.clone();
        for (name, ty) in &captures {
            let resolved = self.lookup(name).cloned().ok_or_else(|| {
                self.name_error(
                    line,
                    format!(
                        "captured variable `{}` not found at call to `{}`",
                        name, mangled
                    ),
                )
            })?;
            if &resolved != ty {
                return Err(self.type_error(
                    line,
                    format!(
                        "captured variable `{}` type mismatch at call to `{}`: expected `{}`, got `{}`",
                        name, mangled, ty, resolved
                    ),
                ));
            }
            args.push(TirExpr {
                kind: TirExprKind::Var(name.clone()),
                ty: self.value_type_from_type(ty),
            });
        }
        Ok(())
    }

    pub(super) fn check_call_args(
        &mut self,
        line: usize,
        func_name: &str,
        func_type: &Type,
        args: &[TirExpr],
    ) -> Result<Type> {
        match func_type {
            Type::Function {
                params,
                return_type,
            } => {
                if args.len() != params.len() {
                    return Err(self.type_error(
                        line,
                        format!(
                            "function `{}` expects {} argument{}, got {}",
                            func_name,
                            params.len(),
                            if params.len() == 1 { "" } else { "s" },
                            args.len()
                        ),
                    ));
                }
                for (i, (arg, expected)) in args.iter().zip(params.iter()).enumerate() {
                    let expected_vty = self.value_type_from_type(expected);
                    if arg.ty != expected_vty {
                        return Err(self.type_error(
                            line,
                            format!(
                                "argument {} type mismatch in call to `{}`: expected `{}`, got `{}`",
                                i, func_name, expected, arg.ty
                            ),
                        ));
                    }
                }
                Ok(*return_type.clone())
            }
            _ => Err(self.type_error(line, "cannot call non-function type")),
        }
    }
}
