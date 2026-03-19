//! # Compiler String Input Tests
//!
//! These tests verify that the `Compiler` can accept source code as 
//! a string, decoupling it from the filesystem (S-04).

use crate::application::compiler::{Compiler, Backend};
use crate::infrastructure::llvm::LlvmBackend;
use std::fs;
use std::process::Command;

#[test]
fn test_compiler_compile_str() {
    let backend = LlvmBackend::new();
    let compiler = Compiler::new(&backend);
    
    // A simple linear usage example that should pass.
    let source = "id : Integer -> Integer\nid x = x";
    
    let result = compiler.compile_str(source, "test_output_id", "test.idr");
    assert!(result.is_ok());
}

#[test]
fn test_compiler_no_qtt_mode() {
    let backend = LlvmBackend::new();
    let compiler = Compiler::new(&backend).with_qtt(false);
    
    // A non-linear usage example that would fail QTT.
    let source = "dup : Integer -> Integer\ndup x = x + x";
    
    let result = compiler.compile_str(source, "test_output_dup", "test.idr");
    assert!(result.is_ok(), "Should succeed in non-QTT mode even with non-linear usage");
}

#[test]
fn test_compiler_nested_where_capture() {
    let backend = LlvmBackend::new();
    let compiler = Compiler::new(&backend).with_qtt(false);

    let source = "d : Integer -> Integer\nd y = c (y + 1 + z y)\n      where c : Integer -> Integer\n            c x = 42 + x\n\n            z : Integer -> Integer\n            z w = y + w\n\nmain : Integer\nmain = print (d 2)";

    let output_path = "test_output_nested_where";
    let result = compiler.compile_str(source, output_path, "nested_where.idr");
    assert!(result.is_ok(), "Nested where with capture should compile");

    let output = Command::new(output_path)
        .output()
        .expect("Failed to execute nested where binary");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("49"), "Expected nested where result in stdout, got: {stdout}");

    let _ = fs::remove_file(output_path);
}

