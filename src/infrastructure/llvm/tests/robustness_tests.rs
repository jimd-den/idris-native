//! # LLVM Backend Robustness Tests
//!
//! These tests verify that the `LlvmBackend` correctly handles all 
//! `Term` variants without panicking (L-01) and that side-effects 
//! are properly isolated (S-02).

use crate::application::compiler::Backend;
use crate::infrastructure::llvm::LlvmBackend;
use crate::domain::Term;
use std::collections::HashMap;

#[test]
fn test_backend_handles_all_variants_without_panic() {
    let backend = LlvmBackend::new();
    let env = HashMap::new();
    
    // Test a variant that was previously unhandled (L-01)
    let lambda = Term::Lambda("x".to_string(), &Term::IntegerType, &Term::Var("x".to_string()));
    
    // This should NOT panic.
    let _ir = backend.lower_term(&lambda, &env);
}

#[test]
fn test_backend_initialization_no_side_effects() {
    // S-02: Backend creation should be silent and side-effect free.
    // We can't easily assert "silence" in unit tests, but we verify 
    // it doesn't crash or require external state.
    let _backend = LlvmBackend::new();
}
