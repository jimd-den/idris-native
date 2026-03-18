//! # Final Integration Test (Native Binary)
//!
//! This module contains the final integration test for the Idris Native 
//! compiler, verifying that it can generate and execute a native binary.
//!
//! # Strategic Architecture
//! These tests sit at the outermost layer, exercising the entire 
//! stack from CLI driver to LLVM code generation.

use idris_native::application::compiler::{Compiler, Backend};
use idris_native::infrastructure::llvm::LlvmBackend;
use std::process::Command;
use std::fs;

#[test]
fn test_end_to_end_id() {
    let backend = LlvmBackend::new();
    let compiler = Compiler::new(&backend);
    
    let source = "id : Integer -> Integer\nid x = x";
    let filepath = "id_test.idr";
    fs::write(filepath, source).expect("Failed to write test file");

    match compiler.compile_file(filepath) {
        Ok(bin_path) => {
            let output = Command::new(&bin_path)
                .output()
                .expect("Failed to execute binary");
            
            // Result for id(2) is 2
            let stdout = String::from_utf8_lossy(&output.stdout);
            assert!(stdout.contains("2"));
            
            let _ = fs::remove_file(filepath);
            let _ = fs::remove_file(bin_path);
        },
        Err(e) => panic!("Compilation failed: {}", e),
    }
}

#[test]
fn test_end_to_end_add() {
    let backend = LlvmBackend::new();
    let compiler = Compiler::new(&backend);
    
    let source = "plus : Integer -> Integer -> Integer\nplus a b = a + b";
    let filepath = "plus_test.idr";
    fs::write(filepath, source).expect("Failed to write test file");

    match compiler.compile_file(filepath) {
        Ok(bin_path) => {
            let output = Command::new(&bin_path)
                .output()
                .expect("Failed to execute binary");
            
            // Result for plus(2, 2) is 4
            let stdout = String::from_utf8_lossy(&output.stdout);
            assert!(stdout.contains("4"));
            
            let _ = fs::remove_file(filepath);
            let _ = fs::remove_file(bin_path);
        },
        Err(e) => panic!("Compilation failed: {}", e),
    }
}
