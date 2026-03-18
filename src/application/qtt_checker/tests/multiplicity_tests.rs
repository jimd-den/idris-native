//! # Multiplicity Enforcement Tests
//!
//! This module verifies that the `QttChecker` correctly enforces 
//! QTT multiplicity rules (0, 1, Unrestricted).

use crate::domain::{Term, arena::Arena, multiplicity::Multiplicity};
use crate::application::qtt_checker::QttChecker;
use crate::common::test_helpers::arena_alloc;

#[test]
fn test_linear_variable_used_once() {
    let mut arena: Arena<Term> = Arena::new();
    let checker = QttChecker::new();
    
    unsafe {
        // (1 x : Integer) -> x
        let x_ptr = arena_alloc(&mut arena, Term::Var("x".to_string()));
        let x_var = &*x_ptr;
        let body = x_var;
        
        assert!(checker.check_multiplicity("x", Multiplicity::One, body));
    }
}

#[test]
fn test_linear_variable_used_twice_fails() {
    let mut arena: Arena<Term> = Arena::new();
    let checker = QttChecker::new();
    
    unsafe {
        // (1 x : Integer) -> x + x
        let x_ptr = arena_alloc(&mut arena, Term::Var("x".to_string()));
        let x_var = &*x_ptr;
        let body = &*arena_alloc(&mut arena, Term::Add(x_var, x_var));
        
        assert!(!checker.check_multiplicity("x", Multiplicity::One, body));
    }
}

#[test]
fn test_linear_variable_in_if_branches() {
    let mut arena: Arena<Term> = Arena::new();
    let checker = QttChecker::new();
    
    unsafe {
        // (1 x : Integer) -> if cond then x else x
        let x_ptr = arena_alloc(&mut arena, Term::Var("x".to_string()));
        let x_var = &*x_ptr;
        let cond = &*arena_alloc(&mut arena, Term::Integer(1));
        let body = &*arena_alloc(&mut arena, Term::If(cond, x_var, x_var));
        
        // This should SUCCEED because only one branch is taken.
        assert!(checker.check_multiplicity("x", Multiplicity::One, body));
    }
}

#[test]
fn test_linear_variable_in_recursion() {
    let mut arena: Arena<Term> = Arena::new();
    let checker = QttChecker::new();
    
    unsafe {
        // ack m n = ack m (n - 1)
        let m_ptr = arena_alloc(&mut arena, Term::Var("m".to_string()));
        let n_ptr = arena_alloc(&mut arena, Term::Var("n".to_string()));
        let m_var = &*m_ptr;
        let n_var = &*n_ptr;
        
        let one = &*arena_alloc(&mut arena, Term::Integer(1));
        let n_minus_1 = &*arena_alloc(&mut arena, Term::Sub(n_var, one));
        
        let ack_var = &*arena_alloc(&mut arena, Term::Var("ack".to_string()));
        let ack_m = &*arena_alloc(&mut arena, Term::App(ack_var, m_var));
        let body = &*arena_alloc(&mut arena, Term::App(ack_m, n_minus_1));
        
        // 'm' is used exactly once in the recursive call.
        assert!(checker.check_multiplicity("m", Multiplicity::One, body));
    }
}

#[test]
fn test_linear_variable_used_twice_recursive() {
    let mut arena: Arena<Term> = Arena::new();
    let checker = QttChecker::new();
    
    unsafe {
        // ack m n = m + ack m (n - 1)
        let m_ptr = arena_alloc(&mut arena, Term::Var("m".to_string()));
        let n_ptr = arena_alloc(&mut arena, Term::Var("n".to_string()));
        let m_var = &*m_ptr;
        let n_var = &*n_ptr;
        
        let one = &*arena_alloc(&mut arena, Term::Integer(1));
        let n_minus_1 = &*arena_alloc(&mut arena, Term::Sub(n_var, one));
        
        let ack_var = &*arena_alloc(&mut arena, Term::Var("ack".to_string()));
        let ack_m = &*arena_alloc(&mut arena, Term::App(ack_var, m_var));
        let ack_call = &*arena_alloc(&mut arena, Term::App(ack_m, n_minus_1));
        
        let body = &*arena_alloc(&mut arena, Term::Add(m_var, ack_call));
        
        // 'm' is used twice. This should return false.
        assert!(!checker.check_multiplicity("m", Multiplicity::One, body));
    }
}

#[test]
fn test_linear_variable_shadowed_in_recursion() {
    let mut arena: Arena<Term> = Arena::new();
    let checker = QttChecker::new();
    
    unsafe {
        // ack m n = let m = 42 in ack m n
        let forty_two = &*arena_alloc(&mut arena, Term::Integer(42));
        let n_var = &*arena_alloc(&mut arena, Term::Var("n".to_string()));
        let ack_var = &*arena_alloc(&mut arena, Term::Var("ack".to_string()));
        let m_inner = &*arena_alloc(&mut arena, Term::Var("m".to_string()));
        let ack_m = &*arena_alloc(&mut arena, Term::App(ack_var, m_inner));
        let ack_call = &*arena_alloc(&mut arena, Term::App(ack_m, n_var));
        
        let body = &*arena_alloc(&mut arena, Term::Let("m".to_string(), forty_two, ack_call));
        
        // 'm' (outer) is used 0 times because it is shadowed.
        assert!(checker.check_multiplicity("m", Multiplicity::Zero, body));
    }
}
