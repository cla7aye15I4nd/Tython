use anyhow::{bail, Result};
use pyo3::prelude::*;
use pyo3::types::{PyList, PyModule};
use std::collections::HashMap;
use std::path::Path;

use super::builtin;
use super::{
    ArithBinOp, BitwiseBinOp, CallResult, CallTarget, CastKind, CmpOp, FunctionParam, LogicalOp,
    TirExpr, TirExprKind, TirFunction, TirModule, TirStmt, TypedBinOp, UnaryOpKind, ValueType,
};
use crate::ast::{ClassField, ClassInfo, ClassMethod, Type};
use crate::errors::{ErrorCategory, TythonError};
use crate::{ast_get_list, ast_get_string, ast_getattr, ast_type_name};

pub struct Lowering {
    symbol_table: HashMap<String, Type>,

    current_module_name: String,
    current_return_type: Option<Type>,
    scopes: Vec<HashMap<String, Type>>,
    current_file: String,
    source_lines: Vec<String>,
    current_function_name: Option<String>,

    class_registry: HashMap<String, ClassInfo>,
    current_class: Option<String>,

    // Accumulated from classes defined inside function/method bodies
    deferred_functions: Vec<TirFunction>,
    deferred_classes: Vec<ClassInfo>,
}

impl Default for Lowering {
    fn default() -> Self {
        Self::new()
    }
}

impl Lowering {
    pub fn new() -> Self {
        Self {
            symbol_table: HashMap::new(),
            current_module_name: String::new(),
            current_return_type: None,
            scopes: Vec::new(),
            current_file: String::new(),
            source_lines: Vec::new(),
            current_function_name: None,
            class_registry: HashMap::new(),
            current_class: None,
            deferred_functions: Vec::new(),
            deferred_classes: Vec::new(),
        }
    }

    // ── error helpers ──────────────────────────────────────────────────

    fn make_error(&self, category: ErrorCategory, line: usize, message: String) -> anyhow::Error {
        TythonError {
            category,
            message,
            file: self.current_file.clone(),
            line,
            source_line: self.source_lines.get(line.wrapping_sub(1)).cloned(),
            function_name: self.current_function_name.clone(),
        }
        .into()
    }

    fn type_error(&self, line: usize, msg: impl Into<String>) -> anyhow::Error {
        self.make_error(ErrorCategory::TypeError, line, msg.into())
    }

    fn name_error(&self, line: usize, msg: impl Into<String>) -> anyhow::Error {
        self.make_error(ErrorCategory::NameError, line, msg.into())
    }

    fn syntax_error(&self, line: usize, msg: impl Into<String>) -> anyhow::Error {
        self.make_error(ErrorCategory::SyntaxError, line, msg.into())
    }

    fn value_error(&self, line: usize, msg: impl Into<String>) -> anyhow::Error {
        self.make_error(ErrorCategory::ValueError, line, msg.into())
    }

    fn attribute_error(&self, line: usize, msg: impl Into<String>) -> anyhow::Error {
        self.make_error(ErrorCategory::AttributeError, line, msg.into())
    }

    // ── scope helpers ──────────────────────────────────────────────────

    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare(&mut self, name: String, ty: Type) {
        self.scopes.last_mut().unwrap().insert(name, ty);
    }

    fn lookup(&self, name: &str) -> Option<&Type> {
        for scope in self.scopes.iter().rev() {
            if let Some(ty) = scope.get(name) {
                return Some(ty);
            }
        }
        None
    }

    // ── helpers: Type → ValueType conversion ────────────────────────

    fn to_value_type(ty: &Type) -> ValueType {
        ValueType::from_type(ty).expect("ICE: expected a value type")
    }

    fn to_opt_value_type(ty: &Type) -> Option<ValueType> {
        match ty {
            Type::Unit => None,
            other => Some(Self::to_value_type(other)),
        }
    }

    // ── module / function lowering ─────────────────────────────────────

    pub fn lower_module(
        &mut self,
        canonical_path: &Path,
        module_path: &str,
        imports: &HashMap<String, Type>,
    ) -> Result<TirModule> {
        self.scopes.clear();
        self.current_return_type = None;
        self.current_module_name = module_path.to_string();
        self.current_file = canonical_path.display().to_string();
        self.current_function_name = None;

        self.push_scope();

        for (local_name, ty) in imports {
            if let Type::Module(mangled) = ty {
                self.declare(local_name.clone(), Type::Module(mangled.clone()));
            }
        }

        Python::attach(|py| -> Result<_> {
            let source = std::fs::read_to_string(canonical_path)?;
            self.source_lines = source.lines().map(String::from).collect();
            let ast_module = PyModule::import(py, "ast")?;
            let py_ast = ast_module.call_method1("parse", (source.as_str(),))?;

            self.lower_py_ast(&py_ast)
        })
    }

    fn lower_py_ast(&mut self, py_ast: &Bound<PyAny>) -> Result<TirModule> {
        let body_list = ast_get_list!(py_ast, "body");

        // Pass 1: Collect class definitions (two sub-phases for cross-referencing)
        // Phase 1a: Register all class names (with module-qualified names), recursing into nested
        self.discover_classes(&body_list, &self.current_module_name.clone())?;
        // Phase 1b: Fill in fields and methods, recursing into nested
        self.collect_classes(&body_list, &self.current_module_name.clone())?;

        // Pass 2: Collect function signatures
        for node in body_list.iter() {
            if ast_type_name!(node) == "FunctionDef" {
                self.collect_function_signature(&node)?;
            }
        }

        // Pass 3: Lower everything
        let mut functions = HashMap::new();
        let mut module_level_stmts = Vec::new();
        let mut classes = HashMap::new();

        for node in body_list.iter() {
            match ast_type_name!(node).as_str() {
                "ClassDef" => {
                    let raw_name = ast_get_string!(node, "name");
                    let qualified = format!("{}${}", self.current_module_name, raw_name);
                    let (class_infos, class_functions) = self.lower_class_def(&node, &qualified)?;
                    for func in class_functions {
                        functions.insert(func.name.clone(), func);
                    }
                    for ci in class_infos {
                        classes.insert(ci.name.clone(), ci);
                    }
                }
                "FunctionDef" => {
                    let tir_func = self.lower_function(&node)?;
                    functions.insert(tir_func.name.clone(), tir_func);
                }
                "Import" | "ImportFrom" => {}
                _ => {
                    module_level_stmts.extend(self.lower_stmt(&node)?);
                }
            }
        }

        if !module_level_stmts.is_empty() {
            let main_func = self.build_synthetic_main(module_level_stmts);
            functions.insert(main_func.name.clone(), main_func);
        }

        // Drain classes/functions discovered inside function/method bodies
        for ci in self.deferred_classes.drain(..) {
            classes.insert(ci.name.clone(), ci);
        }
        for func in self.deferred_functions.drain(..) {
            functions.insert(func.name.clone(), func);
        }

        for func in functions.values() {
            let func_type = Type::Function {
                params: func.params.iter().map(|p| p.ty.to_type()).collect(),
                return_type: Box::new(
                    func.return_type
                        .as_ref()
                        .map(|vt| vt.to_type())
                        .unwrap_or(Type::Unit),
                ),
            };
            self.symbol_table.insert(func.name.clone(), func_type);
        }

        Ok(TirModule { functions, classes })
    }

