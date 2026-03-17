//! # QTT Multiplicities (Entities)
//!
//! This module defines the multiplicities used in Quantitative Type Theory 
//! (QTT) for resource tracking and zero-GC memory management.
//!
//! # Strategic Architecture
//! Multiplicities are a core part of our domain entities, as they define 
//! how many times a term can be used at runtime.
//!
//! # QTT Multiplicities
//! - **0 (Erased)**: The term is only used at compile-time (types).
//! - **1 (Linear)**: The term is used exactly once at runtime (deterministic free).
//! - **$\omega$ (Many)**: The term can be used zero or more times.

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Multiplicity {
    /// Erased (Compile-time only)
    Zero,
    /// Linear (Used exactly once)
    One,
    /// Many (Unrestricted use)
    Many,
}
