//! # SHA-256 Syntax Parser Tests
//!
//! This module verifies the parsing of official Idris 2 syntax.

use crate::core_terms::{Term, arena::Arena};
use crate::syntax_parser::{lex, Parser};

#[test]
fn test_parse_official_bitwise_operators() {
    let mut arena: Arena<Term> = Arena::new();
    
    // Official Idris 2 style syntax
    let source = "bitwise_test a b = (a `xor` b) .&. (a .|. b) .&. (complement a) .&. (a `shiftL` 1)";
    let tokens = lex(source);
    let mut parser = Parser::new(tokens, &mut arena);
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
    // Note: `-- comment` is skipped by lexer
    assert_eq!(tokens, vec![".&.", ".|.", "complement", "`", "xor", "`", "`", "shiftL", "`", "`", "shiftR", "`", "(", ")", "=", "+", "=="]);
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
        Term::Let(name, _, _) => assert_eq!(name, "x"),
        _ => panic!("Expected Let, got {:?}", body),
    }
}

#[test]
fn test_parse_type_signature() {
    let mut arena: Arena<Term> = Arena::new();
    
    // Simple signature: name : type -> type
    let source = "ack : Integer -> Integer -> Integer";
    let tokens = lex(source);
    let mut parser = Parser::new(tokens, &mut arena);
    let (name, sig) = parser.parse_signature();
    
    assert_eq!(name, "ack");
    match sig {
        Term::Pi(_, _, _) => (),
        _ => panic!("Expected Pi type for signature, got {:?}", sig),
    }
}

#[test]
fn test_parse_multiplicity_annotation() {
    let mut arena: Arena<Term> = Arena::new();
    
    // Signature with multiplicity: (1 x : Integer) -> Integer
    let source = "duplicate : (1 x : Integer) -> Integer";
    let tokens = lex(source);
    let mut parser = Parser::new(tokens, &mut arena);
    let (name, sig) = parser.parse_signature();
    
    assert_eq!(name, "duplicate");
    match sig {
        Term::Pi(name, ty, body) => {
            assert_eq!(name, "x");
            // In a full implementation, we'd check the quantity on the Pi node.
        },
        _ => panic!("Expected Pi type, got {:?}", sig),
    }
}

#[test]
fn test_parse_adt_declaration() {
    let mut arena: Arena<Term> = Arena::new();
    
    // data Maybe a = Nothing | Just a
    let source = "data Maybe a = Nothing | Just a";
    let tokens = lex(source);
    let mut parser = Parser::new(tokens, &mut arena);
    let adt = parser.parse_adt();
    
    assert_eq!(adt.name, "Maybe");
    assert_eq!(adt.params, vec!["a".to_string()]);
    assert_eq!(adt.constructors.len(), 2);
}

#[test]
fn test_parse_case_expression() {
    let mut arena: Arena<Term> = Arena::new();
    
    // case x of { Nothing => 0 ; Just y => y }
    let source = "case x of Nothing => 0 | Just y => y";
    let tokens = lex(source);
    let mut parser = Parser::new(tokens, &mut arena);
    let body = parser.parse_expr();
    
    match body {
        Term::Case(_, _) => (),
        _ => panic!("Expected Case expression, got {:?}", body),
    }
}
