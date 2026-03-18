//! # SHA-256 Syntax Parser Tests
//!
//! This module verifies the parsing of official Idris 2 syntax.

use crate::domain::{Term, arena::Arena};
use crate::adapters::syntax_parser::{lex, Parser, Token};

#[test]
fn test_parse_official_bitwise_operators() {
    let mut arena: Arena<Term> = Arena::new();
    
    // Official Idris 2 style syntax
    let source = "bitwise_test a b = (a `xor` b) .&. (a .|. b) .&. (complement a) .&. (a `shiftL` 1)";
    let tokens = lex(source);
    let mut parser = Parser::new(tokens, &mut arena);
    // Since parse_def is public, we use it directly. 
    // We assume parse_program might be better but let's stick to what we have.
    let (body, name, args) = parser.parse_def();
    
    assert_eq!(name, "bitwise_test");
    assert_eq!(args, vec!["a".to_string(), "b".to_string()]);
    
    match body {
        Term::BitAnd(_, _) => (),
        _ => panic!("Expected BitAnd at the top level of this expression, got {:?}", body),
    }
}

#[test]
fn test_lex_official_tokens() {
    let source = ".&. .|. complement `xor` `shiftL` `shiftR` ( ) = + == -- comment";
    let tokens = lex(source);
    // Expected tokens including the EOF
    let expected = vec![
        Token::BitAnd, Token::BitOr, Token::Complement, 
        Token::Backtick, Token::Xor, Token::Backtick, 
        Token::Backtick, Token::ShiftL, Token::Backtick, 
        Token::Backtick, Token::ShiftR, Token::Backtick, 
        Token::LParen, Token::RParen, Token::Assign, 
        Token::Plus, Token::Eq, Token::EOF
    ];
    assert_eq!(tokens, expected);
}

#[test]
fn test_parse_word_types() {
    let mut arena: Arena<Term> = Arena::new();
    
    let source = "types_test = i32 i8";
    let tokens = lex(source);
    let mut parser = Parser::new(tokens, &mut arena);
    let (body, name, args) = parser.parse_def();
    
    assert_eq!(name, "types_test");
    assert!(args.is_empty());
    
    match body {
        Term::App(lhs, rhs) => {
            match lhs {
                Term::I32Type => (),
                _ => panic!("Expected I32Type on LHS, got {:?}", lhs),
            }
            match rhs {
                Term::I8Type => (),
                _ => panic!("Expected I8Type on RHS, got {:?}", rhs),
            }
        },
        _ => panic!("Expected App(I32Type, I8Type), got {:?}", body),
    }
}

#[test]
fn test_parse_buffer_ops() {
    let mut arena: Arena<Term> = Arena::new();
    
    let source = "buffer_test b = setBits64 b 0 (getBits64 b 1)";
    let tokens = lex(source);
    let mut parser = Parser::new(tokens, &mut arena);
    let (body, name, args) = parser.parse_def();
    
    assert_eq!(name, "buffer_test");
    assert_eq!(args, vec!["b".to_string()]);
    
    match body {
        Term::BufferStore(_, _, _) => (),
        _ => panic!("Expected BufferStore, got {:?}", body),
    }
}

#[test]
fn test_parse_let_binding() {
    let mut arena: Arena<Term> = Arena::new();
    
    let source = "let_test = let x = 42 in x + 1";
    let tokens = lex(source);
    let mut parser = Parser::new(tokens, &mut arena);
    let (body, name, args) = parser.parse_def();
    
    assert_eq!(name, "let_test");
    assert!(args.is_empty());
    
    match body {
        Term::Let(n, _, _) => assert_eq!(n, "x"),
        _ => panic!("Expected Let, got {:?}", body),
    }
}
