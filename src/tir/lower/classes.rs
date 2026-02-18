use anyhow::Result;
use pyo3::prelude::*;
use pyo3::types::PyList;
use std::collections::{HashMap, HashSet};

use crate::ast::{ClassField, ClassInfo, ClassMethod, Type};
use crate::tir::{FunctionParam, TirExpr, TirExprKind, TirFunction, TirStmt, ValueType};
use crate::{ast_get_list, ast_get_string, ast_getattr, ast_type_name};

use super::Lowering;

impl Lowering {
    pub(super) fn discover_classes(
        &mut self,
        body_list: &Bound<PyList>,
        parent_prefix: &str,
    ) -> Result<()> {
        for node in body_list.iter() {
            if ast_type_name!(node) == "ClassDef" {
                let raw_name = ast_get_string!(node, "name");
                let qualified = format!("{}${}", parent_prefix, raw_name);
                self.class_registry.insert(
                    qualified.clone(),
                    ClassInfo {
                        name: qualified.clone(),
                        fields: Vec::new(),
                        methods: HashMap::new(),
                        field_map: HashMap::new(),
                    },
                );
                self.declare_non_capture_symbol(raw_name, Type::Class(qualified.clone()));
                // Recurse into nested classes
                let nested_body = ast_get_list!(node, "body");
                self.discover_classes(&nested_body, &qualified)?;
            }
        }
        Ok(())
    }

    pub(super) fn collect_classes(
        &mut self,
        body_list: &Bound<PyList>,
        parent_prefix: &str,
    ) -> Result<()> {
        for node in body_list.iter() {
            if ast_type_name!(node) == "ClassDef" {
                let raw_name = ast_get_string!(node, "name");
                let qualified = format!("{}${}", parent_prefix, raw_name);
                self.collect_class_definition(&node, &qualified)?;
                // Recurse into nested classes
                let nested_body = ast_get_list!(node, "body");
                self.collect_classes(&nested_body, &qualified)?;
            }
        }
        Ok(())
    }

