//! # Multiplicity Enforcement Tests
//!
//! This module verifies that the `QttChecker` correctly enforces 
//! QTT multiplicity rules (0, 1, Unrestricted).

use crate::core_terms::{Term, arena::Arena};
use crate::qtt_checker::QttChecker;

#[test]
fn test_linear_variable_used_once() {
    let mut arena: Arena<Term> = Arena::new();
    let checker = QttChecker::new();
    
    unsafe {
        // (1 x : Integer) -> x
        let x_var = &*arena.alloc(Term::Var("x".to_string()));
        let body = x_var;
        
        // We simulate a context where x is linear (quantity 1)
        assert!(checker.check_multiplicity("x", 1, body));
    }
}

#[test]
fn test_linear_variable_used_twice_fails() {
    let mut arena: Arena<Term> = Arena::new();
    let checker = QttChecker::new();
    
    unsafe {
        // (1 x : Integer) -> x + x
        let x_var = &*arena.alloc(Term::Var("x".to_string()));
        let body = &*arena.alloc(Term::Add(x_var, x_var));
        
        // This should fail because x is used twice.
        assert!(!checker.check_multiplicity("x", 1, body));
    }
}

#[test]
fn test_erased_variable_used_fails() {
    let mut arena: Arena<Term> = Arena::new();
    let checker = QttChecker::new();
    
    unsafe {
        // (0 x : Integer) -> x
        let x_var = &*arena.alloc(Term::Var("x".to_string()));
        let body = x_var;
        
        // This should fail because x is used in a relevant position.
        assert!(!checker.check_multiplicity("x", 0, body));
    }
}
