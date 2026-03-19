//! # Type Parser (Adapter)
//!
//! This module implements type-signature and Pi-type parsing for the
//! Idris 2 compiler. It handles:
//!
//! - Type signatures: `name : Type`
//! - Pi types with QTT multiplicity: `(1 x : Integer) -> Integer`
//! - Named Pi binders: `(n : Nat) -> Type`
//! - Anonymous arrow types: `Integer -> Integer`
//! - Parenthesized type expressions: `(a -> Bool)`
//!
//! # Design Pattern: Composition via Extension Impl
//! Like `expression_parser`, this extends `Parser` with an `impl` block
//! in a separate file. This is the Strategy pattern applied at compile time
//! — each file handles one grammar region without runtime dispatch overhead.
//!
//! # Business Logic
//! Pi types are the backbone of Idris 2's dependent type system. Each Pi
//! binder carries a QTT multiplicity (0 = erased, 1 = linear, ω = many)
//! that governs how many times the bound variable may be used at runtime.

use crate::domain::{Term, multiplicity::Multiplicity};
use crate::adapters::syntax_parser::scanner::Token;
use crate::common::errors::{CompilerError, ParseError, Spanned};

use super::parser::Parser;

impl<'a, 'arena> Parser<'a, 'arena> {
    /// Parses a top-level type signature: `name : type_expression`.
    ///
    /// Returns both the name and the parsed type term so the caller
    /// can pair signatures with their corresponding definitions.
    pub fn parse_signature(&mut self) -> Result<(String, &'a Term<'a>), CompilerError> {
        let name_token = self.advance();
        let name = match name_token.node {
            Token::Identifier(n) => n,
            t => return Err(CompilerError::Parse(ParseError {
                span: name_token.span,
                token: t,
                expected: Some("Identifier".to_string()),
                message: "Expected identifier in signature".to_string(),
            })),
        };
        self.consume(Token::Colon, "Expected : after name in signature")?;
        let sig = self.parse_pi()?;
        Ok((name, sig))
    }

    /// Parses a Pi type (dependent function type).
    ///
    /// # Grammar
    /// ```text
    /// pi_type = '(' quantity name ':' type ')' '->' pi_type   -- QTT Pi
    ///         | '(' name ':' type ')' '->' pi_type            -- named Pi
    ///         | '(' type ')' ['->' pi_type]                   -- parenthesized type
    ///         | app_type '->' pi_type                         -- anonymous arrow
    ///         | app_type                                      -- base case
    /// ```
    ///
    /// # Business Logic
    /// When we see `(`, we must disambiguate between:
    /// 1. A QTT Pi binder: `(1 x : Integer)` — digit followed by name and `:`
    /// 2. A named Pi binder: `(n : Nat)` — identifier followed by `:`
    /// 3. A parenthesized type: `(a -> Bool)` — contains arrows
    /// 4. A plain grouping: `(Integer)` — just a sub-expression
    ///
    /// We use multi-token lookahead to disambiguate.
    pub fn parse_pi(&mut self) -> Result<&'a Term<'a>, CompilerError> {
        let mut lhs = if self.peek() == &Token::LParen {
            // Peek ahead to decide which paren form we have
            let paren_kind = self.classify_paren_contents();

            match paren_kind {
                ParenKind::QttPi(quantity) => {
                    self.advance(); // (
                    self.advance(); // quantity digit
                    let name_token = self.advance();
                    let name = match name_token.node {
                        Token::Identifier(n) => n,
                        _ => "_".to_string(),
                    };
                    self.consume(Token::Colon, "Expected : in Pi binder")?;
                    let ty = self.parse_pi()?;
                    self.consume(Token::RParen, "Expected ) in Pi binder")?;
                    if self.peek() == &Token::Arrow {
                        self.advance();
                        let body = self.parse_pi()?;
                        return Ok(unsafe { &*self.arena.alloc(Term::Pi(name, quantity, ty, body)) });
                    }
                    ty
                }
                ParenKind::NamedPi => {
                    // (name : Type) -> body
                    self.advance(); // (
                    let name_token = self.advance();
                    let name = match name_token.node {
                        Token::Identifier(n) => n,
                        _ => "_".to_string(),
                    };
                    self.consume(Token::Colon, "Expected : in named Pi binder")?;
                    let ty = self.parse_pi_inner()?;
                    self.consume(Token::RParen, "Expected ) in named Pi binder")?;
                    if self.peek() == &Token::Arrow {
                        self.advance();
                        let body = self.parse_pi()?;
                        return Ok(unsafe { &*self.arena.alloc(Term::Pi(name, Multiplicity::Many, ty, body)) });
                    }
                    ty
                }
                ParenKind::GroupedExpr => {
                    // Parenthesized type expression
                    self.advance(); // (
                    let expr = self.parse_pi_inner()?;
                    self.consume(Token::RParen, "Expected )")?;
                    expr
                }
            }
        } else {
            self.parse_app()?
        };

        if self.peek() == &Token::Arrow {
            self.advance();
            let rhs = self.parse_pi()?;
            lhs = unsafe { &*self.arena.alloc(Term::Pi("_".to_string(), Multiplicity::Many, lhs, rhs)) };
        }
        Ok(lhs)
    }

