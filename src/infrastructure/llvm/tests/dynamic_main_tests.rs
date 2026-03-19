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
    
    // sig: Integer, body: 42
    let decls = vec![
        Term::Def("no_args".to_string(), vec![], &Term::Integer(42))
    ];
    
    let ir = backend.lower_program(&decls);
    
    // main should call no_args() with no arguments
    assert!(ir.contains("call i64 @\"no_args\"()"));
}

#[test]
fn test_lower_program_with_three_args() {
    let backend = LlvmBackend::new();
    
    let args = vec!["a".to_string(), "b".to_string(), "c".to_string()];
    let decls = vec![
        Term::Def("three_args".to_string(), args, &Term::Integer(42))
    ];
    
    let ir = backend.lower_program(&decls);
    
    // main should call three_args(i64 2, i64 2, i64 2) 
    assert!(ir.contains("call i64 @\"three_args\"(i64 2, i64 2, i64 2)"));
}
