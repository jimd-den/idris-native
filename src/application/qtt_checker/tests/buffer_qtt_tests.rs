//! # Buffer QTT Verification Tests
//!
//! This module verifies that the `QttChecker` correctly enforces 
//! resource bounds and multiplicities for buffer primitives.

use crate::domain::{Term, arena::Arena};
use crate::application::qtt_checker::QttChecker;

#[test]
fn test_buffer_valid_access() {
    let mut arena: Arena<Term> = Arena::new();
    let checker = QttChecker::new();
    
    unsafe {
        let buffer = &*arena.alloc(Term::Buffer(64));
        let index = &*arena.alloc(Term::Integer(10));
        let load_term = &*arena.alloc(Term::BufferLoad(buffer, index));
        
        // We expect this to pass as index 10 is within bounds [0, 64)
        assert!(checker.check_term(load_term));
    }
}

#[test]
fn test_buffer_out_of_bounds_access() {
    let mut arena: Arena<Term> = Arena::new();
    let checker = QttChecker::new();
    
    unsafe {
        let buffer = &*arena.alloc(Term::Buffer(64));
        let index = &*arena.alloc(Term::Integer(100)); // Out of bounds
        let load_term = &*arena.alloc(Term::BufferLoad(buffer, index));
        
        // We expect this to fail
        assert!(!checker.check_term(load_term));
    }
}
