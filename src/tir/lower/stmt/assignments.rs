use anyhow::Result;
use pyo3::prelude::*;

use crate::ast::Type;
use crate::tir::{
    builtin, ArithBinOp, BitwiseBinOp, CallResult, CallTarget, IntrinsicOp, RawBinOp, TirExpr,
    TirExprKind, TirStmt, ValueType,
};
use crate::{ast_get_list, ast_get_string, ast_getattr, ast_type_name};

use crate::tir::lower::Lowering;

/// Map augmented assignment operators to their corresponding magic method names.
fn aug_op_to_dunder(op: RawBinOp) -> Option<&'static str> {
    use ArithBinOp::*;
    use BitwiseBinOp::*;
    use RawBinOp::*;
    Some(match op {
        Arith(Add) => "__iadd__",
        Arith(Sub) => "__isub__",
        Arith(Mul) => "__imul__",
        Arith(Div) => "__itruediv__",
        Arith(FloorDiv) => "__ifloordiv__",
        Arith(Mod) => "__imod__",
        Arith(Pow) => "__ipow__",
        Bitwise(BitAnd) => "__iand__",
        Bitwise(BitOr) => "__ior__",
        Bitwise(BitXor) => "__ixor__",
        Bitwise(LShift) => "__ilshift__",
        Bitwise(RShift) => "__irshift__",
    })
}

