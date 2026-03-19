//! # Syntax Parser (Adapter)
//!
//! This module orchestrates the lexical analysis and parsing of Idris 2 source.
//!
//! # Strategic Architecture
//! As an Adapter, this module decouples the core domain from the 
//! raw source text. It delegates to the `Scanner` for tokenization 
//! and the `Parser` for AST construction.

pub mod scanner;
pub mod parser;
pub mod expression_parser;
pub mod type_parser;
pub mod declaration_parser;

pub use scanner::{Scanner, Token};
pub use parser::Parser;
use crate::common::errors::{CompilerError, Spanned};

/// Higher-level lex helper for backward compatibility and simplicity.
pub fn lex(input: &str) -> Result<Vec<Spanned<Token>>, CompilerError> {
    let mut scanner = Scanner::new(input);
    scanner.scan_tokens()
}

#[cfg(test)]
mod tests {
    pub mod sha256_syntax_tests;
    pub mod error_tests;
    pub mod qtt_syntax_tests;
    pub mod full_lang_tests;
    pub mod sample_parse_tests;
}
