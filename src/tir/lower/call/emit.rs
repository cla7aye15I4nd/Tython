use anyhow::Result;

use crate::ast::{ClassInfo, Type};
use crate::tir::{
    ArithBinOp, CallResult, CallTarget, RawBinOp, TirExpr, TirExprKind, TirStmt, ValueType,
};

use super::super::Lowering;

impl Lowering {
    pub(super) fn cast_to_float_if_needed(&self, arg: TirExpr) -> TirExpr {
        if arg.ty == ValueType::Float {
            return arg;
        }
        let kind = Self::compute_cast_kind(&arg.ty, &ValueType::Float);
        TirExpr {
            kind: TirExprKind::Cast {
                kind,
                arg: Box::new(arg),
            },
            ty: ValueType::Float,
        }
    }

    pub(super) fn lower_sum_class_list(
        &mut self,
        line: usize,
        list_expr: TirExpr,
        start_expr: TirExpr,
        elem_ty: ValueType,
    ) -> Result<TirExpr> {
        let list_var = self.fresh_internal("sum_list");
        let acc_var = self.fresh_internal("sum_acc");
        let loop_var = self.fresh_internal("sum_item");
        let idx_var = self.fresh_internal("sum_idx");
        let len_var = self.fresh_internal("sum_len");

        self.declare(list_var.clone(), list_expr.ty.to_type());
        self.declare(acc_var.clone(), elem_ty.to_type());
        self.declare(loop_var.clone(), elem_ty.to_type());
        self.declare(idx_var.clone(), Type::Int);
        self.declare(len_var.clone(), Type::Int);

        let lhs = TirExpr {
            kind: TirExprKind::Var(acc_var.clone()),
            ty: elem_ty.clone(),
        };
        let rhs = TirExpr {
            kind: TirExprKind::Var(loop_var.clone()),
            ty: elem_ty.clone(),
        };
        let add_expr = self.resolve_binop(line, RawBinOp::Arith(ArithBinOp::Add), lhs, rhs)?;

        self.pre_stmts.push(TirStmt::Let {
            name: list_var.clone(),
            ty: list_expr.ty.clone(),
            value: list_expr,
        });
        self.pre_stmts.push(TirStmt::Let {
            name: acc_var.clone(),
            ty: elem_ty.clone(),
            value: start_expr,
        });
        self.pre_stmts.push(TirStmt::ForList {
            loop_var: loop_var.clone(),
            loop_var_ty: elem_ty.clone(),
            list_var,
            index_var: idx_var,
            len_var,
            body: vec![TirStmt::Let {
                name: acc_var.clone(),
                ty: elem_ty.clone(),
                value: add_expr,
            }],
            else_body: vec![],
        });

        Ok(TirExpr {
            kind: TirExprKind::Var(acc_var),
            ty: elem_ty,
        })
    }

    pub(super) fn lower_constructor_call(
        &mut self,
        line: usize,
        qualified_name: &str,
        class_info: &ClassInfo,
        tir_args: Vec<TirExpr>,
    ) -> Result<CallResult> {
        let init_method = class_info.methods.get("__init__").ok_or_else(|| {
            self.syntax_error(
                line,
                format!("class `{}` has no __init__ method", qualified_name),
            )
        })?;
        let init_type = Type::Function {
            params: init_method.params.clone(),
            return_type: Box::new(Type::Unit),
        };
        let bound_args = self.bind_user_function_args(
            line,
            qualified_name,
            &init_method.mangled_name,
            &init_type,
            tir_args,
            vec![],
        )?;
        let bound_args = self.coerce_args_to_param_types(bound_args, &init_method.params);
        self.check_call_args(line, qualified_name, &init_type, &bound_args)?;

        Ok(CallResult::Expr(TirExpr {
            kind: TirExprKind::Construct {
                class_name: qualified_name.to_string(),
                init_mangled_name: init_method.mangled_name.clone(),
                args: bound_args,
            },
            ty: ValueType::Class(qualified_name.to_string()),
        }))
    }

