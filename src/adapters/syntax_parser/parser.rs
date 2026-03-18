//! # Parser (Adapter)
//!
//! This module implements the Recursive Descent parser for the Idris 2 compiler.
//!
//! # Strategic Architecture
//! As an Adapter, the `Parser` translates a stream of `Token`s into our 
//! internal `core_terms` (Entities). It encapsulates the syntax rules 
//! and ensures that the resulting AST is structurally sound.

use crate::domain::{Term, arena::Arena};
use crate::adapters::syntax_parser::scanner::Token;
use crate::adapters::diagnostics;

pub struct Parser<'a, 'arena> {
    tokens: Vec<Token>,
    current: usize,
    arena: &'arena mut Arena<Term<'a>>,
}

impl<'a, 'arena> Parser<'a, 'arena> {
    pub fn new(tokens: Vec<Token>, arena: &'arena mut Arena<Term<'a>>) -> Self {
        diagnostics::log("PARSER", "INITIALIZE");
        Self {
            tokens,
            current: 0,
            arena,
        }
    }

    pub fn peek(&self) -> &Token {
        self.tokens.get(self.current).unwrap_or(&Token::EOF)
    }

    fn advance(&mut self) -> Token {
        let t = self.peek().clone();
        if !self.is_at_end() {
            self.current += 1;
        }
        t
    }

    fn is_at_end(&self) -> bool {
        matches!(self.peek(), Token::EOF)
    }

    fn check(&self, token: &Token) -> bool {
        if self.is_at_end() { return false; }
        self.peek() == token
    }

    fn consume(&mut self, token: Token, message: &str) -> Token {
        if self.check(&token) { return self.advance(); }
        panic!("{}: Expected {:?}, got {:?}", message, token, self.peek());
    }

    /// Entry point for parsing a full program (signature + definition).
    pub fn parse_program(&mut self) -> (String, &'a Term<'a>, &'a Term<'a>, Vec<String>) {
        diagnostics::log("PARSER", "ENTER parse_program()");
        let (name_sig, sig) = self.parse_signature();
        let (body, name_def, args) = self.parse_def();
        if name_sig != name_def {
            panic!("Name mismatch: {} vs {}", name_sig, name_def);
        }
        diagnostics::log("PARSER", &format!("EXIT parse_program() -> Success: {}", name_sig));
        (name_sig, sig, body, args)
    }

    pub fn parse_signature(&mut self) -> (String, &'a Term<'a>) {
        let name = match self.advance() {
            Token::Identifier(n) => n,
            t => panic!("Expected identifier in signature, got {:?}", t),
        };
        self.consume(Token::Colon, "Expected : after name in signature");
        let sig = self.parse_pi();
        (name, sig)
    }

    fn parse_pi(&mut self) -> &'a Term<'a> {
        let mut lhs = if self.check(&Token::LParen) {
            let mut is_multiplicity = false;
            let mut name_opt = None;
            if let Some(Token::Integer(q)) = self.tokens.get(self.current + 1) {
                if *q == 0 || *q == 1 {
                    if let Some(Token::Identifier(name)) = self.tokens.get(self.current + 2) {
                        if let Some(Token::Colon) = self.tokens.get(self.current + 3) {
                            is_multiplicity = true;
                            name_opt = Some(name.clone());
                        }
                    }
                }
            }

            if is_multiplicity {
                self.advance(); // (
                self.advance(); // quantity
                let name = name_opt.unwrap();
                self.advance(); // identifier
                self.consume(Token::Colon, "Expected : in Pi binder");
                let ty = self.parse_primary(); // Use parse_primary to avoid over-consumption
                self.consume(Token::RParen, "Expected ) in Pi binder");
                if self.check(&Token::Arrow) {
                    self.advance();
                    let body = self.parse_pi();
                    return unsafe { &*self.arena.alloc(Term::Pi(name, ty, body)) };
                }
                ty
            } else {
                self.parse_primary()
            }
        } else {
            self.parse_primary()
        };

        if self.check(&Token::Arrow) {
            self.advance();
            let rhs = self.parse_pi();
            lhs = unsafe { &*self.arena.alloc(Term::Pi("_".to_string(), lhs, rhs)) };
        }
        lhs
    }