    pub(super) fn collect_class_definition(
        &mut self,
        node: &Bound<PyAny>,
        qualified_name: &str,
    ) -> Result<()> {
        let line = Self::get_line(node);

        let bases_list = ast_get_list!(node, "bases");
        if !bases_list.is_empty() {
            return Err(self.syntax_error(line, "class inheritance is not supported"));
        }

        let body_list = ast_get_list!(node, "body");
        let mut fields = Vec::new();
        let mut field_map = HashMap::new();
        let mut methods = HashMap::new();
        let mut const_names = HashSet::new();
        let mut index = 0;
        self.push_scope();
        let result: Result<()> = (|| {
            for item in body_list.iter() {
                match ast_type_name!(item).as_str() {
                    "AnnAssign" => {
                        let target_node = ast_getattr!(item, "target");
                        if ast_type_name!(target_node) != "Name" {
                            return Err(self.syntax_error(
                                Self::get_line(&item),
                                "only simple names are supported",
                            ));
                        }
                        let value_node = ast_getattr!(item, "value");
                        if value_node.is_none() {
                            let field_name = ast_get_string!(target_node, "id");
                            if const_names.contains(&field_name)
                                || methods.contains_key(&field_name)
                            {
                                return Err(self.syntax_error(
                                    Self::get_line(&item),
                                    format!("duplicate symbol `{}` in class body", field_name),
                                ));
                            }
                            let annotation = ast_getattr!(item, "annotation");
                            let field_ty = self.convert_type_annotation(&annotation)?;

                            fields.push(ClassField {
                                name: field_name.clone(),
                                ty: field_ty,
                                index,
                            });
                            field_map.insert(field_name, index);
                            index += 1;
                        } else {
                            self.collect_class_constant_decl(
                                &item,
                                qualified_name,
                                &field_map,
                                &methods,
                                &mut const_names,
                            )?;
                        }
                    }
                    "Assign" => {
                        self.collect_class_constant_decl(
                            &item,
                            qualified_name,
                            &field_map,
                            &methods,
                            &mut const_names,
                        )?;
                    }
                    "FunctionDef" => {
                        let method_name = ast_get_string!(item, "name");
                        let method_line = Self::get_line(&item);
                        if field_map.contains_key(&method_name)
                            || const_names.contains(&method_name)
                        {
                            return Err(self.syntax_error(
                                method_line,
                                format!("duplicate symbol `{}` in class body", method_name),
                            ));
                        }
                        let args_node = ast_getattr!(item, "args");
                        let py_args = ast_get_list!(&args_node, "args");

                        if py_args.is_empty() {
                            return Err(self.syntax_error(
                                method_line,
                                format!(
                                    "method `{}` must have `self` as first parameter",
                                    method_name
                                ),
                            ));
                        }
                        let first_arg = py_args.get_item(0)?;
                        let first_name = ast_get_string!(first_arg, "arg");
                        if first_name != "self" {
                            return Err(self.syntax_error(
                                method_line,
                                format!(
                                    "first parameter of method `{}` must be `self`",
                                    method_name
                                ),
                            ));
                        }

                        let mut param_types = Vec::new();
                        for i in 1..py_args.len() {
                            let arg = py_args.get_item(i)?;
                            let p_name = ast_get_string!(arg, "arg");
                            let annotation = ast_getattr!(arg, "annotation");
                            if annotation.is_none() {
                                return Err(self.syntax_error(
                                    method_line,
                                    format!("parameter `{}` requires a type annotation", p_name),
                                ));
                            }
                            param_types.push(self.convert_type_annotation(&annotation)?);
                        }

                        let return_type = self.convert_return_type(&item)?;
                        let mangled_name = format!("{}${}", qualified_name, method_name);

                        if method_name == "__init__" && return_type != Type::Unit {
                            return Err(self.type_error(
                                method_line,
                                format!("__init__ must return None, got `{}`", return_type),
                            ));
                        }

                        methods.insert(
                            method_name.clone(),
                            ClassMethod {
                                name: method_name,
                                params: param_types,
                                return_type,
                                mangled_name,
                            },
                        );
                    }
                    "Pass" | "ClassDef" => {}
                    "Expr" => {
                        // Allow expression statements only if they are docstrings or ellipsis
                        let value_node = ast_getattr!(item, "value");
                        if ast_type_name!(value_node) == "Constant" {
                            let value = ast_getattr!(value_node, "value");
                            let is_ellipsis = value
                                .get_type()
                                .name()
                                .is_ok_and(|type_name| type_name == "ellipsis");

                            // Allow ellipsis and string literals (docstrings) in class body
                            if is_ellipsis || value.is_instance_of::<pyo3::types::PyString>() {
                                // These are allowed in class body but don't generate code
                                continue;
                            }
                        }

                        // Other expression statements are not allowed in class body
                        return Err(self.syntax_error(
                            Self::get_line(&item),
                            "only field declarations, method definitions, and nested classes are allowed in class body",
                        ));
                    }
                    _ => {
                        return Err(self.syntax_error(
                            Self::get_line(&item),
                            "only field declarations, method definitions, and nested classes are allowed in class body",
                        ));
                    }
                }
            }

            // Auto-generate `new()` factory method if `__init__` exists.
            if let Some(init_method) = methods.get("__init__") {
                let new_mangled = format!("{}$new", qualified_name);
                methods.insert(
                    "new".to_string(),
                    ClassMethod {
                        name: "new".to_string(),
                        params: init_method.params.clone(),
                        return_type: Type::Class(qualified_name.to_string()),
                        mangled_name: new_mangled,
                    },
                );
            }

            let class_info = ClassInfo {
                name: qualified_name.to_string(),
                fields,
                methods,
                field_map,
            };

            self.class_registry
                .insert(qualified_name.to_string(), class_info);
            Ok(())
        })();
        self.pop_scope();
        result
    }

