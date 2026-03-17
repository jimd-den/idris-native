//! # Core Terms - Internal Arena Allocation Tests
//!
//! This module verifies our custom, zero-dependency `Arena` allocator. 
//!
//! Why we are building this:
//! To achieve performance exceeding well-optimized C, we avoid standard 
//! heap allocation overhead. By implementing our own Arena, we maintain 
//! **Dependency Minimalism** while ensuring O(1) allocation and O(1) bulk 
//! deallocation for compiler terms.

use crate::core_terms::arena::Arena;

#[test]
fn test_arena_allocation() {
    // We create our custom arena with an initial capacity.
    let mut arena = Arena::new();
    
    // We attempt to allocate a value within the arena.
    let value_ref = arena.alloc(42);
    
    // The test ensures the value is correctly stored and retrievable.
    unsafe {
        assert_eq!(*value_ref, 42);
    }
}
