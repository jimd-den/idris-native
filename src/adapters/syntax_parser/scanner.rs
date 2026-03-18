//! # Scanner (Adapter)
//!
//! This module implements the lexical analysis for the Idris 2 compiler.
//!
//! # Strategic Architecture
//! As an Adapter, the `Scanner` is responsible for converting raw source 
//! text into a stream of structured `Token`s. It uses the `common::Cursor` 
//! to maintain a sliding window over the source characters.
//!
//! # Literate Documentation
//! Tokenization is the first step in the compilation pipeline. The scanner 
//! partitions the input string into meaningful symbols (identifiers, 
//! keywords, literals), which are then consumed by the parser.

use crate::common::cursor::Cursor;

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

/// Helper function to tokenize source text.
pub fn lex(source: &str) -> Vec<Token> {
    let mut scanner = Scanner::new(source);
    scanner.scan_tokens()
}

pub struct Scanner<'a> {
    source: &'a str,
    cursor: Cursor<char>,
    tokens: Vec<Token>,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            cursor: Cursor::new(source.chars().collect()),
            tokens: Vec::new(),
        }
    }

    /// Scans the source text and returns a vector of tokens.
    pub fn scan_tokens(&mut self) -> Vec<Token> {
        let _span = crate::trace_span!("SCANNER", "scan_tokens");
        
        while !self.cursor.is_at_end() {
            self.scan_token();
        }
        
        self.tokens.push(Token::EOF);
        self.tokens.clone()
    }

    fn scan_token(&mut self) {
        let c = match self.cursor.advance() {
            Some(c) => *c,
            None => return,
        };

        match c {
            '(' => self.tokens.push(Token::LParen),
            ')' => self.tokens.push(Token::RParen),
            ':' => self.tokens.push(Token::Colon),
            '=' => {
                if self.cursor.match_item(&'>') {
                    self.tokens.push(Token::FatArrow);
                } else if self.cursor.match_item(&'=') {
                    self.tokens.push(Token::Eq);
                } else {
                    self.tokens.push(Token::Assign);
                }
            }
            '|' => self.tokens.push(Token::Pipe),
            '`' => self.tokens.push(Token::Backtick),
            '+' => self.tokens.push(Token::Plus),
            '-' => {
                if self.cursor.match_item(&'>') {
                    self.tokens.push(Token::Arrow);
                } else if self.cursor.match_item(&'-') {
                    // Comment: skip until end of line
                    while self.cursor.peek() != Some(&'\n') && !self.cursor.is_at_end() {
                        self.cursor.advance();
                    }
                } else {
                    self.tokens.push(Token::Minus);
                }
            }
            '.' => {
                if self.cursor.match_item(&'&') && self.cursor.match_item(&'.') {
                    self.tokens.push(Token::BitAnd);
                } else if self.cursor.match_item(&'|') && self.cursor.match_item(&'.') {
                    self.tokens.push(Token::BitOr);
                }
            }
            ' ' | '\r' | '\t' | '\n' => (), // Ignore whitespace
            _ => {
                if c.is_digit(10) {
                    self.number(c);
                } else if c.is_alphabetic() || c == '_' {
                    self.identifier(c);
                }
            }
        }
    }

    fn identifier(&mut self, first: char) {
        let start = self.cursor.current_pos() - 1;
        while let Some(c) = self.cursor.peek() {
            if c.is_alphanumeric() || *c == '_' {
                self.cursor.advance();
            } else {
                break;
            }
        }
        let end = self.cursor.current_pos();
        // Since we collected chars into a Vec, we need to map back to original source 
        // or just use the collected chars. For efficiency we'll rebuild string.
        let mut text = String::new();
        text.push(first);
        // This is a bit inefficient due to char collection, but fine for MVP.
        // We can optimize by tracking byte offsets if needed.
        
        // Let's just use the source slice if possible.
        // For now, let's just collect the chars we passed.
        let tokens_slice: String = self.source.chars().skip(start).take(end - start).collect();
        
        let token = match tokens_slice.as_str() {
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
            _ => Token::Identifier(tokens_slice),
        };
        self.tokens.push(token);
    }

    fn number(&mut self, _first: char) {
        let start = self.cursor.current_pos() - 1;
        while let Some(c) = self.cursor.peek() {
            if c.is_digit(10) {
                self.cursor.advance();
            } else {
                break;
            }
        }
        let end = self.cursor.current_pos();
        let text: String = self.source.chars().skip(start).take(end - start).collect();
        let value = text.parse::<i64>().unwrap_or(0);
        self.tokens.push(Token::Integer(value));
    }
}
