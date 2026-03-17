//! # Term Structure Tests
//!
//! This module contains tests for the Idris Native AST entities.

use crate::core_terms::arena::Arena;
use crate::core_terms::Term;

#[test]
fn test_create_pi_type() {
    let mut arena = Arena::new();
    
    // Create 'Integer -> Integer' (Pi x:Integer. Integer)
    unsafe {
        let integer_type = &*arena.alloc(Term::IntegerType);
        let pi_type_ptr = arena.alloc(Term::Pi("x".to_string(), integer_type, integer_type));
        
        match &*pi_type_ptr {
            Term::Pi(name, _, _) => assert_eq!(name, "x"),
            _ => panic!("Expected Pi type"),
        }
    }
}
