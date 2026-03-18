//! # Adapters Layer (Interface Adapters)
//!
//! This module contains the interface adapters that translate 
//! data between the internal use cases and external formats 
//! (e.g., source text, CLI input, terminal output).

pub mod syntax_parser;
pub mod diagnostics;
pub mod repl_session;
