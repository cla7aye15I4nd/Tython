use std::io::IsTerminal;
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
    let c = std::io::stderr().is_terminal();

    if let Some(te) = err.chain().find_map(|e| e.downcast_ref::<TythonError>()) {
        print_tython_error(te, c);
    } else {
        let message = err.chain().last().unwrap().to_string();
        if c {
            eprintln!("\x1b[1;31merror\x1b[0m\x1b[1m: {}\x1b[0m", message);
            eprintln!("  \x1b[1;34m-->\x1b[0m {}", file.display());
        } else {
            eprintln!("error: {}", message);
            eprintln!("  --> {}", file.display());
        }
    }
}

fn print_tython_error(te: &TythonError, c: bool) {
    let line_num = te.line.to_string();
    let pad = line_num.len();

    if c {
        eprintln!(
            "\x1b[1;31m{}\x1b[0m\x1b[1m: {}\x1b[0m",
            te.category, te.message
        );
    } else {
        eprintln!("{}: {}", te.category, te.message);
    }

    if c {
        eprint!(" {:>pad$} \x1b[1;34m-->\x1b[0m ", "", pad = pad);
    } else {
        eprint!(" {:>pad$} --> ", "", pad = pad);
    }
    eprint!("{}:{}", te.file, te.line);
    if let Some(ref func) = te.function_name {
        eprint!(", in {}", func);
    }
    eprintln!();

    if let Some(ref src) = te.source_line {
        let trimmed = src.trim();
        if !trimmed.is_empty() {
            if c {
                eprintln!(" {:>pad$} \x1b[1;34m|\x1b[0m", "", pad = pad);
                eprintln!(
                    " \x1b[1;34m{}\x1b[0m \x1b[1;34m|\x1b[0m   {}",
                    line_num, trimmed
                );
                eprintln!(" {:>pad$} \x1b[1;34m|\x1b[0m", "", pad = pad);
            } else {
                eprintln!(" {:>pad$} |", "", pad = pad);
                eprintln!(" {} |   {}", line_num, trimmed);
                eprintln!(" {:>pad$} |", "", pad = pad);
            }
        }
    }
}
