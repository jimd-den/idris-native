//! # WASM Target Support Tests
//!
//! This module contains tests for the Idris Native LLVM backend's 
//! WebAssembly (WASM) code generation.
//!
//! # Strategic Architecture
//! These tests verify that the `llvm_native` infrastructure correctly 
//! handles the `wasm32-unknown-unknown` target triple and generates 
//! appropriate IR.

use super::LlvmBackend;

#[test]
fn test_wasm_target_configuration() {
    let mut backend = LlvmBackend::new();
    backend.set_target("wasm32-unknown-unknown");
    // We expect the backend to correctly identify as a WASM target.
    assert_eq!(backend.get_target(), "wasm32-unknown-unknown");
}

#[test]
fn test_wasm_print_ir_generation() {
    let mut backend = LlvmBackend::new();
    backend.set_target("wasm32-unknown-unknown");
    
    let (decl, body) = backend.gen_print_ir("WASM Test");
    
    assert!(decl.contains("declare void @__wasm_print"));
    assert!(body.contains("call void @__wasm_print"));
    assert!(!decl.contains("@puts"));
}