impl Lowering {
    pub(in crate::tir::lower) fn handle_ann_assign(
        &mut self,
        node: &Bound<PyAny>,
        line: usize,
    ) -> Result<Vec<TirStmt>> {
        let target_node = ast_getattr!(node, "target");
        if ast_type_name!(target_node) != "Name" {
            return Err(self.syntax_error(line, "only simple variable assignments are supported"));
        }
        let target = ast_get_string!(target_node, "id");

        let annotation = ast_getattr!(node, "annotation");
        let annotated_ty = (!annotation.is_none())
            .then(|| self.convert_type_annotation(&annotation))
            .transpose()?;

        let value_node = ast_getattr!(node, "value");

        // Handle empty list literal `[]` with type annotation
        if ast_type_name!(value_node) == "List" {
            let elts = ast_get_list!(value_node, "elts");
            if elts.is_empty() {
                let list_ty = annotated_ty.ok_or_else(|| {
                    self.syntax_error(line, "empty list literal `[]` requires a type annotation")
                })?;
                let inner_ty = match &list_ty {
                    Type::List(inner) => self.value_type_from_type(inner),
                    _ => {
                        return Err(self.type_error(
                            line,
                            format!("type mismatch: expected `list[...]`, got `{}`", list_ty),
                        ))
                    }
                };
                let vty = ValueType::List(Box::new(inner_ty.clone()));
                self.declare(target.clone(), list_ty);
                return Ok(vec![TirStmt::Let {
                    name: target,
                    ty: vty.clone(),
                    value: TirExpr {
                        kind: TirExprKind::ListLiteral {
                            element_type: inner_ty,
                            elements: vec![],
                        },
                        ty: vty,
                    },
                }]);
            }
        }

        // Handle empty dict literal `{}` with type annotation
        if ast_type_name!(value_node) == "Dict" {
            let keys = ast_get_list!(value_node, "keys");
            if keys.is_empty() {
                let dict_ty = annotated_ty.ok_or_else(|| {
                    self.syntax_error(line, "empty dict literal `{}` requires a type annotation")
                })?;
                let (key_ty, value_ty) = match &dict_ty {
                    Type::Dict(key, value) => (
                        self.value_type_from_type(key),
                        self.value_type_from_type(value),
                    ),
                    _ => {
                        return Err(self.type_error(
                            line,
                            format!(
                                "type mismatch: expected `dict[..., ...]`, got `{}`",
                                dict_ty
                            ),
                        ))
                    }
                };
                let vty = ValueType::Dict(Box::new(key_ty), Box::new(value_ty));
                self.declare(target.clone(), dict_ty);
                return Ok(vec![TirStmt::Let {
                    name: target,
                    ty: vty.clone(),
                    value: TirExpr {
                        kind: TirExprKind::ExternalCall {
                            func: builtin::BuiltinFn::DictEmpty,
                            args: vec![],
                        },
                        ty: vty,
                    },
                }]);
            }
        }

        // Handle empty set constructor set() with type annotation
        if ast_type_name!(value_node) == "Call" {
            let func_node_inner = ast_getattr!(value_node, "func");
            if ast_type_name!(func_node_inner) == "Name"
                && ast_get_string!(func_node_inner, "id") == "set"
                && ast_get_list!(value_node, "args").is_empty()
            {
                let set_ty = annotated_ty.ok_or_else(|| {
                    self.syntax_error(line, "set() requires a type annotation in this context")
                })?;
                let inner_ty = match &set_ty {
                    Type::Set(inner) => self.value_type_from_type(inner),
                    _ => {
                        return Err(self.type_error(
                            line,
                            format!("type mismatch: expected `set[...]`, got `{}`", set_ty),
                        ))
                    }
                };
                let vty = ValueType::Set(Box::new(inner_ty));
                self.declare(target.clone(), set_ty);
                return Ok(vec![TirStmt::Let {
                    name: target,
                    ty: vty.clone(),
                    value: TirExpr {
                        kind: TirExprKind::ExternalCall {
                            func: builtin::BuiltinFn::SetEmpty,
                            args: vec![],
                        },
                        ty: vty,
                    },
                }]);
            }
        }

        // Handle tuple(genexpr) with type annotation
        if ast_type_name!(value_node) == "Call" {
            let func_node_inner = ast_getattr!(value_node, "func");
            if ast_type_name!(func_node_inner) == "Name"
                && ast_get_string!(func_node_inner, "id") == "tuple"
            {
                let call_args = ast_get_list!(value_node, "args");
                if call_args.len() == 1 {
                    let arg0 = call_args.get_item(0).unwrap();
                    if ast_type_name!(arg0) == "GeneratorExp" {
                        let tuple_ty = annotated_ty.clone().ok_or_else(|| {
                            self.syntax_error(line, "tuple(genexpr) requires a type annotation")
                        })?;
                        let element_types = match &tuple_ty {
                            Type::Tuple(elements) => elements
                                .iter()
                                .map(|t| self.value_type_from_type(t))
                                .collect::<Vec<_>>(),
                            _ => {
                                return Err(self.type_error(
                                    line,
                                    format!(
                                    "expected tuple type annotation for tuple(genexpr), got `{}`",
                                    tuple_ty
                                ),
                                ))
                            }
                        };

                        let elt_node = ast_getattr!(arg0, "elt");
                        let generators = ast_get_list!(arg0, "generators");
                        let list_expr = self.lower_comp_impl(&elt_node, &generators, line)?;

                        // Build tuple elements: list_get(list, 0), list_get(list, 1), ...
                        let mut tuple_elements = Vec::new();
                        for (i, elem_ty) in element_types.iter().enumerate() {
                            tuple_elements.push(TirExpr {
                                kind: TirExprKind::ExternalCall {
                                    func: builtin::BuiltinFn::ListGet,
                                    args: vec![
                                        list_expr.clone(),
                                        TirExpr {
                                            kind: TirExprKind::IntLiteral(i as i64),
                                            ty: ValueType::Int,
                                        },
                                    ],
                                },
                                ty: elem_ty.clone(),
                            });
                        }

                        let class_name = self.get_or_create_tuple_class(&element_types);
                        let init_mangled = format!("{}$__init__", class_name);
                        let vty = ValueType::Class(class_name.clone());
                        self.declare(target.clone(), tuple_ty);

                        return Ok(vec![TirStmt::Let {
                            name: target,
                            ty: vty.clone(),
                            value: TirExpr {
                                kind: TirExprKind::Construct {
                                    class_name: class_name.clone(),
                                    init_mangled_name: init_mangled,
                                    args: tuple_elements,
                                },
                                ty: vty,
                            },
                        }]);
                    }
                }
            }
        }

        let saved_empty_list_hint = self.empty_list_hint.clone();
        if let Some(Type::List(inner)) = &annotated_ty {
            self.empty_list_hint = Some(self.value_type_from_type(inner));
        }
        let tir_value = self.lower_expr(&value_node)?;
        self.empty_list_hint = saved_empty_list_hint;

        // Compare at the ValueType level so that Type::Tuple and the
        // corresponding auto-generated tuple class are treated as equal.
        if let Some(ref ann_ty) = annotated_ty {
            let ann_vty = self.value_type_from_type(ann_ty);
            if ann_vty != tir_value.ty {
                return Err(self.type_error(
                    line,
                    format!(
                        "type mismatch: expected `{}`, got `{}`",
                        ann_ty,
                        tir_value.ty.to_type()
                    ),
                ));
            }
        }
        let tir_value_ast_ty = tir_value.ty.to_type();

        let var_type = annotated_ty.unwrap_or(tir_value_ast_ty);
        self.declare(target.clone(), var_type.clone());

        Ok(vec![TirStmt::Let {
            name: target,
            ty: self.value_type_from_type(&var_type),
            value: tir_value,
        }])
    }

