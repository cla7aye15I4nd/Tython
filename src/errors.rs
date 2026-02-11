use std::path::Path;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCategory {
    TypeError,
    NameError,
    SyntaxError,
    ValueError,
    AttributeError,
}

impl std::fmt::Display for ErrorCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorCategory::TypeError => write!(f, "TypeError"),
            ErrorCategory::NameError => write!(f, "NameError"),
            ErrorCategory::SyntaxError => write!(f, "SyntaxError"),
            ErrorCategory::ValueError => write!(f, "ValueError"),
            ErrorCategory::AttributeError => write!(f, "AttributeError"),
        }
    }
}

#[derive(Debug, Error)]
#[error("{category}: {message}")]
pub struct TythonError {
    pub category: ErrorCategory,
    pub message: String,
    pub file: String,
    pub line: usize,
    pub source_line: Option<String>,
    pub function_name: Option<String>,
}

pub fn print_error(file: &Path, err: &anyhow::Error) {
    if let Some(te) = err.chain().find_map(|e| e.downcast_ref::<TythonError>()) {
        print_tython_error(te);
    } else {
        let message = err.chain().last().unwrap().to_string();
        eprintln!("error: {}", message);
        eprintln!("  --> {}", file.display());
    }
}

fn print_tython_error(te: &TythonError) {
    let line_num = te.line.to_string();
    let pad = line_num.len();

    eprintln!("{}: {}", te.category, te.message);

    eprint!(" {:>pad$} --> ", "", pad = pad);
    eprint!("{}:{}", te.file, te.line);
    if let Some(ref func) = te.function_name {
        eprint!(", in {}", func);
    }
    eprintln!();

    if let Some(ref src) = te.source_line {
        let trimmed = src.trim();
        if !trimmed.is_empty() {
            eprintln!(" {:>pad$} |", "", pad = pad);
            eprintln!(" {} |   {}", line_num, trimmed);
            eprintln!(" {:>pad$} |", "", pad = pad);
        }
    }
}
