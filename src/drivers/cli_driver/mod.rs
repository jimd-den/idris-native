//! # CLI Driver (Infrastructure)
//!
//! This module implements the Command Line Interface (CLI) for the 
//! Idris Native compiler, providing the primary entrypoint for 
//! user interactions.
//!
//! # Strategic Architecture
//! As a Framework/Driver, the `drivers::cli_driver` is responsible for parsing 
//! command-line arguments and routing requests to the `compiler` 
//! Use Case. Sit sits at the outermost layer of our Clean Architecture.
//!
//! # User Experience
//! In accordance with our Product Guidelines, the CLI driver should 
//! follow the Unix philosophy while also offering interactive features 
//! for a modern developer experience.

use std::env;
use crate::application::compiler::Compiler;
use crate::infrastructure::llvm::LlvmBackend;
use crate::adapters::diagnostics;

/// The primary entrypoint for the CLI driver.
/// 
/// Why this exists:
/// Thin Infrastructure layer. This function only parses arguments and 
/// delegates all application logic to the `Compiler` Use Case.
pub fn run() {
    diagnostics::log("CLI_DRIVER", "ENTER run()");
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        println!("Idris Native Compiler");
        println!("Usage: idris_native <file.idr>");
        diagnostics::log("CLI_DRIVER", "EXIT run() -> Missing arguments");
        return;
    }

    let filepath = &args[1];
    
    // Dependency Injection: Inject the LLVM Infrastructure into the Compiler Use Case.
    let backend = LlvmBackend::new();
    let compiler = Compiler::new(&backend);

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
