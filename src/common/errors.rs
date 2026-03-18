//! # Compiler Error Types
//!
//! This module defines the structured error types for all stages of 
//! the Idris Native compiler pipeline.

use crate::adapters::syntax_parser::scanner::Token;
use crate::domain::multiplicity::Multiplicity;

/// Represents a source location (line, column, and length).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub line: usize,
    pub col: usize,
    pub len: usize,
}

impl Span {
    pub fn new(line: usize, col: usize, len: usize) -> Self {
        Self { line, col, len }
    }
}

/// Errors occurring during lexical analysis (Scanning).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexError {
    pub span: Span,
    pub character: char,
    pub message: String,
}

/// Errors occurring during syntactic analysis (Parsing).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub span: Span,
    pub token: Token,
    pub expected: Option<String>,
    pub message: String,
}

/// Errors occurring during QTT multiplicity and boundary checking.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QttError {
    pub span: Span,
    pub variable: String,
    pub declared: Multiplicity,
    pub actual: usize,
    pub context: String,
    pub hint: Option<String>,
}

/// Unified error type for the entire compiler pipeline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompilerError {
    Lex(LexError),
    Parse(ParseError),
    Qtt(QttError),
}

/// A wrapper that attaches a source span to any value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Spanned<T> {
    pub node: T,
    pub span: Span,
}

impl<T> Spanned<T> {
    pub fn new(node: T, span: Span) -> Self {
        Self { node, span }
    }
}
