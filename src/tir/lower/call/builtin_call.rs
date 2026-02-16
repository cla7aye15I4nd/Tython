use anyhow::Result;

use crate::tir::builtin::BuiltinFn;
use crate::tir::{CallResult, TirExpr, TirExprKind, ValueType};

use super::super::Lowering;

// ── Predicates ───────────────────────────────────────────────────────

pub fn is_builtin_call(name: &str) -> bool {
    matches!(
        name,
        "str"
            | "repr"
            | "bytes"
            | "bytearray"
            | "int"
            | "float"
            | "bool"
            | "len"
            | "abs"
            | "round"
            | "pow"
            | "min"
            | "max"
            | "sum"
            | "all"
            | "any"
            | "sorted"
            | "dict"
            | "set"
            | "iter"
            | "next"
            | "range"
            | "reversed"
    )
}

// ── Error messages ───────────────────────────────────────────────────

pub fn builtin_call_error_message(name: &str, arg_types: &[&ValueType], provided: usize) -> String {
    match name {
        "str" | "bytes" | "int" | "float" | "bool" => {
            if provided != 1 {
                format!("{}() expects exactly 1 argument, got {}", name, provided)
            } else {
                format!("{}() cannot convert `{}`", name, arg_types[0])
            }
        }
        "repr" => {
            debug_assert_ne!(
                provided, 1,
                "single-argument repr() calls are handled in lower_builtin_call"
            );
            format!("repr() expects exactly 1 argument, got {}", provided)
        }
        "bytearray" => {
            if provided > 1 {
                format!("bytearray() expects 0 or 1 arguments, got {}", provided)
            } else {
                format!("bytearray() cannot convert `{}`", arg_types[0])
            }
        }
        "dict" | "set" => {
            format!("{}() expects no arguments, got {}", name, provided)
        }
        "len" => {
            debug_assert_ne!(
                provided, 1,
                "single-argument len() calls are handled in lower_builtin_call"
            );
            format!("len() expects exactly 1 argument, got {}", provided)
        }
        "abs" => {
            if provided != 1 {
                format!("abs() expects exactly 1 argument, got {}", provided)
            } else {
                format!("abs() requires a numeric argument, got `{}`", arg_types[0])
            }
        }
        "round" => {
            if provided != 1 {
                format!("round() expects exactly 1 argument, got {}", provided)
            } else {
                format!(
                    "round() requires a `float` argument, got `{}`",
                    arg_types[0]
                )
            }
        }
        "pow" => {
            if provided != 2 {
                format!("pow() expects 2 arguments, got {}", provided)
            } else if arg_types[0] != arg_types[1] {
                format!(
                    "pow() arguments must have the same type: got `{}` and `{}`",
                    arg_types[0], arg_types[1]
                )
            } else {
                format!("pow() requires numeric arguments, got `{}`", arg_types[0])
            }
        }
        "min" | "max" => {
            if provided < 2 {
                format!("{}() expects at least 2 arguments, got {}", name, provided)
            } else if arg_types.iter().any(|ty| *ty != arg_types[0]) {
                format!(
                    "{}() arguments must have the same type: got `{}` and `{}`",
                    name,
                    arg_types[0],
                    arg_types
                        .iter()
                        .copied()
                        .find(|ty| *ty != arg_types[0])
                        .unwrap_or(arg_types[0])
                )
            } else {
                format!(
                    "{}() requires numeric arguments, got `{}`",
                    name, arg_types[0]
                )
            }
        }
        "sum" => {
            if provided == 0 || provided > 2 {
                format!("sum() expects 1 or 2 arguments, got {}", provided)
            } else {
                format!(
                    "sum() requires a list of numbers and optional start value, got `{}`",
                    arg_types[0]
                )
            }
        }
        "all" | "any" => {
            if provided != 1 {
                format!("{}() expects exactly 1 argument, got {}", name, provided)
            } else {
                format!("{}() requires a list, got `{}`", name, arg_types[0])
            }
        }
        "sorted" => {
            if provided != 1 {
                format!("sorted() expects exactly 1 argument, got {}", provided)
            } else {
                format!(
                    "sorted() requires a list whose elements support ordering (`__lt__`), got `{}`",
                    arg_types[0]
                )
            }
        }
        "iter" => {
            if provided != 1 {
                format!("iter() expects 1 argument, got {}", provided)
            } else {
                format!(
                    "iter() argument must be a class with `__iter__`, got `{}`",
                    arg_types[0]
                )
            }
        }
        "next" => {
            if provided != 1 {
                format!("next() expects 1 argument, got {}", provided)
            } else {
                format!(
                    "next() argument must be a class with `__next__`, got `{}`",
                    arg_types[0]
                )
            }
        }
        _ => unreachable!("not a built-in call: {}", name),
    }
}

