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
    
    let result = compiler.compile_str(source, "test_output", "test.idr");
    assert!(result.is_ok());
}
