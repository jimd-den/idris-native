//! # CLI Driver (Infrastructure)
//!
//! This module implements the Command Line Interface (CLI) for the 
//! Idris Native compiler, providing the primary entrypoint for 
//! user interactions.
//!
//! # Strategic Architecture
//! As a Framework/Driver, the `drivers::cli_driver` is responsible for parsing 
//! command-line arguments and routing requests to the `compiler` 
//! Use Case. It sits at the outermost layer of our Clean Architecture.
//!
//! # User Experience
//! In accordance with our Product Guidelines, the CLI driver should 
//! follow the Unix philosophy while also offering interactive features 
//! for a modern developer experience.

use std::env;
use crate::application::compiler::{Compiler, Backend};
use crate::adapters::diagnostics;

/// The primary entrypoint for the CLI driver.
/// 
/// # Dependency Injection (CA-02, D-01)
/// This function now receives its `Backend` implementation as an argument, 
/// typically from the composition root (`main.rs`). This allows us to 
/// easily swap the production LLVM backend for a mock or alternative 
/// backend during testing or cross-compilation.
pub fn run(backend: &dyn Backend) {
    diagnostics::log("CLI_DRIVER", "ENTER run()");
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        println!("Idris Native Compiler");
        println!("Usage: idris_native <file.idr>");
        diagnostics::log("CLI_DRIVER", "EXIT run() -> Missing arguments");
        return;
    }

    let filepath = &args[1];
    
    // Use the injected backend
    let compiler = Compiler::new(backend);

    match compiler.compile_file(filepath) {
        Ok(bin_path) => {
            println!("Successfully compiled to {}", bin_path);
            diagnostics::log("CLI_DRIVER", &format!("EXIT run() -> Success: {}", bin_path));
        },
        Err(e) => {
            eprintln!("Error: {}", e);
            diagnostics::log("CLI_DRIVER", &format!("EXIT run() -> Error: {}", e));
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    pub mod mock_tests;
}
