use anyhow::Result;
use pyo3::prelude::*;
use pyo3::types::PyList;
use std::collections::HashMap;

use crate::ast::{ClassField, ClassInfo, ClassMethod, Type};
use crate::tir::{FunctionParam, TirFunction, ValueType};
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
                self.declare(raw_name, Type::Class(qualified.clone()));
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
        let mut index = 0;

        for item in body_list.iter() {
            match ast_type_name!(item).as_str() {
                "AnnAssign" => {
                    let target_node = ast_getattr!(item, "target");
                    let field_name = ast_get_string!(target_node, "id");
                    let annotation = ast_getattr!(item, "annotation");
                    let field_ty = self.convert_type_annotation(&annotation)?;

                    fields.push(ClassField {
                        name: field_name.clone(),
                        ty: field_ty,
                        index,
                    });
                    field_map.insert(field_name, index);
                    index += 1;
                }
                "FunctionDef" => {
                    let method_name = ast_get_string!(item, "name");
                    let method_line = Self::get_line(&item);
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
                            format!("first parameter of method `{}` must be `self`", method_name),
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
                _ => {
                    return Err(self.syntax_error(
                        Self::get_line(&item),
                        "only field declarations, method definitions, and nested classes are allowed in class body",
                    ));
                }
            }
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

        self.current_class = None;
        Ok((all_classes, functions))
    }

    fn lower_method(&mut self, node: &Bound<PyAny>, class_info: &ClassInfo) -> Result<TirFunction> {
        let method_name = ast_get_string!(node, "name");
        let method_info = &class_info.methods[&method_name];
        let mangled_name = method_info.mangled_name.clone();

        let args_node = ast_getattr!(node, "args");
        let py_args = ast_get_list!(&args_node, "args");

        let mut params = Vec::new();
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
            params.push(FunctionParam::new(param_name, Self::to_value_type(&ty)));
        }

        let return_type = Self::to_opt_value_type(&method_info.return_type);

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
                continue;
            }
            tir_body.extend(self.lower_stmt(&stmt_node)?);
        }

        self.pop_scope();
        self.current_return_type = None;
        self.current_function_name = None;

        Ok(TirFunction {
            name: mangled_name,
            params,
            return_type,
            body: tir_body,
        })
    }
}
