use anyhow::Result;
use pyo3::prelude::*;

use crate::ast::Type;
use crate::tir::{ExceptClause, TirStmt, ValueType};
use crate::{ast_get_list, ast_get_string, ast_getattr, ast_type_name};

use crate::tir::lower::Lowering;

impl Lowering {
    pub(in crate::tir::lower) fn handle_raise(
        &mut self,
        node: &Bound<PyAny>,
        line: usize,
    ) -> Result<Vec<TirStmt>> {
        let exc_node = ast_getattr!(node, "exc");
        if exc_node.is_none() {
            return Ok(vec![TirStmt::Raise {
                exc_type_tag: None,
                message: None,
            }]);
        }

        match ast_type_name!(exc_node).as_str() {
            "Call" => {
                let func_node = ast_getattr!(exc_node, "func");
                let exc_name = if ast_type_name!(func_node) == "Name" {
                    ast_get_string!(func_node, "id")
                } else {
                    return Err(self.syntax_error(line, "unsupported raise expression"));
                };

                let type_tag = Self::resolve_exception_tag(&exc_name).ok_or_else(|| {
                    self.syntax_error(line, format!("unsupported exception type `{}`", exc_name))
                })?;

                let args_list = ast_get_list!(exc_node, "args");
                let message = if args_list.is_empty() {
                    None
                } else if args_list.len() == 1 {
                    let msg_expr = self.lower_expr(&args_list.get_item(0)?)?;
                    if msg_expr.ty != ValueType::Str {
                        return Err(self.type_error(
                            line,
                            format!("exception message must be `str`, got `{}`", msg_expr.ty),
                        ));
                    }
                    Some(msg_expr)
                } else {
                    return Err(
                        self.syntax_error(line, format!("{}() takes 0 or 1 arguments", exc_name))
                    );
                };

                Ok(vec![TirStmt::Raise {
                    exc_type_tag: Some(type_tag),
                    message,
                }])
            }
            "Name" => {
                let exc_name = ast_get_string!(exc_node, "id");
                let type_tag = Self::resolve_exception_tag(&exc_name).ok_or_else(|| {
                    self.syntax_error(line, format!("unsupported exception type `{}`", exc_name))
                })?;
                Ok(vec![TirStmt::Raise {
                    exc_type_tag: Some(type_tag),
                    message: None,
                }])
            }
            _ => Err(self.syntax_error(line, "unsupported raise expression")),
        }
    }

    fn resolve_exception_tag(name: &str) -> Option<i64> {
        match name {
            "Exception" => Some(1),
            "StopIteration" => Some(2),
            "ValueError" => Some(3),
            "TypeError" => Some(4),
            "KeyError" => Some(5),
            "RuntimeError" => Some(6),
            "ZeroDivisionError" => Some(7),
            "OverflowError" => Some(8),
            "IndexError" => Some(9),
            "AttributeError" => Some(10),
            "NotImplementedError" => Some(11),
            "NameError" => Some(12),
            "ArithmeticError" => Some(13),
            "LookupError" => Some(14),
            "AssertionError" => Some(15),
            "ImportError" => Some(16),
            "ModuleNotFoundError" => Some(17),
            "FileNotFoundError" => Some(18),
            "PermissionError" => Some(19),
            "OSError" | "IOError" => Some(20),
            _ => None,
        }
    }

    pub(in crate::tir::lower) fn handle_try(
        &mut self,
        node: &Bound<PyAny>,
        line: usize,
    ) -> Result<Vec<TirStmt>> {
        let finalbody = ast_get_list!(node, "finalbody");
        let has_finally = !finalbody.is_empty();

        // `return` is forbidden inside try/except/finally blocks that have
        // a finally clause, because the codegen cannot guarantee the finally
        // block runs before the function exits.
        if has_finally {
            self.in_try_finally_depth += 1;
        }

        let try_body = self.lower_block(&ast_get_list!(node, "body"))?;

        let handlers_list = ast_get_list!(node, "handlers");
        let mut except_clauses = Vec::new();
        for handler in handlers_list.iter() {
            let type_node = ast_getattr!(handler, "type");
            let exc_type_tag = if type_node.is_none() {
                None // bare except
            } else {
                let type_name = if ast_type_name!(type_node) == "Name" {
                    ast_get_string!(type_node, "id")
                } else {
                    return Err(
                        self.syntax_error(line, "unsupported exception type in except clause")
                    );
                };
                Some(Self::resolve_exception_tag(&type_name).ok_or_else(|| {
                    self.syntax_error(line, format!("unsupported exception type `{}`", type_name))
                })?)
            };

            let name_node = ast_getattr!(handler, "name");
            let var_name = if name_node.is_none() {
                None
            } else {
                let name = name_node.extract::<String>()?;
                // Declare the exception variable as str in the handler scope
                self.declare(name.clone(), Type::Str);
                Some(name)
            };

            let handler_body = self.lower_block(&ast_get_list!(handler, "body"))?;
            except_clauses.push(ExceptClause {
                exc_type_tag,
                var_name,
                body: handler_body,
            });
        }

        let orelse = ast_get_list!(node, "orelse");
        let else_body = if orelse.is_empty() {
            vec![]
        } else {
            self.lower_block(&orelse)?
        };

        let finally_body = if finalbody.is_empty() {
            vec![]
        } else {
            self.lower_block(&finalbody)?
        };

        if has_finally {
            self.in_try_finally_depth -= 1;
        }

        Ok(vec![TirStmt::TryCatch {
            try_body,
            except_clauses,
            else_body,
            finally_body,
            has_finally,
        }])
    }
}
