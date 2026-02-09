use super::context::TypeContext;
use crate::ast::{Expr, ExprKind, Module, Stmt, StmtKind, Type};
use anyhow::{bail, Context as _, Result};

pub struct TypeInferencer {
    context: TypeContext,
}

impl TypeInferencer {
    pub fn new() -> Self {
        Self {
            context: TypeContext::new(),
        }
    }

    /// Add a function definition (for cross-module references)
    pub fn add_function(&mut self, name: String, ty: Type) -> Result<()> {
        self.context.define_function(name, ty)
    }

    /// Run type inference on a module, mutating expressions to add types
    pub fn infer_module(&mut self, module: &mut Module) -> Result<()> {
        // First pass: collect function signatures
        for stmt in &module.body {
            if let StmtKind::FunctionDef {
                name,
                params,
                return_type,
                ..
            } = &stmt.kind
            {
                let param_types = params.iter().map(|p| p.ty.clone()).collect();
                let func_type = Type::Function {
                    params: param_types,
                    return_type: Box::new(return_type.clone()),
                };
                log::debug!("Registering function '{}' in module", name);
                self.context.define_function(name.clone(), func_type)?;
            }
        }
        log::debug!(
            "First pass complete, registered {} functions",
            self.context.get_all_functions().len()
        );

        // Second pass: type check function bodies
        for stmt in &mut module.body {
            self.infer_stmt(stmt)?;
        }

        Ok(())
    }

    fn infer_stmt(&mut self, stmt: &mut Stmt) -> Result<()> {
        match &mut stmt.kind {
            StmtKind::FunctionDef {
                name,
                params,
                return_type,
                body,
            } => {
                // Save the current context to restore function definitions
                let saved_functions = self.context.get_all_functions();
                log::debug!(
                    "Type-checking function '{}', {} functions available",
                    name,
                    saved_functions.len()
                );

                // Create new scope for function variables
                let mut func_context = TypeContext::new();

                // Restore function definitions in new context
                for (fname, ftype) in saved_functions {
                    log::debug!("  Restoring function '{}' to context", fname);
                    func_context.define_function(fname, ftype)?;
                }

                // Add parameters to scope
                for param in params {
                    func_context.define_var(param.name.clone(), param.ty.clone())?;
                }

                func_context.enter_function(return_type.clone());

                // Type check body with new context
                let old_context = std::mem::replace(&mut self.context, func_context);

                for stmt in body {
                    self.infer_stmt(stmt).with_context(|| {
                        format!("In function '{}' at line {}", name, stmt.span.line)
                    })?;
                }

                self.context = old_context;
            }

            StmtKind::Assign { target, ty, value } => {
                // Infer expression type
                self.infer_expr(value)?;

                // Check annotation matches inferred type
                if let Some(annotated_ty) = ty {
                    if let Some(inferred_ty) = &value.ty {
                        if annotated_ty != inferred_ty {
                            bail!(
                                "Type mismatch at line {}: expected {:?}, got {:?}",
                                stmt.span.line,
                                annotated_ty,
                                inferred_ty
                            );
                        }
                    }
                }

                // Register variable
                let var_type = ty
                    .clone()
                    .or_else(|| value.ty.clone())
                    .context("Cannot infer variable type")?;
                self.context.define_var(target.clone(), var_type)?;
            }

            StmtKind::Return(expr_opt) => {
                if let Some(expr) = expr_opt {
                    self.infer_expr(expr)?;

                    // Check return type matches function signature
                    if let Some(expected) = self.context.get_return_type() {
                        if let Some(actual) = &expr.ty {
                            if expected != actual {
                                bail!(
                                    "Return type mismatch at line {}: expected {:?}, got {:?}",
                                    stmt.span.line,
                                    expected,
                                    actual
                                );
                            }
                        }
                    }
                } else {
                    // Return without value - should be Unit
                    if let Some(expected) = self.context.get_return_type() {
                        if !expected.is_unit() {
                            bail!(
                                "Return without value at line {}, but function expects {:?}",
                                stmt.span.line,
                                expected
                            );
                        }
                    }
                }
            }

            StmtKind::Expr(expr) => {
                self.infer_expr(expr)?;
            }
        }

        Ok(())
    }

    fn infer_expr(&mut self, expr: &mut Expr) -> Result<()> {
        let inferred_type = match &mut expr.kind {
            ExprKind::IntLiteral(_) => Type::Int,

            ExprKind::Var(name) => self.context.lookup_var(name).with_context(|| {
                format!("At line {}, column {}", expr.span.line, expr.span.column)
            })?,

            ExprKind::BinOp { op, left, right } => {
                self.infer_expr(left)?;
                self.infer_expr(right)?;

                // For arithmetic ops, both operands must be int
                if !matches!(left.ty, Some(Type::Int)) || !matches!(right.ty, Some(Type::Int)) {
                    bail!(
                        "Binary operator {:?} at line {} requires int operands, got {:?} and {:?}",
                        op,
                        expr.span.line,
                        left.ty,
                        right.ty
                    );
                }

                Type::Int
            }

            ExprKind::Attribute { value, attr: _ } => {
                // For now, only support module.function
                // Infer the value type (should be a module)
                self.infer_expr(value)?;

                // For module attributes, we treat them as function references
                // The type will be inferred when used in a call
                // For now, return Unknown - the call site will resolve it
                Type::Unknown
            }

            ExprKind::Call { func, args } => {
                // Infer argument types
                for arg in args.iter_mut() {
                    self.infer_expr(arg)?;
                }

                // Get function name - could be direct or via attribute
                let func_name = match &func.kind {
                    ExprKind::Var(name) => name.clone(),
                    ExprKind::Attribute { value: _, attr } => {
                        // module.function - for now we'll just use the attr name
                        // In a full implementation, we'd need to resolve the module
                        // and look up the function in that module's scope
                        attr.clone()
                    }
                    _ => bail!(
                        "Only direct function calls and module.function calls supported at line {}",
                        expr.span.line
                    ),
                };

                // Special case for print (built-in)
                if func_name == "print" {
                    expr.ty = Some(Type::Unit);
                    return Ok(());
                }

                let func_type = self.context.lookup_function(&func_name).with_context(|| {
                    format!("At line {}, column {}", expr.span.line, expr.span.column)
                })?;

                match func_type {
                    Type::Function {
                        params,
                        return_type,
                    } => {
                        // Check argument count
                        if args.len() != params.len() {
                            bail!(
                                "Function '{}' at line {} expects {} arguments, got {}",
                                func_name,
                                expr.span.line,
                                params.len(),
                                args.len()
                            );
                        }

                        // Check argument types
                        for (i, (arg, expected)) in args.iter().zip(params.iter()).enumerate() {
                            if let Some(actual) = &arg.ty {
                                if actual != expected {
                                    bail!(
                                        "Argument {} type mismatch in call to '{}' at line {}: expected {:?}, got {:?}",
                                        i,
                                        func_name,
                                        expr.span.line,
                                        expected,
                                        actual
                                    );
                                }
                            }
                        }

                        *return_type
                    }
                    _ => bail!("Cannot call non-function type at line {}", expr.span.line),
                }
            }
        };

        expr.ty = Some(inferred_type);
        Ok(())
    }
}

impl Default for TypeInferencer {
    fn default() -> Self {
        Self::new()
    }
}
