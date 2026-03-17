//! # Bare-Metal Target Support Tests
//!
//! This module contains tests for the Idris Native LLVM backend's 
//! bare-metal/embedded code generation.
//!
//! # Strategic Architecture
//! These tests verify that the `llvm_native` infrastructure correctly 
//! handles bare-metal target triples and generates IR that doesn't 
//! rely on any OS services or standard C libraries.

use super::LlvmBackend;

#[test]
fn test_bare_metal_target_configuration() {
    let mut backend = LlvmBackend::new();
    // arm-none-eabi is a common bare-metal target.
    backend.set_target("arm-none-eabi");
    assert_eq!(backend.get_target(), "arm-none-eabi");
}

#[test]
fn test_bare_metal_print_ir_generation() {
    let mut backend = LlvmBackend::new();
    backend.set_target("arm-none-eabi");
    
    let ir = backend.gen_print_ir("Bare Metal Test");
    
    // For bare-metal, we expect it to use a low-level, implementation-defined 
    // print routine that doesn't rely on 'puts'.
    assert!(ir.contains("declare void @__bare_metal_print"));
    assert!(!ir.contains("@puts"));
}
