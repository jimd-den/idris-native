//! # Multiplicity Enforcement Tests
//!
//! This module verifies that the `QttChecker` correctly enforces 
//! QTT multiplicity rules (0, 1, Unrestricted).

use crate::domain::{Term, arena::Arena};
use crate::application::qtt_checker::QttChecker;

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
fn test_linear_variable_in_if_branches() {
    let mut arena: Arena<Term> = Arena::new();
    let checker = QttChecker::new();
    use crate::common::test_helpers::arena_alloc;
    
    unsafe {
        // (1 x : Integer) -> if cond then x else x
        let x_var = &*arena_alloc(&mut arena, Term::Var("x".to_string()));
        let cond = &*arena_alloc(&mut arena, Term::Integer(1));
        let body = &*arena_alloc(&mut arena, Term::If(cond, x_var, x_var));
        
        // This should SUCCEED because only one branch is taken.
        // Currently it FAILS (returns 2) because it incorrectly sums.
        assert!(checker.check_multiplicity("x", 1, body));
    }
}