    pub(in crate::tir::lower) fn handle_assign(
        &mut self,
        node: &Bound<PyAny>,
        line: usize,
    ) -> Result<Vec<TirStmt>> {
        let targets_list = ast_get_list!(node, "targets");
        if targets_list.len() != 1 {
            return Err(self.syntax_error(line, "multiple assignment targets are not supported"));
        }

        let target_node = targets_list.get_item(0)?;
        match ast_type_name!(target_node).as_str() {
            "Name" => {
                let target = ast_get_string!(target_node, "id");
                let value_node = ast_getattr!(node, "value");
                let saved_empty_list_hint = self.empty_list_hint.clone();
                if let Some(Type::List(inner)) = self.lookup(&target).cloned() {
                    self.empty_list_hint = Some(self.value_type_from_type(&inner));
                }
                let tir_value = self.lower_expr(&value_node)?;
                self.empty_list_hint = saved_empty_list_hint;
                let var_type = tir_value.ty.to_type();
                self.declare(target.clone(), var_type);

                Ok(vec![TirStmt::Let {
                    name: target,
                    ty: tir_value.ty.clone(),
                    value: tir_value,
                }])
            }
            "Attribute" => self.lower_attribute_assign(&target_node, node, line),
            "Subscript" => self.lower_subscript_assign(&target_node, node, line),
            _ => Err(self.syntax_error(
                line,
                "only variable, attribute, or subscript assignments are supported",
            )),
        }
    }

    pub(in crate::tir::lower) fn handle_aug_assign(
        &mut self,
        node: &Bound<PyAny>,
        line: usize,
    ) -> Result<Vec<TirStmt>> {
        let target_node = ast_getattr!(node, "target");
        match ast_type_name!(target_node).as_str() {
            "Name" => {
                let target = ast_get_string!(target_node, "id");

                let target_ty = self.lookup(&target).cloned().ok_or_else(|| {
                    self.name_error(line, format!("undefined variable `{}`", target))
                })?;

                let op = Self::convert_binop(&ast_getattr!(node, "op"))?;
                let value_expr = self.lower_expr(&ast_getattr!(node, "value"))?;

                if op == RawBinOp::Arith(ArithBinOp::Div) && target_ty == Type::Int {
                    return Err(self.type_error(
                        line,
                        format!("`/=` on `int` variable `{}` would change type to `float`; use `//=` for integer division", target),
                    ));
                }

                let target_ref = TirExpr {
                    kind: TirExprKind::Var(target.clone()),
                    ty: self.value_type_from_type(&target_ty),
                };

                // Try to use magic method for augmented assignment
                let result_expr = if let Some(dunder) = aug_op_to_dunder(op) {
                    match &target_ref.ty {
                        ValueType::List(_inner) => {
                            // Try to call the in-place magic method
                            match self.lower_method_call(
                                line,
                                target_ref.clone(),
                                dunder,
                                vec![value_expr.clone()],
                            ) {
                                Ok(CallResult::Expr(e)) => e,
                                Ok(CallResult::VoidStmt(_)) => {
                                    unreachable!("in-place methods should return self")
                                }
                                Err(_) => {
                                    // Fallback to binop if magic method not available
                                    self.resolve_binop(line, op, target_ref, value_expr)?
                                }
                            }
                        }
                        _ => {
                            // For other types, use regular binop
                            self.resolve_binop(line, op, target_ref, value_expr)?
                        }
                    }
                } else {
                    self.resolve_binop(line, op, target_ref, value_expr)?
                };

                self.declare(target.clone(), result_expr.ty.to_type());

                Ok(vec![TirStmt::Let {
                    name: target,
                    ty: result_expr.ty.clone(),
                    value: result_expr,
                }])
            }
            "Attribute" => self.lower_attribute_aug_assign(&target_node, node, line),
            "Subscript" => self.lower_subscript_aug_assign(&target_node, node, line),
            _ => Err(self.syntax_error(
                line,
                "only variable, attribute, or subscript augmented assignments are supported",
            )),
        }
    }

