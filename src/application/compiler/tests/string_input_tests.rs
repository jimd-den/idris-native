//! # Compiler String Input Tests
//!
//! These tests verify that the `Compiler` can accept source code as 
//! a string, decoupling it from the filesystem (S-04).

use crate::application::compiler::{Compiler, Backend};
use crate::infrastructure::llvm::LlvmBackend;

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