    fn collect_class_constant_decl(
        &mut self,
        node: &Bound<PyAny>,
        qualified_name: &str,
        field_map: &HashMap<String, usize>,
        methods: &HashMap<String, ClassMethod>,
        const_names: &mut HashSet<String>,
    ) -> Result<()> {
        let line = Self::get_line(node);
        let node_type = ast_type_name!(node);

        let (name, annotated_ty, value_node) = match node_type.as_str() {
            "AnnAssign" => {
                let target_node = ast_getattr!(node, "target");
                if ast_type_name!(target_node) != "Name" {
                    return Err(self.syntax_error(line, "only simple names are supported"));
                }
                let name = ast_get_string!(target_node, "id");
                let value_node = ast_getattr!(node, "value");
                if value_node.is_none() {
                    return Err(
                        self.syntax_error(line, "class constant declaration requires a value")
                    );
                }
                let annotation = ast_getattr!(node, "annotation");
                let annotated_ty = if annotation.is_none() {
                    None
                } else {
                    Some(self.convert_type_annotation(&annotation)?)
                };
                (name, annotated_ty, value_node)
            }
            "Assign" => {
                let targets_list = ast_get_list!(node, "targets");
                if targets_list.len() != 1 {
                    return Err(
                        self.syntax_error(line, "multiple assignment targets are not supported")
                    );
                }
                let target_node = targets_list.get_item(0)?;
                if ast_type_name!(target_node) != "Name" {
                    return Err(self.syntax_error(line, "only simple names are supported"));
                }
                let name = ast_get_string!(target_node, "id");
                let value_node = ast_getattr!(node, "value");
                (name, None, value_node)
            }
            _ => unreachable!("collect_class_constant_decl only handles Assign/AnnAssign"),
        };

        if field_map.contains_key(&name)
            || methods.contains_key(&name)
            || const_names.contains(&name)
        {
            return Err(
                self.syntax_error(line, format!("duplicate symbol `{}` in class body", name))
            );
        }

        let pre_len = self.pre_stmts.len();
        let const_expr = self.lower_expr(&value_node)?;
        if self.pre_stmts.len() != pre_len {
            self.pre_stmts.truncate(pre_len);
            return Err(
                self.syntax_error(line, "class constant value must be a constant expression")
            );
        }

        self.ensure_supported_default_expr(line, &const_expr)
            .map_err(|_| {
                self.syntax_error(line, "class constant value must be a constant expression")
            })?;

        if let Some(ref ann_ty) = annotated_ty {
            let ann_vty = self.value_type_from_type(ann_ty);
            if ann_vty != const_expr.ty {
                return Err(self.type_error(
                    line,
                    format!(
                        "type mismatch: expected `{}`, got `{}`",
                        ann_ty,
                        const_expr.ty.to_type()
                    ),
                ));
            }
        }

        if !Self::is_supported_global_constant_type(&const_expr.ty) {
            return Err(self.type_error(
                line,
                format!(
                    "class constant `{}` must have type int, float, bool, str, bytes, or list[...] of those; got `{}`",
                    name, const_expr.ty
                ),
            ));
        }

        let qualified_symbol = Self::mangle_class_symbol(qualified_name, &name);
        self.class_constants
            .insert(qualified_symbol, const_expr.clone());
        const_names.insert(name.clone());
        self.declare(name, const_expr.ty.to_type());
        Ok(())
    }

    pub(super) fn lower_class_def(
        &mut self,
        node: &Bound<PyAny>,
        qualified_name: &str,
    ) -> Result<(Vec<ClassInfo>, Vec<TirFunction>)> {
        let class_info = self.class_registry.get(qualified_name).unwrap().clone();
        let body_list = ast_get_list!(node, "body");

        let mut functions = Vec::new();
        let mut all_classes = vec![class_info.clone()];
        self.current_class = Some(qualified_name.to_string());

        for item in body_list.iter() {
            match ast_type_name!(item).as_str() {
                "FunctionDef" => {
                    let func = self.lower_method(&item, &class_info)?;
                    functions.push(func);
                }
                "ClassDef" => {
                    let raw_name = ast_get_string!(item, "name");
                    let nested_qualified = format!("{}${}", qualified_name, raw_name);
                    let (nested_classes, nested_fns) =
                        self.lower_class_def(&item, &nested_qualified)?;
                    all_classes.extend(nested_classes);
                    functions.extend(nested_fns);
                }
                _ => {}
            }
        }

        // Generate `new()` factory TirFunction if __init__ exists.
        if let Some(init_method) = class_info.methods.get("__init__") {
            if let Some(new_method) = class_info.methods.get("new") {
                let new_fn = self.generate_new_factory(
                    qualified_name,
                    &new_method.mangled_name,
                    &init_method.mangled_name,
                    &init_method.params,
                );
                functions.push(new_fn);
            }
        }

        self.current_class = None;
        Ok((all_classes, functions))
    }