    /// Parses a type expression inside parentheses, where `->` and `:` are valid.
    /// This is like `parse_pi` but also handles infix operators and stops at `)`.
    fn parse_pi_inner(&mut self) -> Result<&'a Term<'a>, CompilerError> {
        let mut lhs = self.parse_app()?;

        // Handle arrows inside parens: (a -> Bool)
        if self.peek() == &Token::Arrow {
            self.advance();
            let rhs = self.parse_pi_inner()?;
            lhs = unsafe { &*self.arena.alloc(Term::Pi("_".to_string(), Multiplicity::Many, lhs, rhs)) };
        }

        // Handle dependent pairs: (x ** Vect x t)
        if self.peek() == &Token::DoubleStar {
            self.advance();
            let _ = self.parse_pi_inner()?;
        }

        // Handle commas (tuples): (a, b)
        while self.peek() == &Token::Comma {
            self.advance();
            let _ = self.parse_pi_inner()?;
        }

        // Handle `=` in types (equality type): Z = S n
        if self.peek() == &Token::Assign {
            self.advance();
            let rhs = self.parse_pi_inner()?;
            lhs = unsafe { &*self.arena.alloc(Term::Eq(lhs, rhs)) };
        }

        Ok(lhs)
    }

    /// Classifies what kind of parenthesized expression we're looking at,
    /// using multi-token lookahead without consuming any tokens.
    ///
    /// # Lookahead patterns
    /// - `( 0 name : ` → QTT Pi with Zero
    /// - `( 1 name : ` → QTT Pi with One
    /// - `( name : `   → Named Pi
    /// - Everything else → Grouped expression
    fn classify_paren_contents(&self) -> ParenKind {
        // Check position 1 (first token after `(`)
        let pos1 = self.cursor.peek_at(1);

        if let Some(Spanned { node: Token::Integer(q), .. }) = pos1 {
            let q_val = *q;
            if q_val == 0 || q_val == 1 {
                // Check if position 2 is an identifier and position 3 is ':'
                if let Some(Spanned { node: Token::Identifier(_), .. }) = self.cursor.peek_at(2) {
                    if let Some(Spanned { node: Token::Colon, .. }) = self.cursor.peek_at(3) {
                        let mult = if q_val == 0 { Multiplicity::Zero } else { Multiplicity::One };
                        return ParenKind::QttPi(mult);
                    }
                }
            }
        }

        // Check for named Pi: ( identifier : ... )
        if let Some(Spanned { node: Token::Identifier(_), .. }) = pos1 {
            if let Some(Spanned { node: Token::Colon, .. }) = self.cursor.peek_at(2) {
                return ParenKind::NamedPi;
            }
        }

        ParenKind::GroupedExpr
    }
}

/// Classification of parenthesized forms in type position.
///
/// # Business Logic
/// Idris 2 uses parentheses for three distinct purposes in type syntax:
/// 1. QTT-annotated Pi: `(1 x : Type) -> ...`
/// 2. Named-but-unrestricted Pi: `(n : Nat) -> ...`
/// 3. Grouping: `(a -> Bool)` for clarity
enum ParenKind {
    QttPi(Multiplicity),
    NamedPi,
    GroupedExpr,
}
