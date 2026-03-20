//! # Scanner (Adapter)
//!
//! This module implements the lexical analysis for the Idris 2 compiler.

use crate::common::cursor::Cursor;
use crate::common::errors::{CompilerError, LexError, Span, Spanned};

/// The types of tokens recognized by the Idris Native compiler.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Identifier(String),
    Integer(i64),
    Float(u64), // Bits of f64
    String(String),
    Char(char),
    
    // Keywords
    If, Then, Else, Let, In, Case, Of, Data, Interface, Implementation, Record,
    Mutual, Where, Do, Module, Import,
    
    // Symbols
    Assign,      // =
    Arrow,       // ->
    FatArrow,    // =>
    Bind,        // <-
    Colon,       // :
    Pipe,        // |
    LParen,      // (
    RParen,      // )
    Backtick,    // `
    LBrace,      // {
    RBrace,      // }
    LBracket,    // [
    RBracket,    // ]
    Semi,        // ;
    Comma,       // ,
    Dot,         // .
    Append,      // ++
    Newline,     // \n
    Question,    // ?
    Backslash,   // \
    Dollar,      // $
    DoubleStar,  // **
    Hash,        // #
    Underscore,  // _ (standalone, not part of identifier)
    At,          // @
    
    // Operators
    Eq,          // ==
    Plus,        // +
    Minus,       // -
    Star,        // *
    Slash,       // /
    Lt,          // <
    Gt,          // >
    
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
            '{' => self.add_token(Token::LBrace, 1),
            '}' => self.add_token(Token::RBrace, 1),
            '[' => self.add_token(Token::LBracket, 1),
            ']' => self.add_token(Token::RBracket, 1),
            ';' => self.add_token(Token::Semi, 1),
            ',' => self.add_token(Token::Comma, 1),
            ':' => self.add_token(Token::Colon, 1),
            '?' => self.add_token(Token::Question, 1),
            '\\' => self.add_token(Token::Backslash, 1),
            '$' => self.add_token(Token::Dollar, 1),
            '#' => self.add_token(Token::Hash, 1),
            '@' => self.add_token(Token::At, 1),
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
            '|' => {
                if self.cursor.match_item(&'|') {
                    self.col += 1;
                    // For now no LogicalOr token, skip or add if needed
                } else {
                    self.add_token(Token::Pipe, 1);
                }
            }
            '`' => self.add_token(Token::Backtick, 1),
            '+' => {
                if self.cursor.match_item(&'+') {
                    self.col += 1;
                    self.add_token(Token::Append, 2);
                } else {
                    self.add_token(Token::Plus, 1);
                }
            }
            '-' => {
                if self.cursor.match_item(&'>') {
                    self.col += 1;
                    self.add_token(Token::Arrow, 2);
                } else if self.cursor.match_item(&'-') {
                    while self.cursor.peek() != Some(&'\n') && !self.cursor.is_at_end() {
                        self.advance();
                    }
                } else {
                    self.add_token(Token::Minus, 1);
                }
            }
            '*' => {
                if self.cursor.match_item(&'*') {
                    self.col += 1;
                    self.add_token(Token::DoubleStar, 2);
                } else {
                    self.add_token(Token::Star, 1);
                }
            }
            '/' => self.add_token(Token::Slash, 1),
            '<' => {
                if self.cursor.match_item(&'-') {
                    self.col += 1;
                    self.add_token(Token::Bind, 2);
                } else {
                    self.add_token(Token::Lt, 1);
                }
            }
            '>' => self.add_token(Token::Gt, 1),
            '.' => {
                if self.cursor.match_item(&'&') && self.cursor.match_item(&'.') {
                    self.col += 2;
                    self.add_token(Token::BitAnd, 3);
                } else if self.cursor.match_item(&'|') && self.cursor.match_item(&'.') {
                    self.col += 2;
                    self.add_token(Token::BitOr, 3);
                } else {
                    self.add_token(Token::Dot, 1);
                }
            }
            '"' => self.string()?,
            '\'' => {
                // Disambiguate: char literal 'c' vs standalone tick
                // A char literal requires exactly one char followed by closing '
                if let Some(&next_ch) = self.cursor.peek() {
                    if let Some(&after) = self.cursor.peek_at(1) {
                        if after == '\'' && next_ch != '\'' {
                            // Pattern: 'c' — this is a character literal
                            self.character()?;
                        } else {
                            // Not a char literal; treat ' as an identifier char
                            // (e.g., standalone prime, or start of multi-char sequence)
                            self.add_token(Token::Identifier("'".to_string()), 1);
                        }
                    } else {
                        self.add_token(Token::Identifier("'".to_string()), 1);
                    }
                } else {
                    self.add_token(Token::Identifier("'".to_string()), 1);
                }
            }
            ' ' | '\r' | '\t' => {
                // Whitespace is insignificant; just advance column tracking
            }
            '&' => {
                if self.cursor.match_item(&'&') {
                    self.col += 1;
                    self.add_token(Token::Identifier("&&".to_string()), 2);
                } else {
                    self.add_token(Token::Identifier("&".to_string()), 1);
                }
            }
            '!' => self.add_token(Token::Identifier("!".to_string()), 1),
            '~' => self.add_token(Token::Identifier("~".to_string()), 1),
            '%' => {
                // Idris 2 pragma directive (e.g. %default total, %inline).
                // Skip the rest of the line — our backend does not implement
                // these compiler hints.
                while self.cursor.peek() != Some(&'\n') && !self.cursor.is_at_end() {
                    self.cursor.advance();
                }
            }
            '\n' => {
                self.add_token(Token::Newline, 1);
            }
            _ => {
                if c.is_digit(10) {
                    self.number(c);
                } else if c.is_alphabetic() || c == '_' {
                    self.identifier(c);
                }
                // Silently skip unrecognized characters. This allows the
                // scanner to resilient-parse imported library files that
                // may use advanced Idris 2 syntax our backend doesn't
                // implement (e.g. operator chars from contrib libs).
            }
        }
        Ok(())
    }

    fn string(&mut self) -> Result<(), CompilerError> {
        let start_col = self.col - 1;
        let mut value = String::new();
        while self.cursor.peek() != Some(&'"') && !self.cursor.is_at_end() {
            if let Some(c) = self.advance() {
                value.push(c);
            }
        }
        if self.cursor.is_at_end() {
            return Err(CompilerError::Lex(LexError {
                span: Span::new(self.line, start_col, value.len() + 1),
                character: '"',
                message: "Unterminated string".to_string(),
            }));
        }
        self.advance(); // Closing "
        self.add_token(Token::String(value.clone()), value.len() + 2);
        Ok(())
    }

    fn character(&mut self) -> Result<(), CompilerError> {
        let start_col = self.col - 1;
        let c = match self.advance() {
            Some(c) => c,
            None => return Err(CompilerError::Lex(LexError {
                span: Span::new(self.line, start_col, 1),
                character: '\'',
                message: "Empty character literal".to_string(),
            })),
        };
        if self.cursor.peek() != Some(&'\'') {
            return Err(CompilerError::Lex(LexError {
                span: Span::new(self.line, start_col, 2),
                character: c,
                message: "Unterminated character literal".to_string(),
            }));
        }
        self.advance(); // Closing '
        self.add_token(Token::Char(c), 3);
        Ok(())
    }

    fn identifier(&mut self, first: char) {
        let mut text = String::from(first);
        while let Some(&c) = self.cursor.peek() {
            // Idris 2 allows alphanumeric, underscore, dot (qualified names),
            // and prime/tick (e.g., `show'`, `xs'`, `d''`) in identifiers.
            if c.is_alphanumeric() || c == '_' || c == '.' || c == '\'' {
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
            "interface" => Token::Interface,
            "implementation" => Token::Implementation,
            "record" => Token::Record,
            "mutual" => Token::Mutual,
            "where" => Token::Where,
            "do" => Token::Do,
            "module" => Token::Module,
            "import" => Token::Import,
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
        let mut is_float = false;
        while let Some(&c) = self.cursor.peek() {
            if c.is_digit(10) {
                text.push(c);
                self.advance();
            } else if c == '.' {
                if let Some(next) = self.cursor.peek_next() {
                    if next.is_digit(10) {
                        is_float = true;
                        text.push(c);
                        self.advance();
                        continue;
                    }
                }
                break;
            } else {
                break;
            }
        }
        let len = text.len();
        if is_float {
            let val = text.parse::<f64>().unwrap_or(0.0);
            self.add_token(Token::Float(val.to_bits()), len);
        } else {
            let value = text.parse::<i64>().unwrap_or(0);
            self.add_token(Token::Integer(value), len);
        }
    }
}
