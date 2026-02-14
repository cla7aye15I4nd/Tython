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

    fn push_print_str_literal(stmts: &mut Vec<TirStmt>, s: impl Into<String>) {
        stmts.push(TirStmt::VoidCall {
            target: CallTarget::Builtin(builtin::BuiltinFn::PrintStr),
            args: vec![TirExpr {
                kind: TirExprKind::StrLiteral(s.into()),
                ty: ValueType::Str,
            }],
        });
    }

    fn lower_print_class_as_str(&self, line: usize, object: TirExpr) -> Result<TirExpr> {
        let rule = crate::tir::type_rules::lookup_builtin_call("str", &[&object.ty])
            .expect("ICE: missing builtin rule for str() on class");
        match rule {
            crate::tir::type_rules::BuiltinCallRule::ClassMagic {
                method_names,
                return_type,
            } => self.lower_class_magic_method(line, object, method_names, return_type, "str"),
            _ => unreachable!("ICE: str() on class should resolve to ClassMagic"),
        }
    }

    fn lower_print_value_stmts(
        &mut self,
        line: usize,
        arg: TirExpr,
        stmts: &mut Vec<TirStmt>,
    ) -> Result<()> {
        macro_rules! push_direct_print {
            ($fn_name:expr, $value:expr) => {{
                stmts.push(TirStmt::VoidCall {
                    target: CallTarget::Builtin($fn_name),
                    args: vec![$value],
                });
                Ok(())
            }};
        }

        match &arg.ty {
            ValueType::Tuple(element_types) => {
                let tuple_var = self.fresh_internal("print_tuple");
                let tuple_ty = arg.ty.clone();
                let tuple_element_types = element_types.clone();

                stmts.push(TirStmt::Let {
                    name: tuple_var.clone(),
                    ty: tuple_ty.clone(),
                    value: arg,
                });

                Self::push_print_str_literal(stmts, "(");

                for (i, element_ty) in tuple_element_types.iter().enumerate() {
                    if i > 0 {
                        Self::push_print_str_literal(stmts, ", ");
                    }

                    let element_expr = TirExpr {
                        kind: TirExprKind::GetField {
                            object: Box::new(TirExpr {
                                kind: TirExprKind::Var(tuple_var.clone()),
                                ty: tuple_ty.clone(),
                            }),
                            field_index: i,
                        },
                        ty: element_ty.clone(),
                    };

                    self.lower_print_repr_stmts(line, element_expr, stmts)?;
                }

                if tuple_element_types.len() == 1 {
                    Self::push_print_str_literal(stmts, ",");
                }
                Self::push_print_str_literal(stmts, ")");
                Ok(())
            }
            ValueType::Class(_) => {
                let print_arg = self.lower_print_class_as_str(line, arg)?;
                self.lower_print_value_stmts(line, print_arg, stmts)
            }
            ValueType::Float => push_direct_print!(builtin::BuiltinFn::PrintFloat, arg),
            ValueType::Bool => push_direct_print!(builtin::BuiltinFn::PrintBool, arg),
            ValueType::Int => push_direct_print!(builtin::BuiltinFn::PrintInt, arg),
            ValueType::Str => push_direct_print!(builtin::BuiltinFn::PrintStr, arg),
            ValueType::Bytes => push_direct_print!(builtin::BuiltinFn::PrintBytes, arg),
            ValueType::ByteArray => push_direct_print!(builtin::BuiltinFn::PrintByteArray, arg),
            ValueType::List(_) => {
                let inner_ty = match arg.ty.clone() {
                    ValueType::List(inner) => *inner,
                    _ => unreachable!(),
                };
                self.lower_print_list_stmts(line, arg, inner_ty, stmts)
            }
            ValueType::Function { .. } => {
                Err(self.type_error(line, format!("cannot print value of type `{}`", arg.ty)))
            }
            ValueType::Dict(_, _) | ValueType::Set(_) => {
                Err(self.type_error(line, format!("cannot print value of type `{}`", arg.ty)))
            }
        }
    }

    /// Auto-generate list printing for any element type by iterating and
    /// printing each element's repr. Replaces per-type C++ print_list_*
    /// runtime functions.
    fn lower_print_list_stmts(
        &mut self,
        line: usize,
        arg: TirExpr,
        loop_var_ty: ValueType,
        stmts: &mut Vec<TirStmt>,
    ) -> Result<()> {
        let list_var = self.fresh_internal("print_list");
        let idx_var = self.fresh_internal("print_idx");
        let len_var = self.fresh_internal("print_len");
        let loop_var = self.fresh_internal("print_elem");
        let list_ty = arg.ty.clone();

        stmts.push(TirStmt::Let {
            name: list_var.clone(),
            ty: list_ty,
            value: arg,
        });
        Self::push_print_str_literal(stmts, "[");

        let mut body = Vec::new();
        let idx_gt_zero = TirExpr {
            kind: TirExprKind::IntGt(
                Box::new(TirExpr {
                    kind: TirExprKind::Var(idx_var.clone()),
                    ty: ValueType::Int,
                }),
                Box::new(TirExpr {
                    kind: TirExprKind::IntLiteral(0),
                    ty: ValueType::Int,
                }),
            ),
            ty: ValueType::Bool,
        };
        body.push(TirStmt::If {
            condition: idx_gt_zero,
            then_body: vec![TirStmt::VoidCall {
                target: CallTarget::Builtin(builtin::BuiltinFn::PrintStr),
                args: vec![TirExpr {
                    kind: TirExprKind::StrLiteral(", ".to_string()),
                    ty: ValueType::Str,
                }],
            }],
            else_body: vec![],
        });

        let elem_expr = TirExpr {
            kind: TirExprKind::Var(loop_var.clone()),
            ty: loop_var_ty.clone(),
        };
        self.lower_print_repr_stmts(line, elem_expr, &mut body)?;

        stmts.push(TirStmt::ForList {
            loop_var,
            loop_var_ty,
            list_var,
            index_var: idx_var,
            len_var,
            body,
            else_body: vec![],
        });
        Self::push_print_str_literal(stmts, "]");
        Ok(())
    }

    /// Print the repr of a value (used inside list/tuple printing).
    /// Strings are wrapped in quotes; all other types delegate to
    /// `lower_print_value_stmts` (which already outputs the repr form
    /// for bytes, bytearray, nested lists, etc.).
    fn lower_print_repr_stmts(
        &mut self,
        line: usize,
        arg: TirExpr,
        stmts: &mut Vec<TirStmt>,
    ) -> Result<()> {
        match &arg.ty {
            ValueType::Str => {
                Self::push_print_str_literal(stmts, "'");
                stmts.push(TirStmt::VoidCall {
                    target: CallTarget::Builtin(builtin::BuiltinFn::PrintStr),
                    args: vec![arg],
                });
                Self::push_print_str_literal(stmts, "'");
                Ok(())
            }
            _ => self.lower_print_value_stmts(line, arg, stmts),
        }
    }
}
