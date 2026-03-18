//! # SHA-256 Primitives Tests
//!
//! This module verifies the new bitwise and word type primitives 
//! required for the SHA-256 implementation.

use crate::domain::Term;
use crate::domain::arena::Arena;

#[test]
fn test_word_types_creation() {
    let mut arena: Arena<Term> = Arena::new();
    
    // We expect to be able to create i32 and i8 type terms.
    unsafe {
        let i32_type = arena.alloc(Term::I32Type);
        let i8_type = arena.alloc(Term::I8Type);
        
        match &*i32_type {
            Term::I32Type => (),
            _ => panic!("Expected I32Type"),
        }
        
        match &*i8_type {
            Term::I8Type => (),
            _ => panic!("Expected I8Type"),
        }
    }
}

#[test]
fn test_bitwise_operators_creation() {
    let mut arena: Arena<Term> = Arena::new();
    
    unsafe {
        let a = &*arena.alloc(Term::Integer(1));
        let b = &*arena.alloc(Term::Integer(2));
        
        // Test Xor, And, Or, Not, ShiftLeft, ShiftRight
        let xor_term = arena.alloc(Term::BitXor(a, b));
        let and_term = arena.alloc(Term::BitAnd(a, b));
        let or_term = arena.alloc(Term::BitOr(a, b));
        let not_term = arena.alloc(Term::BitNot(a));
        let shl_term = arena.alloc(Term::Shl(a, b));
        let shr_term = arena.alloc(Term::Shr(a, b));
        
        match &*xor_term {
            Term::BitXor(_, _) => (),
            _ => panic!("Expected BitXor"),
        }
        match &*and_term {
            Term::BitAnd(_, _) => (),
            _ => panic!("Expected BitAnd"),
        }
        match &*or_term {
            Term::BitOr(_, _) => (),
            _ => panic!("Expected BitOr"),
        }
        match &*not_term {
            Term::BitNot(_) => (),
            _ => panic!("Expected BitNot"),
        }
        match &*shl_term {
            Term::Shl(_, _) => (),
            _ => panic!("Expected Shl"),
        }
        match &*shr_term {
            Term::Shr(_, _) => (),
            _ => panic!("Expected Shr"),
        }
    }
}
