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
    
    let (decl, body) = backend.gen_print_ir("Bare Metal Test");
    
    assert!(decl.contains("declare void @__bare_metal_print"));
    assert!(body.contains("call void @__bare_metal_print"));
    assert!(!decl.contains("@puts"));
}
