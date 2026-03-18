//! # Parser (Adapter)
//!
//! This module implements the Recursive Descent parser for the Idris 2 compiler.
//!
//! # Strategic Architecture
//! As an Adapter, the `Parser` translates a stream of `Token`s into our 
//! internal `Term` (Entities). It encapsulates the syntax rules and 
//! ensures that the resulting AST is structurally sound.

use crate::domain::{Term, arena::Arena, multiplicity::Multiplicity};
use crate::adapters::syntax_parser::scanner::Token;
use crate::common::cursor::Cursor;
use crate::common::errors::{CompilerError, ParseError, Span, Spanned};

pub struct Parser<'a, 'arena> {
    cursor: Cursor<Spanned<Token>>,
    arena: &'arena mut Arena<Term<'a>>,
}

impl<'a, 'arena> Parser<'a, 'arena> {
    pub fn new(tokens: Vec<Spanned<Token>>, arena: &'arena mut Arena<Term<'a>>) -> Self {
        Self {
            cursor: Cursor::new(tokens),
            arena,
        }
    }

    pub fn peek(&self) -> &Token {
        match self.cursor.peek() {
            Some(s) => &s.node,
            None => &Token::EOF,
        }
    }

    fn peek_span(&self) -> Span {
        match self.cursor.peek() {
            Some(s) => s.span,
            None => Span::new(0, 0, 0),
        }
    }

    fn advance(&mut self) -> Spanned<Token> {
        self.cursor.advance().cloned().unwrap_or_else(|| {
            Spanned::new(Token::EOF, Span::new(0, 0, 0))
        })
    }

