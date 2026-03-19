//! # QTT Syntax Tests
//!
//! These tests verify that the parser correctly handles explicit 
//! QTT multiplicity annotations in Pi types.

use crate::adapters::syntax_parser::{lex, Token, Parser};
use crate::domain::{Term, arena::Arena, multiplicity::Multiplicity};

#[test]
fn test_parse_linear_pi_type() {
    let mut arena = Arena::new();
    let source = "linear_use : (1 x : Integer) -> Integer\nlinear_use x = x";
    let tokens = lex(source).expect("Lexing failed");
    let mut parser = Parser::new(tokens, &mut arena);
    
    let decls = parser.parse_program().expect("Parsing failed");
    let sig = &decls[0];
    
    match sig {
        Term::Pi(name, mult, _ty, _body) => {
            assert_eq!(name, "x");
            assert_eq!(*mult, Multiplicity::One);
        },
        _ => panic!("Expected Pi type, got {:?}", sig),
    }
}

#[test]
fn test_parse_erased_pi_type() {
    let mut arena = Arena::new();
    let source = "erased_use : (0 ty : Integer) -> Integer\nerased_use x = 42";
    let tokens = lex(source).expect("Lexing failed");
    let mut parser = Parser::new(tokens, &mut arena);
    
    let decls = parser.parse_program().expect("Parsing failed");
    let sig = &decls[0];
    
    match sig {
        Term::Pi(name, mult, _ty, _body) => {
            assert_eq!(name, "ty");
            assert_eq!(*mult, Multiplicity::Zero);
        },
        _ => panic!("Expected Pi type, got {:?}", sig),
    }
}
