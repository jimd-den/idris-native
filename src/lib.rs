//! # Idris Native Compiler Library
//!
//! This library contains the core logic for the Idris 2 compiler.

pub mod domain;
pub mod application;
pub mod adapters;
pub mod drivers;
pub mod infrastructure;
pub mod common;

#[macro_use]
pub mod macros {
    pub use crate::common::logging::LogSpan;
}
