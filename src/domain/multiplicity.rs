//! # Multiplicity Entity (Domain Layer)
//!
//! This module defines the `Multiplicity` type used by the QTT checker.
//!
//! # Literate Documentation
//! Quantitative Type Theory (QTT) assigns a 'count' or 'multiplicity' to 
//! every resource. This allows the compiler to track whether a variable 
//! is erased (Zero), used exactly once (One), or used any number of 
//! times (Many).
//!
//! # Performance
//! By resolving these multiplicities at compile-time, we can eliminate 
//! unnecessary runtime checks and safely manage memory without a 
//! Garbage Collector.

/// QTT Multiplicity representing how many times a resource can be used.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Multiplicity {
    /// The resource is erased at runtime (e.g., types, proofs).
    Zero,
    /// The resource must be used exactly once (linear).
    One,
    /// The resource can be used zero or more times (unrestricted).
    Many,
}
