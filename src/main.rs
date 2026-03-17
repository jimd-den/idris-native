//! # Idris Native Compiler & REPL
//!
//! This is the entrypoint for the Idris Native project, a high-performance 
//! Idris 2 compiler built with a QTT Screaming Architecture in Rust.
//!
//! # Strategic Architecture
//! This project adheres to Clean Architecture principles, ensuring that 
//! business logic (Entities and Use Cases) is completely decoupled from 
//! external frameworks and implementation details.
//!
//! # QTT & Zero-GC
//! We achieve performance better than well-optimized C by using Quantitative 
//! Type Theory (QTT) for deterministic, compile-time memory management.
//! This approach eliminates the need for a runtime Garbage Collector (GC).
//!
//! # Performance
//! Extreme performance is realized through Data-Oriented Design, cache-friendly 
//! memory layouts (SoA), and Arena-based allocation.

// Core Layers (Entities & Use Cases)
pub mod core_terms;
pub mod qtt_checker;
pub mod evaluator;
pub mod compiler;

// Adapter Layer
pub mod syntax_parser;
pub mod repl_session;
pub mod diagnostics;

// Infrastructure Layer (Frameworks & Drivers)
pub mod cli_driver;
pub mod llvm_native;

fn main() {
    // In accordance with Clean Architecture, we route strictly through 
    // the CLI driver (Infrastructure) to begin execution.
    cli_driver::run();
}
