//! # SHA-256 Syntax Parser Tests
//!
//! This module verifies the parsing of bitwise operators and 
//! word types required for SHA-256.

use crate::core_terms::{Term, arena::Arena};
use crate::syntax_parser::{lex, Parser};

#[test]
fn test_parse_bitwise_operators() {
    let mut arena: Arena<Term> = Arena::new();
    
    // We expect the parser to handle: ^, &, |, ~, <<, >>
    let source = "bitwise_test a b = (a ^ b) & (a | b) & (~a) & (a << 1) & (b >> 2)";
    let tokens = lex(source);
    let mut parser = Parser::new(tokens, &mut arena);
    let (body, name, args) = parser.parse_def();
    
    assert_eq!(name, "bitwise_test");
    assert_eq!(args, vec!["a".to_string(), "b".to_string()]);
    
    // The structure should be a deeply nested series of BitAnds
    // depending on the precedence we implement.
    // For now, let's just ensure it doesn't panic and we can match some parts.
    match body {
        Term::BitAnd(_, _) => (),
        _ => panic!("Expected BitAnd at the top level of this expression, got {:?}", body),
    }
}

#[test]
fn test_lex_bitwise_tokens() {
    let source = "^ & | ~ << >>";
    let tokens = lex(source);
    assert_eq!(tokens, vec!["^", "&", "|", "~", "<<", ">>"]);
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
