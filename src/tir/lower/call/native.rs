use anyhow::Result;

use crate::tir::{CallResult, CallTarget, TirExpr, TirExprKind, TirStmt, ValueType};

use super::super::Lowering;

impl Lowering {
    pub(super) fn lower_native_module_call(
        &self,
        line: usize,
        module: &str,
        attr: &str,
        mut positional_args: Vec<TirExpr>,
        keyword_args: Vec<(String, TirExpr)>,
    ) -> Result<CallResult> {
        match (module, attr) {
            ("math", "log") | ("math", "exp") => {
                if !keyword_args.is_empty() {
                    return Err(self.syntax_error(
                        line,
                        format!("{}.{}() does not accept keywords", module, attr),
                    ));
                }
                if positional_args.len() != 1 {
                    return Err(self.type_error(
                        line,
                        format!(
                            "{}.{}() expects 1 argument, got {}",
                            module,
                            attr,
                            positional_args.len()
                        ),
                    ));
                }
                let arg = self.cast_to_float_if_needed(positional_args.remove(0));
                let func = if attr == "log" {
                    crate::tir::builtin::BuiltinFn::MathLog
                } else {
                    crate::tir::builtin::BuiltinFn::MathExp
                };
                Ok(CallResult::Expr(TirExpr {
                    kind: TirExprKind::ExternalCall {
                        func,
                        args: vec![arg],
                    },
                    ty: ValueType::Float,
                }))
            }
            ("random", "seed") => {
                if !keyword_args.is_empty() {
                    return Err(self.syntax_error(line, "random.seed() does not accept keywords"));
                }
                if positional_args.len() != 1 || positional_args[0].ty != ValueType::Int {
                    return Err(
                        self.type_error(line, "random.seed() expects exactly one `int` argument")
                    );
                }
                Ok(CallResult::VoidStmt(Box::new(TirStmt::VoidCall {
                    target: CallTarget::Builtin(crate::tir::builtin::BuiltinFn::RandomSeed),
                    args: positional_args,
                })))
            }
            ("random", "gauss") => {
                if !keyword_args.is_empty() {
                    return Err(self.syntax_error(line, "random.gauss() does not accept keywords"));
                }
                if positional_args.len() != 2 {
                    return Err(self.type_error(
                        line,
                        format!(
                            "random.gauss() expects 2 arguments, got {}",
                            positional_args.len()
                        ),
                    ));
                }
                let mu = self.cast_to_float_if_needed(positional_args.remove(0));
                let sigma = self.cast_to_float_if_needed(positional_args.remove(0));
                Ok(CallResult::Expr(TirExpr {
                    kind: TirExprKind::ExternalCall {
                        func: crate::tir::builtin::BuiltinFn::RandomGauss,
                        args: vec![mu, sigma],
                    },
                    ty: ValueType::Float,
                }))
            }
            ("random", "shuffle") => {
                if !keyword_args.is_empty() {
                    return Err(
                        self.syntax_error(line, "random.shuffle() does not accept keywords")
                    );
                }
                if positional_args.len() != 1 {
                    return Err(self.type_error(
                        line,
                        format!(
                            "random.shuffle() expects 1 argument, got {}",
                            positional_args.len()
                        ),
                    ));
                }
                let list_arg = positional_args.remove(0);
                if !matches!(list_arg.ty, ValueType::List(_)) {
                    return Err(self.type_error(
                        line,
                        format!("random.shuffle() expects `list`, got `{}`", list_arg.ty),
                    ));
                }
                Ok(CallResult::VoidStmt(Box::new(TirStmt::VoidCall {
                    target: CallTarget::Builtin(crate::tir::builtin::BuiltinFn::RandomShuffle),
                    args: vec![list_arg],
                })))
            }
            ("random", "choices") => {
                if positional_args.len() != 1 {
                    return Err(self.type_error(
                        line,
                        format!(
                            "random.choices() expects population as 1 positional argument, got {}",
                            positional_args.len()
                        ),
                    ));
                }
                let population = positional_args.remove(0);
                if population.ty != ValueType::List(Box::new(ValueType::Int)) {
                    return Err(self.type_error(
                        line,
                        format!(
                            "random.choices() population must be `list[int]`, got `{}`",
                            population.ty
                        ),
                    ));
                }

                if keyword_args.len() != 1 || keyword_args[0].0 != "weights" {
                    return Err(self.type_error(
                        line,
                        "random.choices() currently requires exactly keyword argument `weights=`",
                    ));
                }
                let weights = keyword_args[0].1.clone();
                if weights.ty != ValueType::List(Box::new(ValueType::Float)) {
                    return Err(self.type_error(
                        line,
                        format!(
                            "random.choices() weights must be `list[float]`, got `{}`",
                            weights.ty
                        ),
                    ));
                }

                Ok(CallResult::Expr(TirExpr {
                    kind: TirExprKind::ExternalCall {
                        func: crate::tir::builtin::BuiltinFn::RandomChoicesInt,
                        args: vec![population, weights],
                    },
                    ty: ValueType::List(Box::new(ValueType::Int)),
                }))
            }
            _ => Err(self.name_error(
                line,
                format!("unsupported native module function {}.{}", module, attr),
            )),
        }
    }
}