    pub fn parse_def(&mut self) -> (&'a Term<'a>, String, Vec<String>) {
        let name = match self.advance() {
            Token::Identifier(n) => n,
            t => panic!("Expected identifier in definition, got {:?}", t),
        };
        let mut args = Vec::new();
        while !self.check(&Token::Assign) && !self.is_at_end() {
            match self.advance() {
                Token::Identifier(arg) => args.push(arg),
                t => panic!("Expected argument name, got {:?}", t),
            }
        }
        self.consume(Token::Assign, "Expected = in definition");
        let body = self.parse_expr();
        (body, name, args)
    }

    fn parse_expr(&mut self) -> &'a Term<'a> {
        match self.peek() {
            Token::If => {
                self.advance();
                let cond = self.parse_expr();
                self.consume(Token::Then, "Expected then");
                let then_br = self.parse_expr();
                self.consume(Token::Else, "Expected else");
                let else_br = self.parse_expr();
                let term = Term::If(cond, then_br, else_br);
                unsafe { &*self.arena.alloc(term) }
            }
            Token::Let => {
                self.advance();
                let name = match self.advance() {
                    Token::Identifier(n) => n,
                    _ => panic!("Expected identifier in let"),
                };
                self.consume(Token::Assign, "Expected = in let");
                let val = self.parse_expr();
                self.consume(Token::In, "Expected in in let");
                let body = self.parse_expr();
                let term = Term::Let(name, val, body);
                unsafe { &*self.arena.alloc(term) }
            }
            Token::Case => {
                self.advance();
                let target = self.parse_expr();
                self.consume(Token::Of, "Expected of in case");
                let mut branches = Vec::new();
                while !self.is_at_end() {
                    let pat_name = match self.advance() {
                        Token::Identifier(n) => n,
                        Token::Integer(i) => i.to_string(),
                        _ => panic!("Expected pattern"),
                    };
                    let mut pat_args = Vec::new();
                    while !self.check(&Token::FatArrow) {
                        match self.advance() {
                            Token::Identifier(a) => pat_args.push(a),
                            _ => panic!("Expected pattern argument"),
                        }
                    }
                    self.consume(Token::FatArrow, "Expected =>");
                    let body = self.parse_expr();
                    branches.push((pat_name, pat_args, body));
                    if self.check(&Token::Pipe) { self.advance(); } else { break; }
                }
                let term = Term::Case(target, branches);
                unsafe { &*self.arena.alloc(term) }
            }
            _ => self.parse_comparison(),
        }
    }

    fn parse_comparison(&mut self) -> &'a Term<'a> {
        let mut lhs = self.parse_bitwise_or();
        while self.check(&Token::Eq) {
            self.advance();
            let rhs = self.parse_bitwise_or();
            lhs = unsafe { &*self.arena.alloc(Term::Eq(lhs, rhs)) };
        }
        lhs
    }

    fn parse_bitwise_or(&mut self) -> &'a Term<'a> {
        let mut lhs = self.parse_bitwise_xor();
        while self.check(&Token::BitOr) {
            self.advance();
            let rhs = self.parse_bitwise_xor();
            lhs = unsafe { &*self.arena.alloc(Term::BitOr(lhs, rhs)) };
        }
        lhs
    }

    fn parse_bitwise_xor(&mut self) -> &'a Term<'a> {
        let mut lhs = self.parse_bitwise_and();
        while self.check(&Token::Backtick) {
            // Check for `xor`
            if let Some(Token::Xor) = self.tokens.get(self.current + 1) {
                self.advance(); // `
                self.advance(); // xor
                self.consume(Token::Backtick, "Expected ` after xor");
                let rhs = self.parse_bitwise_and();
                lhs = unsafe { &*self.arena.alloc(Term::BitXor(lhs, rhs)) };
            } else {
                break;
            }
        }
        lhs
    }

