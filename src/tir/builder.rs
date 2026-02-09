use anyhow::{bail, Result};
use std::collections::HashMap;

use super::{TirExpr, TirExprKind, TirFunction, TirModule, TirStmt};
use crate::ast::{Expr, ExprKind, Module, Stmt, StmtKind};

pub struct TirBuilder {
    /// The dotted module path (e.g. "imports.module_a")
    module_path: String,
    /// Direct call names -> mangled LLVM names (for `func_a()` style calls)
    call_resolution_map: HashMap<String, String>,
    /// Module aliases -> dotted module paths (for `module_a.func_a()` attribute calls)
    module_import_map: HashMap<String, String>,
}

impl TirBuilder {
    pub fn new(
        module_path: String,
        call_resolution_map: HashMap<String, String>,
        module_import_map: HashMap<String, String>,
    ) -> Self {
        Self {
            module_path,
            call_resolution_map,
            module_import_map,
        }
    }

    pub fn build_module(
        ast_module: Module,
        module_path: &str,
        call_resolution_map: HashMap<String, String>,
        module_import_map: HashMap<String, String>,
    ) -> Result<TirModule> {
        let mut builder = Self::new(
            module_path.to_string(),
            call_resolution_map,
            module_import_map,
        );
        let mut functions = HashMap::new();

        for stmt in ast_module.body {
            if let StmtKind::FunctionDef {
                name,
                params,
                return_type,
                body,
            } = stmt.kind
            {
                let tir_func = builder.build_function(name.clone(), params, return_type, body)?;
                functions.insert(tir_func.name.clone(), tir_func);
            }
        }

        Ok(TirModule {
            path: ast_module.path,
            functions,
        })
    }

    /// Mangle a function name with module path prefix.
    /// Synthetic "main" (module-level code) -> "module.path$$main$"
    /// Regular functions -> "module.path$func_name"
    fn mangle_name(&self, name: &str) -> String {
        if name == "main" {
            format!("{}$$main$", self.module_path)
        } else {
            format!("{}${}", self.module_path, name)
        }
    }

    fn build_function(
        &mut self,
        name: String,
        params: Vec<crate::ast::FunctionParam>,
        return_type: crate::ast::Type,
        body: Vec<Stmt>,
    ) -> Result<TirFunction> {
        let mangled_name = self.mangle_name(&name);

        let mut tir_body = Vec::new();
        for stmt in body {
            tir_body.push(self.build_stmt(stmt)?);
        }

        Ok(TirFunction {
            name: mangled_name,
            params,
            return_type,
            body: tir_body,
        })
    }

    fn build_stmt(&mut self, stmt: Stmt) -> Result<TirStmt> {
        match stmt.kind {
            StmtKind::Assign { target, ty, value } => {
                let tir_value = self.build_expr(value)?;
                let var_type = ty.unwrap_or_else(|| tir_value.ty.clone());
                Ok(TirStmt::Let {
                    name: target,
                    ty: var_type,
                    value: tir_value,
                })
            }

            StmtKind::Return(expr_opt) => Ok(TirStmt::Return(if let Some(e) = expr_opt {
                Some(self.build_expr(e)?)
            } else {
                None
            })),

            StmtKind::Expr(expr) => Ok(TirStmt::Expr(self.build_expr(expr)?)),

            StmtKind::FunctionDef { .. } => {
                bail!("Nested functions not supported")
            }
        }
    }

    fn build_expr(&mut self, expr: Expr) -> Result<TirExpr> {
        let ty = expr
            .ty
            .ok_or_else(|| anyhow::anyhow!("Expression has no type at line {}", expr.span.line))?;

        let kind = match expr.kind {
            ExprKind::IntLiteral(val) => TirExprKind::IntLiteral(val),

            ExprKind::Var(name) => TirExprKind::Var(name),

            ExprKind::BinOp { op, left, right } => TirExprKind::BinOp {
                op,
                left: Box::new(self.build_expr(*left)?),
                right: Box::new(self.build_expr(*right)?),
            },

            ExprKind::Call { func, args } => {
                let resolved_name = match func.kind {
                    ExprKind::Var(ref name) => {
                        // Direct call: look up in call_resolution_map
                        // "print" falls through unmapped (special-cased in codegen)
                        self.call_resolution_map
                            .get(name)
                            .cloned()
                            .unwrap_or_else(|| name.clone())
                    }
                    ExprKind::Attribute {
                        ref value,
                        ref attr,
                    } => {
                        // module.function() style call
                        if let ExprKind::Var(ref mod_name) = value.kind {
                            if let Some(mod_path) = self.module_import_map.get(mod_name) {
                                format!("{}${}", mod_path, attr)
                            } else {
                                bail!("Unknown module: {}", mod_name)
                            }
                        } else {
                            bail!("Complex attribute access not supported")
                        }
                    }
                    _ => bail!("Complex function expressions not supported"),
                };

                let tir_args = args
                    .into_iter()
                    .map(|arg| self.build_expr(arg))
                    .collect::<Result<Vec<_>>>()?;

                TirExprKind::Call {
                    func: resolved_name,
                    args: tir_args,
                }
            }

            ExprKind::Attribute { .. } => {
                bail!("Attribute access outside of function calls not yet supported")
            }
        };

        Ok(TirExpr { kind, ty })
    }
}
