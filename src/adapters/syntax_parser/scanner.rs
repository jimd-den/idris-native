//! # Scanner (Infrastructure)
//!
//! This module implements the lexical analysis for the Idris 2 compiler.
//!
//! # Strategic Architecture
//! As an Infrastructure component, the `Scanner` is responsible for 
//! converting raw source text into a stream of structured `Token`s. 
//! This decouples the parser from string manipulation and whitespace handling.

use crate::adapters::diagnostics;

/// The types of tokens recognized by the Idris Native compiler.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Identifier(String),
    Integer(i64),
    // Keywords
    If, Then, Else, Let, In, Case, Of, Data,
    // Symbols
    Assign,      // =
    Arrow,       // ->
    FatArrow,    // =>
    Colon,       // :
    Pipe,        // |
    LParen,      // (
    RParen,      // )
    Backtick,    // `
    // Operators
    Eq,          // ==
    Plus,        // +
    Minus,       // -
    BitAnd,      // .&.
    BitOr,       // .|.
    Xor,         // xor
    ShiftL,      // shiftL
    ShiftR,      // shiftR
    Complement,  // complement
    // Control
    EOF,
}

pub struct Scanner<'a> {
    source: &'a str,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Self {
        diagnostics::log("SCANNER", "INITIALIZE");
        Self {
            source,
            tokens: Vec::new(),
            start: 0,
            current: 0,
        }
    }

    /// Scans the source text and returns a vector of tokens.
    pub fn scan_tokens(&mut self) -> Vec<Token> {
        diagnostics::log("SCANNER", "ENTER scan_tokens()");
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }
        self.tokens.push(Token::EOF);
        diagnostics::log("SCANNER", &format!("EXIT scan_tokens() -> {} tokens", self.tokens.len()));
        self.tokens.clone()
    }

    fn scan_token(&mut self) {
        let c = self.advance();
        match c {
            '(' => self.add_token(Token::LParen),
            ')' => self.add_token(Token::RParen),
            ':' => self.add_token(Token::Colon),
            '=' => {
                if self.match_char('>') {
                    self.add_token(Token::FatArrow);
                } else if self.match_char('=') {
                    self.add_token(Token::Eq);
                } else {
                    self.add_token(Token::Assign);
                }
            }
            '|' => self.add_token(Token::Pipe),
            '`' => self.add_token(Token::Backtick),
            '+' => self.add_token(Token::Plus),
            '-' => {
                if self.match_char('>') {
                    self.add_token(Token::Arrow);
                } else if self.match_char('-') {
                    // Comment: skip until end of line
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else {
                    self.add_token(Token::Minus);
                }
            }
            '.' => {
                if self.match_char('&') && self.match_char('.') {
                    self.add_token(Token::BitAnd);
                } else if self.match_char('|') && self.match_char('.') {
                    self.add_token(Token::BitOr);
                }
            }
            ' ' | '\r' | '\t' | '\n' => (), // Ignore whitespace
            _ => {
                if c.is_digit(10) {
                    self.number();
                } else if c.is_alphabetic() || c == '_' {
                    self.identifier();
                } else {
                    // Ignore unknown characters for MVP
                }
            }
        }
    }

    fn identifier(&mut self) {
        while self.peek().is_alphanumeric() || self.peek() == '_' {
            self.advance();
        }
        let text = &self.source[self.start..self.current];
        let token = match text {
            "if" => Token::If,
            "then" => Token::Then,
            "else" => Token::Else,
            "let" => Token::Let,
            "in" => Token::In,
            "case" => Token::Case,
            "of" => Token::Of,
            "data" => Token::Data,
            "xor" => Token::Xor,
            "shiftL" => Token::ShiftL,
            "shiftR" => Token::ShiftR,
            "complement" => Token::Complement,
            _ => Token::Identifier(text.to_string()),
        };
        self.add_token(token);
    }

    fn number(&mut self) {
        while self.peek().is_digit(10) {
            self.advance();
        }
        let value = self.source[self.start..self.current].parse::<i64>().unwrap();
        self.add_token(Token::Integer(value));
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn advance(&mut self) -> char {
        let c = self.source.chars().nth(self.current).unwrap();
        self.current += 1;
        c
    }

    fn peek(&self) -> char {
        if self.is_at_end() { return '\0'; }
        self.source.chars().nth(self.current).unwrap()
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.is_at_end() { return false; }
        if self.source.chars().nth(self.current).unwrap() != expected {
            return false;
        }
        self.current += 1;
        true
    }

    fn add_token(&mut self, token: Token) {
        diagnostics::log("SCANNER", &format!("TOKEN: {:?}", token));
        self.tokens.push(token);
    }
}
