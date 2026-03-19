//! # Dynamic Main Tests
//!
//! These tests verify that the `LlvmBackend` correctly generates a 
//! `main()` wrapper that matches the signature of the program being compiled.

use crate::infrastructure::llvm::LlvmBackend;
use crate::application::compiler::Backend;
use crate::domain::Term;

#[test]
fn test_lower_program_with_no_args() {
    let backend = LlvmBackend::new();
    let name = "no_args";
    let sig = Term::IntegerType;
    let body = Term::Integer(42);
    let args = vec![];
    
    let ir = backend.lower_program(name, &sig, &body, &args);
    
    // main should call no_args() with no arguments
    assert!(ir.contains("call i64 @no_args()"));
}

#[test]
fn test_lower_program_with_three_args() {
    let backend = LlvmBackend::new();
    let name = "three_args";
    let sig = Term::IntegerType; // Simplified signature
    let body = Term::Integer(42);
    let args = vec!["a".to_string(), "b".to_string(), "c".to_string()];
    
    let ir = backend.lower_program(name, &sig, &body, &args);
    
    // main should call three_args(i64 2, i64 2, i64 2) 
    // (using 2 as default for now as per current logic, but three of them)
    assert!(ir.contains("call i64 @three_args(i64 2, i64 2, i64 2)"));
}
