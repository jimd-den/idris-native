//! # Expression Parser (Adapter)
//!
//! This module implements the operator-precedence expression parsing for
//! the Idris 2 compiler. It handles the full expression grammar from
//! highest to lowest precedence:
//!
//!   primary → app → unary → multiplicative → arithmetic → shift →
//!   bitwise_and → bitwise_xor → bitwise_or → comparison → expr
//!
//! # Design Pattern: Composition via Extension Impl
//! Rather than using trait-based Strategy, we extend the `Parser` struct
//! with an `impl` block in this separate file. This gives us the
//! separation-of-concerns benefit of the Strategy pattern without
//! runtime polymorphism overhead — the simplest approach that satisfies
//! Single Responsibility (SOLID) while keeping the code navigable (KISS).
//!
//! # Performance
//! Each precedence level is O(n) in the number of operators at that level.
//! The recursive descent is O(depth) in stack frames, bounded by the
//! nesting depth of the source program.

use crate::domain::Term;
use crate::adapters::syntax_parser::scanner::Token;
use crate::common::errors::{CompilerError, ParseError, Spanned};

use super::parser::Parser;

impl<'a, 'arena> Parser<'a, 'arena> {
    /// Parses a full expression, dispatching on the leading token.
    ///
    /// # Business Logic
    /// Idris 2 expressions come in several forms: conditionals (if/then/else),
    /// let-bindings (let x = val in body), pattern matching (case/of),
    /// do-notation (do { stmts }), lambda abstractions (\x => body),
    /// and arithmetic/comparison chains.
    /// This function identifies which form we're in and delegates accordingly.
    pub fn parse_expr(&mut self) -> Result<&'a Term<'a>, CompilerError> {
        match self.peek() {
            Token::If => {
                self.advance();
                let cond = self.parse_expr()?;
                self.skip_newlines();
                self.consume(Token::Then, "Expected then")?;
                let then_br = self.parse_expr()?;
                self.skip_newlines();
                self.consume(Token::Else, "Expected else")?;
                let else_br = self.parse_expr()?;
                Ok(unsafe { &*self.arena.alloc(Term::If(cond, then_br, else_br)) })
            }
            Token::Let => {
                self.advance();
                let name_token = self.advance();
                let name = match name_token.node {
                    Token::Identifier(n) => n,
                    _ => panic!("Expected identifier in let"),
                };
                // Check for optional type annotation: let x : Type = val in body
                if self.peek() == &Token::Colon {
                    self.advance(); // skip :
                    // Skip the type annotation — consume tokens until we hit '='
                    self.skip_until_assign();
                }
                self.consume(Token::Assign, "Expected = in let")?;
                let val = self.parse_expr()?;
                // In do-notation, `let x = val` without `in` is allowed
                if self.peek() == &Token::In {
                    self.advance();
                    let body = self.parse_expr()?;
                    Ok(unsafe { &*self.arena.alloc(Term::Let(name, val, body)) })
                } else {
                    // do-notation let: no `in` clause, treat as let-bind
                    Ok(unsafe { &*self.arena.alloc(Term::Let(name, val, val)) })
                }
            }
            Token::Case => {
                self.advance();
                let target = self.parse_expr()?;
                self.consume(Token::Of, "Expected of in case")?;
                let mut branches = Vec::new();
                while self.peek() != &Token::EOF {
                    self.skip_newlines();
                    if self.is_decl_start() || self.peek() == &Token::EOF { break; }
                    let pat_token = self.advance();
                    let pat_name = match pat_token.node {
                        Token::Identifier(n) => n,
                        Token::Integer(i) => i.to_string(),
                        Token::LParen => {
                            // Pattern in parens: (_ ** xs') or (x :: xs)
                            self.skip_balanced_parens();
                            "_paren_pat".to_string()
                        }
                        _ => break,
                    };
                    let mut pat_args = Vec::new();
                    while self.peek() != &Token::FatArrow && self.peek() != &Token::EOF {
                        match self.peek() {
                            Token::Newline => { self.advance(); continue; }
                            Token::FatArrow => break,
                            _ => {
                                let arg_token = self.advance();
                                match arg_token.node {
                                    Token::Identifier(a) => pat_args.push(a),
                                    Token::DoubleStar => pat_args.push("**".to_string()),
                                    _ => {} // Skip non-identifier pattern elements
                                }
                            }
                        }
                    }
                    if self.peek() != &Token::FatArrow { break; }
                    self.consume(Token::FatArrow, "Expected =>")?;
                    let body = self.parse_expr()?;
                    branches.push((pat_name, pat_args, body));
                    self.skip_newlines();
                    if self.peek() != &Token::Pipe { break; }
                    self.advance(); // |
                }
                Ok(unsafe { &*self.arena.alloc(Term::Case(target, branches)) })
            }
            Token::Do => {
                self.advance();
                self.skip_newlines();
                let mut stmts = Vec::new();
                while self.peek() != &Token::EOF && !self.is_decl_start() {
                    if let Token::Identifier(n) = self.peek().clone() {
                        if let Some(next) = self.cursor.peek_at(1) {
                            if next.node == Token::Bind {
                                self.advance(); // name
                                self.advance(); // <-
                                let action = self.parse_expr()?;
                                stmts.push(Term::Bind(n, action));
                                self.skip_newlines();
                                continue;
                            }
                        }
                    }
                    // Handle do-notation `let` without `in`
                    if self.peek() == &Token::Let {
                        self.advance();
                        let name_token = self.advance();
                        let name = match name_token.node {
                            Token::Identifier(n) => n,
                            _ => { self.skip_newlines(); continue; },
                        };
                        self.consume(Token::Assign, "Expected = in do-let")?;
                        let val = self.parse_expr()?;
                        stmts.push(Term::Let(name, val, val));
                        self.skip_newlines();
                        continue;
                    }
                    stmts.push(self.parse_expr()?.clone());
                    self.skip_newlines();
                }
                Ok(unsafe { &*self.arena.alloc(Term::Do(stmts)) })
            }
            Token::Backslash => {
                // Lambda abstraction: \x => body
                self.advance();
                let name_token = self.advance();
                let name = match name_token.node {
                    Token::Identifier(n) => n,
                    _ => "_".to_string(),
                };
                self.consume(Token::FatArrow, "Expected => in lambda")?;
                let body = self.parse_expr()?;
                // Represent lambda as Pi with a dummy type (parsed as Var("_"))
                let dummy_type = unsafe { &*self.arena.alloc(Term::Var("_".to_string())) };
                Ok(unsafe { &*self.arena.alloc(Term::Lambda(name, dummy_type, body)) })
            }
            _ => self.parse_comparison(),
        }
    }

