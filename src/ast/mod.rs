pub mod convert;
pub mod expr;
pub mod stmt;
pub mod types;

pub use expr::{BinOpKind, Expr, ExprKind, Span};
pub use stmt::{FunctionParam, Stmt, StmtKind};
pub use types::Type;

use std::path::PathBuf;

/// A Tython module (single Python file)
#[derive(Debug, Clone)]
pub struct Module {
    pub path: PathBuf,
    pub body: Vec<Stmt>,
}

impl Module {
    pub fn new(path: PathBuf, body: Vec<Stmt>) -> Self {
        Self { path, body }
    }

    /// If there are module-level statements and no main(), wrap them in a synthetic main()
    pub fn ensure_main(&mut self) {
        let has_main = self
            .body
            .iter()
            .any(|stmt| matches!(&stmt.kind, StmtKind::FunctionDef { name, .. } if name == "main"));

        if has_main {
            return; // Already has main
        }

        // Collect module-level statements (non-functions)
        let mut module_level_stmts = Vec::new();
        let mut functions = Vec::new();

        for stmt in std::mem::take(&mut self.body) {
            match &stmt.kind {
                StmtKind::FunctionDef { .. } => functions.push(stmt),
                _ => module_level_stmts.push(stmt),
            }
        }

        if module_level_stmts.is_empty() {
            // No module-level code, just restore functions
            self.body = functions;
            return;
        }

        // Create synthetic main() function
        module_level_stmts.push(Stmt::new(
            StmtKind::Return(Some(Expr::new(ExprKind::IntLiteral(0), Span::new(0, 0)))),
            Span::new(0, 0),
        ));

        let main_func = Stmt::new(
            StmtKind::FunctionDef {
                name: "main".to_string(),
                params: Vec::new(),
                return_type: Type::Int,
                body: module_level_stmts,
            },
            Span::new(0, 0),
        );

        // Rebuild body with functions + synthetic main
        self.body = functions;
        self.body.push(main_func);
    }
}
