//! # Diagnostics (Adapter)
//!
//! This module implements user-friendly diagnostic reporting, 
//! translating compiler and type-checking errors into Idris-style output.
//!
//! # Strategic Architecture
//! `diagnostics` is an Adapter responsible for formatting internal 
//! domain errors for human consumption, ensuring a consistent and 
//! helpful developer experience.
//!
//! # Communication Guidelines
//! In accordance with our Product Guidelines, diagnostics must be 
//! Idris-compatible and provide clear, context-aware instructions 
//! for resolving errors.