    pub(in crate::tir::lower) fn lower_attribute_assign(
        &mut self,
        target_node: &Bound<PyAny>,
        assign_node: &Bound<PyAny>,
        line: usize,
    ) -> Result<Vec<TirStmt>> {
        let obj_node = ast_getattr!(target_node, "value");
        let field_name = ast_get_string!(target_node, "attr");
        let obj_expr = self.lower_expr(&obj_node)?;

        let class_name = match &obj_expr.ty {
            ValueType::Class(name) => name.clone(),
            other => {
                return Err(self.type_error(
                    line,
                    format!("cannot set attribute on non-class type `{}`", other),
                ))
            }
        };

        let class_info = self.lookup_class(line, &class_name)?;
        let field_index = self.lookup_field_index(line, &class_info, &field_name)?;
        let field = &class_info.fields[field_index];

        // Enforce reference-type field immutability outside __init__
        if field.ty.is_reference_type() {
            let inside_init = self.current_class.as_ref() == Some(&class_name)
                && self
                    .current_function_name
                    .as_ref()
                    .map(|n| n.ends_with(".__init__"))
                    .unwrap_or(false);
            let is_self = matches!(&obj_expr.kind, TirExprKind::Var(name) if name == "self");

            if !(inside_init && is_self) {
                return Err(self.type_error(
                    line,
                    format!(
                        "cannot reassign reference field `{}.{}` of type `{}` outside of __init__",
                        class_name, field_name, field.ty
                    ),
                ));
            }
        }

        let value_node = ast_getattr!(assign_node, "value");

        // Handle empty list/dict literals using the field's type annotation
        let tir_value = if ast_type_name!(value_node) == "List" {
            let elts = ast_get_list!(value_node, "elts");
            if elts.is_empty() {
                let inner_ty = match &field.ty {
                    Type::List(inner) => self.value_type_from_type(inner),
                    _ => {
                        return Err(self.type_error(
                            line,
                            format!(
                                "cannot assign empty list `[]` to field `{}.{}` of type `{}`",
                                class_name, field_name, field.ty
                            ),
                        ))
                    }
                };
                let vty = ValueType::List(Box::new(inner_ty.clone()));
                TirExpr {
                    kind: TirExprKind::ListLiteral {
                        element_type: inner_ty,
                        elements: vec![],
                    },
                    ty: vty,
                }
            } else {
                self.lower_expr(&value_node)?
            }
        } else if ast_type_name!(value_node) == "Dict" {
            let keys = ast_get_list!(value_node, "keys");
            if keys.is_empty() {
                let (key_ty, value_ty) = match &field.ty {
                    Type::Dict(key, value) => (
                        self.value_type_from_type(key),
                        self.value_type_from_type(value),
                    ),
                    _ => {
                        return Err(self.type_error(
                            line,
                            format!(
                                "cannot assign empty dict `{{}}` to field `{}.{}` of type `{}`",
                                class_name, field_name, field.ty
                            ),
                        ))
                    }
                };
                let vty = ValueType::Dict(Box::new(key_ty), Box::new(value_ty));
                TirExpr {
                    kind: TirExprKind::ExternalCall {
                        func: builtin::BuiltinFn::DictEmpty,
                        args: vec![],
                    },
                    ty: vty,
                }
            } else {
                self.lower_expr(&value_node)?
            }
        } else {
            self.lower_expr(&value_node)?
        };

        let field_vty = self.value_type_from_type(&field.ty);
        if tir_value.ty != field_vty {
            return Err(self.type_error(
                line,
                format!(
                    "cannot assign `{}` to field `{}.{}` of type `{}`",
                    tir_value.ty, class_name, field_name, field.ty
                ),
            ));
        }

        Ok(vec![TirStmt::SetField {
            object: obj_expr,
            class_name,
            field_index,
            value: tir_value,
        }])
    }

