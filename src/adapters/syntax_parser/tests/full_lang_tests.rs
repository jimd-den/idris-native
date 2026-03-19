//! # Full Language Parsing Tests
//!
//! These tests verify that the `Parser` can correctly parse full Idris 2 
//! constructs such as modules, imports, ADTs, interfaces, and records.

use crate::adapters::syntax_parser::{lex, Parser};
use crate::domain::{Term, arena::Arena};

#[test]
fn test_parse_module_and_import() {
    let mut arena = Arena::new();
    let source = "module Main\nimport Data.Buffer\nmain : Integer\nmain = 42";
    let tokens = lex(source).expect("Lexing failed");
    let mut parser = Parser::new(tokens, &mut arena);
    
    let decls = parser.parse_program().expect("Parsing failed");
    assert_eq!(decls.len(), 4);
    
    match &decls[0] {
        Term::Module(n) => assert_eq!(n, "Main"),
        _ => panic!("Expected Module, got {:?}", decls[0]),
    }
    
    match &decls[1] {
        Term::Import(n) => assert_eq!(n, "Data.Buffer"),
        _ => panic!("Expected Import, got {:?}", decls[1]),
    }
}

#[test]
fn test_parse_data_type() {
    let mut arena = Arena::new();
    let source = "data Bool = False | True";
    let tokens = lex(source).expect("Lexing failed");
    let mut parser = Parser::new(tokens, &mut arena);
    
    let decls = parser.parse_program().expect("Parsing failed");
    assert_eq!(decls.len(), 1);
    
    match &decls[0] {
        Term::Data(name, _params, constructors) => {
            assert_eq!(name, "Bool");
            assert_eq!(constructors.len(), 2);
            assert_eq!(constructors[0].name, "False");
            assert_eq!(constructors[1].name, "True");
        },
        _ => panic!("Expected Data, got {:?}", decls[0]),
    }
}
