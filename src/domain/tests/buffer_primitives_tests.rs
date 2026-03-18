//! # Buffer Primitives Tests
//!
//! This module verifies the new fixed-size buffer primitives 
//! required for storing SHA-256 blocks and state.

use crate::domain::Term;
use crate::domain::arena::Arena;

#[test]
fn test_buffer_primitives_creation() {
    let mut arena: Arena<Term> = Arena::new();
    
    unsafe {
        // Create a buffer of size 64 (512-bit block)
        let size = 64;
        let buffer_ptr = arena.alloc(Term::Buffer(size));
        let buffer_term = &*buffer_ptr;
        
        match buffer_term {
            Term::Buffer(s) => assert_eq!(*s, 64),
            _ => panic!("Expected Buffer(64)"),
        }
        
        // Test BufferLoad: load from buffer at index i
        let index = &*arena.alloc(Term::Integer(0));
        let load_term = arena.alloc(Term::BufferLoad(buffer_term, index));
        
        match &*load_term {
            Term::BufferLoad(_, _) => (),
            _ => panic!("Expected BufferLoad"),
        }
        
        // Test BufferStore: store value v into buffer at index i
        let value = &*arena.alloc(Term::Integer(42));
        let store_term = arena.alloc(Term::BufferStore(buffer_term, index, value));
        
        match &*store_term {
            Term::BufferStore(_, _, _) => (),
            _ => panic!("Expected BufferStore"),
        }
    }
}
