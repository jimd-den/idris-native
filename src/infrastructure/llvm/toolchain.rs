//! # LLVM Toolchain (Infrastructure)
//!
//! This module provides the interface to the external LLVM toolchain 
//! (clang, llc) for compiling generated IR into native binaries.

use std::process::Command;
use std::fs;
use std::io;

/// Compiles a string of LLVM IR into a native binary.
pub fn compile_ir_to_binary(ir: String, output_path: &str) -> io::Result<bool> {
    let ir_path = format!("{}.ll", output_path);
    
    // 1. Write IR to disk
    fs::write(&ir_path, ir)?;
    
    // 2. Invoke clang to compile and link
    let status = Command::new("clang")
        .arg(&ir_path)
        .arg("-o")
        .arg(output_path)
        .status()?;
    
    // 3. Cleanup temporary IR file if successful
    if status.success() {
        let _ = fs::remove_file(ir_path);
    }
    
    Ok(status.success())
}
