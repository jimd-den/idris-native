//! # Evaluator (Use Case)
//!
//! This module implements compile-time normalization and term reduction 
//! logic for the Idris 2 compiler.
//!
//! # Strategic Architecture
//! The `evaluator` is a pure use-case responsible for reducing terms 
//! during the type-checking and optimization phases, ensuring that 
//! Idris terms are evaluated correctly before code generation.
//!
//! # Performance
//! To achieve performance exceeding well-optimized C, the evaluator 
//! leverages data-oriented structures and arena-based allocation 
//! from the `core_terms` (Entities) layer.