    pub(in crate::tir::lower) fn lower_subscript_assign(
        &mut self,
        target_node: &Bound<PyAny>,
        assign_node: &Bound<PyAny>,
        line: usize,
    ) -> Result<Vec<TirStmt>> {
        let value_node_target = ast_getattr!(target_node, "value");
        let slice_node = ast_getattr!(target_node, "slice");
        let value_node = ast_getattr!(assign_node, "value");

        let list_expr = self.lower_expr(&value_node_target)?;
        match &list_expr.ty {
            ValueType::List(inner) => {
                let index_expr = self.lower_expr(&slice_node)?;
                if index_expr.ty != ValueType::Int {
                    return Err(self.type_error(
                        line,
                        format!("list index must be `int`, got `{}`", index_expr.ty),
                    ));
                }
                let tir_value = self.lower_expr(&value_node)?;
                let expected_elem_ty = inner.as_ref();
                if &tir_value.ty != expected_elem_ty {
                    return Err(self.type_error(
                        line,
                        format!(
                            "cannot assign `{}` to element of `list[{}]`",
                            tir_value.ty, expected_elem_ty
                        ),
                    ));
                }
                Ok(vec![TirStmt::ListSet {
                    list: list_expr,
                    index: index_expr,
                    value: tir_value,
                }])
            }
            ValueType::Dict(key_ty, value_ty) => {
                let key_expr = self.lower_expr(&slice_node)?;
                if key_expr.ty != **key_ty {
                    return Err(self.type_error(
                        line,
                        format!("dict key index must be `{}`, got `{}`", key_ty, key_expr.ty),
                    ));
                }
                let tir_value = self.lower_expr(&value_node)?;
                if tir_value.ty != **value_ty {
                    return Err(self.type_error(
                        line,
                        format!(
                            "cannot assign `{}` to value of `dict[{}, {}]`",
                            tir_value.ty, key_ty, value_ty
                        ),
                    ));
                }
                self.require_intrinsic_eq_support(line, key_ty);
                let key_eq_tag = self.register_intrinsic_instance(IntrinsicOp::Eq, key_ty);
                Ok(vec![TirStmt::VoidCall {
                    target: CallTarget::Builtin(builtin::BuiltinFn::DictSetByTag),
                    args: vec![
                        list_expr,
                        key_expr,
                        tir_value,
                        TirExpr {
                            kind: TirExprKind::IntLiteral(key_eq_tag),
                            ty: ValueType::Int,
                        },
                    ],
                }])
            }
            ValueType::Class(ref name) if self.is_tuple_class(name) => {
                Err(self.type_error(line, "tuple does not support index assignment".to_string()))
            }
            other => Err(self.type_error(
                line,
                format!("type `{}` does not support index assignment", other),
            )),
        }
    }