    fn parse_bitwise_and(&mut self) -> &'a Term<'a> {
        let mut lhs = self.parse_shift();
        while self.check(&Token::BitAnd) {
            self.advance();
            let rhs = self.parse_shift();
            lhs = unsafe { &*self.arena.alloc(Term::BitAnd(lhs, rhs)) };
        }
        lhs
    }

    fn parse_shift(&mut self) -> &'a Term<'a> {
        let mut lhs = self.parse_arithmetic();
        while self.check(&Token::Backtick) {
            let op = self.tokens.get(self.current + 1);
            if matches!(op, Some(Token::ShiftL) | Some(Token::ShiftR)) {
                self.advance(); // `
                let t = self.advance();
                self.consume(Token::Backtick, "Expected `");
                let rhs = self.parse_arithmetic();
                let term = if t == Token::ShiftL { Term::Shl(lhs, rhs) } else { Term::Shr(lhs, rhs) };
                lhs = unsafe { &*self.arena.alloc(term) };
            } else {
                break;
            }
        }
        lhs
    }

    fn parse_arithmetic(&mut self) -> &'a Term<'a> {
        let mut lhs = self.parse_unary();
        while self.check(&Token::Plus) || self.check(&Token::Minus) {
            let op = self.advance();
            let rhs = self.parse_unary();
            let term = if op == Token::Plus { Term::Add(lhs, rhs) } else { Term::Sub(lhs, rhs) };
            lhs = unsafe { &*self.arena.alloc(term) };
        }
        lhs
    }

    fn parse_unary(&mut self) -> &'a Term<'a> {
        if self.check(&Token::Complement) {
            self.advance();
            let body = self.parse_unary();
            return unsafe { &*self.arena.alloc(Term::BitNot(body)) };
        }
        self.parse_app()
    }

    fn parse_app(&mut self) -> &'a Term<'a> {
        let mut expr = self.parse_primary();
        while !self.is_at_end() {
            match self.peek() {
                Token::Identifier(_) | Token::Integer(_) | Token::LParen |
                Token::ShiftL | Token::ShiftR | Token::Xor | Token::Complement |
                Token::If | Token::Let | Token::Case | Token::Data => {
                    // Check if the token is a delimiter that should terminate application
                    if self.check(&Token::In) || self.check(&Token::Then) || self.check(&Token::Else) || 
                       self.check(&Token::Of) || self.check(&Token::Arrow) || self.check(&Token::FatArrow) || 
                       self.check(&Token::Pipe) {
                        break;
                    }
                    let arg = self.parse_primary();
                    expr = unsafe { &*self.arena.alloc(Term::App(expr, arg)) };
                }
                _ => break,
            }
        }
        expr
    }

    fn parse_primary(&mut self) -> &'a Term<'a> {
        match self.advance() {
            Token::LParen => {
                let expr = self.parse_expr();
                self.consume(Token::RParen, "Expected )");
                expr
            }
            Token::Integer(val) => unsafe { &*self.arena.alloc(Term::Integer(val)) },
            Token::Identifier(n) => {
                let term = match n.as_str() {
                    "i32" => Term::I32Type,
                    "i8" => Term::I8Type,
                    "Integer" => Term::IntegerType,
                    "Bits64" => Term::Bits64Type,
                    "IO" => Term::IOType,
                    "buffer" => {
                        let size = match self.advance() {
                            Token::Integer(s) => s as usize,
                            _ => panic!("Expected buffer size"),
                        };
                        Term::Buffer(size)
                    }
                    _ => Term::Var(n),
                };
                unsafe { &*self.arena.alloc(term) }
            }
            t => panic!("Unexpected token in parse_primary: {:?}", t),
        }
    }
}
