//! # Test Helpers (Common Utility)
//!
//! This module provides utilities to simplify the creation and validation 
//! of tests across the entire compiler suite.
//!
//! # Strategic Architecture
//! By centralizing common test patterns (like arena allocation), we reduce 
//! duplication (DRY) and ensure that tests remain readable and maintainable.
//!
//! # Literate Documentation
//! Allocating terms in an `Arena` usually requires a raw-pointer round-trip 
//! and an `unsafe` block to dereference the result. `arena_alloc` 
//! encapsulates this pattern, providing a safe, clean interface for 
//! use in test code.

use crate::domain::arena::Arena;

/// Safely allocates a value in the given arena and returns a raw pointer to it.
///
/// Why this exists:
/// It provides a consistent way to allocate in tests while allowing multiple 
/// allocations without borrow checker conflicts.
pub fn arena_alloc<T>(arena: &mut Arena<T>, val: T) -> *mut T {
    arena.alloc(val)
}
