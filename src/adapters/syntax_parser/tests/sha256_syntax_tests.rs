//! # SHA-256 Syntax Tests
//!
//! These tests verify that the parser correctly handles the 
//! bitwise and buffer primitives used in the SHA-256 reference implementation.

use crate::adapters::syntax_parser::{lex, Token, Parser};
use crate::domain::{Term, arena::Arena};

#[test]
fn test_lex_official_tokens() {
    let source = "xor .&. .|. shiftL shiftR complement buffer getBits64 setBits64";
    let tokens = lex(source).expect("Lexing failed");
    
    // Check nodes, ignoring EOF
    let nodes: Vec<Token> = tokens.iter().map(|t| t.node.clone()).collect();
    assert!(nodes.contains(&Token::Xor));
    assert!(nodes.contains(&Token::BitAnd));
    assert!(nodes.contains(&Token::BitOr));
    assert!(nodes.contains(&Token::ShiftL));
    assert!(nodes.contains(&Token::ShiftR));
    assert!(nodes.contains(&Token::Complement));
}

#[test]
fn test_parse_buffer_ops() {
    let mut arena = Arena::new();
    let source = "buf_test : Integer -> Integer
buf_test n = let b = buffer 64 in setBits64 b 0 n";
    let tokens = lex(source).expect("Lexing failed");
    let mut parser = Parser::new(tokens, &mut arena);
    
    let decls = parser.parse_program().expect("Parsing failed");
    // Should have 2 decls: signature (Integer -> Integer) and body
    assert_eq!(decls.len(), 2);
    
    // Decls[1] is Term::Def("buf_test", ["n"], body)
    match &decls[1] {
        Term::Def(name, args, body) => {
            assert_eq!(name, "buf_test");
            assert_eq!(args.len(), 1);
            // Verify AST structure for BufferStore
            match body {
                Term::Let(_, _val, inner) => {
                    match inner {
                        Term::BufferStore(_, _, _) => (),
                        _ => panic!("Expected BufferStore, got {:?}", inner),
                    }
                },
                _ => panic!("Expected Let for buffer binding, got {:?}", body),
            }
        },
        _ => panic!("Expected Def, got {:?}", decls[1]),
    }
}

#[test]
fn test_parse_official_bitwise_operators() {
    let mut arena = Arena::new();
    let source = "bit_test : Bits64 -> Bits64 -> Bits64
bit_test a b = ( a `xor` b ) .&. ( a .|. b )";
    let tokens = lex(source).expect("Lexing failed");
    let mut parser = Parser::new(tokens, &mut arena);
    
    let decls = parser.parse_program().expect("Parsing failed");
    match &decls[1] {
        Term::Def(name, args, body) => {
            assert_eq!(name, "bit_test");
            assert_eq!(args.len(), 2);
            // Verify AST structure for Bitwise operators
            match body {
                Term::BitAnd(_, _) => (),
                _ => panic!("Expected BitAnd at top level, got {:?}", body),
            }
        },
        _ => panic!("Expected Def, got {:?}", decls[1]),
    }
}

#[test]
fn test_parse_word_types() {
    let mut arena = Arena::new();
    let source = "type_test : Bits64 -> I32 -> I8 -> Integer
type_test a b c = 42";
    let tokens = lex(source).expect("Lexing failed");
    let mut parser = Parser::new(tokens, &mut arena);
    
    let decls = parser.parse_program().expect("Parsing failed");
    assert_eq!(decls.len(), 2);
}

#[test]
fn test_parse_let_binding() {
    let mut arena = Arena::new();
    let source = "let_test : Integer -> Integer
let_test x = let y = x + 1 in y";
    let tokens = lex(source).expect("Lexing failed");
    let mut parser = Parser::new(tokens, &mut arena);
    
    let decls = parser.parse_program().expect("Parsing failed");
    match &decls[1] {
        Term::Def(name, _args, body) => {
            assert_eq!(name, "let_test");
            match body {
                Term::Let(n, _, _) => assert_eq!(n, "y"),
                _ => panic!("Expected Let, got {:?}", body),
            }
        },
        _ => panic!("Expected Def, got {:?}", decls[1]),
    }
}
