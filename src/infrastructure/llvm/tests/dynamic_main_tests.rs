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
    
    // A zero-arg fallback is safe to call when there is no explicit main.
    assert!(ir.contains("call i64 @\"no_args\"()"));
    // The fallback @main should NOT call print_int directly;
    // only the runtime's @print helper does that.
    let main_fn = ir.split("define i32 @main()")
        .nth(1)
        .expect("Expected fallback @main in IR");
    assert!(!main_fn.contains("call void @print_int"));
}

#[test]
fn test_lower_program_with_three_args() {
    let backend = LlvmBackend::new();
    
    let args = vec!["a".to_string(), "b".to_string(), "c".to_string()];
    let decls = vec![
        Term::Def("three_args".to_string(), args, &Term::Integer(42))
    ];
    
    let ir = backend.lower_program(&decls);
    
    // The backend must not fabricate dummy arguments for a non-main function.
    assert!(ir.contains("define i32 @main()"));
    assert!(!ir.contains("@\"three_args\"(i64 2, i64 2, i64 2)"));
    assert!(!ir.contains("call i64 @\"three_args\""));
}
