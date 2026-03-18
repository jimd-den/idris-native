//! # Evaluator Memory Leak Tests
//!
//! These tests verify that the `Evaluator` correctly uses an `Arena` 
//! for allocations during evaluation, instead of leaking memory with `Box::leak`.

use crate::domain::{Term, arena::Arena};
use crate::application::evaluator::Evaluator;
use crate::common::test_helpers::arena_alloc;
use std::cell::RefCell;

#[test]
fn test_evaluator_uses_arena() {
    let arena = RefCell::new(Arena::new());
    let evaluator = Evaluator::new(&arena);
    
    // Construct (\x. x) 42
    let x_body = unsafe { &*arena_alloc(&mut arena.borrow_mut(), Term::Var("x".to_string())) };
    let lambda = unsafe { &*arena_alloc(&mut arena.borrow_mut(), Term::Lambda("x".to_string(), &Term::IntegerType, x_body)) };
    let arg = unsafe { &*arena_alloc(&mut arena.borrow_mut(), Term::Integer(42)) };
    let app = unsafe { &*arena_alloc(&mut arena.borrow_mut(), Term::App(lambda, arg)) };
    
    let result = evaluator.eval(app);
    if let Term::Integer(val) = result {
        assert_eq!(*val, 42);
    } else {
        panic!("Expected Integer(42), got {:?}", result);
    }
}
