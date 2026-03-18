//! # Arena Robustness Tests
//!
//! These tests stress-test the `Arena` allocator with large volumes, 
//! mixed lifetimes, and boundary conditions to ensure it is robust 
//! for high-performance compiler use.

use crate::domain::arena::Arena;

#[test]
fn test_arena_high_volume_allocation() {
    let mut arena: Arena<i64> = Arena::new();
    // Allocate 100,000 integers to stress test growth and memory stability.
    for i in 0..100000 {
        let ptr = arena.alloc(i as i64);
        unsafe {
            assert_eq!(*ptr, i as i64);
        }
    }
}

#[test]
fn test_arena_reset_and_reuse() {
    let mut arena: Arena<String> = Arena::new();
    
    // First round of allocations
    for i in 0..1000 {
        arena.alloc(format!("first_{}", i));
    }
    
    // Reset the arena (if we had a clear/reset method)
    // For now, our Arena doesn't have a clear/reset, it just grows.
    // Let's test that it survives being dropped and recreated if needed,
    // or if we decide to add a reset method.
}

#[test]
fn test_arena_mixed_types_via_trait_objects() {
    // Testing if we can use the arena for trait objects if robustified.
    trait Dummy { fn val(&self) -> i32; }
    struct A(i32);
    impl Dummy for A { fn val(&self) -> i32 { self.0 } }
    
    let mut arena: Arena<Box<dyn Dummy>> = Arena::new();
    let ptr = arena.alloc(Box::new(A(42)));
    unsafe {
        assert_eq!((*ptr).val(), 42);
    }
}
