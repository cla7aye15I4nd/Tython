use anyhow::{bail, Result};
use pyo3::prelude::*;
use pyo3::types::{PyList, PyModule};
use std::collections::HashMap;
use std::path::Path;

use super::{
    ArithBinOp, BitwiseBinOp, CmpOp, TirFunction, TirModule, TirStmt, TypedBinOp, UnaryOpKind,
    ValueType,
};
use crate::ast::{ClassInfo, Type};
use crate::errors::{ErrorCategory, TythonError};
use crate::{ast_get_list, ast_get_string, ast_getattr, ast_type_name};

mod calls;
mod classes;
mod expressions;
mod functions;
mod statements;

macro_rules! define_error_helpers {
    ($($name:ident => $category:ident),* $(,)?) => {
        $(
            fn $name(&self, line: usize, msg: impl Into<String>) -> anyhow::Error {
                self.make_error(ErrorCategory::$category, line, msg.into())
            }
        )*
    }
}

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

    define_error_helpers! {
        type_error     => TypeError,
        name_error     => NameError,
        syntax_error   => SyntaxError,
        value_error    => ValueError,
        attribute_error => AttributeError,
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

    /// Lower a block of statements inside a new scope.
    fn lower_block(&mut self, stmts: &Bound<PyList>) -> Result<Vec<TirStmt>> {
        self.push_scope();
        let mut body = Vec::new();
        for stmt_node in stmts.iter() {
            body.extend(self.lower_stmt(&stmt_node)?);
        }
        self.pop_scope();
        Ok(body)
    }

    /// Look up a class in the registry, or return a NameError.
    fn lookup_class(&self, line: usize, class_name: &str) -> Result<ClassInfo> {
        self.class_registry
            .get(class_name)
            .cloned()
            .ok_or_else(|| self.name_error(line, format!("unknown class `{}`", class_name)))
    }

    /// Look up a field index in a class, or return an AttributeError.
    fn lookup_field_index(
        &self,
        line: usize,
        class_info: &ClassInfo,
        field_name: &str,
    ) -> Result<usize> {
        class_info
            .field_map
            .get(field_name)
            .copied()
            .ok_or_else(|| {
                self.attribute_error(
                    line,
                    format!("class `{}` has no field `{}`", class_info.name, field_name),
                )
            })
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

    // ── utility helpers ───────────────────────────────────────────────

    fn get_line(node: &Bound<PyAny>) -> usize {
        ast_getattr!(node, "lineno")
            .extract::<usize>()
            .unwrap_or_default()
    }

    fn mangle_name(&self, name: &str) -> String {
        format!("{}${}", self.current_module_name, name)
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
                    "str" => Ok(Type::Str),
                    "bytes" => Ok(Type::Bytes),
                    "bytearray" => Ok(Type::ByteArray),
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
}
