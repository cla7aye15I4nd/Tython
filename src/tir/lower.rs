use anyhow::{bail, Result};
use pyo3::prelude::*;
use pyo3::types::PyModule;
use std::collections::HashMap;
use std::path::Path;

use super::builtin;
use super::{
    BinOpKind, CmpOp, FunctionParam, LogicalOp, TirExpr, TirExprKind, TirFunction, TirModule,
    TirStmt, UnaryOpKind,
};
use crate::ast::Type;
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

        for node in body_list.iter() {
            if ast_type_name!(node) == "FunctionDef" {
                self.collect_function_signature(&node)?;
            }
        }

        let mut functions = HashMap::new();
        let mut module_level_stmts = Vec::new();

        for node in body_list.iter() {
            match ast_type_name!(node).as_str() {
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

        for func in functions.values() {
            let func_type = Type::Function {
                params: func.params.iter().map(|p| p.ty.clone()).collect(),
                return_type: Box::new(func.return_type.clone()),
            };
            self.symbol_table.insert(func.name.clone(), func_type);
        }

        Ok(TirModule { functions })
    }

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
            param_types.push(Self::convert_type_annotation(&annotation)?);
        }

        let return_type = Self::convert_return_type(node)?;
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
            let ty = Self::convert_type_annotation(&annotation)?;
            params.push(FunctionParam::new(param_name, ty));
        }

        let return_type = Self::convert_return_type(node)?;

        self.push_scope();
        for param in &params {
            self.declare(param.name.clone(), param.ty.clone());
        }
        self.current_return_type = Some(return_type.clone());
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
            ty: Type::Int,
        })));

        TirFunction {
            name: self.mangle_name("$main$"),
            params: Vec::new(),
            return_type: Type::Int,
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
                    .then(|| Self::convert_type_annotation(&annotation))
                    .transpose()?;

                let value_node = ast_getattr!(node, "value");
                let tir_value = self.lower_expr(&value_node)?;

                if let Some(ref ann_ty) = annotated_ty {
                    if ann_ty != &tir_value.ty {
                        return Err(self.type_error(
                            line,
                            format!(
                                "type mismatch: expected `{}`, got `{}`",
                                ann_ty, tir_value.ty
                            ),
                        ));
                    }
                }

                let var_type = annotated_ty.unwrap_or_else(|| tir_value.ty.clone());
                self.declare(target.clone(), var_type.clone());

                Ok(vec![TirStmt::Let {
                    name: target,
                    ty: var_type,
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
                if ast_type_name!(target_node) != "Name" {
                    return Err(
                        self.syntax_error(line, "only simple variable assignments are supported")
                    );
                }
                let target = ast_get_string!(target_node, "id");

                let value_node = ast_getattr!(node, "value");
                let tir_value = self.lower_expr(&value_node)?;
                let var_type = tir_value.ty.clone();
                self.declare(target.clone(), var_type.clone());

                Ok(vec![TirStmt::Let {
                    name: target,
                    ty: var_type,
                    value: tir_value,
                }])
            }

            "AugAssign" => {
                let target_node = ast_getattr!(node, "target");
                if ast_type_name!(target_node) != "Name" {
                    return Err(self.syntax_error(
                        line,
                        "only simple variable augmented assignments are supported",
                    ));
                }
                let target = ast_get_string!(target_node, "id");

                let target_ty = self.lookup(&target).cloned().ok_or_else(|| {
                    self.name_error(line, format!("undefined variable `{}`", target))
                })?;

                let op = Self::convert_binop(&ast_getattr!(node, "op"))?;
                let value_expr = self.lower_expr(&ast_getattr!(node, "value"))?;

                // Disallow /= on int variables (would change type)
                if op == BinOpKind::Div && target_ty == Type::Int {
                    return Err(self.type_error(
                        line,
                        format!("`/=` on `int` variable `{}` would change type to `float`; use `//=` for integer division", target),
                    ));
                }

                let target_ref = TirExpr {
                    kind: TirExprKind::Var(target.clone()),
                    ty: target_ty,
                };

                let (final_left, final_right, result_ty) =
                    self.resolve_binop_types(line, op, target_ref, value_expr)?;

                let binop_expr = TirExpr {
                    kind: TirExprKind::BinOp {
                        op,
                        left: Box::new(final_left),
                        right: Box::new(final_right),
                    },
                    ty: result_ty.clone(),
                };

                self.declare(target.clone(), result_ty.clone());

                Ok(vec![TirStmt::Let {
                    name: target,
                    ty: result_ty,
                    value: binop_expr,
                }])
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
                        if expected != &tir_expr.ty {
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

                // Detect print() calls and expand to value-print + newline statements
                if ast_type_name!(value_node) == "Call" {
                    let func_node = ast_getattr!(value_node, "func");
                    if ast_type_name!(func_node) == "Name"
                        && ast_get_string!(func_node, "id") == "print"
                    {
                        return self.lower_print_stmt(&value_node);
                    }
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
                let bool_condition = TirExpr {
                    kind: TirExprKind::Cast {
                        target: Type::Bool,
                        arg: Box::new(condition),
                    },
                    ty: Type::Bool,
                };
                Ok(vec![TirStmt::Expr(TirExpr {
                    kind: TirExprKind::ExternalCall {
                        func: builtin::BuiltinFn::Assert,
                        args: vec![bool_condition],
                    },
                    ty: Type::Unit,
                })])
            }

            _ => {
                Err(self.syntax_error(line, format!("unsupported statement type: `{}`", node_type)))
            }
        }
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
                        ty: Type::Bool,
                    })
                } else if let Ok(int_val) = value.extract::<i64>() {
                    Ok(TirExpr {
                        kind: TirExprKind::IntLiteral(int_val),
                        ty: Type::Int,
                    })
                } else if let Ok(float_val) = value.extract::<f64>() {
                    Ok(TirExpr {
                        kind: TirExprKind::FloatLiteral(float_val),
                        ty: Type::Float,
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
                Ok(TirExpr {
                    kind: TirExprKind::Var(id),
                    ty,
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
                    ty: result_ty,
                })
            }

            "Compare" => {
                let left = self.lower_expr(&ast_getattr!(node, "left"))?;
                let ops_list = ast_get_list!(node, "ops");
                let comparators_list = ast_get_list!(node, "comparators");

                if ops_list.len() == 1 {
                    // Single comparison
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
                        ty: Type::Bool,
                    });
                }

                // Chained comparison: a < b < c => (a < b) and (b < c)
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
                        ty: Type::Bool,
                    });

                    current_left = right;
                }

                // Chain with LogicalOp::And
                let mut result = comparisons.remove(0);
                for cmp in comparisons {
                    result = TirExpr {
                        kind: TirExprKind::LogicalOp {
                            op: LogicalOp::And,
                            left: Box::new(result),
                            right: Box::new(cmp),
                        },
                        ty: Type::Bool,
                    };
                }

                Ok(result)
            }

            "UnaryOp" => {
                let op_node = ast_getattr!(node, "op");
                let op_type = ast_type_name!(op_node);
                let operand = self.lower_expr(&ast_getattr!(node, "operand"))?;

                let op = Self::convert_unaryop(&op_type)?;

                let rule = super::type_rules::lookup_unaryop(op, &operand.ty).ok_or_else(|| {
                    self.type_error(
                        line,
                        super::type_rules::unaryop_type_error_message(op, &operand.ty),
                    )
                })?;

                Ok(TirExpr {
                    kind: TirExprKind::UnaryOp {
                        op,
                        operand: Box::new(operand),
                    },
                    ty: rule.result_type,
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

                // All values must have the same type
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

                // Chain: a and b and c => (a and b) and c
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

            "Call" => self.lower_call(node, line),

            "Attribute" => Err(self.syntax_error(
                line,
                "attribute access outside of function calls is not yet supported",
            )),

            _ => Err(self.syntax_error(
                line,
                format!("unsupported expression type: `{}`", node_type),
            )),
        }
    }

    // ── print statement ────────────────────────────────────────────────

    fn lower_print_stmt(&mut self, call_node: &Bound<PyAny>) -> Result<Vec<TirStmt>> {
        let args_list = ast_get_list!(call_node, "args");

        let mut tir_args = Vec::new();
        for arg in args_list.iter() {
            tir_args.push(self.lower_expr(&arg)?);
        }

        if tir_args.is_empty() {
            return Ok(vec![TirStmt::Expr(TirExpr {
                kind: TirExprKind::ExternalCall {
                    func: builtin::BuiltinFn::PrintNewline,
                    args: vec![],
                },
                ty: Type::Unit,
            })]);
        }

        let mut stmts = Vec::new();
        for (i, arg) in tir_args.into_iter().enumerate() {
            if i > 0 {
                stmts.push(TirStmt::Expr(TirExpr {
                    kind: TirExprKind::ExternalCall {
                        func: builtin::BuiltinFn::PrintSpace,
                        args: vec![],
                    },
                    ty: Type::Unit,
                }));
            }
            let print_fn = builtin::resolve_print(&arg.ty);
            stmts.push(TirStmt::Expr(TirExpr {
                kind: TirExprKind::ExternalCall {
                    func: print_fn,
                    args: vec![arg],
                },
                ty: Type::Unit,
            }));
        }
        stmts.push(TirStmt::Expr(TirExpr {
            kind: TirExprKind::ExternalCall {
                func: builtin::BuiltinFn::PrintNewline,
                args: vec![],
            },
            ty: Type::Unit,
        }));

        Ok(stmts)
    }

    // ── call lowering ──────────────────────────────────────────────────

    fn lower_call(&mut self, node: &Bound<PyAny>, line: usize) -> Result<TirExpr> {
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
                            if arg.ty != Type::Int && arg.ty != Type::Float && arg.ty != Type::Bool
                            {
                                return Err(self.type_error(
                                    line,
                                    format!("int() cannot convert `{}`", arg.ty),
                                ));
                            }
                            Type::Int
                        }
                        "float" => {
                            if arg.ty != Type::Int && arg.ty != Type::Float && arg.ty != Type::Bool
                            {
                                return Err(self.type_error(
                                    line,
                                    format!("float() cannot convert `{}`", arg.ty),
                                ));
                            }
                            Type::Float
                        }
                        "bool" => {
                            if arg.ty != Type::Int && arg.ty != Type::Float && arg.ty != Type::Bool
                            {
                                return Err(self.type_error(
                                    line,
                                    format!("bool() cannot convert `{}`", arg.ty),
                                ));
                            }
                            Type::Bool
                        }
                        _ => unreachable!(),
                    };
                    return Ok(TirExpr {
                        kind: TirExprKind::Cast {
                            target: target_ty.clone(),
                            arg: Box::new(arg),
                        },
                        ty: target_ty,
                    });
                }

                // Built-in numeric functions
                if func_name == "abs" {
                    if tir_args.len() != 1 {
                        return Err(self.type_error(
                            line,
                            format!("abs() expects 1 argument, got {}", tir_args.len()),
                        ));
                    }
                    let (builtin_fn, ret_ty) = match tir_args[0].ty {
                        Type::Int => (builtin::BuiltinFn::AbsInt, Type::Int),
                        Type::Float => (builtin::BuiltinFn::AbsFloat, Type::Float),
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
                    return Ok(TirExpr {
                        kind: TirExprKind::ExternalCall {
                            func: builtin_fn,
                            args: tir_args,
                        },
                        ty: ret_ty,
                    });
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
                    match tir_args[0].ty {
                        Type::Int => {
                            return Ok(TirExpr {
                                kind: TirExprKind::ExternalCall {
                                    func: builtin::BuiltinFn::PowInt,
                                    args: tir_args,
                                },
                                ty: Type::Int,
                            });
                        }
                        Type::Float => {
                            let right = tir_args.remove(1);
                            let left = tir_args.remove(0);
                            return Ok(TirExpr {
                                kind: TirExprKind::BinOp {
                                    op: BinOpKind::Pow,
                                    left: Box::new(left),
                                    right: Box::new(right),
                                },
                                ty: Type::Float,
                            });
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
                    let (builtin_fn, ret_ty) = match (&tir_args[0].ty, func_name.as_str()) {
                        (Type::Int, "min") => (builtin::BuiltinFn::MinInt, Type::Int),
                        (Type::Int, "max") => (builtin::BuiltinFn::MaxInt, Type::Int),
                        (Type::Float, "min") => (builtin::BuiltinFn::MinFloat, Type::Float),
                        (Type::Float, "max") => (builtin::BuiltinFn::MaxFloat, Type::Float),
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
                    return Ok(TirExpr {
                        kind: TirExprKind::ExternalCall {
                            func: builtin_fn,
                            args: tir_args,
                        },
                        ty: ret_ty,
                    });
                }

                if func_name == "round" {
                    if tir_args.len() != 1 {
                        return Err(self.type_error(
                            line,
                            format!("round() expects 1 argument, got {}", tir_args.len()),
                        ));
                    }
                    if tir_args[0].ty != Type::Float {
                        return Err(self.type_error(
                            line,
                            format!(
                                "round() requires a `float` argument, got `{}`",
                                tir_args[0].ty
                            ),
                        ));
                    }
                    return Ok(TirExpr {
                        kind: TirExprKind::ExternalCall {
                            func: builtin::BuiltinFn::RoundFloat,
                            args: tir_args,
                        },
                        ty: Type::Int,
                    });
                }

                let scope_type = self.lookup(&func_name).cloned().ok_or_else(|| {
                    self.name_error(line, format!("undefined function `{}`", func_name))
                })?;

                match &scope_type {
                    Type::Function { .. } => {
                        let return_type =
                            self.check_call_args(line, &func_name, &scope_type, &tir_args)?;
                        let mangled = self.mangle_name(&func_name);
                        Ok(TirExpr {
                            kind: TirExprKind::Call {
                                func: mangled,
                                args: tir_args,
                            },
                            ty: return_type,
                        })
                    }
                    Type::Module(mangled) => {
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
                        Ok(TirExpr {
                            kind: TirExprKind::Call {
                                func: mangled.clone(),
                                args: tir_args,
                            },
                            ty: return_type,
                        })
                    }
                    _ => Err(self.type_error(line, format!("`{}` is not callable", func_name))),
                }
            }

            "Attribute" => {
                let value_node = ast_getattr!(func_node, "value");
                let attr = ast_get_string!(func_node, "attr");

                if ast_type_name!(value_node) != "Name" {
                    return Err(
                        self.syntax_error(line, "complex attribute access is not supported")
                    );
                }
                let mod_name = ast_get_string!(value_node, "id");

                let mod_type = self.lookup(&mod_name).cloned().ok_or_else(|| {
                    self.name_error(line, format!("unknown module `{}`", mod_name))
                })?;

                let mod_path = match &mod_type {
                    Type::Module(path) => path.clone(),
                    _ => {
                        return Err(self.type_error(line, format!("`{}` is not a module", mod_name)))
                    }
                };

                let resolved = format!("{}${}", mod_path, attr);

                let func_type = self
                    .symbol_table
                    .get(&resolved)
                    .ok_or_else(|| {
                        self.name_error(line, format!("undefined function `{}.{}`", mod_name, attr))
                    })?
                    .clone();

                let return_type = {
                    let label = format!("{}.{}", mod_name, attr);
                    self.check_call_args(line, &label, &func_type, &tir_args)?
                };

                Ok(TirExpr {
                    kind: TirExprKind::Call {
                        func: resolved,
                        args: tir_args,
                    },
                    ty: return_type,
                })
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
                    if &arg.ty != expected {
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

    fn mangle_name(&self, name: &str) -> String {
        format!("{}${}", self.current_module_name, name)
    }

    // ── type promotion / binary ops ────────────────────────────────────

    fn resolve_binop_types(
        &self,
        line: usize,
        op: BinOpKind,
        left: TirExpr,
        right: TirExpr,
    ) -> Result<(TirExpr, TirExpr, Type)> {
        let rule = super::type_rules::lookup_binop(op, &left.ty, &right.ty).ok_or_else(|| {
            self.type_error(
                line,
                super::type_rules::binop_type_error_message(op, &left.ty, &right.ty),
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
                if expr.ty == Type::Float {
                    expr
                } else {
                    TirExpr {
                        kind: TirExprKind::Cast {
                            target: Type::Float,
                            arg: Box::new(expr),
                        },
                        ty: Type::Float,
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
        } else if (left.ty == Type::Int && right.ty == Type::Float)
            || (left.ty == Type::Float && right.ty == Type::Int)
        {
            let pl = if left.ty == Type::Int {
                TirExpr {
                    kind: TirExprKind::Cast {
                        target: Type::Float,
                        arg: Box::new(left),
                    },
                    ty: Type::Float,
                }
            } else {
                left
            };
            let pr = if right.ty == Type::Int {
                TirExpr {
                    kind: TirExprKind::Cast {
                        target: Type::Float,
                        arg: Box::new(right),
                    },
                    ty: Type::Float,
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

    // ── type / operator conversion helpers ─────────────────────────────

    fn convert_return_type(node: &Bound<PyAny>) -> Result<Type> {
        let returns = ast_getattr!(node, "returns");
        if returns.is_none() {
            Ok(Type::Unit)
        } else {
            Self::convert_type_annotation(&returns)
        }
    }

    fn convert_type_annotation(node: &Bound<PyAny>) -> Result<Type> {
        let node_type = ast_type_name!(node);
        match node_type.as_str() {
            "Name" => {
                let id = ast_get_string!(node, "id");
                match id.as_str() {
                    "int" => Ok(Type::Int),
                    "float" => Ok(Type::Float),
                    "bool" => Ok(Type::Bool),
                    _ => bail!("unsupported type `{}`", id),
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

    fn convert_binop(node: &Bound<PyAny>) -> Result<BinOpKind> {
        let op_type = ast_type_name!(node);
        match op_type.as_str() {
            "Add" => Ok(BinOpKind::Add),
            "Sub" => Ok(BinOpKind::Sub),
            "Mult" => Ok(BinOpKind::Mul),
            "Div" => Ok(BinOpKind::Div),
            "FloorDiv" => Ok(BinOpKind::FloorDiv),
            "Mod" => Ok(BinOpKind::Mod),
            "Pow" => Ok(BinOpKind::Pow),
            "BitAnd" => Ok(BinOpKind::BitAnd),
            "BitOr" => Ok(BinOpKind::BitOr),
            "BitXor" => Ok(BinOpKind::BitXor),
            "LShift" => Ok(BinOpKind::LShift),
            "RShift" => Ok(BinOpKind::RShift),
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