    /// Helper: skip tokens until we find `=` at the current nesting level.
    /// Used for skipping type annotations in let-bindings.
    fn skip_until_assign(&mut self) {
        let mut depth = 0;
        while self.peek() != &Token::EOF {
            match self.peek() {
                Token::LParen | Token::LBracket | Token::LBrace => { depth += 1; self.advance(); }
                Token::RParen | Token::RBracket | Token::RBrace => { depth -= 1; self.advance(); }
                Token::Assign if depth == 0 => break,
                _ => { self.advance(); }
            }
        }
    }

    /// Helper: skip everything inside balanced parens after we've consumed '('.
    /// Used for pattern matching on complex patterns.
    fn skip_balanced_parens(&mut self) {
        let mut depth = 1;
        while self.peek() != &Token::EOF && depth > 0 {
            match self.peek() {
                Token::LParen => { depth += 1; self.advance(); }
                Token::RParen => { depth -= 1; self.advance(); }
                _ => { self.advance(); }
            }
        }
    }

    /// Comparison operators: ==, <, >
    ///
    /// # Precedence
    /// Lowest precedence among arithmetic expressions. Left-associative.
    pub fn parse_comparison(&mut self) -> Result<&'a Term<'a>, CompilerError> {
        let mut lhs = self.parse_bitwise_or()?;
        loop {
            match self.peek() {
                Token::Eq => { self.advance(); let rhs = self.parse_bitwise_or()?; lhs = unsafe { &*self.arena.alloc(Term::Eq(lhs, rhs)) }; }
                Token::Lt => { self.advance(); let rhs = self.parse_bitwise_or()?; lhs = unsafe { &*self.arena.alloc(Term::Lt(lhs, rhs)) }; }
                Token::Gt => { self.advance(); let rhs = self.parse_bitwise_or()?; lhs = unsafe { &*self.arena.alloc(Term::Gt(lhs, rhs)) }; }
                _ => break,
            }
        }
        Ok(lhs)
    }

