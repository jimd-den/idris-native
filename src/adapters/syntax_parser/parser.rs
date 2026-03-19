//! # Parser (Adapter)
//!
//! This module implements the Recursive Descent parser for the Idris 2 compiler.

use crate::domain::{Term, arena::Arena, multiplicity::Multiplicity, term::Constructor};
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

    /// Entry point for parsing a full program (multiple declarations).
    pub fn parse_program(&mut self) -> Result<Vec<Term<'a>>, CompilerError> {
        let _span = crate::trace_span!("PARSER", "parse_program");
        let mut declarations = Vec::new();
        
        while self.peek() != &Token::EOF {
            declarations.push(self.parse_declaration()?);
        }
        
        Ok(declarations)
    }

    fn parse_declaration(&mut self) -> Result<Term<'a>, CompilerError> {
        match self.peek() {
            Token::Module => self.parse_module(),
            Token::Import => self.parse_import(),
            Token::Data => self.parse_data(),
            Token::Interface => self.parse_interface(),
            Token::Implementation => self.parse_implementation(),
            Token::Record => self.parse_record(),
            Token::Mutual => self.parse_mutual(),
            Token::Identifier(_) => {
                // Peek ahead to see if it's a signature or a definition
                if let Some(next) = self.cursor.peek_next() {
                    if next.node == Token::Colon {
                        self.parse_signature_decl()
                    } else {
                        self.parse_definition_decl()
                    }
                } else {
                    self.parse_definition_decl()
                }
            }
            _ => {
                let token = self.advance();
                Err(CompilerError::Parse(ParseError {
                    span: token.span,
                    token: token.node,
                    expected: Some("Declaration".to_string()),
                    message: "Expected top-level declaration".to_string(),
                }))
            }
        }
    }

    fn parse_module(&mut self) -> Result<Term<'a>, CompilerError> {
        self.advance(); // module
        let name_token = self.advance();
        match name_token.node {
            Token::Identifier(n) => Ok(Term::Module(n)),
            _ => Err(CompilerError::Parse(ParseError {
                span: name_token.span,
                token: name_token.node,
                expected: Some("Module Name".to_string()),
                message: "Expected module name".to_string(),
            })),
        }
    }

    fn parse_import(&mut self) -> Result<Term<'a>, CompilerError> {
        self.advance(); // import
        let name_token = self.advance();
        match name_token.node {
            Token::Identifier(n) => Ok(Term::Import(n)),
            _ => Err(CompilerError::Parse(ParseError {
                span: name_token.span,
                token: name_token.node,
                expected: Some("Import Name".to_string()),
                message: "Expected import name".to_string(),
            })),
        }
    }

    fn parse_data(&mut self) -> Result<Term<'a>, CompilerError> {
        self.advance(); // data
        let name = match self.advance().node {
            Token::Identifier(n) => n,
            _ => panic!("Expected data type name"),
        };
        let mut params = Vec::new();
        while self.peek() != &Token::Assign && self.peek() != &Token::EOF {
            match self.advance().node {
                Token::Identifier(p) => params.push(p),
                _ => break,
            }
        }
        self.consume(Token::Assign, "Expected = in data definition")?;
        let mut constructors = Vec::new();
        while self.peek() != &Token::EOF {
            let c_name = match self.advance().node {
                Token::Identifier(n) => n,
                _ => break,
            };
            let mut fields = Vec::new();
            // Basic field parsing: just identifiers or types for now
            while !matches!(self.peek(), Token::Pipe | Token::EOF | Token::Identifier(_)) {
                 // Simplified: we'll just stop at Pipe or next decl
                 break;
            }
            constructors.push(Constructor { name: c_name, fields });
            if self.peek() == &Token::Pipe { self.advance(); } else { break; }
        }
        Ok(Term::Data(name, params, constructors))
    }

    fn parse_interface(&mut self) -> Result<Term<'a>, CompilerError> {
        self.advance(); // interface
        let name = match self.advance().node {
            Token::Identifier(n) => n,
            _ => panic!("Expected interface name"),
        };
        let mut params = Vec::new();
        while self.peek() != &Token::Where && self.peek() != &Token::EOF {
            match self.advance().node {
                Token::Identifier(p) => params.push(p),
                _ => break,
            }
        }
        self.consume(Token::Where, "Expected where in interface")?;
        let mut methods = Vec::new();
        // Simplified method parsing
        Ok(Term::Interface(name, params, methods))
    }

    fn parse_implementation(&mut self) -> Result<Term<'a>, CompilerError> {
        self.advance(); // implementation
        let iface = match self.advance().node {
            Token::Identifier(n) => n,
            _ => panic!("Expected interface name"),
        };
        let target = match self.advance().node {
            Token::Identifier(n) => n,
            _ => panic!("Expected implementation target"),
        };
        Ok(Term::Implementation(iface, target, Vec::new()))
    }

    fn parse_record(&mut self) -> Result<Term<'a>, CompilerError> {
        self.advance(); // record
        let name = match self.advance().node {
            Token::Identifier(n) => n,
            _ => panic!("Expected record name"),
        };
        self.consume(Token::Where, "Expected where in record")?;
        Ok(Term::Record(name, Vec::new()))
    }

    fn parse_mutual(&mut self) -> Result<Term<'a>, CompilerError> {
        self.advance(); // mutual
        Ok(Term::Mutual(Vec::new()))
    }

    fn parse_signature_decl(&mut self) -> Result<Term<'a>, CompilerError> {
        let (_name, sig) = self.parse_signature()?;
        Ok(sig.clone())
    }

    fn parse_definition_decl(&mut self) -> Result<Term<'a>, CompilerError> {
        let (body, name, args) = self.parse_def()?;
        Ok(Term::Def(name, args, body))
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
                    _ => panic!("Expected identifier in Pi"),
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
            _ => panic!("Expected identifier in definition"),
        };
        let mut args = Vec::new();
        while self.peek() != &Token::Assign && self.peek() != &Token::EOF {
            let arg_token = self.advance();
            match arg_token.node {
                Token::Identifier(arg) => args.push(arg),
                _ => panic!("Expected argument name"),
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
                    _ => panic!("Expected identifier in let"),
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
                        _ => panic!("Expected pattern"),
                    };
                    let mut pat_args = Vec::new();
                    while self.peek() != &Token::FatArrow {
                        let arg_token = self.advance();
                        match arg_token.node {
                            Token::Identifier(a) => pat_args.push(a),
                            _ => panic!("Expected pattern argument"),
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
            Token::Do => {
                self.advance();
                let mut stmts = Vec::new();
                // Basic do block parsing (needs improvements for semi/newline)
                Ok(unsafe { &*self.arena.alloc(Term::Do(stmts)) })
            }
            _ => self.parse_comparison(),
        }
    }

    fn parse_comparison(&mut self) -> Result<&'a Term<'a>, CompilerError> {
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
            } else {
                break;
            }
        }
        Ok(lhs)
    }

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
                Token::Identifier(_) | Token::Integer(_) | Token::Float(_) | Token::String(_) | Token::Char(_) |
                Token::LParen | Token::Complement => {
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
            Token::Float(bits) => Ok(unsafe { &*self.arena.alloc(Term::Float(bits)) }),
            Token::String(val) => Ok(unsafe { &*self.arena.alloc(Term::String(val)) }),
            Token::Char(val) => Ok(unsafe { &*self.arena.alloc(Term::Char(val)) }),
            Token::Identifier(n) => {
                let term = match n.as_str() {
                    "i32" => Term::I32Type,
                    "i8" => Term::I8Type,
                    "Integer" => Term::IntegerType,
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
                            _ => panic!("Expected buffer size"),
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