    pub(in crate::tir::lower) fn lower_subscript_aug_assign(
        &mut self,
        target_node: &Bound<PyAny>,
        aug_node: &Bound<PyAny>,
        line: usize,
    ) -> Result<Vec<TirStmt>> {
        let value_node = ast_getattr!(target_node, "value");
        let slice_node = ast_getattr!(target_node, "slice");

        let list_expr = self.lower_expr(&value_node)?;
        match &list_expr.ty {
            ValueType::List(inner) => {
                let index_expr = self.lower_expr(&slice_node)?;
                if index_expr.ty != ValueType::Int {
                    return Err(self.type_error(
                        line,
                        format!("list index must be `int`, got `{}`", index_expr.ty),
                    ));
                }
                let elem_ty = inner.as_ref().clone();

                let current_val = TirExpr {
                    kind: TirExprKind::ExternalCall {
                        func: builtin::BuiltinFn::ListGet,
                        args: vec![list_expr.clone(), index_expr.clone()],
                    },
                    ty: elem_ty,
                };

                let op = Self::convert_binop(&ast_getattr!(aug_node, "op"))?;
                let rhs = self.lower_expr(&ast_getattr!(aug_node, "value"))?;
                let binop_expr = self.resolve_binop(line, op, current_val, rhs)?;

                if &binop_expr.ty != inner.as_ref() {
                    return Err(self.type_error(
                        line,
                        format!(
                            "augmented assignment would change list element type from `{}` to `{}`",
                            inner, binop_expr.ty
                        ),
                    ));
                }

                Ok(vec![TirStmt::ListSet {
                    list: list_expr,
                    index: index_expr,
                    value: binop_expr,
                }])
            }
            ValueType::Dict(key_ty, value_ty) => {
                let key_expr = self.lower_expr(&slice_node)?;
                if key_expr.ty != **key_ty {
                    return Err(self.type_error(
                        line,
                        format!("dict key index must be `{}`, got `{}`", key_ty, key_expr.ty),
                    ));
                }
                self.require_intrinsic_eq_support(line, key_ty);
                let key_eq_tag = self.register_intrinsic_instance(IntrinsicOp::Eq, key_ty);

                let current_val = TirExpr {
                    kind: TirExprKind::ExternalCall {
                        func: builtin::BuiltinFn::DictGetByTag,
                        args: vec![
                            list_expr.clone(),
                            key_expr.clone(),
                            TirExpr {
                                kind: TirExprKind::IntLiteral(key_eq_tag),
                                ty: ValueType::Int,
                            },
                        ],
                    },
                    ty: (**value_ty).clone(),
                };

                let op = Self::convert_binop(&ast_getattr!(aug_node, "op"))?;
                let rhs = self.lower_expr(&ast_getattr!(aug_node, "value"))?;
                let binop_expr = self.resolve_binop(line, op, current_val, rhs)?;

                if binop_expr.ty != **value_ty {
                    return Err(self.type_error(
                        line,
                        format!(
                            "augmented assignment would change dict value type from `{}` to `{}`",
                            value_ty, binop_expr.ty
                        ),
                    ));
                }

                Ok(vec![TirStmt::VoidCall {
                    target: CallTarget::Builtin(builtin::BuiltinFn::DictSetByTag),
                    args: vec![
                        list_expr,
                        key_expr,
                        binop_expr,
                        TirExpr {
                            kind: TirExprKind::IntLiteral(key_eq_tag),
                            ty: ValueType::Int,
                        },
                    ],
                }])
            }
            ValueType::Class(ref name) if self.is_tuple_class(name) => {
                Err(self.type_error(line, "tuple does not support index assignment".to_string()))
            }
            other => Err(self.type_error(
                line,
                format!("type `{}` does not support index assignment", other),
            )),
        }
    }

    pub(in crate::tir::lower) fn lower_attribute_aug_assign(
        &mut self,
        target_node: &Bound<PyAny>,
        aug_node: &Bound<PyAny>,
        line: usize,
    ) -> Result<Vec<TirStmt>> {
        let obj_node = ast_getattr!(target_node, "value");
        let field_name = ast_get_string!(target_node, "attr");
        let obj_expr = self.lower_expr(&obj_node)?;

        let class_name = match &obj_expr.ty {
            ValueType::Class(name) => name.clone(),
            other => {
                return Err(self.type_error(
                    line,
                    format!("cannot set attribute on non-class type `{}`", other),
                ))
            }
        };

        let class_info = self.lookup_class(line, &class_name)?;
        let field_index = self.lookup_field_index(line, &class_info, &field_name)?;
        let field = &class_info.fields[field_index];
        let field_vty = self.value_type_from_type(&field.ty);

        // Read current field value
        let current_val = TirExpr {
            kind: TirExprKind::GetField {
                object: Box::new(obj_expr.clone()),
                class_name: class_name.clone(),
                field_index,
            },
            ty: field_vty.clone(),
        };

        let op = Self::convert_binop(&ast_getattr!(aug_node, "op"))?;
        let rhs = self.lower_expr(&ast_getattr!(aug_node, "value"))?;

        let binop_expr = self.resolve_binop(line, op, current_val, rhs)?;

        if binop_expr.ty != field_vty {
            return Err(self.type_error(
                line,
                format!(
                    "augmented assignment would change field `{}.{}` type from `{}` to `{}`",
                    class_name, field_name, field.ty, binop_expr.ty
                ),
            ));
        }

        Ok(vec![TirStmt::SetField {
            object: obj_expr,
            class_name,
            field_index,
            value: binop_expr,
        }])
    }
}
