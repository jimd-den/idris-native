//! # Hypothesis Tests: Parser Consumption
//!
//! This module contains targeted tests to prove/disprove the hypothesis 
//! that the parser is over-consuming tokens from signatures.

use crate::domain::{Term, arena::Arena};
use crate::adapters::syntax_parser::{Scanner, Parser, Token};

#[test]
fn test_hypothesis_signature_boundary() {
    let mut arena: Arena<Term> = Arena::new();
    
    // source: "name : type -> type\nnext_name = ..."
    let source = "ack : Integer -> Integer\nack m = m";
    let mut scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens();
    
    // Expected tokens: 
    // 0: Identifier("ack")
    // 1: Colon
    // 2: Identifier("Integer")
    // 3: Arrow
    // 4: Identifier("Integer")
    // 5: Identifier("ack")  <-- PARSER SHOULD STOP BEFORE THIS
    // 6: Identifier("m")
    // 7: Assign
    // ...
    
    let mut parser = Parser::new(tokens, &mut arena);
    let (_name, _sig) = parser.parse_signature();
    
    // PROOF: If the parser is at index 5, it has over-consumed.
    // Official Idris 2 would stop at index 5.
    let next_token = parser.peek();
    assert!(matches!(next_token, Token::Identifier(n) if n == "ack"), 
            "Hypothesis Check: Parser should be at 'ack', but is at {:?}", next_token);
}

