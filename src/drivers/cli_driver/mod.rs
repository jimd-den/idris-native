//! # CLI Driver (Infrastructure)
//!
//! This module implements the Command Line Interface (CLI) for the 
//! Idris Native compiler, providing the primary entrypoint for 
//! user interactions.

use crate::application::compiler::{Compiler, Backend};
use crate::adapters::diagnostics;

/// The primary entrypoint for the CLI driver.
pub fn run(backend: &dyn Backend, args: Vec<String>) {
    diagnostics::log("CLI_DRIVER", "ENTER run()");
    
    if args.len() < 2 {
        println!("Idris Native Compiler");
        println!("Usage: idris_native <file.idr> [--no-qtt]");
        diagnostics::log("CLI_DRIVER", "EXIT run() -> Missing arguments");
        return;
    }

    let filepath = &args[1];
    let qtt_enabled = !args.contains(&"--no-qtt".to_string());
    
    if !qtt_enabled {
        diagnostics::log("CLI_DRIVER", "MODE: Non-QTT (Linearity checks disabled)");
    }

    let compiler = Compiler::new(backend).with_qtt(qtt_enabled);

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