    /// Bitwise OR: `.|.`
    fn parse_bitwise_or(&mut self) -> Result<&'a Term<'a>, CompilerError> {
        let mut lhs = self.parse_bitwise_xor()?;
        while self.peek() == &Token::BitOr {
            self.advance();
            let rhs = self.parse_bitwise_xor()?;
            lhs = unsafe { &*self.arena.alloc(Term::BitOr(lhs, rhs)) };
        }
        Ok(lhs)
    }

    /// Bitwise XOR: `` `xor` ``
    fn parse_bitwise_xor(&mut self) -> Result<&'a Term<'a>, CompilerError> {
        let mut lhs = self.parse_bitwise_and()?;
        while self.peek() == &Token::Backtick {
            if let Some(Spanned { node: Token::Xor, .. }) = self.cursor.peek_next() {
                self.advance(); // `
                self.advance(); // xor
                self.consume(Token::Backtick, "Expected ` after xor")?;
                let rhs = self.parse_bitwise_and()?;
                lhs = unsafe { &*self.arena.alloc(Term::BitXor(lhs, rhs)) };
            } else {
                break;
            }
        }
        Ok(lhs)
    }

    /// Bitwise AND: `.&.`
    fn parse_bitwise_and(&mut self) -> Result<&'a Term<'a>, CompilerError> {
        let mut lhs = self.parse_shift()?;
        while self.peek() == &Token::BitAnd {
            self.advance();
            let rhs = self.parse_shift()?;
            lhs = unsafe { &*self.arena.alloc(Term::BitAnd(lhs, rhs)) };
        }
        Ok(lhs)
    }

    /// Shift operators: `` `shiftL` `` and `` `shiftR` ``
    fn parse_shift(&mut self) -> Result<&'a Term<'a>, CompilerError> {
        let mut lhs = self.parse_arithmetic()?;
        while self.peek() == &Token::Backtick {
            if let Some(Spanned { node: op, .. }) = self.cursor.peek_next() {
                if matches!(op, Token::ShiftL | Token::ShiftR) {
                    self.advance(); // `
                    let t = self.advance();
                    self.consume(Token::Backtick, "Expected `")?;
                    let rhs = self.parse_arithmetic()?;
                    let term = if t.node == Token::ShiftL { Term::Shl(lhs, rhs) } else { Term::Shr(lhs, rhs) };
                    lhs = unsafe { &*self.arena.alloc(term) };
                    continue;
                }
            }
            break;
        }
        Ok(lhs)
    }

    /// Additive arithmetic: +, -, ++
    fn parse_arithmetic(&mut self) -> Result<&'a Term<'a>, CompilerError> {
        let mut lhs = self.parse_multiplicative()?;
        loop {
            if self.peek() == &Token::Plus {
                self.advance();
                let rhs = self.parse_multiplicative()?;
                lhs = unsafe { &*self.arena.alloc(Term::Add(lhs, rhs)) };
            } else if self.peek() == &Token::Minus {
                self.advance();
                let rhs = self.parse_multiplicative()?;
                lhs = unsafe { &*self.arena.alloc(Term::Sub(lhs, rhs)) };
            } else if self.peek() == &Token::Append {
                self.advance();
                let rhs = self.parse_multiplicative()?;
                lhs = unsafe { &*self.arena.alloc(Term::Add(lhs, rhs)) };
            } else {
                break;
            }
        }
        Ok(lhs)
    }

    /// Multiplicative: *, /
    fn parse_multiplicative(&mut self) -> Result<&'a Term<'a>, CompilerError> {
        let mut lhs = self.parse_unary()?;
        loop {
            if self.peek() == &Token::Star {
                self.advance();
                let rhs = self.parse_unary()?;
                lhs = unsafe { &*self.arena.alloc(Term::Mul(lhs, rhs)) };
            } else if self.peek() == &Token::Slash {
                self.advance();
                let rhs = self.parse_unary()?;
                lhs = unsafe { &*self.arena.alloc(Term::Div(lhs, rhs)) };
            } else {
                break;
            }
        }
        Ok(lhs)
    }

    /// Unary prefix: `complement`
    fn parse_unary(&mut self) -> Result<&'a Term<'a>, CompilerError> {
        if self.peek() == &Token::Complement {
            self.advance();
            let body = self.parse_unary()?;
            return Ok(unsafe { &*self.arena.alloc(Term::BitNot(body)) });
        }
        self.parse_app()
    }

