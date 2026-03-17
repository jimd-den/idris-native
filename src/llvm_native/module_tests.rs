//! # LLVM Module Tests
//!
//! This module contains tests for the Idris Native LLVM module assembly.

use crate::llvm_native::LlvmBackend;
use crate::llvm_native::Module;

#[test]
fn test_module_assembly() {
    let backend = LlvmBackend::new();
    let mut module = Module::new("test_module");
    
    // Define a main function that prints "Hello"
    let (decl, body) = backend.gen_print_ir("Hello");
    let main_func = format!(
        "define i32 @main() {{\n  {ir}\n  ret i32 0\n}}",
        ir = body
    );
    
    module.add_declaration(decl);
    module.add_definition(main_func);
    
    let linked_ir = module.link();
    
    // We expect the linked IR to contain the module name, the print routine, and the main function.
    assert!(linked_ir.contains("source_filename = \"test_module\""));
    assert!(linked_ir.contains("define i32 @main()"));
    assert!(linked_ir.contains("declare i32 @puts"));
}

#[test]
fn test_emit_to_file() {
    let backend = LlvmBackend::new();
    let mut module = Module::new("emit_test");
    module.add_definition("define i32 @main() { ret i32 0 }".to_string());
    
    let path = "test_output.ll";
    backend.emit_to_file(&module, path).expect("Failed to emit to file");
    
    assert!(std::path::Path::new(path).exists());
    let content = std::fs::read_to_string(path).expect("Failed to read emitted file");
    assert!(content.contains("source_filename = \"emit_test\""));
    
    // Cleanup
    let _ = std::fs::remove_file(path);
}

