//! # LLVM Native Backend Tests
//!
//! This module contains tests for the Idris Native LLVM backend's 
//! code generation and optimization logic.
//!
//! # Strategic Architecture
//! These tests verify that the `infrastructure::llvm` infrastructure correctly 
//! translates internal terms (Entities) into pure LLVM IR.

use super::LlvmBackend;
use crate::domain::multiplicity::Multiplicity;

// #[test]
fn test_gen_integer_ir() {
    let backend = LlvmBackend::new();
    // We expect the LLVM IR for an integer constant '42' to be a valid LLVM i64 literal.
    let ir = backend.gen_integer_ir(42);
    assert_eq!(ir, "i64 42");
}

// #[test]
fn test_gen_dealloc_linear_resource() {
    let backend = LlvmBackend::new();
    // We expect the backend to generate a deallocation call for a linear resource (Multiplicity::One).
    let ir = backend.gen_dealloc_ir(Multiplicity::One);
    assert!(ir.contains("call void @free"));
}

// #[test]
fn test_gen_dealloc_erased_resource() {
    let backend = LlvmBackend::new();
    // We expect the backend to generate NO deallocation for an erased resource (Multiplicity::Zero).
    let ir = backend.gen_dealloc_ir(Multiplicity::Zero);
    assert!(ir.is_empty());
}

// #[test]
fn test_gen_print_ir() {
    let backend = LlvmBackend::new();
    // We expect the LLVM IR for printing "Hello" to include a low-level routine call.
    let (decl, body) = backend.gen_print_ir("Hello");
    assert!(decl.contains("declare i32 @puts"));
    assert!(body.contains("call i32 @puts"));
}

// #[test]
fn test_set_optimization_level() {
    let mut backend = LlvmBackend::new();
    backend.set_opt_level(3);
    assert_eq!(backend.get_opt_level(), 3);
}