    // ── class lowering ────────────────────────────────────────────────

    fn discover_classes(&mut self, body_list: &Bound<PyList>, parent_prefix: &str) -> Result<()> {
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

    fn collect_classes(&mut self, body_list: &Bound<PyList>, parent_prefix: &str) -> Result<()> {
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

    fn collect_class_definition(
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

    fn lower_class_def(
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

    // ── function lowering ─────────────────────────────────────────────

    fn collect_function_signature(&mut self, node: &Bound<PyAny>) -> Result<()> {
        let name = ast_get_string!(node, "name");
        let line = Self::get_line(node);

        let args_node = ast_getattr!(node, "args");
        let py_args = ast_get_list!(&args_node, "args");

        let mut param_types = Vec::new();
        for arg in py_args.iter() {
            let param_name = ast_get_string!(arg, "arg");
            let annotation = ast_getattr!(arg, "annotation");
            if annotation.is_none() {
                return Err(self.syntax_error(
                    line,
                    format!(
                        "parameter `{}` in function `{}` requires a type annotation",
                        param_name, name
                    ),
                ));
            }
            param_types.push(self.convert_type_annotation(&annotation)?);
        }

        let return_type = self.convert_return_type(node)?;
        let func_type = Type::Function {
            params: param_types,
            return_type: Box::new(return_type),
        };

        self.declare(name, func_type);
        Ok(())
    }

    fn lower_function(&mut self, node: &Bound<PyAny>) -> Result<TirFunction> {
        let name = ast_get_string!(node, "name");
        let mangled_name = self.mangle_name(&name);

        let args_node = ast_getattr!(node, "args");
        let py_args = ast_get_list!(&args_node, "args");
        let mut params = Vec::new();
        for arg in py_args.iter() {
            let param_name = ast_get_string!(arg, "arg");
            let annotation = ast_getattr!(arg, "annotation");
            let ty = self.convert_type_annotation(&annotation)?;
            let vty = Self::to_value_type(&ty);
            params.push(FunctionParam::new(param_name, vty));
        }

        let return_type_ast = self.convert_return_type(node)?;
        let return_type = Self::to_opt_value_type(&return_type_ast);

        self.push_scope();
        for param in &params {
            self.declare(param.name.clone(), param.ty.to_type());
        }
        self.current_return_type = Some(return_type_ast);
        self.current_function_name = Some(name.clone());

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

    fn build_synthetic_main(&self, mut stmts: Vec<TirStmt>) -> TirFunction {
        stmts.push(TirStmt::Return(Some(TirExpr {
            kind: TirExprKind::IntLiteral(0),
            ty: ValueType::Int,
        })));

        TirFunction {
            name: self.mangle_name("$main$"),
            params: Vec::new(),
            return_type: Some(ValueType::Int),
            body: stmts,
        }
    }

    // ── statement lowering ─────────────────────────────────────────────

    fn lower_stmt(&mut self, node: &Bound<PyAny>) -> Result<Vec<TirStmt>> {
        let node_type = ast_type_name!(node);
        let line = Self::get_line(node);

        match node_type.as_str() {
            "FunctionDef" => {
                Err(self.syntax_error(line, "nested function definitions are not supported"))
            }

            "ClassDef" => {
                let raw_name = ast_get_string!(node, "name");
                let fn_name = self.current_function_name.as_deref().unwrap_or("_");
                let qualified = format!("{}${}${}", self.current_module_name, fn_name, raw_name);

                // Register the class (Phase 1a-equivalent)
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

                // Discover nested classes inside this class
                let body = ast_get_list!(node, "body");
                self.discover_classes(&body, &qualified)?;

                // Collect fields and methods (Phase 1b-equivalent)
                self.collect_class_definition(node, &qualified)?;
                self.collect_classes(&body, &qualified)?;

                // Lower the class definition — results go into deferred state
                let (class_infos, methods) = self.lower_class_def(node, &qualified)?;
                self.deferred_classes.extend(class_infos);
                self.deferred_functions.extend(methods);

                Ok(vec![])
            }

            "AnnAssign" => {
                let target_node = ast_getattr!(node, "target");
                if ast_type_name!(target_node) != "Name" {
                    return Err(
                        self.syntax_error(line, "only simple variable assignments are supported")
                    );
                }
                let target = ast_get_string!(target_node, "id");

                let annotation = ast_getattr!(node, "annotation");
                let annotated_ty = (!annotation.is_none())
                    .then(|| self.convert_type_annotation(&annotation))
                    .transpose()?;

                let value_node = ast_getattr!(node, "value");
                let tir_value = self.lower_expr(&value_node)?;

                let tir_value_ast_ty = tir_value.ty.to_type();
                if let Some(ref ann_ty) = annotated_ty {
                    if ann_ty != &tir_value_ast_ty {
                        return Err(self.type_error(
                            line,
                            format!(
                                "type mismatch: expected `{}`, got `{}`",
                                ann_ty, tir_value_ast_ty
                            ),
                        ));
                    }
                }

                let var_type = annotated_ty.unwrap_or(tir_value_ast_ty);
                self.declare(target.clone(), var_type.clone());

                Ok(vec![TirStmt::Let {
                    name: target,
                    ty: Self::to_value_type(&var_type),
                    value: tir_value,
                }])
            }

            "Assign" => {
                let targets_list = ast_get_list!(node, "targets");
                if targets_list.len() != 1 {
                    return Err(
                        self.syntax_error(line, "multiple assignment targets are not supported")
                    );
                }

                let target_node = targets_list.get_item(0)?;
                match ast_type_name!(target_node).as_str() {
                    "Name" => {
                        let target = ast_get_string!(target_node, "id");
                        let value_node = ast_getattr!(node, "value");
                        let tir_value = self.lower_expr(&value_node)?;
                        let var_type = tir_value.ty.to_type();
                        self.declare(target.clone(), var_type);

                        Ok(vec![TirStmt::Let {
                            name: target,
                            ty: tir_value.ty.clone(),
                            value: tir_value,
                        }])
                    }
                    "Attribute" => self.lower_attribute_assign(&target_node, node, line),
                    _ => Err(self.syntax_error(
                        line,
                        "only variable or attribute assignments are supported",
                    )),
                }
            }

            "AugAssign" => {
                let target_node = ast_getattr!(node, "target");
                match ast_type_name!(target_node).as_str() {
                    "Name" => {
                        let target = ast_get_string!(target_node, "id");

                        let target_ty = self.lookup(&target).cloned().ok_or_else(|| {
                            self.name_error(line, format!("undefined variable `{}`", target))
                        })?;

                        let op = Self::convert_binop(&ast_getattr!(node, "op"))?;
                        let value_expr = self.lower_expr(&ast_getattr!(node, "value"))?;

                        if op == TypedBinOp::Arith(ArithBinOp::Div) && target_ty == Type::Int {
                            return Err(self.type_error(
                                line,
                                format!("`/=` on `int` variable `{}` would change type to `float`; use `//=` for integer division", target),
                            ));
                        }

                        let target_ref = TirExpr {
                            kind: TirExprKind::Var(target.clone()),
                            ty: Self::to_value_type(&target_ty),
                        };

                        let (final_left, final_right, result_ty) =
                            self.resolve_binop_types(line, op, target_ref, value_expr)?;

                        let result_vty = Self::to_value_type(&result_ty);
                        let binop_expr = TirExpr {
                            kind: TirExprKind::BinOp {
                                op,
                                left: Box::new(final_left),
                                right: Box::new(final_right),
                            },
                            ty: result_vty.clone(),
                        };

                        self.declare(target.clone(), result_ty);

                        Ok(vec![TirStmt::Let {
                            name: target,
                            ty: result_vty,
                            value: binop_expr,
                        }])
                    }
                    "Attribute" => self.lower_attribute_aug_assign(&target_node, node, line),
                    _ => Err(self.syntax_error(
                        line,
                        "only variable or attribute augmented assignments are supported",
                    )),
                }
            }

            "Return" => {
                let value_node = ast_getattr!(node, "value");
                if value_node.is_none() {
                    if let Some(ref expected) = self.current_return_type {
                        if *expected != Type::Unit {
                            return Err(self.type_error(
                                line,
                                format!(
                                    "return without value, but function expects `{}`",
                                    expected
                                ),
                            ));
                        }
                    }
                    Ok(vec![TirStmt::Return(None)])
                } else {
                    let tir_expr = self.lower_expr(&value_node)?;
                    if let Some(ref expected) = self.current_return_type {
                        if *expected != tir_expr.ty.to_type() {
                            return Err(self.type_error(
                                line,
                                format!(
                                    "return type mismatch: expected `{}`, got `{}`",
                                    expected, tir_expr.ty
                                ),
                            ));
                        }
                    }
                    Ok(vec![TirStmt::Return(Some(tir_expr))])
                }
            }

            "Expr" => {
                let value_node = ast_getattr!(node, "value");

                if ast_type_name!(value_node) == "Call" {
                    let func_node = ast_getattr!(value_node, "func");
                    if ast_type_name!(func_node) == "Name"
                        && ast_get_string!(func_node, "id") == "print"
                    {
                        return self.lower_print_stmt(&value_node);
                    }

                    let call_result = self.lower_call(&value_node, line)?;
                    return match call_result {
                        CallResult::Expr(expr) => Ok(vec![TirStmt::Expr(expr)]),
                        CallResult::VoidStmt(stmt) => Ok(vec![stmt]),
                    };
                }

                Ok(vec![TirStmt::Expr(self.lower_expr(&value_node)?)])
            }

            "If" => {
                let test_node = ast_getattr!(node, "test");
                let condition = self.lower_expr(&test_node)?;

                let body_list = ast_get_list!(node, "body");
                self.push_scope();
                let mut then_body = Vec::new();
                for stmt_node in body_list.iter() {
                    then_body.extend(self.lower_stmt(&stmt_node)?);
                }
                self.pop_scope();

                let orelse_list = ast_get_list!(node, "orelse");
                self.push_scope();
                let mut else_body = Vec::new();
                for stmt_node in orelse_list.iter() {
                    else_body.extend(self.lower_stmt(&stmt_node)?);
                }
                self.pop_scope();

                Ok(vec![TirStmt::If {
                    condition,
                    then_body,
                    else_body,
                }])
            }

            "While" => {
                let test_node = ast_getattr!(node, "test");
                let condition = self.lower_expr(&test_node)?;

                let body_list = ast_get_list!(node, "body");
                self.push_scope();
                let mut body = Vec::new();
                for stmt_node in body_list.iter() {
                    body.extend(self.lower_stmt(&stmt_node)?);
                }
                self.pop_scope();

                Ok(vec![TirStmt::While { condition, body }])
            }

            "Break" => Ok(vec![TirStmt::Break]),

            "Continue" => Ok(vec![TirStmt::Continue]),

            "Assert" => {
                let test_node = ast_getattr!(node, "test");
                let condition = self.lower_expr(&test_node)?;

                let bool_condition = if condition.ty == ValueType::Bool {
                    condition
                } else {
                    let cast_kind = match &condition.ty {
                        ValueType::Int => CastKind::IntToBool,
                        ValueType::Float => CastKind::FloatToBool,
                        _ => {
                            return Err(self.type_error(
                                line,
                                format!("cannot use `{}` in assert", condition.ty),
                            ))
                        }
                    };
                    TirExpr {
                        kind: TirExprKind::Cast {
                            kind: cast_kind,
                            arg: Box::new(condition),
                        },
                        ty: ValueType::Bool,
                    }
                };

                Ok(vec![TirStmt::VoidCall {
                    target: CallTarget::Builtin(builtin::BuiltinFn::Assert),
                    args: vec![bool_condition],
                }])
            }

            _ => {
                Err(self.syntax_error(line, format!("unsupported statement type: `{}`", node_type)))
            }
        }
    }

    // ── attribute assignment ───────────────────────────────────────────

    fn lower_attribute_assign(
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

        let class_info = self
            .class_registry
            .get(&class_name)
            .ok_or_else(|| self.name_error(line, format!("unknown class `{}`", class_name)))?
            .clone();

        let field_index = *class_info.field_map.get(&field_name).ok_or_else(|| {
            self.attribute_error(
                line,
                format!("class `{}` has no field `{}`", class_name, field_name),
            )
        })?;

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
        let tir_value = self.lower_expr(&value_node)?;

        if tir_value.ty.to_type() != field.ty {
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
            field_name,
            field_index,
            value: tir_value,
        }])
    }

    fn lower_attribute_aug_assign(
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

        let class_info = self
            .class_registry
            .get(&class_name)
            .ok_or_else(|| self.name_error(line, format!("unknown class `{}`", class_name)))?
            .clone();

        let field_index = *class_info.field_map.get(&field_name).ok_or_else(|| {
            self.attribute_error(
                line,
                format!("class `{}` has no field `{}`", class_name, field_name),
            )
        })?;

        let field = &class_info.fields[field_index];
        let field_vty = Self::to_value_type(&field.ty);

        // Read current field value
        let current_val = TirExpr {
            kind: TirExprKind::GetField {
                object: Box::new(obj_expr.clone()),
                field_name: field_name.clone(),
                field_index,
            },
            ty: field_vty,
        };

        let op = Self::convert_binop(&ast_getattr!(aug_node, "op"))?;
        let rhs = self.lower_expr(&ast_getattr!(aug_node, "value"))?;

        let (final_left, final_right, result_ty) =
            self.resolve_binop_types(line, op, current_val, rhs)?;

        if result_ty != field.ty {
            return Err(self.type_error(
                line,
                format!(
                    "augmented assignment would change field `{}.{}` type from `{}` to `{}`",
                    class_name, field_name, field.ty, result_ty
                ),
            ));
        }

        let result_vty = Self::to_value_type(&result_ty);
        let binop_expr = TirExpr {
            kind: TirExprKind::BinOp {
                op,
                left: Box::new(final_left),
                right: Box::new(final_right),
            },
            ty: result_vty,
        };

        Ok(vec![TirStmt::SetField {
            object: obj_expr,
            field_name,
            field_index,
            value: binop_expr,
        }])
    }

    // ── expression lowering ────────────────────────────────────────────

    fn lower_expr(&mut self, node: &Bound<PyAny>) -> Result<TirExpr> {
        let node_type = ast_type_name!(node);
        let line = Self::get_line(node);

        match node_type.as_str() {
            "Constant" => {
                let value = ast_getattr!(node, "value");
                if value.is_instance_of::<pyo3::types::PyBool>() {
                    let bool_val = value.extract::<bool>()?;
                    Ok(TirExpr {
                        kind: TirExprKind::IntLiteral(if bool_val { 1 } else { 0 }),
                        ty: ValueType::Bool,
                    })
                } else if let Ok(int_val) = value.extract::<i64>() {
                    Ok(TirExpr {
                        kind: TirExprKind::IntLiteral(int_val),
                        ty: ValueType::Int,
                    })
                } else if let Ok(float_val) = value.extract::<f64>() {
                    Ok(TirExpr {
                        kind: TirExprKind::FloatLiteral(float_val),
                        ty: ValueType::Float,
                    })
                } else {
                    Err(self.value_error(line, "unsupported constant type"))
                }
            }

            "Name" => {
                let id = ast_get_string!(node, "id");
                let ty = self
                    .lookup(&id)
                    .cloned()
                    .ok_or_else(|| self.name_error(line, format!("undefined variable `{}`", id)))?;
                let vty = Self::to_value_type(&ty);
                Ok(TirExpr {
                    kind: TirExprKind::Var(id),
                    ty: vty,
                })
            }

            "BinOp" => {
                let left = self.lower_expr(&ast_getattr!(node, "left"))?;
                let right = self.lower_expr(&ast_getattr!(node, "right"))?;
                let op = Self::convert_binop(&ast_getattr!(node, "op"))?;

                let (final_left, final_right, result_ty) =
                    self.resolve_binop_types(line, op, left, right)?;

                Ok(TirExpr {
                    kind: TirExprKind::BinOp {
                        op,
                        left: Box::new(final_left),
                        right: Box::new(final_right),
                    },
                    ty: Self::to_value_type(&result_ty),
                })
            }

            "Compare" => {
                let left = self.lower_expr(&ast_getattr!(node, "left"))?;
                let ops_list = ast_get_list!(node, "ops");
                let comparators_list = ast_get_list!(node, "comparators");

                if ops_list.len() == 1 {
                    let op_node = ops_list.get_item(0)?;
                    let cmp_op = Self::convert_cmpop(&op_node)?;
                    let right = self.lower_expr(&comparators_list.get_item(0)?)?;
                    let (fl, fr) = self.promote_for_comparison(line, left, right)?;
                    return Ok(TirExpr {
                        kind: TirExprKind::Compare {
                            op: cmp_op,
                            left: Box::new(fl),
                            right: Box::new(fr),
                        },
                        ty: ValueType::Bool,
                    });
                }

                let mut comparisons: Vec<TirExpr> = Vec::new();
                let mut current_left = left;

                for i in 0..ops_list.len() {
                    let op_node = ops_list.get_item(i)?;
                    let cmp_op = Self::convert_cmpop(&op_node)?;
                    let right = self.lower_expr(&comparators_list.get_item(i)?)?;

                    let (fl, fr) =
                        self.promote_for_comparison(line, current_left.clone(), right.clone())?;

                    comparisons.push(TirExpr {
                        kind: TirExprKind::Compare {
                            op: cmp_op,
                            left: Box::new(fl),
                            right: Box::new(fr),
                        },
                        ty: ValueType::Bool,
                    });

                    current_left = right;
                }

                let mut result = comparisons.remove(0);
                for cmp in comparisons {
                    result = TirExpr {
                        kind: TirExprKind::LogicalOp {
                            op: LogicalOp::And,
                            left: Box::new(result),
                            right: Box::new(cmp),
                        },
                        ty: ValueType::Bool,
                    };
                }

                Ok(result)
            }

            "UnaryOp" => {
                let op_node = ast_getattr!(node, "op");
                let op_type = ast_type_name!(op_node);
                let operand = self.lower_expr(&ast_getattr!(node, "operand"))?;

                let op = Self::convert_unaryop(&op_type)?;

                let rule = super::type_rules::lookup_unaryop(op, &operand.ty.to_type())
                    .ok_or_else(|| {
                        self.type_error(
                            line,
                            super::type_rules::unaryop_type_error_message(
                                op,
                                &operand.ty.to_type(),
                            ),
                        )
                    })?;

                Ok(TirExpr {
                    kind: TirExprKind::UnaryOp {
                        op,
                        operand: Box::new(operand),
                    },
                    ty: Self::to_value_type(&rule.result_type),
                })
            }

            "BoolOp" => {
                let op_node = ast_getattr!(node, "op");
                let op_type = ast_type_name!(op_node);
                let values_list = ast_get_list!(node, "values");

                let logical_op = match op_type.as_str() {
                    "And" => LogicalOp::And,
                    "Or" => LogicalOp::Or,
                    _ => {
                        return Err(self.syntax_error(
                            line,
                            format!("unsupported logical operator: `{}`", op_type),
                        ))
                    }
                };

                let mut exprs: Vec<TirExpr> = Vec::new();
                for val in values_list.iter() {
                    exprs.push(self.lower_expr(&val)?);
                }

                let result_ty = exprs[0].ty.clone();
                for (i, e) in exprs.iter().enumerate().skip(1) {
                    if e.ty != result_ty {
                        return Err(self.type_error(
                            line,
                            format!(
                                "all operands of `{}` must have the same type: operand {} is `{}`, expected `{}`",
                                op_type, i, e.ty, result_ty
                            ),
                        ));
                    }
                }

                let mut result = exprs.remove(0);
                for operand in exprs {
                    result = TirExpr {
                        kind: TirExprKind::LogicalOp {
                            op: logical_op,
                            left: Box::new(result),
                            right: Box::new(operand),
                        },
                        ty: result_ty.clone(),
                    };
                }

                Ok(result)
            }

            "Call" => match self.lower_call(node, line)? {
                CallResult::Expr(expr) => Ok(expr),
                CallResult::VoidStmt(_) => {
                    Err(self.type_error(line, "void function cannot be used as a value expression"))
                }
            },

            "Attribute" => {
                let value_node = ast_getattr!(node, "value");
                let attr_name = ast_get_string!(node, "attr");
                let obj_expr = self.lower_expr(&value_node)?;

                let class_name = match &obj_expr.ty {
                    ValueType::Class(name) => name.clone(),
                    other => {
                        return Err(self.type_error(
                            line,
                            format!("cannot access attribute on non-class type `{}`", other),
                        ))
                    }
                };

                let class_info = self.class_registry.get(&class_name).ok_or_else(|| {
                    self.name_error(line, format!("unknown class `{}`", class_name))
                })?;

                let field_index = *class_info.field_map.get(&attr_name).ok_or_else(|| {
                    self.attribute_error(
                        line,
                        format!("class `{}` has no field `{}`", class_name, attr_name),
                    )
                })?;

                let field_ty = Self::to_value_type(&class_info.fields[field_index].ty);

                Ok(TirExpr {
                    kind: TirExprKind::GetField {
                        object: Box::new(obj_expr),
                        field_name: attr_name,
                        field_index,
                    },
                    ty: field_ty,
                })
            }

            _ => Err(self.syntax_error(
                line,
                format!("unsupported expression type: `{}`", node_type),
            )),
        }
    }

    // ── print statement ────────────────────────────────────────────────

    fn lower_print_stmt(&mut self, call_node: &Bound<PyAny>) -> Result<Vec<TirStmt>> {
        let line = Self::get_line(call_node);
        let args_list = ast_get_list!(call_node, "args");

        let mut tir_args = Vec::new();
        for arg in args_list.iter() {
            tir_args.push(self.lower_expr(&arg)?);
        }

        if tir_args.is_empty() {
            return Ok(vec![TirStmt::VoidCall {
                target: CallTarget::Builtin(builtin::BuiltinFn::PrintNewline),
                args: vec![],
            }]);
        }

        let mut stmts = Vec::new();
        for (i, arg) in tir_args.into_iter().enumerate() {
            if i > 0 {
                stmts.push(TirStmt::VoidCall {
                    target: CallTarget::Builtin(builtin::BuiltinFn::PrintSpace),
                    args: vec![],
                });
            }
            let print_fn = builtin::resolve_print(&arg.ty).ok_or_else(|| {
                self.type_error(line, format!("cannot print value of type `{}`", arg.ty))
            })?;
            stmts.push(TirStmt::VoidCall {
                target: CallTarget::Builtin(print_fn),
                args: vec![arg],
            });
        }
        stmts.push(TirStmt::VoidCall {
            target: CallTarget::Builtin(builtin::BuiltinFn::PrintNewline),
            args: vec![],
        });

        Ok(stmts)
    }

    // ── call lowering ──────────────────────────────────────────────────

    fn lower_call(&mut self, node: &Bound<PyAny>, line: usize) -> Result<CallResult> {
        let func_node = ast_getattr!(node, "func");
        let args_list = ast_get_list!(node, "args");

        let mut tir_args = Vec::new();
        for arg in args_list.iter() {
            tir_args.push(self.lower_expr(&arg)?);
        }

        let func_node_type = ast_type_name!(func_node);
        match func_node_type.as_str() {
            "Name" => {
                let func_name = ast_get_string!(func_node, "id");

                if func_name == "print" {
                    return Err(self.syntax_error(line, "print() can only be used as a statement"));
                }

                if func_name == "int" || func_name == "float" || func_name == "bool" {
                    if tir_args.len() != 1 {
                        return Err(self.type_error(
                            line,
                            format!(
                                "{}() expects exactly 1 argument, got {}",
                                func_name,
                                tir_args.len()
                            ),
                        ));
                    }
                    let arg = tir_args.remove(0);
                    let target_ty = match func_name.as_str() {
                        "int" => {
                            if !arg.ty.is_primitive() {
                                return Err(self.type_error(
                                    line,
                                    format!("int() cannot convert `{}`", arg.ty),
                                ));
                            }
                            ValueType::Int
                        }
                        "float" => {
                            if !arg.ty.is_primitive() {
                                return Err(self.type_error(
                                    line,
                                    format!("float() cannot convert `{}`", arg.ty),
                                ));
                            }
                            ValueType::Float
                        }
                        "bool" => {
                            if !arg.ty.is_primitive() {
                                return Err(self.type_error(
                                    line,
                                    format!("bool() cannot convert `{}`", arg.ty),
                                ));
                            }
                            ValueType::Bool
                        }
                        _ => unreachable!(),
                    };

                    if arg.ty == target_ty {
                        return Ok(CallResult::Expr(arg));
                    }

                    let cast_kind = Self::compute_cast_kind(&arg.ty, &target_ty);
                    return Ok(CallResult::Expr(TirExpr {
                        kind: TirExprKind::Cast {
                            kind: cast_kind,
                            arg: Box::new(arg),
                        },
                        ty: target_ty,
                    }));
                }

                // Built-in numeric functions
                if func_name == "abs" {
                    if tir_args.len() != 1 {
                        return Err(self.type_error(
                            line,
                            format!("abs() expects 1 argument, got {}", tir_args.len()),
                        ));
                    }
                    let (builtin_fn, ret_vty) = match &tir_args[0].ty {
                        ValueType::Int => (builtin::BuiltinFn::AbsInt, ValueType::Int),
                        ValueType::Float => (builtin::BuiltinFn::AbsFloat, ValueType::Float),
                        _ => {
                            return Err(self.type_error(
                                line,
                                format!(
                                    "abs() requires a numeric argument, got `{}`",
                                    tir_args[0].ty
                                ),
                            ))
                        }
                    };
                    return Ok(CallResult::Expr(TirExpr {
                        kind: TirExprKind::ExternalCall {
                            func: builtin_fn,
                            args: tir_args,
                        },
                        ty: ret_vty,
                    }));
                }

                if func_name == "pow" {
                    if tir_args.len() != 2 {
                        return Err(self.type_error(
                            line,
                            format!("pow() expects 2 arguments, got {}", tir_args.len()),
                        ));
                    }
                    if tir_args[0].ty != tir_args[1].ty {
                        return Err(self.type_error(
                            line,
                            format!(
                                "pow() arguments must have the same type: got `{}` and `{}`",
                                tir_args[0].ty, tir_args[1].ty
                            ),
                        ));
                    }
                    match &tir_args[0].ty {
                        ValueType::Int => {
                            return Ok(CallResult::Expr(TirExpr {
                                kind: TirExprKind::ExternalCall {
                                    func: builtin::BuiltinFn::PowInt,
                                    args: tir_args,
                                },
                                ty: ValueType::Int,
                            }));
                        }
                        ValueType::Float => {
                            let right = tir_args.remove(1);
                            let left = tir_args.remove(0);
                            return Ok(CallResult::Expr(TirExpr {
                                kind: TirExprKind::BinOp {
                                    op: TypedBinOp::Arith(ArithBinOp::Pow),
                                    left: Box::new(left),
                                    right: Box::new(right),
                                },
                                ty: ValueType::Float,
                            }));
                        }
                        _ => {
                            return Err(self.type_error(
                                line,
                                format!(
                                    "pow() requires numeric arguments, got `{}`",
                                    tir_args[0].ty
                                ),
                            ))
                        }
                    }
                }

                if func_name == "min" || func_name == "max" {
                    if tir_args.len() != 2 {
                        return Err(self.type_error(
                            line,
                            format!(
                                "{}() expects 2 arguments, got {}",
                                func_name,
                                tir_args.len()
                            ),
                        ));
                    }
                    if tir_args[0].ty != tir_args[1].ty {
                        return Err(self.type_error(
                            line,
                            format!(
                                "{}() arguments must have the same type: got `{}` and `{}`",
                                func_name, tir_args[0].ty, tir_args[1].ty
                            ),
                        ));
                    }
                    let (builtin_fn, ret_vty) = match (&tir_args[0].ty, func_name.as_str()) {
                        (ValueType::Int, "min") => (builtin::BuiltinFn::MinInt, ValueType::Int),
                        (ValueType::Int, "max") => (builtin::BuiltinFn::MaxInt, ValueType::Int),
                        (ValueType::Float, "min") => {
                            (builtin::BuiltinFn::MinFloat, ValueType::Float)
                        }
                        (ValueType::Float, "max") => {
                            (builtin::BuiltinFn::MaxFloat, ValueType::Float)
                        }
                        _ => {
                            return Err(self.type_error(
                                line,
                                format!(
                                    "{}() requires numeric arguments, got `{}`",
                                    func_name, tir_args[0].ty
                                ),
                            ))
                        }
                    };
                    return Ok(CallResult::Expr(TirExpr {
                        kind: TirExprKind::ExternalCall {
                            func: builtin_fn,
                            args: tir_args,
                        },
                        ty: ret_vty,
                    }));
                }

                if func_name == "round" {
                    if tir_args.len() != 1 {
                        return Err(self.type_error(
                            line,
                            format!("round() expects 1 argument, got {}", tir_args.len()),
                        ));
                    }
                    if tir_args[0].ty != ValueType::Float {
                        return Err(self.type_error(
                            line,
                            format!(
                                "round() requires a `float` argument, got `{}`",
                                tir_args[0].ty
                            ),
                        ));
                    }
                    return Ok(CallResult::Expr(TirExpr {
                        kind: TirExprKind::ExternalCall {
                            func: builtin::BuiltinFn::RoundFloat,
                            args: tir_args,
                        },
                        ty: ValueType::Int,
                    }));
                }

                let scope_type = self.lookup(&func_name).cloned().ok_or_else(|| {
                    self.name_error(line, format!("undefined function `{}`", func_name))
                })?;

                match &scope_type {
                    Type::Function { .. } => {
                        let return_type =
                            self.check_call_args(line, &func_name, &scope_type, &tir_args)?;
                        let mangled = self.mangle_name(&func_name);
                        if return_type == Type::Unit {
                            Ok(CallResult::VoidStmt(TirStmt::VoidCall {
                                target: CallTarget::Named(mangled),
                                args: tir_args,
                            }))
                        } else {
                            Ok(CallResult::Expr(TirExpr {
                                kind: TirExprKind::Call {
                                    func: mangled,
                                    args: tir_args,
                                },
                                ty: Self::to_value_type(&return_type),
                            }))
                        }
                    }
                    Type::Module(mangled) => {
                        // Check if this is an imported class constructor
                        if let Some(class_info) = self.class_registry.get(mangled).cloned() {
                            return self.lower_constructor_call(
                                line,
                                mangled,
                                &class_info,
                                tir_args,
                            );
                        }

                        let func_type = self
                            .symbol_table
                            .get(mangled)
                            .ok_or_else(|| {
                                self.name_error(
                                    line,
                                    format!(
                                        "imported symbol `{}` not found in symbol table",
                                        func_name
                                    ),
                                )
                            })?
                            .clone();
                        let return_type =
                            self.check_call_args(line, &func_name, &func_type, &tir_args)?;
                        if return_type == Type::Unit {
                            Ok(CallResult::VoidStmt(TirStmt::VoidCall {
                                target: CallTarget::Named(mangled.clone()),
                                args: tir_args,
                            }))
                        } else {
                            Ok(CallResult::Expr(TirExpr {
                                kind: TirExprKind::Call {
                                    func: mangled.clone(),
                                    args: tir_args,
                                },
                                ty: Self::to_value_type(&return_type),
                            }))
                        }
                    }
                    Type::Class(name) => {
                        // Constructor call
                        let class_info = self
                            .class_registry
                            .get(name)
                            .ok_or_else(|| {
                                self.name_error(line, format!("unknown class `{}`", name))
                            })?
                            .clone();
                        self.lower_constructor_call(line, name, &class_info, tir_args)
                    }
                    _ => Err(self.type_error(line, format!("`{}` is not callable", func_name))),
                }
            }

            "Attribute" => {
                let value_node = ast_getattr!(func_node, "value");
                let attr = ast_get_string!(func_node, "attr");

                // Check if value_node is a Name that resolves to a module
                // (modules are not value types, so we must handle them before lower_expr)
                if ast_type_name!(value_node) == "Name" {
                    let name = ast_get_string!(value_node, "id");
                    if let Some(Type::Module(mod_path)) = self.lookup(&name).cloned() {
                        let resolved = format!("{}${}", mod_path, attr);

                        // Check for class constructor first
                        if let Some(class_info) = self.class_registry.get(&resolved).cloned() {
                            return self.lower_constructor_call(
                                line,
                                &resolved,
                                &class_info,
                                tir_args,
                            );
                        }

                        let func_type = self
                            .symbol_table
                            .get(&resolved)
                            .ok_or_else(|| {
                                self.name_error(line, format!("undefined function `{}`", attr))
                            })?
                            .clone();

                        let return_type = {
                            let label = attr.to_string();
                            self.check_call_args(line, &label, &func_type, &tir_args)?
                        };

                        return if return_type == Type::Unit {
                            Ok(CallResult::VoidStmt(TirStmt::VoidCall {
                                target: CallTarget::Named(resolved),
                                args: tir_args,
                            }))
                        } else {
                            Ok(CallResult::Expr(TirExpr {
                                kind: TirExprKind::Call {
                                    func: resolved,
                                    args: tir_args,
                                },
                                ty: Self::to_value_type(&return_type),
                            }))
                        };
                    }
                }

                // Check if the full dotted path resolves to a class
                // (e.g., Outer.Inner(...), Deep.Mid.Leaf(...))
                if let Some(qualified) = self.try_resolve_class_path(&func_node) {
                    if let Some(class_info) = self.class_registry.get(&qualified).cloned() {
                        return self.lower_constructor_call(
                            line,
                            &qualified,
                            &class_info,
                            tir_args,
                        );
                    }
                }

                // Not a class path — lower value as an expression (must be a class instance)
                let obj_expr = self.lower_expr(&value_node)?;

                match &obj_expr.ty {
                    ValueType::Class(class_name) => {
                        // Method call on a class instance
                        let class_info = self
                            .class_registry
                            .get(class_name)
                            .ok_or_else(|| {
                                self.name_error(line, format!("unknown class `{}`", class_name))
                            })?
                            .clone();

                        let method = class_info.methods.get(&attr).ok_or_else(|| {
                            self.attribute_error(
                                line,
                                format!("class `{}` has no method `{}`", class_name, attr),
                            )
                        })?;

                        if tir_args.len() != method.params.len() {
                            return Err(self.type_error(
                                line,
                                format!(
                                    "{}.{}() expects {} argument{}, got {}",
                                    class_name,
                                    attr,
                                    method.params.len(),
                                    if method.params.len() == 1 { "" } else { "s" },
                                    tir_args.len()
                                ),
                            ));
                        }
                        for (i, (arg, expected)) in
                            tir_args.iter().zip(method.params.iter()).enumerate()
                        {
                            if arg.ty.to_type() != *expected {
                                return Err(self.type_error(
                                    line,
                                    format!(
                                        "argument {} type mismatch in {}.{}(): expected `{}`, got `{}`",
                                        i, class_name, attr, expected, arg.ty
                                    ),
                                ));
                            }
                        }

                        let return_type = &method.return_type;
                        let mangled = method.mangled_name.clone();

                        if *return_type == Type::Unit {
                            Ok(CallResult::VoidStmt(TirStmt::VoidCall {
                                target: CallTarget::MethodCall {
                                    mangled_name: mangled,
                                    object: obj_expr,
                                },
                                args: tir_args,
                            }))
                        } else {
                            Ok(CallResult::Expr(TirExpr {
                                kind: TirExprKind::MethodCall {
                                    object: Box::new(obj_expr),
                                    method_mangled_name: mangled,
                                    args: tir_args,
                                },
                                ty: Self::to_value_type(return_type),
                            }))
                        }
                    }
                    _ => {
                        Err(self
                            .type_error(line, format!("`{}` is not a class instance", obj_expr.ty)))
                    }
                }
            }

            _ => Err(self.syntax_error(
                line,
                "only direct function calls and module.function calls are supported",
            )),
        }
    }

    // ── call argument checking ─────────────────────────────────────────

    fn check_call_args(
        &self,
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
                    if arg.ty.to_type() != *expected {
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

    fn lower_constructor_call(
        &self,
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

        if tir_args.len() != init_method.params.len() {
            return Err(self.type_error(
                line,
                format!(
                    "{}() expects {} argument{}, got {}",
                    qualified_name,
                    init_method.params.len(),
                    if init_method.params.len() == 1 {
                        ""
                    } else {
                        "s"
                    },
                    tir_args.len()
                ),
            ));
        }
        for (i, (arg, expected)) in tir_args.iter().zip(init_method.params.iter()).enumerate() {
            if arg.ty.to_type() != *expected {
                return Err(self.type_error(
                    line,
                    format!(
                        "argument {} type mismatch in {}(): expected `{}`, got `{}`",
                        i, qualified_name, expected, arg.ty
                    ),
                ));
            }
        }

        Ok(CallResult::Expr(TirExpr {
            kind: TirExprKind::Construct {
                class_name: qualified_name.to_string(),
                init_mangled_name: init_method.mangled_name.clone(),
                args: tir_args,
            },
            ty: ValueType::Class(qualified_name.to_string()),
        }))
    }

    /// Try to resolve an AST node as a dotted class/module path to a qualified class name.
    /// E.g., `Outer.Inner` → `module$Outer$Inner`, `mod.Class` → `mod$Class`
    fn try_resolve_class_path(&self, node: &Bound<PyAny>) -> Option<String> {
        match ast_type_name!(node).as_str() {
            "Name" => {
                let name = ast_get_string!(node, "id");
                match self.lookup(&name)? {
                    Type::Class(qualified) => Some(qualified.clone()),
                    Type::Module(mod_path) => Some(mod_path.clone()),
                    _ => None,
                }
            }
            "Attribute" => {
                let value_node = ast_getattr!(node, "value");
                let attr = ast_get_string!(node, "attr");
                let parent = self.try_resolve_class_path(&value_node)?;
                let candidate = format!("{}${}", parent, attr);
                if self.class_registry.contains_key(&candidate) {
                    Some(candidate)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn mangle_name(&self, name: &str) -> String {
        format!("{}${}", self.current_module_name, name)
    }

    // ── type promotion / binary ops ────────────────────────────────────

    fn resolve_binop_types(
        &self,
        line: usize,
        op: TypedBinOp,
        left: TirExpr,
        right: TirExpr,
    ) -> Result<(TirExpr, TirExpr, Type)> {
        let left_ast = left.ty.to_type();
        let right_ast = right.ty.to_type();
        let rule = super::type_rules::lookup_binop(op, &left_ast, &right_ast).ok_or_else(|| {
            self.type_error(
                line,
                super::type_rules::binop_type_error_message(op, &left_ast, &right_ast),
            )
        })?;

        let final_left = Self::apply_coercion(left, rule.left_coercion);
        let final_right = Self::apply_coercion(right, rule.right_coercion);

        Ok((final_left, final_right, rule.result_type))
    }

    fn apply_coercion(expr: TirExpr, coercion: super::type_rules::Coercion) -> TirExpr {
        match coercion {
            super::type_rules::Coercion::None => expr,
            super::type_rules::Coercion::ToFloat => {
                if expr.ty == ValueType::Float {
                    expr
                } else {
                    let cast_kind = match &expr.ty {
                        ValueType::Int => CastKind::IntToFloat,
                        ValueType::Bool => CastKind::BoolToFloat,
                        _ => unreachable!(),
                    };
                    TirExpr {
                        kind: TirExprKind::Cast {
                            kind: cast_kind,
                            arg: Box::new(expr),
                        },
                        ty: ValueType::Float,
                    }
                }
            }
        }
    }

    fn promote_for_comparison(
        &self,
        line: usize,
        left: TirExpr,
        right: TirExpr,
    ) -> Result<(TirExpr, TirExpr)> {
        if left.ty == right.ty {
            Ok((left, right))
        } else if (left.ty == ValueType::Int && right.ty == ValueType::Float)
            || (left.ty == ValueType::Float && right.ty == ValueType::Int)
        {
            let pl = if left.ty == ValueType::Int {
                TirExpr {
                    kind: TirExprKind::Cast {
                        kind: CastKind::IntToFloat,
                        arg: Box::new(left),
                    },
                    ty: ValueType::Float,
                }
            } else {
                left
            };
            let pr = if right.ty == ValueType::Int {
                TirExpr {
                    kind: TirExprKind::Cast {
                        kind: CastKind::IntToFloat,
                        arg: Box::new(right),
                    },
                    ty: ValueType::Float,
                }
            } else {
                right
            };
            Ok((pl, pr))
        } else {
            Err(self.type_error(
                line,
                format!(
                    "comparison operands must have compatible types: `{}` vs `{}`",
                    left.ty, right.ty
                ),
            ))
        }
    }

    // ── cast kind computation ──────────────────────────────────────────

    fn compute_cast_kind(from: &ValueType, to: &ValueType) -> CastKind {
        match (from, to) {
            (ValueType::Int, ValueType::Float) => CastKind::IntToFloat,
            (ValueType::Float, ValueType::Int) => CastKind::FloatToInt,
            (ValueType::Bool, ValueType::Float) => CastKind::BoolToFloat,
            (ValueType::Int, ValueType::Bool) => CastKind::IntToBool,
            (ValueType::Float, ValueType::Bool) => CastKind::FloatToBool,
            (ValueType::Bool, ValueType::Int) => CastKind::BoolToInt,
            _ => unreachable!("identity cast should have been eliminated"),
        }
    }

    // ── type / operator conversion helpers ─────────────────────────────

    fn convert_return_type(&self, node: &Bound<PyAny>) -> Result<Type> {
        let returns = ast_getattr!(node, "returns");
        if returns.is_none() {
            Ok(Type::Unit)
        } else {
            self.convert_type_annotation(&returns)
        }
    }

    fn convert_type_annotation(&self, node: &Bound<PyAny>) -> Result<Type> {
        let node_type = ast_type_name!(node);
        match node_type.as_str() {
            "Name" => {
                let id = ast_get_string!(node, "id");
                match id.as_str() {
                    "int" => Ok(Type::Int),
                    "float" => Ok(Type::Float),
                    "bool" => Ok(Type::Bool),
                    other => {
                        if let Some(ty) = self.lookup(other).cloned() {
                            match ty {
                                Type::Class(_) => Ok(ty),
                                Type::Module(ref mangled) => {
                                    // from-import of a class: `from mod import ClassName`
                                    if self.class_registry.contains_key(mangled) {
                                        Ok(Type::Class(mangled.clone()))
                                    } else {
                                        bail!("'{}' is not a type", other)
                                    }
                                }
                                _ => bail!("'{}' is not a type", other),
                            }
                        } else {
                            bail!("unsupported type `{}`", id)
                        }
                    }
                }
            }
            "Constant" => {
                let value = ast_getattr!(node, "value");
                if value.is_none() {
                    Ok(Type::Unit)
                } else {
                    bail!("unsupported constant type annotation")
                }
            }
            "Attribute" => {
                if let Some(qualified) = self.try_resolve_class_path(node) {
                    if self.class_registry.contains_key(&qualified) {
                        return Ok(Type::Class(qualified));
                    }
                }
                bail!("unsupported type annotation: `{}`", node_type)
            }
            _ => bail!("unsupported type annotation: `{}`", node_type),
        }
    }

    fn convert_cmpop(node: &Bound<PyAny>) -> Result<CmpOp> {
        let op_type = ast_type_name!(node);
        match op_type.as_str() {
            "Eq" => Ok(CmpOp::Eq),
            "NotEq" => Ok(CmpOp::NotEq),
            "Lt" => Ok(CmpOp::Lt),
            "LtE" => Ok(CmpOp::LtEq),
            "Gt" => Ok(CmpOp::Gt),
            "GtE" => Ok(CmpOp::GtEq),
            _ => bail!("unsupported comparison operator: `{}`", op_type),
        }
    }

    fn convert_binop(node: &Bound<PyAny>) -> Result<TypedBinOp> {
        let op_type = ast_type_name!(node);
        match op_type.as_str() {
            "Add" => Ok(TypedBinOp::Arith(ArithBinOp::Add)),
            "Sub" => Ok(TypedBinOp::Arith(ArithBinOp::Sub)),
            "Mult" => Ok(TypedBinOp::Arith(ArithBinOp::Mul)),
            "Div" => Ok(TypedBinOp::Arith(ArithBinOp::Div)),
            "FloorDiv" => Ok(TypedBinOp::Arith(ArithBinOp::FloorDiv)),
            "Mod" => Ok(TypedBinOp::Arith(ArithBinOp::Mod)),
            "Pow" => Ok(TypedBinOp::Arith(ArithBinOp::Pow)),
            "BitAnd" => Ok(TypedBinOp::Bitwise(BitwiseBinOp::BitAnd)),
            "BitOr" => Ok(TypedBinOp::Bitwise(BitwiseBinOp::BitOr)),
            "BitXor" => Ok(TypedBinOp::Bitwise(BitwiseBinOp::BitXor)),
            "LShift" => Ok(TypedBinOp::Bitwise(BitwiseBinOp::LShift)),
            "RShift" => Ok(TypedBinOp::Bitwise(BitwiseBinOp::RShift)),
            _ => bail!("unsupported binary operator: `{}`", op_type),
        }
    }

    fn convert_unaryop(op_type: &str) -> Result<UnaryOpKind> {
        match op_type {
            "USub" => Ok(UnaryOpKind::Neg),
            "UAdd" => Ok(UnaryOpKind::Pos),
            "Not" => Ok(UnaryOpKind::Not),
            "Invert" => Ok(UnaryOpKind::BitNot),
            _ => bail!("unsupported unary operator: `{}`", op_type),
        }
    }

    fn get_line(node: &Bound<PyAny>) -> usize {
        ast_getattr!(node, "lineno")
            .extract::<usize>()
            .unwrap_or_default()
    }
}
