//! # Test Helper Tests
//!
//! These tests verify that our test utilities (e.g., `arena_alloc`) correctly 
//! simplify test boilerplate without introducing regressions.

use crate::domain::arena::Arena;
use crate::common::test_helpers::arena_alloc;

#[test]
fn test_arena_alloc_helper() {
    let mut arena = Arena::new();
    
    // Using the helper.
    let x = arena_alloc(&mut arena, 42);
    let y = arena_alloc(&mut arena, 100);
    
    unsafe {
        assert_eq!(*x, 42);
        assert_eq!(*y, 100);
    }
}
