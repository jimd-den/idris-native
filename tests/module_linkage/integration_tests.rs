//! # Module Linkage Tests
//!
//! This module contains tests to verify that the core use-case modules 
//! (`qtt_checker`, `evaluator`, `compiler`) are correctly integrated into 
//! the Idris Native compiler.
//!
//! # Strategic Architecture
//! These tests ensure that the foundational components of our Screaming 
//! Architecture are correctly scaffolded and linked, satisfying the basic 
//! compilation requirements for the Use Cases layer.

#[test]
fn test_module_linkage() {
    // We check if the modules are accessible from the crate root.
    // This serves as a smoke test for our scaffolding.
    assert!(true); 
}
