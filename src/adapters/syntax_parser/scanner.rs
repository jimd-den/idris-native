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
use crate::common::errors::{CompilerError, LexError, Span, Spanned};

/// The types of tokens recognized by the Idris Native compiler.
#[derive(Debug, Clone, PartialEq, Eq)]
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
pub fn lex(source: &str) -> Result<Vec<Spanned<Token>>, CompilerError> {
    let mut scanner = Scanner::new(source);
    scanner.scan_tokens()
}

pub struct Scanner<'a> {
    source: &'a str,
    cursor: Cursor<char>,
    tokens: Vec<Spanned<Token>>,
    line: usize,
    col: usize,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            cursor: Cursor::new(source.chars().collect()),
            tokens: Vec::new(),
            line: 1,
            col: 1,
        }
    }

    /// Scans the source text and returns a vector of tokens or an error.
    pub fn scan_tokens(&mut self) -> Result<Vec<Spanned<Token>>, CompilerError> {
        let _span = crate::trace_span!("SCANNER", "scan_tokens");
        
        while !self.cursor.is_at_end() {
            self.scan_token()?;
        }
        
        self.add_token(Token::EOF, 1);
        Ok(self.tokens.clone())
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.cursor.advance().copied();
        if let Some(ch) = c {
            if ch == '\n' {
                self.line += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }
        }
        c
    }

    fn add_token(&mut self, node: Token, len: usize) {
        let span = Span::new(self.line, self.col.saturating_sub(len), len);
        self.tokens.push(Spanned::new(node, span));
    }

    fn scan_token(&mut self) -> Result<(), CompilerError> {
        let start_col = self.col;
        let c = match self.advance() {
            Some(c) => c,
            None => return Ok(()),
        };

        match c {
            '(' => self.add_token(Token::LParen, 1),
            ')' => self.add_token(Token::RParen, 1),
            ':' => self.add_token(Token::Colon, 1),
            '=' => {
                if self.cursor.match_item(&'>') {
                    self.col += 1;
                    self.add_token(Token::FatArrow, 2);
                } else if self.cursor.match_item(&'=') {
                    self.col += 1;
                    self.add_token(Token::Eq, 2);
                } else {
                    self.add_token(Token::Assign, 1);
                }
            }
            '|' => self.add_token(Token::Pipe, 1),
            '`' => self.add_token(Token::Backtick, 1),
            '+' => self.add_token(Token::Plus, 1),
            '-' => {
                if self.cursor.match_item(&'>') {
                    self.col += 1;
                    self.add_token(Token::Arrow, 2);
                } else if self.cursor.match_item(&'-') {
                    // Comment: skip until end of line
                    while self.cursor.peek() != Some(&'\n') && !self.cursor.is_at_end() {
                        self.advance();
                    }
                } else {
                    self.add_token(Token::Minus, 1);
                }
            }
            '.' => {
                if self.cursor.match_item(&'&') && self.cursor.match_item(&'.') {
                    self.col += 2;
                    self.add_token(Token::BitAnd, 3);
                } else if self.cursor.match_item(&'|') && self.cursor.match_item(&'.') {
                    self.col += 2;
                    self.add_token(Token::BitOr, 3);
                }
            }
            ' ' | '\r' | '\t' | '\n' => (), // Ignore whitespace
            _ => {
                if c.is_digit(10) {
                    self.number(c);
                } else if c.is_alphabetic() || c == '_' {
                    self.identifier(c);
                } else {
                    return Err(CompilerError::Lex(LexError {
                        span: Span::new(self.line, start_col, 1),
                        character: c,
                        message: format!("Unexpected character: '{}'", c),
                    }));
                }
            }
        }
        Ok(())
    }

    fn identifier(&mut self, first: char) {
        let mut text = String::from(first);
        while let Some(&c) = self.cursor.peek() {
            if c.is_alphanumeric() || c == '_' {
                text.push(c);
                self.advance();
            } else {
                break;
            }
        }
        
        let len = text.len();
        let token = match text.as_str() {
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
            _ => Token::Identifier(text),
        };
        self.add_token(token, len);
    }

    fn number(&mut self, first: char) {
        let mut text = String::from(first);
        while let Some(&c) = self.cursor.peek() {
            if c.is_digit(10) {
                text.push(c);
                self.advance();
            } else {
                break;
            }
        }
        let len = text.len();
        let value = text.parse::<i64>().unwrap_or(0);
        self.add_token(Token::Integer(value), len);
    }
}
