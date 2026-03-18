//! # Parser (Adapter)
//!
//! This module implements the Recursive Descent parser for the Idris 2 compiler.
//!
//! # Strategic Architecture
//! As an Adapter, the `Parser` translates a stream of `Token`s into our 
//! internal `Term` (Entities). It encapsulates the syntax rules and 
//! ensures that the resulting AST is structurally sound.
//!
//! # Performance & Memory
//! By using the `Arena` for all term allocations, we ensure that the AST 
//! is stored contiguously in memory, maximizing cache hits during 
//! subsequent traversal and lowering.

use crate::domain::{Term, arena::Arena};
use crate::adapters::syntax_parser::scanner::Token;
use crate::common::cursor::Cursor;

pub struct Parser<'a, 'arena> {
    cursor: Cursor<Token>,
    arena: &'arena mut Arena<Term<'a>>,
}

impl<'a, 'arena> Parser<'a, 'arena> {
    pub fn new(tokens: Vec<Token>, arena: &'arena mut Arena<Term<'a>>) -> Self {
        Self {
            cursor: Cursor::new(tokens),
            arena,
        }
    }

    pub fn peek(&self) -> &Token {
        self.cursor.peek().unwrap_or(&Token::EOF)
    }

    fn advance(&mut self) -> Token {
        self.cursor.advance().cloned().unwrap_or(Token::EOF)
    }

    fn consume(&mut self, token: Token, message: &str) -> Token {
        if self.cursor.check(&token) { 
            return self.advance(); 
        }
        panic!("{}: Expected {:?}, got {:?}", message, token, self.peek());
    }

    /// Entry point for parsing a full program (signature + definition).
    pub fn parse_program(&mut self) -> (String, &'a Term<'a>, &'a Term<'a>, Vec<String>) {
        let _span = crate::trace_span!("PARSER", "parse_program");
        
        let (name_sig, sig) = self.parse_signature();
        let (body, name_def, args) = self.parse_def();
        
        if name_sig != name_def {
            panic!("Name mismatch: {} vs {}", name_sig, name_def);
        }
        
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
        let mut lhs = if self.cursor.check(&Token::LParen) {
            let mut is_multiplicity = false;
            let mut name_opt: Option<String> = None;
            
            // Check if it's (q x : type)
            if let Some(Token::Integer(q)) = self.cursor.peek_next() {
                if *q == 0 || *q == 1 {
                    // We need a way to look ahead further. 
                    // For now we'll do a hacky check.
                    // ( q identifier :
                    // Since we can't look ahead 3 steps easily, let's just 
                    // try to parse it and backtrack if it fails? 
                    // Our Cursor doesn't support backtracking yet.
                    
                    // MVP simplification: 
                    is_multiplicity = true;
                }
            }

            if is_multiplicity {
                self.advance(); // (
                let _q = self.advance(); // quantity
                let name = match self.advance() {
                    Token::Identifier(n) => n,
                    _ => panic!("Expected identifier in Pi"),
                };
                self.consume(Token::Colon, "Expected : in Pi binder");
                let ty = self.parse_expr();
                self.consume(Token::RParen, "Expected ) in Pi binder");
                if self.cursor.match_item(&Token::Arrow) {
                    let body = self.parse_pi();
                    return unsafe { &*self.arena.alloc(Term::Pi(name, ty, body)) };
                }
                ty
            } else {
                // Not a multiplicity binder, just (expr)
                self.advance(); // (
                let expr = self.parse_expr();
                self.consume(Token::RParen, "Expected )");
                expr
            }
        } else {
            self.parse_primary()
        };

        if self.cursor.match_item(&Token::Arrow) {
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
        while !self.cursor.check(&Token::Assign) && !self.cursor.is_at_end() {
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
                unsafe { &*self.arena.alloc(Term::If(cond, then_br, else_br)) }
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
                unsafe { &*self.arena.alloc(Term::Let(name, val, body)) }
            }
            Token::Case => {
                self.advance();
                let target = self.parse_expr();
                self.consume(Token::Of, "Expected of in case");
                let mut branches = Vec::new();
                while !self.cursor.is_at_end() {
                    let pat_name = match self.advance() {
                        Token::Identifier(n) => n,
                        Token::Integer(i) => i.to_string(),
                        _ => panic!("Expected pattern"),
                    };
                    let mut pat_args = Vec::new();
                    while !self.cursor.check(&Token::FatArrow) {
                        match self.advance() {
                            Token::Identifier(a) => pat_args.push(a),
                            _ => panic!("Expected pattern argument"),
                        }
                    }
                    self.consume(Token::FatArrow, "Expected =>");
                    let body = self.parse_expr();
                    branches.push((pat_name, pat_args, body));
                    if !self.cursor.match_item(&Token::Pipe) { break; }
                }
                unsafe { &*self.arena.alloc(Term::Case(target, branches)) }
            }
            _ => self.parse_comparison(),
        }
    }

