//! # Core Terms (Entities)
//!
//! This module defines the core terms and data structures for the Idris 2 
//! compiler, including the Abstract Syntax Tree (AST) and QTT-related types.
//!
//! # Strategic Architecture
//! This module is placed at the top level to explicitly announce its role 
//! as the source of truth for all compiler entities.
//!
//! # Performance & Purity
//! To achieve performance exceeding well-optimized C, all terms in this 
//! module are allocated within our custom internal `Arena`, ensuring 
//! high cache locality and O(1) bulk deallocation.
//!
//! # QTT & Zero-GC
//! By leveraging Quantitative Type Theory (QTT) for memory management, we 
//! avoid the overhead of a garbage collector. Our `Arena` provides 
//! the physical memory pool for these QTT-checked terms.

pub mod arena;

#[cfg(test)]
mod tests {
    mod arena_tests;
}
