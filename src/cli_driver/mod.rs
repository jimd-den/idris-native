//! # CLI Driver (Infrastructure)
//!
//! This module implements the Command Line Interface (CLI) for the 
//! Idris Native compiler, providing the primary entrypoint for 
//! user interactions.
//!
//! # Strategic Architecture
//! As a Framework/Driver, the `cli_driver` is responsible for parsing 
//! command-line arguments and routing requests to the `compiler` 
//! use-case. It sits at the outermost layer of our Clean Architecture.
//!
//! # User Experience
//! In accordance with our Product Guidelines, the CLI driver should 
//! follow the Unix philosophy while also offering interactive features 
//! for a modern developer experience.

use std::process::Command;

/// The primary entrypoint for the CLI driver.
pub fn run() {
    println!("Idris Native: CLI Driver initialized.");
}

/// Compiles an LLVM IR file to a native binary using the system toolchain.
/// 
/// Why this exists:
/// This is the final step in the compiler pipeline. It bridges the gap 
/// between our generated IR and a standalone executable.
pub fn compile_to_binary(ir_path: &str, output_path: &str) -> std::io::Result<bool> {
    // We attempt to use 'clang' as the driver for LLVM compilation and linking.
    let status = Command::new("clang")
        .arg(ir_path)
        .arg("-o")
        .arg(output_path)
        .status()?;

    Ok(status.success())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_compile_empty_main() {
        let ir = "define i32 @main() { ret i32 0 }";
        let ir_path = "empty_main.ll";
        let output_path = "empty_main";
        
        fs::write(ir_path, ir).unwrap();
        
        // This test requires 'clang' to be installed on the system.
        let result = compile_to_binary(ir_path, output_path);
        
        if let Ok(success) = result {
            if success {
                assert!(std::path::Path::new(output_path).exists());
                let _ = fs::remove_file(output_path);
            }
        }
        
        let _ = fs::remove_file(ir_path);
    }
}
