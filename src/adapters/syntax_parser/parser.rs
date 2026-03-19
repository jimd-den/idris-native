//! # Parser Orchestrator (Adapter)
//!
//! This module defines the `Parser` struct — the shared state for parsing
//! Idris 2 source code. The actual parsing logic is decomposed into three
//! focused sub-modules that extend `Parser` via separate `impl` blocks:
//!
//! - `expression_parser` — operator precedence and expression forms
//! - `type_parser` — Pi types, signatures, and QTT multiplicity
//! - `declaration_parser` — top-level declarations (module, data, etc.)
//!
//! # Design Pattern: Composition via Extension Impl
//! Each sub-parser file adds methods to `Parser` without trait indirection.
//! This is the Composition pattern applied at compile time — the simplest
//! thing that satisfies Single Responsibility (SOLID) while keeping the
//! cursor and arena accessible to all grammar rules (KISS).
//!
//! # Strategic Architecture
//! As an Adapter, the parser converts the raw token stream from the Scanner
//! into domain-layer `Term` entities, bridging external syntax with internal
//! representation.

use crate::domain::Term;
use crate::adapters::syntax_parser::scanner::Token;
use crate::common::cursor::Cursor;
use crate::common::errors::{CompilerError, ParseError, Span, Spanned};
use crate::domain::arena::Arena;

/// The shared parsing state.
///
/// # Fields
/// - `cursor` — a `Cursor` over the spanned token stream, providing
///   peek, advance, and multi-token lookahead.
/// - `arena` — the memory arena where all `Term` nodes are allocated,
///   ensuring cache locality and O(1) bulk deallocation.
pub struct Parser<'a, 'arena> {
    pub(super) cursor: Cursor<Spanned<Token>>,
    pub(super) arena: &'arena mut Arena<Term<'a>>,
}

impl<'a, 'arena> Parser<'a, 'arena> {
    /// Creates a new parser from a token stream and an arena allocator.
    pub fn new(tokens: Vec<Spanned<Token>>, arena: &'arena mut Arena<Term<'a>>) -> Self {
        Self {
            cursor: Cursor::new(tokens),
            arena,
        }
    }

    // ── Shared Cursor Utilities ─────────────────────────────────────
    // These are used by all three sub-parser modules.

    /// Returns the current token without advancing.
    pub fn peek(&self) -> &Token {
        match self.cursor.peek() {
            Some(s) => &s.node,
            None => &Token::EOF,
        }
    }

    /// Returns the span of the current token.
    pub(super) fn peek_span(&self) -> Span {
        match self.cursor.peek() {
            Some(s) => s.span,
            None => Span::new(0, 0, 0),
        }
    }

    /// Advances the cursor by one token and returns the consumed token.
    pub(super) fn advance(&mut self) -> Spanned<Token> {
        let t = self.cursor.advance().cloned().unwrap_or_else(|| {
            Spanned::new(Token::EOF, Span::new(0, 0, 0))
        });
        t
    }

    /// Consumes a specific token or returns a structured error.
    ///
    /// Skips leading newlines before checking, since Idris 2 allows
    /// flexible whitespace between tokens in many positions.
    pub(super) fn consume(&mut self, token: Token, message: &str) -> Result<Spanned<Token>, CompilerError> {
        self.skip_newlines();
        if self.peek() == &token {
            return Ok(self.advance());
        }
        Err(CompilerError::Parse(ParseError {
            span: self.peek_span(),
            token: self.peek().clone(),
            expected: Some(format!("{:?}", token)),
            message: message.to_string(),
        }))
    }

    /// Skips any sequence of newline tokens.
    pub(super) fn skip_newlines(&mut self) {
        while self.peek() == &Token::Newline {
            self.advance();
        }
    }

    // ── Program Entry Point ─────────────────────────────────────────

    /// Entry point for parsing a full program (multiple declarations).
    ///
    /// # Business Logic
    /// A program is a sequence of top-level declarations separated by
    /// newlines. We keep consuming declarations until we hit EOF.
    /// The actual declaration parsing is delegated to `declaration_parser`.
    pub fn parse_program(&mut self) -> Result<Vec<Term<'a>>, CompilerError> {
        let _span = crate::trace_span!("PARSER", "parse_program");
        let mut declarations = Vec::new();

        while self.peek() != &Token::EOF {
            self.skip_newlines();
            if self.peek() == &Token::EOF { break; }
            declarations.push(self.parse_declaration()?);
        }

        Ok(declarations)
    }
}
