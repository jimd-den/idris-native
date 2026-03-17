//! # Idris Native Compiler Library
//!
//! This library contains the core logic for the Idris 2 compiler.

pub mod core_terms;
pub mod qtt_checker;
pub mod evaluator;
pub mod compiler;
pub mod syntax_parser;
pub mod repl_session;
pub mod diagnostics;
pub mod cli_driver;
pub mod llvm_native;
