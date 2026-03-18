//! # Diagnostics (Adapter)
//!
//! This module implements user-friendly diagnostic reporting, 
//! translating compiler and type-checking errors into Idris-style output.
//!
//! # Strategic Architecture
//! `diagnostics` is an Adapter responsible for formatting internal 
//! domain errors for human consumption, ensuring a consistent and 
//! helpful developer experience.

use crate::common::errors::{CompilerError, Span};
use std::time::{SystemTime, UNIX_EPOCH};

/// Logs a message with a timestamp for telemetry.
pub fn log(tag: &str, message: &str) {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    
    // We use a simple seconds-since-epoch for now to avoid external dependencies like chrono.
    println!("[{}] {}: {}", now, tag, message);
}

/// Renders a compiler error in a detailed Idris 2 style with carets and hints.
pub fn report_error(error: &CompilerError, source: &str, filename: &str) {
    match error {
        CompilerError::Lex(e) => render_snippet(filename, source, e.span, &e.message, None),
        CompilerError::Parse(e) => render_snippet(filename, source, e.span, &e.message, e.expected.as_deref()),
        CompilerError::Qtt(e) => {
            let message = format!(
                "Variable '{}' declared {:?} but used {} times",
                e.variable, e.declared, e.actual
            );
            render_snippet(filename, source, e.span, &message, e.hint.as_deref());
        }
    }
}

fn render_snippet(filename: &str, source: &str, span: Span, message: &str, hint: Option<&str>) {
    println!("Error: {}", message);
    println!("  --> {}:{}:{}", filename, span.line, span.col);
    
    let lines: Vec<&str> = source.lines().collect();
    if let Some(line_text) = lines.get(span.line.saturating_sub(1)) {
        println!("{:>4} | {}", span.line, line_text);
        let padding = " ".repeat(span.col.saturating_sub(1));
        let carets = "^".repeat(span.len.max(1));
        println!("     | {}{}", padding, carets);
    }
    
    if let Some(h) = hint {
        println!("     | Hint: {}", h);
    }
    println!();
}