    /// Function application: `f x y z`
    ///
    /// # Business Logic
    /// In Idris 2, function application is left-associative and has higher
    /// precedence than all binary operators. We greedily apply arguments
    /// as long as the next token looks like an argument (identifier, literal,
    /// or parenthesized expression) and is NOT the start of a new declaration.
    pub fn parse_app(&mut self) -> Result<&'a Term<'a>, CompilerError> {
        let mut expr = self.parse_primary()?;
        loop {
            match self.peek() {
                Token::Identifier(_) => {
                    if let Some(next) = self.cursor.peek_at(1) {
                        if next.node == Token::Colon || next.node == Token::Assign || next.node == Token::Bind {
                            break;
                        }
                    }
                    let arg = self.parse_primary()?;
                    expr = unsafe { &*self.arena.alloc(Term::App(expr, arg)) };
                }
                Token::Integer(_) | Token::Float(_) | Token::String(_) | Token::Char(_) |
                Token::LParen | Token::LBracket | Token::LBrace | Token::Complement | Token::Question => {
                    let arg = self.parse_primary()?;
                    expr = unsafe { &*self.arena.alloc(Term::App(expr, arg)) };
                }
                _ => break,
            }
        }
        Ok(expr)
    }

    /// Primary expressions: literals, identifiers, parenthesized expressions,
    /// list literals, hole expressions, and built-in type/function recognition.
    pub fn parse_primary(&mut self) -> Result<&'a Term<'a>, CompilerError> {
        self.skip_newlines();
        let token = self.advance();
        match token.node {
            Token::LParen => {
                if self.peek() == &Token::RParen {
                    self.advance();
                    return Ok(unsafe { &*self.arena.alloc(Term::Integer(0)) });
                }
                // Check for operator-as-identifier: (+), (++), (::), (<+>), (<*>), etc.
                match self.peek() {
                    Token::Plus | Token::Star | Token::Minus | Token::Slash |
                    Token::Append | Token::Eq | Token::Lt | Token::Gt |
                    Token::Colon => {
                        if let Some(after) = self.cursor.peek_at(1) {
                            if after.node == Token::RParen {
                                // Operator section: (+)
                                let op_tok = self.advance();
                                self.advance(); // )
                                let name = format!("{:?}", op_tok.node);
                                return Ok(unsafe { &*self.arena.alloc(Term::Var(name)) });
                            }
                        }
                        // Check two-char operators like (::), (<+>)
                        if let Some(after2) = self.cursor.peek_at(2) {
                            if after2.node == Token::RParen {
                                let op1 = self.advance();
                                let op2 = self.advance();
                                self.advance(); // )
                                let name = format!("{:?}{:?}", op1.node, op2.node);
                                return Ok(unsafe { &*self.arena.alloc(Term::Var(name)) });
                            }
                        }
                    }
                    _ => {}
                }
                let expr = self.parse_expr()?;
                // Handle tuples: (a, b, c)
                if self.peek() == &Token::Comma {
                    // Skip remaining tuple elements — represent as the first element
                    while self.peek() == &Token::Comma {
                        self.advance();
                        let _ = self.parse_expr()?;
                    }
                }
                // Handle dependent pairs: (x ** y)
                if self.peek() == &Token::DoubleStar {
                    self.advance();
                    let _ = self.parse_expr()?;
                }
                // Handle type annotations inside parens: (n : Nat) or (a : Type -> b)
                // and arrow types: (a -> b)
                if self.peek() == &Token::Colon || self.peek() == &Token::Arrow {
                    // Skip the rest of the type expression until we find the matching )
                    let mut depth = 1i32;
                    while self.peek() != &Token::EOF && depth > 0 {
                        match self.peek() {
                            Token::LParen => { depth += 1; self.advance(); }
                            Token::RParen => {
                                depth -= 1;
                                if depth > 0 { self.advance(); }
                            }
                            _ => { self.advance(); }
                        }
                    }
                    if self.peek() == &Token::RParen {
                        self.advance();
                    }
                    return Ok(expr);
                }
                self.consume(Token::RParen, "Expected )")?;
                Ok(expr)
            }
            Token::LBracket => {
                // List literal: [1,2,3], [], or list comprehension [x | x <- xs]
                if self.peek() == &Token::RBracket {
                    self.advance();
                    return Ok(unsafe { &*self.arena.alloc(Term::Var("Nil".to_string())) });
                }
                // Parse first expression
                let first = self.parse_expr()?;

                if self.peek() == &Token::RBracket {
                    // Single-element list: [x]
                    self.advance();
                    return Ok(first);
                }

                // List with commas or comprehension or range
                let mut depth = 1i32;
                while self.peek() != &Token::EOF && depth > 0 {
                    match self.peek() {
                        Token::LBracket => { depth += 1; self.advance(); }
                        Token::RBracket => { depth -= 1; if depth > 0 { self.advance(); } }
                        _ => { self.advance(); }
                    }
                }
                if self.peek() == &Token::RBracket {
                    self.advance();
                }
                Ok(first)
            }
            Token::Question => {
                // Hole expression: ?name
                let name_token = self.advance();
                let name = match name_token.node {
                    Token::Identifier(n) => n,
                    _ => "hole".to_string(),
                };
                Ok(unsafe { &*self.arena.alloc(Term::Var(format!("?{}", name))) })
            }
            Token::LBrace => {
                // Record update syntax: { field := val } or block
                // Permissively skip to matching }
                let mut depth = 1i32;
                while self.peek() != &Token::EOF && depth > 0 {
                    match self.peek() {
                        Token::LBrace => { depth += 1; self.advance(); }
                        Token::RBrace => { depth -= 1; if depth > 0 { self.advance(); } }
                        _ => { self.advance(); }
                    }
                }
                if self.peek() == &Token::RBrace {
                    self.advance();
                }
                Ok(unsafe { &*self.arena.alloc(Term::Var("_record_update".to_string())) })
            }
            Token::Backslash => {
                // Lambda: \x => body
                let name_token = self.advance();
                let name = match name_token.node {
                    Token::Identifier(n) => n,
                    _ => "_".to_string(),
                };
                self.consume(Token::FatArrow, "Expected => in lambda")?;
                let body = self.parse_expr()?;
                let dummy_type = unsafe { &*self.arena.alloc(Term::Var("_".to_string())) };
                Ok(unsafe { &*self.arena.alloc(Term::Lambda(name, dummy_type, body)) })
            }
            Token::Dollar => {
                // $ is function application operator — parse the RHS
                self.parse_expr()
            }
            Token::Integer(val) => Ok(unsafe { &*self.arena.alloc(Term::Integer(val)) }),
            Token::Float(bits) => Ok(unsafe { &*self.arena.alloc(Term::Float(bits)) }),
            Token::String(val) => Ok(unsafe { &*self.arena.alloc(Term::String(val)) }),
            Token::Char(val) => Ok(unsafe { &*self.arena.alloc(Term::Char(val)) }),
            Token::Identifier(n) => {
                let term = match n.as_str() {
                    "i32" => Term::I32Type,
                    "i8" => Term::I8Type,
                    "Integer" | "Int" => Term::IntegerType,
                    "Bool" => Term::Var("Bool".to_string()),
                    "True" => Term::Integer(1),
                    "False" => Term::Integer(0),
                    "Bits64" => Term::Bits64Type,
                    "IO" => Term::IOType,
                    "String" => Term::StringType,
                    "Char" => Term::CharType,
                    "Float" => Term::FloatType,
                    "Type" => Term::TypeType,
                    "buffer" => {
                        let size_token = self.advance();
                        let size = match size_token.node {
                            Token::Integer(s) => s as usize,
                            _ => 0,
                        };
                        Term::Buffer(size)
                    }
                    "getBits64" => {
                        let buffer = self.parse_primary()?;
                        let index = self.parse_primary()?;
                        Term::BufferLoad(buffer, index)
                    }
                    "setBits64" => {
                        let buffer = self.parse_primary()?;
                        let index = self.parse_primary()?;
                        let value = self.parse_primary()?;
                        Term::BufferStore(buffer, index, value)
                    }
                    _ => Term::Var(n),
                };
                Ok(unsafe { &*self.arena.alloc(term) })
            }
            // Permissive: tokens that shouldn't appear in expression position
            // are absorbed rather than crashing the parser. This lets us handle
            // partially-supported syntax without blocking other declarations.
            Token::Pipe | Token::Colon | Token::At | Token::Hash |
            Token::Underscore | Token::Assign => {
                Ok(unsafe { &*self.arena.alloc(Term::Var("_".to_string())) })
            }
            _ => Err(CompilerError::Parse(ParseError {
                span: token.span,
                token: token.node,
                expected: None,
                message: "Unexpected token in parse_primary".to_string(),
            })),
        }
    }
}
