//! # QTT Checker (Use Case)
//!
//! This module implements the Quantitative Type Theory (QTT) elaboration 
//! and resource tracking logic for the Idris 2 compiler.
//!
//! # Strategic Architecture
//! As a Use Case, the `qtt_checker` orchestrates domain logic (Entities) 
//! to perform type checking and resource management, adhering to the 
//! dependency rule by not knowing about Adapters or Infrastructure.
//!
//! # QTT & Zero-GC
//! This is where the deterministic, compile-time memory management 
//! happens. The checker ensures that resource multiplicities are correctly 
//! tracked, allowing the compiler to generate GC-free native code.