    /// Generate a `new()` factory function that constructs and returns an instance.
    fn generate_new_factory(
        &mut self,
        qualified_name: &str,
        new_mangled: &str,
        init_mangled: &str,
        init_params: &[Type],
    ) -> TirFunction {
        let params: Vec<FunctionParam> = init_params
            .iter()
            .enumerate()
            .map(|(i, ty)| FunctionParam::new(format!("__arg{}", i), self.value_type_from_type(ty)))
            .collect();

        let arg_exprs: Vec<TirExpr> = params
            .iter()
            .map(|p| TirExpr {
                kind: TirExprKind::Var(p.name.clone()),
                ty: p.ty.clone(),
            })
            .collect();

        let construct = TirExpr {
            kind: TirExprKind::Construct {
                class_name: qualified_name.to_string(),
                init_mangled_name: init_mangled.to_string(),
                args: arg_exprs,
            },
            ty: ValueType::Class(qualified_name.to_string()),
        };

        TirFunction {
            name: new_mangled.to_string(),
            params,
            return_type: Some(ValueType::Class(qualified_name.to_string())),
            body: vec![TirStmt::Return(Some(construct))],
        }
    }

    fn lower_method(&mut self, node: &Bound<PyAny>, class_info: &ClassInfo) -> Result<TirFunction> {
        let method_name = ast_get_string!(node, "name");
        let method_info = &class_info.methods[&method_name];
        let mangled_name = method_info.mangled_name.clone();

        let args_node = ast_getattr!(node, "args");
        let py_args = ast_get_list!(&args_node, "args");
        let all_defaults =
            self.lower_defaults_for_params(&args_node, Self::get_line(node), &method_name)?;

        let mut params = Vec::new();
        let mut param_names = Vec::new();
        // self parameter
        params.push(FunctionParam::new(
            "self".to_string(),
            ValueType::Class(class_info.name.clone()),
        ));

        // remaining parameters
        for i in 1..py_args.len() {
            let arg = py_args.get_item(i)?;
            let param_name = ast_get_string!(arg, "arg");
            let annotation = ast_getattr!(arg, "annotation");
            let ty = self.convert_type_annotation(&annotation)?;
            param_names.push(param_name.clone());
            params.push(FunctionParam::new(
                param_name,
                self.value_type_from_type(&ty),
            ));
        }
        let default_values = all_defaults.into_iter().skip(1).collect();

        let return_type = self.opt_value_type_from_type(&method_info.return_type);

        self.push_scope();
        for param in &params {
            self.declare(param.name.clone(), param.ty.to_type());
        }
        self.current_return_type = Some(method_info.return_type.clone());
        self.current_function_name = Some(format!("{}.{}", class_info.name, method_name));

        let body_list = ast_get_list!(node, "body");
        let mut tir_body = Vec::new();
        for stmt_node in body_list.iter() {
            let node_type = ast_type_name!(stmt_node);
            if node_type == "Import" || node_type == "ImportFrom" {
                return Err(self.syntax_error(
                    Self::get_line(&stmt_node),
                    "imports are only allowed at module top-level",
                ));
            }
            tir_body.extend(self.lower_stmt(&stmt_node)?);
        }

        self.pop_scope();
        self.current_return_type = None;
        self.current_function_name = None;
        self.register_function_signature(mangled_name.clone(), param_names, default_values);

        Ok(TirFunction {
            name: mangled_name,
            params,
            return_type,
            body: tir_body,
        })
    }
}