    pub(in crate::tir::lower) fn lower_class_magic_method(
        &mut self,
        line: usize,
        object: TirExpr,
        method_names: &[&str],
        expected_return_type: Option<ValueType>,
        caller_name: &str,
    ) -> Result<TirExpr> {
        self.lower_class_magic_method_with_args(
            line,
            object,
            method_names,
            expected_return_type,
            caller_name,
            vec![],
        )
    }

    pub(in crate::tir::lower) fn lower_class_magic_method_with_args(
        &mut self,
        line: usize,
        object: TirExpr,
        method_names: &[&str],
        expected_return_type: Option<ValueType>,
        caller_name: &str,
        args: Vec<TirExpr>,
    ) -> Result<TirExpr> {
        if method_names.is_empty() {
            return Err(self.syntax_error(
                line,
                format!(
                    "internal error: {}() magic method list is empty",
                    caller_name
                ),
            ));
        }

        let class_name = match &object.ty {
            ValueType::Class(name) => name.clone(),
            _ => {
                return Err(self.type_error(
                    line,
                    format!("{}() cannot convert `{}`", caller_name, object.ty),
                ))
            }
        };

        let class_info = self.lookup_class(line, &class_name)?;

        let method = method_names
            .iter()
            .find_map(|name| class_info.methods.get(*name))
            .ok_or_else(|| {
                if method_names.len() == 1 {
                    self.attribute_error(
                        line,
                        format!("class `{}` has no method `{}`", class_name, method_names[0]),
                    )
                } else {
                    let choices = method_names
                        .iter()
                        .map(|name| format!("`{}`", name))
                        .collect::<Vec<_>>()
                        .join(" or ");
                    self.attribute_error(
                        line,
                        format!("class `{}` has no method {}", class_name, choices),
                    )
                }
            })?;

        if method.params.len() != args.len() {
            return Err(self.type_error(
                line,
                format!(
                    "{}.{}() expects {} argument{} besides `self`, got {}",
                    class_name,
                    method.name,
                    method.params.len(),
                    if method.params.len() == 1 { "" } else { "s" },
                    args.len()
                ),
            ));
        }
        for (i, (arg, expected)) in args.iter().zip(method.params.iter()).enumerate() {
            let expected_vty = self.value_type_from_type(expected);
            if arg.ty != expected_vty {
                return Err(self.type_error(
                    line,
                    format!(
                        "argument {} type mismatch in {}.{}(): expected `{}`, got `{}`",
                        i, class_name, method.name, expected, arg.ty
                    ),
                ));
            }
        }

        // Clone data from method/class_info before mutable borrow via value_type_from_type
        let method_return_type = method.return_type.clone();
        let method_name = method.name.clone();
        let method_mangled_name = method.mangled_name.clone();

        let return_type = if let Some(ref expected) = expected_return_type {
            let method_return_vty = self.value_type_from_type(&method_return_type);
            if method_return_vty != *expected {
                return Err(self.type_error(
                    line,
                    format!(
                        "{}.{}() must return `{}`, got `{}`",
                        class_name, method_name, expected, method_return_type
                    ),
                ));
            }
            expected.clone()
        } else {
            if method_return_type == Type::Unit {
                return Err(self.type_error(
                    line,
                    format!(
                        "{}.{}() must return a value, got `None`",
                        class_name, method_name
                    ),
                ));
            }
            self.value_type_from_type(&method_return_type)
        };

        let mut call_args = Vec::with_capacity(1 + args.len());
        call_args.push(object);
        call_args.extend(args);

        Ok(TirExpr {
            kind: TirExprKind::Call {
                func: method_mangled_name,
                args: call_args,
            },
            ty: return_type,
        })
    }

    pub(super) fn build_named_call_result(
        &mut self,
        mangled: String,
        return_type: Type,
        args: Vec<TirExpr>,
    ) -> CallResult {
        if return_type == Type::Unit {
            CallResult::VoidStmt(Box::new(TirStmt::VoidCall {
                target: CallTarget::Named(mangled),
                args,
            }))
        } else {
            CallResult::Expr(TirExpr {
                kind: TirExprKind::Call {
                    func: mangled,
                    args,
                },
                ty: self.value_type_from_type(&return_type),
            })
        }
    }
}