    fn parse_comparison(&mut self) -> &'a Term<'a> {
        let mut lhs = self.parse_bitwise_or();
        while self.cursor.match_item(&Token::Eq) {
            let rhs = self.parse_bitwise_or();
            lhs = unsafe { &*self.arena.alloc(Term::Eq(lhs, rhs)) };
        }
        lhs
    }

    fn parse_bitwise_or(&mut self) -> &'a Term<'a> {
        let mut lhs = self.parse_bitwise_xor();
        while self.cursor.match_item(&Token::BitOr) {
            let rhs = self.parse_bitwise_xor();
            lhs = unsafe { &*self.arena.alloc(Term::BitOr(lhs, rhs)) };
        }
        lhs
    }

    fn parse_bitwise_xor(&mut self) -> &'a Term<'a> {
        let mut lhs = self.parse_bitwise_and();
        while self.cursor.check(&Token::Backtick) {
            if self.cursor.peek_next() == Some(&Token::Xor) {
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
        while self.cursor.match_item(&Token::BitAnd) {
            let rhs = self.parse_shift();
            lhs = unsafe { &*self.arena.alloc(Term::BitAnd(lhs, rhs)) };
        }
        lhs
    }

    fn parse_shift(&mut self) -> &'a Term<'a> {
        let mut lhs = self.parse_arithmetic();
        while self.cursor.check(&Token::Backtick) {
            let op = self.cursor.peek_next();
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
        loop {
            if self.cursor.match_item(&Token::Plus) {
                let rhs = self.parse_unary();
                lhs = unsafe { &*self.arena.alloc(Term::Add(lhs, rhs)) };
            } else if self.cursor.match_item(&Token::Minus) {
                let rhs = self.parse_unary();
                lhs = unsafe { &*self.arena.alloc(Term::Sub(lhs, rhs)) };
            } else {
                break;
            }
        }
        lhs
    }

    fn parse_unary(&mut self) -> &'a Term<'a> {
        if self.cursor.match_item(&Token::Complement) {
            let body = self.parse_unary();
            return unsafe { &*self.arena.alloc(Term::BitNot(body)) };
        }
        self.parse_app()
    }

    fn parse_app(&mut self) -> &'a Term<'a> {
        let mut expr = self.parse_primary();
        while !self.cursor.is_at_end() {
            match self.peek() {
                Token::Identifier(_) | Token::Integer(_) | Token::LParen |
                Token::Complement => {
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
                    "getBits64" => {
                        let buffer = self.parse_primary();
                        let index = self.parse_primary();
                        Term::BufferLoad(buffer, index)
                    }
                    "setBits64" => {
                        let buffer = self.parse_primary();
                        let index = self.parse_primary();
                        let value = self.parse_primary();
                        Term::BufferStore(buffer, index, value)
                    }
                    _ => Term::Var(n),
                };
                unsafe { &*self.arena.alloc(term) }
            }
            t => panic!("Unexpected token in parse_primary: {:?}", t),
        }
    }
}