    fn consume(&mut self, token: Token, message: &str) -> Result<Spanned<Token>, CompilerError> {
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

    /// Entry point for parsing a full program (signature + definition).
    pub fn parse_program(&mut self) -> Result<(String, &'a Term<'a>, &'a Term<'a>, Vec<String>), CompilerError> {
        let _span = crate::trace_span!("PARSER", "parse_program");
        
        let (name_sig, sig) = self.parse_signature()?;
        let (body, name_def, args) = self.parse_def()?;
        
        if name_sig != name_def {
            return Err(CompilerError::Parse(ParseError {
                span: self.peek_span(), 
                token: Token::Identifier(name_def.clone()),
                expected: Some(name_sig.clone()),
                message: format!("Name mismatch: {} vs {}", name_sig, name_def),
            }));
        }
        
        Ok((name_sig, sig, body, args))
    }

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

    fn parse_pi(&mut self) -> Result<&'a Term<'a>, CompilerError> {
        let mut lhs = if self.peek() == &Token::LParen {
            let mut is_multiplicity = false;
            let mut quantity = Multiplicity::Many;
            
            // Check if it's (q x : type)
            if let Some(Spanned { node: Token::Integer(q), .. }) = self.cursor.peek_next() {
                if *q == 0 || *q == 1 {
                    is_multiplicity = true;
                    quantity = if *q == 0 { Multiplicity::Zero } else { Multiplicity::One };
                }
            }

            if is_multiplicity {
                self.advance(); // (
                self.advance(); // quantity
                let name_token = self.advance();
                let name = match name_token.node {
                    Token::Identifier(n) => n,
                    t => return Err(CompilerError::Parse(ParseError {
                        span: name_token.span,
                        token: t,
                        expected: Some("Identifier".to_string()),
                        message: "Expected identifier in Pi".to_string(),
                    })),
                };
                self.consume(Token::Colon, "Expected : in Pi binder")?;
                let ty = self.parse_expr()?;
                self.consume(Token::RParen, "Expected ) in Pi binder")?;
                if self.peek() == &Token::Arrow {
                    self.advance();
                    let body = self.parse_pi()?;
                    return Ok(unsafe { &*self.arena.alloc(Term::Pi(name, quantity, ty, body)) });
                }
                ty
            } else {
                self.advance(); // (
                let expr = self.parse_expr()?;
                self.consume(Token::RParen, "Expected )")?;
                expr
            }
        } else {
            self.parse_primary()?
        };

        if self.peek() == &Token::Arrow {
            self.advance();
            let rhs = self.parse_pi()?;
            lhs = unsafe { &*self.arena.alloc(Term::Pi("_".to_string(), Multiplicity::Many, lhs, rhs)) };
        }
        Ok(lhs)
    }

    pub fn parse_def(&mut self) -> Result<(&'a Term<'a>, String, Vec<String>), CompilerError> {
        let name_token = self.advance();
        let name = match name_token.node {
            Token::Identifier(n) => n,
            t => return Err(CompilerError::Parse(ParseError {
                span: name_token.span,
                token: t,
                expected: Some("Identifier".to_string()),
                message: "Expected identifier in definition".to_string(),
            })),
        };
        let mut args = Vec::new();
        while self.peek() != &Token::Assign && self.peek() != &Token::EOF {
            let arg_token = self.advance();
            match arg_token.node {
                Token::Identifier(arg) => args.push(arg),
                t => return Err(CompilerError::Parse(ParseError {
                    span: arg_token.span,
                    token: t,
                    expected: Some("Identifier (argument name)".to_string()),
                    message: "Expected argument name".to_string(),
                })),
            }
        }
        self.consume(Token::Assign, "Expected = in definition")?;
        let body = self.parse_expr()?;
        Ok((body, name, args))
    }

    fn parse_expr(&mut self) -> Result<&'a Term<'a>, CompilerError> {
        match self.peek() {
            Token::If => {
                self.advance();
                let cond = self.parse_expr()?;
                self.consume(Token::Then, "Expected then")?;
                let then_br = self.parse_expr()?;
                self.consume(Token::Else, "Expected else")?;
                let else_br = self.parse_expr()?;
                Ok(unsafe { &*self.arena.alloc(Term::If(cond, then_br, else_br)) })
            }
            Token::Let => {
                self.advance();
                let name_token = self.advance();
                let name = match name_token.node {
                    Token::Identifier(n) => n,
                    t => return Err(CompilerError::Parse(ParseError {
                        span: name_token.span,
                        token: t,
                        expected: Some("Identifier".to_string()),
                        message: "Expected identifier in let".to_string(),
                    })),
                };
                self.consume(Token::Assign, "Expected = in let")?;
                let val = self.parse_expr()?;
                self.consume(Token::In, "Expected in in let")?;
                let body = self.parse_expr()?;
                Ok(unsafe { &*self.arena.alloc(Term::Let(name, val, body)) })
            }
            Token::Case => {
                self.advance();
                let target = self.parse_expr()?;
                self.consume(Token::Of, "Expected of in case")?;
                let mut branches = Vec::new();
                while self.peek() != &Token::EOF {
                    let pat_token = self.advance();
                    let pat_name = match pat_token.node {
                        Token::Identifier(n) => n,
                        Token::Integer(i) => i.to_string(),
                        t => return Err(CompilerError::Parse(ParseError {
                            span: pat_token.span,
                            token: t,
                            expected: Some("Pattern (Identifier or Integer)".to_string()),
                            message: "Expected pattern".to_string(),
                        })),
                    };
                    let mut pat_args = Vec::new();
                    while self.peek() != &Token::FatArrow {
                        let arg_token = self.advance();
                        match arg_token.node {
                            Token::Identifier(a) => pat_args.push(a),
                            t => return Err(CompilerError::Parse(ParseError {
                                span: arg_token.span,
                                token: t,
                                expected: Some("FatArrow (=>) or argument identifier".to_string()),
                                message: "Expected pattern argument".to_string(),
                            })),
                        }
                    }
                    self.consume(Token::FatArrow, "Expected =>")?;
                    let body = self.parse_expr()?;
                    branches.push((pat_name, pat_args, body));
                    if self.peek() != &Token::Pipe { break; }
                    self.advance(); // |
                }
                Ok(unsafe { &*self.arena.alloc(Term::Case(target, branches)) })
            }
            _ => self.parse_comparison(),
        }
    }

    fn parse_comparison(&mut self) -> Result<&'a Term<'a>, CompilerError> {
        let mut lhs = self.parse_bitwise_or()?;
        while self.peek() == &Token::Eq {
            self.advance();
            let rhs = self.parse_bitwise_or()?;
            lhs = unsafe { &*self.arena.alloc(Term::Eq(lhs, rhs)) };
        }
        Ok(lhs)
    }

    fn parse_bitwise_or(&mut self) -> Result<&'a Term<'a>, CompilerError> {
        let mut lhs = self.parse_bitwise_xor()?;
        while self.peek() == &Token::BitOr {
            self.advance();
            let rhs = self.parse_bitwise_xor()?;
            lhs = unsafe { &*self.arena.alloc(Term::BitOr(lhs, rhs)) };
        }
        Ok(lhs)
    }

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

    fn parse_bitwise_and(&mut self) -> Result<&'a Term<'a>, CompilerError> {
        let mut lhs = self.parse_shift()?;
        while self.peek() == &Token::BitAnd {
            self.advance();
            let rhs = self.parse_shift()?;
            lhs = unsafe { &*self.arena.alloc(Term::BitAnd(lhs, rhs)) };
        }
        Ok(lhs)
    }

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

    fn parse_arithmetic(&mut self) -> Result<&'a Term<'a>, CompilerError> {
        let mut lhs = self.parse_unary()?;
        loop {
            if self.peek() == &Token::Plus {
                self.advance();
                let rhs = self.parse_unary()?;
                lhs = unsafe { &*self.arena.alloc(Term::Add(lhs, rhs)) };
            } else if self.peek() == &Token::Minus {
                self.advance();
                let rhs = self.parse_unary()?;
                lhs = unsafe { &*self.arena.alloc(Term::Sub(lhs, rhs)) };
            } else {
                break;
            }
        }
        Ok(lhs)
    }

    fn parse_unary(&mut self) -> Result<&'a Term<'a>, CompilerError> {
        if self.peek() == &Token::Complement {
            self.advance();
            let body = self.parse_unary()?;
            return Ok(unsafe { &*self.arena.alloc(Term::BitNot(body)) });
        }
        self.parse_app()
    }

    fn parse_app(&mut self) -> Result<&'a Term<'a>, CompilerError> {
        let mut expr = self.parse_primary()?;
        loop {
            match self.peek() {
                Token::Identifier(_) | Token::Integer(_) | Token::LParen |
                Token::Complement => {
                    let arg = self.parse_primary()?;
                    expr = unsafe { &*self.arena.alloc(Term::App(expr, arg)) };
                }
                _ => break,
            }
        }
        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<&'a Term<'a>, CompilerError> {
        let token = self.advance();
        match token.node {
            Token::LParen => {
                let expr = self.parse_expr()?;
                self.consume(Token::RParen, "Expected )")?;
                Ok(expr)
            }
            Token::Integer(val) => Ok(unsafe { &*self.arena.alloc(Term::Integer(val)) }),
            Token::Identifier(n) => {
                let term = match n.as_str() {
                    "i32" => Term::I32Type,
                    "i8" => Term::I8Type,
                    "Integer" => Term::IntegerType,
                    "Bits64" => Term::Bits64Type,
                    "IO" => Term::IOType,
                    "buffer" => {
                        let size_token = self.advance();
                        let size = match size_token.node {
                            Token::Integer(s) => s as usize,
                            t => return Err(CompilerError::Parse(ParseError {
                                span: size_token.span,
                                token: t,
                                expected: Some("Integer (buffer size)".to_string()),
                                message: "Expected buffer size".to_string(),
                            })),
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
            t => Err(CompilerError::Parse(ParseError {
                span: token.span,
                token: t,
                expected: None,
                message: "Unexpected token in parse_primary".to_string(),
            })),
        }
    }
}