// ── Helpers for building CallResult values ────────────────────────────

fn external_call_result(func: BuiltinFn, return_type: ValueType, args: Vec<TirExpr>) -> CallResult {
    CallResult::Expr(TirExpr {
        kind: TirExprKind::ExternalCall { func, args },
        ty: return_type,
    })
}

fn identity_result(args: Vec<TirExpr>) -> CallResult {
    CallResult::Expr(
        args.into_iter()
            .next()
            .expect("ICE: identity conversion expects one arg"),
    )
}

fn cast_result(args: Vec<TirExpr>, target_type: ValueType) -> CallResult {
    let arg = args
        .into_iter()
        .next()
        .expect("ICE: primitive cast expects one arg");
    let cast_kind = Lowering::compute_cast_kind(&arg.ty, &target_type);
    CallResult::Expr(TirExpr {
        kind: TirExprKind::Cast {
            kind: cast_kind,
            arg: Box::new(arg),
        },
        ty: target_type,
    })
}

fn fold_call_result(func: BuiltinFn, return_type: ValueType, args: Vec<TirExpr>) -> CallResult {
    let mut iter = args.into_iter();
    let mut acc = iter
        .next()
        .expect("ICE: FoldExternalCall expects at least two args");
    for arg in iter {
        acc = TirExpr {
            kind: TirExprKind::ExternalCall {
                func,
                args: vec![acc, arg],
            },
            ty: return_type.clone(),
        };
    }
    CallResult::Expr(acc)
}

// ── Merged builtin call lowering ─────────────────────────────────────

