//! # Error Handling Tests
//!
//! These tests verify that the scanner and parser return structured 
//! errors (Result::Err) instead of panicking on invalid input.

use crate::adapters::syntax_parser::{lex, Parser};
use crate::domain::arena::Arena;
use crate::common::errors::CompilerError;

#[test]
fn test_scanner_unknown_character_returns_err() {
    let source = "main = @"; // '@' is unknown
    let result = lex(source);
    
    assert!(result.is_err(), "Expected LexError for unknown character '@'");
    if let Err(CompilerError::Lex(e)) = result {
        assert_eq!(e.character, '@');
        assert_eq!(e.span.line, 1);
    } else {
        panic!("Expected Lex error variant");
    }
}

#[test]
fn test_parser_syntax_error_returns_err() {
    let mut arena = Arena::new();
    let source = "main = 42"; // Incorrect signature format
    let tokens = lex(source).expect("Lexing should succeed for valid characters");
    let mut parser = Parser::new(tokens, &mut arena);
    
    let result = parser.parse_program();
    assert!(result.is_err(), "Expected ParseError for invalid signature");
    if let Err(CompilerError::Parse(e)) = result {
        assert!(e.message.contains("Expected :"));
    } else {
        panic!("Expected Parse error variant");
    }
}
