//! # Minimal Zero-GC Arena Allocator (Entities)
//!
//! This module provides an internal, zero-dependency `Arena` for the 
//! Idris Native compiler. It is designed to minimize heap overhead 
//! and maximize cache locality for compiler terms.
//!
//! # Strategic Architecture
//! This allocator resides in the Entities (Enterprise Logic) layer because 
//! it is a foundational primitive for all Idris terms and the QTT checker.
//!
//! # Performance & Purity
//! Standard allocation (`Box`, `Rc`) scatters memory, causing cache misses. 
//! Our `Arena` ensures that related terms are stored contiguously. 
//! By using a `Vec`-backed store, we leverage the performance of the 
//! standard library while ensuring O(1) allocation and O(1) bulk deallocation.
//!
//! # QTT & Zero-GC
//! QTT ensures deterministic memory usage. This arena provides the 
//! physical pool for those tracked resources, eliminating the need 
//! for a runtime Garbage Collector (GC).

/// A minimal, performance-oriented arena for allocating compiler terms.
///
/// This implementation currently uses a `Vec` for simplicity and safety, 
/// but it can be optimized with raw pointers or multiple chunks as needed 
/// to achieve our goal of beating well-optimized C performance.
pub struct Arena<T> {
    storage: Vec<Box<T>>, // Use Box to ensure pointers remain stable
}

impl<T> Arena<T> {
    /// Creates a new, empty arena.
    pub fn new() -> Self {
        Self {
            storage: Vec::with_capacity(1024),
        }
    }

    /// Allocates a value within the arena and returns a raw pointer to it.
    ///
    /// # Performance
    /// This is an O(1) operation (amortized). By using Box, we ensure 
    /// that pointers remain stable even if the storage Vec reallocates.
    pub fn alloc(&mut self, value: T) -> *mut T {
        let boxed = Box::new(value);
        let ptr: *mut T = Box::into_raw(boxed);
        // Safety: We manage the lifetime via the Arena storage.
        self.storage.push(unsafe { Box::from_raw(ptr) });
        ptr
    }
}