impl Lowering {
    /// Lower a builtin function call directly to TIR.
    /// Merges type-checking, rule lookup, and code generation into one step.
    pub(in crate::tir::lower) fn lower_builtin_call(
        &mut self,
        line: usize,
        name: &str,
        mut args: Vec<TirExpr>,
    ) -> Result<CallResult> {
        let arg_types: Vec<&ValueType> = args.iter().map(|a| &a.ty).collect();
        let provided = args.len();

        // Try the main dispatch table.  Arms that need &mut self use early return.
        // Arms that are simple ExternalCall return Some((func, return_type));
        // None means "invalid call".
        let simple = match (name, arg_types.as_slice()) {
            // ── str / repr ───────────────────────────────────────────
            ("str" | "repr", [ValueType::Class(_)]) => {
                let method_names: &[&str] = if name == "repr" {
                    &["__repr__"]
                } else {
                    &["__str__", "__repr__"]
                };
                let arg = args.remove(0);
                return Ok(CallResult::Expr(self.lower_class_magic_method(
                    line,
                    arg,
                    method_names,
                    Some(ValueType::Str),
                    name,
                )?));
            }
            ("str" | "repr", [ValueType::Int]) => Some((BuiltinFn::StrFromInt, ValueType::Str)),
            ("str" | "repr", [ValueType::Float]) => Some((BuiltinFn::StrFromFloat, ValueType::Str)),
            ("str" | "repr", [ValueType::Bool]) => Some((BuiltinFn::StrFromBool, ValueType::Str)),
            ("str", [ValueType::Str]) => return Ok(identity_result(args)),
            ("repr", [ValueType::Str]) => Some((BuiltinFn::ReprStr, ValueType::Str)),
            ("str", [_]) => {
                let arg = args.remove(0);
                return self.lower_method_call(line, arg, "__str__", vec![]);
            }
            ("repr", [_]) => {
                let arg = args.remove(0);
                return self.lower_method_call(line, arg, "__repr__", vec![]);
            }

            // ── bytes ────────────────────────────────────────────────
            ("bytes", [ValueType::Bytes]) => return Ok(identity_result(args)),
            ("bytes", [ValueType::Class(_)]) => {
                let arg = args.remove(0);
                return Ok(CallResult::Expr(self.lower_class_magic_method(
                    line,
                    arg,
                    &["__bytes__"],
                    Some(ValueType::Bytes),
                    "bytes",
                )?));
            }
            ("bytes", [ValueType::Int]) => Some((BuiltinFn::BytesFromInt, ValueType::Bytes)),

            // ── bytearray ────────────────────────────────────────────
            ("bytearray", []) => Some((BuiltinFn::ByteArrayEmpty, ValueType::ByteArray)),
            ("bytearray", [ValueType::ByteArray]) => return Ok(identity_result(args)),
            ("bytearray", [ValueType::Int]) => {
                Some((BuiltinFn::ByteArrayFromInt, ValueType::ByteArray))
            }
            ("bytearray", [ValueType::Bytes]) => {
                Some((BuiltinFn::ByteArrayFromBytes, ValueType::ByteArray))
            }

            // ── dict / set constructors ──────────────────────────────
            ("dict", []) => Some((
                BuiltinFn::DictEmpty,
                ValueType::Dict(Box::new(ValueType::Int), Box::new(ValueType::Int)),
            )),
            ("set", []) => Some((
                BuiltinFn::SetEmpty,
                ValueType::Set(Box::new(ValueType::Int)),
            )),
            ("set", [ValueType::Str]) => Some((
                BuiltinFn::SetFromStr,
                ValueType::List(Box::new(ValueType::Str)),
            )),

            // ── int / float / bool conversions ───────────────────────
            ("int", [ValueType::Int]) => return Ok(identity_result(args)),
            ("int", [ValueType::Class(_)]) => {
                let arg = args.remove(0);
                return Ok(CallResult::Expr(self.lower_class_magic_method(
                    line,
                    arg,
                    &["__int__"],
                    Some(ValueType::Int),
                    "int",
                )?));
            }
            ("int", [ValueType::Float | ValueType::Bool]) => {
                return Ok(cast_result(args, ValueType::Int));
            }
            ("float", [ValueType::Float]) => return Ok(identity_result(args)),
            ("float", [ValueType::Class(_)]) => {
                let arg = args.remove(0);
                return Ok(CallResult::Expr(self.lower_class_magic_method(
                    line,
                    arg,
                    &["__float__"],
                    Some(ValueType::Float),
                    "float",
                )?));
            }
            ("float", [ValueType::Int | ValueType::Bool]) => {
                return Ok(cast_result(args, ValueType::Float));
            }
            ("bool", [ValueType::Bool]) => return Ok(identity_result(args)),
            ("bool", [ValueType::Class(_)]) => {
                let arg = args.remove(0);
                return Ok(CallResult::Expr(self.lower_class_magic_method(
                    line,
                    arg,
                    &["__bool__"],
                    Some(ValueType::Bool),
                    "bool",
                )?));
            }
            ("bool", [ValueType::Int | ValueType::Float]) => {
                return Ok(cast_result(args, ValueType::Bool));
            }

            // ── len ──────────────────────────────────────────────────
            ("len", [ValueType::Class(_)]) => {
                let arg = args.remove(0);
                return Ok(CallResult::Expr(self.lower_class_magic_method(
                    line,
                    arg,
                    &["__len__"],
                    Some(ValueType::Int),
                    "len",
                )?));
            }
            ("len", [_]) => {
                let arg = args.remove(0);
                return self.lower_method_call(line, arg, "__len__", vec![]);
            }

            // ── abs / round ──────────────────────────────────────────
            ("abs", [ValueType::Class(_)]) => {
                let arg = args.remove(0);
                return Ok(CallResult::Expr(self.lower_class_magic_method(
                    line,
                    arg,
                    &["__abs__"],
                    None,
                    "abs",
                )?));
            }
            ("abs", [ValueType::Int]) => Some((BuiltinFn::AbsInt, ValueType::Int)),
            ("abs", [ValueType::Float]) => Some((BuiltinFn::AbsFloat, ValueType::Float)),
            ("round", [ValueType::Class(_)]) => {
                let arg = args.remove(0);
                return Ok(CallResult::Expr(self.lower_class_magic_method(
                    line,
                    arg,
                    &["__round__"],
                    None,
                    "round",
                )?));
            }
            ("round", [ValueType::Float]) => Some((BuiltinFn::RoundFloat, ValueType::Int)),

            // ── min / max ────────────────────────────────────────────
            ("min", _) if provided >= 2 && arg_types.iter().all(|ty| **ty == ValueType::Int) => {
                return Ok(fold_call_result(BuiltinFn::MinInt, ValueType::Int, args));
            }
            ("min", _) if provided >= 2 && arg_types.iter().all(|ty| **ty == ValueType::Float) => {
                return Ok(fold_call_result(
                    BuiltinFn::MinFloat,
                    ValueType::Float,
                    args,
                ));
            }
            ("max", [ValueType::List(inner)]) => match inner.as_ref() {
                ValueType::Int | ValueType::Bool => Some((BuiltinFn::MaxListInt, ValueType::Int)),
                ValueType::Float => Some((BuiltinFn::MaxListFloat, ValueType::Float)),
                _ => None,
            },
            ("max", _) if provided >= 2 && arg_types.iter().all(|ty| **ty == ValueType::Int) => {
                return Ok(fold_call_result(BuiltinFn::MaxInt, ValueType::Int, args));
            }
            ("max", _) if provided >= 2 && arg_types.iter().all(|ty| **ty == ValueType::Float) => {
                return Ok(fold_call_result(
                    BuiltinFn::MaxFloat,
                    ValueType::Float,
                    args,
                ));
            }

            // ── pow ──────────────────────────────────────────────────
            ("pow", [ValueType::Int, ValueType::Int]) => Some((BuiltinFn::PowInt, ValueType::Int)),
            ("pow", [ValueType::Float, ValueType::Float]) => {
                let right = args.remove(1);
                let left = args.remove(0);
                return Ok(CallResult::Expr(TirExpr {
                    kind: TirExprKind::FloatPow(Box::new(left), Box::new(right)),
                    ty: ValueType::Float,
                }));
            }

            // ── sum ──────────────────────────────────────────────────
            ("sum", [ValueType::List(inner)]) => match inner.as_ref() {
                ValueType::Int | ValueType::Bool => Some((BuiltinFn::SumInt, ValueType::Int)),
                ValueType::Float => Some((BuiltinFn::SumFloat, ValueType::Float)),
                _ => None,
            },
            ("sum", [ValueType::List(inner), ValueType::Int]) => match inner.as_ref() {
                ValueType::Int | ValueType::Bool => Some((BuiltinFn::SumIntStart, ValueType::Int)),
                _ => None,
            },
            ("sum", [ValueType::List(inner), ValueType::Float]) => match inner.as_ref() {
                ValueType::Float => Some((BuiltinFn::SumFloatStart, ValueType::Float)),
                _ => None,
            },

            // ── all / any ────────────────────────────────────────────
            ("all", [ValueType::List(_)]) => Some((BuiltinFn::AllList, ValueType::Bool)),
            ("any", [ValueType::List(_)]) => Some((BuiltinFn::AnyList, ValueType::Bool)),

            // ── sorted / reversed ────────────────────────────────────
            ("reversed", [ValueType::List(inner)]) => {
                Some((BuiltinFn::ReversedList, ValueType::List(inner.clone())))
            }

            // ── iter / next ──────────────────────────────────────────
            ("iter", [ValueType::Class(_)]) => {
                let arg = args.remove(0);
                return Ok(CallResult::Expr(self.lower_class_magic_method(
                    line,
                    arg,
                    &["__iter__"],
                    None,
                    "iter",
                )?));
            }
            ("next", [ValueType::Class(_)]) => {
                let arg = args.remove(0);
                return Ok(CallResult::Expr(self.lower_class_magic_method(
                    line,
                    arg,
                    &["__next__"],
                    None,
                    "next",
                )?));
            }

            // ── range ────────────────────────────────────────────────
            ("range", [ValueType::Int]) => {
                Some((BuiltinFn::Range1, ValueType::List(Box::new(ValueType::Int))))
            }
            ("range", [ValueType::Int, ValueType::Int]) => {
                Some((BuiltinFn::Range2, ValueType::List(Box::new(ValueType::Int))))
            }
            ("range", [ValueType::Int, ValueType::Int, ValueType::Int]) => {
                Some((BuiltinFn::Range3, ValueType::List(Box::new(ValueType::Int))))
            }

            _ => None,
        };

        match simple {
            Some((func, return_type)) => Ok(external_call_result(func, return_type, args)),
            None => {
                Err(self.type_error(line, builtin_call_error_message(name, &arg_types, provided)))
            }
        }
    }

    /// Lower a builtin call for a single argument expression (used by f-strings).
    pub(in crate::tir::lower) fn lower_builtin_single_arg_expr(
        &mut self,
        line: usize,
        name: &str,
        arg: TirExpr,
    ) -> Result<TirExpr> {
        let arg_ty = arg.ty.clone();
        let call = self
            .lower_builtin_call(line, name, vec![arg])
            .map_err(|_| {
                self.type_error(
                    line,
                    format!(
                        "f-string conversion `{}` is not defined for type `{}`",
                        name, arg_ty
                    ),
                )
            })?;
        let CallResult::Expr(expr) = call else {
            unreachable!("f-string conversion should always lower to an expression");
        };
        Ok(expr)
    }
}
