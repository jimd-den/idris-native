//! # Idris Native Compiler & REPL
//!
//! This is the entrypoint for the Idris Native project, a high-performance 
//! Idris 2 compiler built with a QTT Screaming Architecture in Rust.
//!
//! # Strategic Architecture
//! This project follows Clean Architecture principles. `main.rs` serves as 
//! the **Composition Root**, where we instantiate our infrastructure components 
//! (like the `LlvmBackend`) and inject them into our drivers and use cases.
//!
//! # Performance
//! Extreme performance is realized through Data-Oriented Design, cache-friendly 
//! memory layouts (SoA), and Arena-based allocation.

use idris_native::drivers::cli_driver;
use idris_native::infrastructure::llvm::LlvmBackend;

fn main() {
    // Composition Root: Initialize the production backend.
    let backend = LlvmBackend::new();
    
    // In accordance with Clean Architecture, we route strictly through 
    // the CLI driver (Drivers Layer), injecting the concrete infrastructure.
    cli_driver::run(&backend, std::env::args().collect());
}
