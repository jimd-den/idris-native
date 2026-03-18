//! # Final Integration Test (Native Binary)
//!
//! This module contains the final integration test for the Idris Native 
//! compiler, verifying that it can generate and execute a native binary.

use idris_native::llvm_native::{LlvmBackend, Module};
use idris_native::cli_driver::compile_to_binary;
use std::process::Command;
use std::fs;

#[test]
fn test_hello_world_compilation() {
    let backend = LlvmBackend::new();
    let mut module = Module::new("hello_world");
    
    // Create IR for printing "Hello World" and returning 0.
    let (decl, body) = backend.gen_print_ir("Hello World");
    let main_func = format!(
        "define i32 @main() {{\n  {ir}\n  ret i32 0\n}}",
        ir = body
    );

    module.add_declaration(decl);
    module.add_definition(main_func);

    let ir_path = "hello_world.ll";
    let bin_path = "./hello_world_bin";

    backend.emit_to_file(&module, ir_path).expect("Failed to emit IR");

    // Compile to binary
    let success = compile_to_binary(ir_path, bin_path).expect("Compilation failed");
    assert!(success, "Clang failed to compile IR");

    // Execute binary
    let output = Command::new(bin_path)
        .output()
        .expect("Failed to execute binary");

    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Hello World"));
    
    // Cleanup
    let _ = fs::remove_file(ir_path);
    let _ = fs::remove_file(bin_path);
}

#[test]
fn test_compile_idris_source_ackermann() {
    use idris_native::core_terms::arena::Arena;
    use idris_native::syntax_parser::{lex, Parser};
    use idris_native::compiler::IRBuilder;
    use std::collections::HashMap;

    let mut arena = Arena::new();
    
    // Idris 2 source code for Ackermann (MVP subset syntax)
    let source = "ack m n = if m == 0 then n + 1 else if n == 0 then ack ( m - 1 ) 1 else ack ( m - 1 ) ( ack m ( n - 1 ) )";
    
    // 1. Lex and Parse into AST
    let tokens = lex(source);
    let mut parser = Parser::new(tokens, &mut arena);
    let (body, name, args) = parser.parse_def();
    
    assert_eq!(name, "ack");
    assert_eq!(args, vec!["m".to_string(), "n".to_string()]);
    
    // 2. Lower AST to LLVM IR
    let mut builder = IRBuilder::new();
    let mut env = HashMap::new();
    env.insert("m".to_string(), "%m".to_string());
    env.insert("n".to_string(), "%n".to_string());
    env.insert("ack".to_string(), "@ack".to_string()); // Bind recursive call
    
    let res_reg = builder.lower_term(body, &env);
    builder.instructions.push(format!("  ret i64 {}", res_reg));
    
    let ack_ir = format!(
        "define i64 @ack(i64 %m, i64 %n) {{\nentry:\n{}\n}}",
        builder.instructions.join("\n")
    );

    // 3. Assemble Module
    let mut module = Module::new("source_ackermann");
    module.add_definition(ack_ir);
    
    let main_func = "define i32 @main() {
entry:
  %res = call i64 @ack(i64 2, i64 2)
  %exit_code = trunc i64 %res to i32
  ret i32 %exit_code
}".to_string();

    module.add_definition(main_func);

    let backend = LlvmBackend::new();
    let ir_path = "source_ackermann.ll";
    let bin_path = "./source_ackermann_bin";

    backend.emit_to_file(&module, ir_path).expect("Failed to emit IR");

    // 4. Compile to binary and Execute
    let success = compile_to_binary(ir_path, bin_path).expect("Compilation failed");
    assert!(success, "Clang failed to compile Idris Source Ackermann IR");

    let status = Command::new(bin_path)
        .status()
        .expect("Failed to execute binary");
    
    // ack(2, 2) should return 7
    assert_eq!(status.code(), Some(7), "Ackermann Source compilation result was incorrect");
    
    // Cleanup
    let _ = fs::remove_file(ir_path);
    let _ = fs::remove_file(bin_path);
}

#[test]
fn test_hex_printing() {
    let backend = LlvmBackend::new();
    let mut module = Module::new("hex_test");
    
    let main_func = "
define i32 @main() {
entry:
  call void @print_hex(i64 255)
  ret i32 0
}
".to_string();

    module.add_definition(main_func);
    
    // Manual inclusion of @print_hex for the test (normally handled by cli_driver)
    let print_hex_ir = "
define void @print_hex(i64 %n) {
entry:
  %buf = alloca [19 x i8]
  %p0 = getelementptr [19 x i8], ptr %buf, i32 0, i32 0
  store i8 48, ptr %p0
  %p1 = getelementptr [19 x i8], ptr %buf, i32 0, i32 1
  store i8 120, ptr %p1
  %p_newline = getelementptr [19 x i8], ptr %buf, i32 0, i32 18
  store i8 10, ptr %p_newline
  br label %loop
loop:
  %i = phi i32 [ 0, %entry ], [ %i_next, %loop ]
  %curr_n = phi i64 [ %n, %entry ], [ %n_next, %loop ]
  %bits = lshr i64 %curr_n, 60
  %digit = trunc i64 %bits to i8
  %is_less_10 = icmp ult i8 %digit, 10
  %base = select i1 %is_less_10, i8 48, i8 87
  %char = add i8 %digit, %base
  %pos = add i32 %i, 2
  %ptr = getelementptr [19 x i8], ptr %buf, i32 0, i32 %pos
  store i8 %char, ptr %ptr
  %n_next = shl i64 %curr_n, 4
  %i_next = add i32 %i, 1
  %done = icmp eq i32 %i_next, 16
  br i1 %done, label %exit, label %loop
exit:
  call void asm sideeffect \"syscall\", \"{rax},{rdi},{rsi},{rdx},~{rcx},~{r11}\"(i64 1, i64 1, ptr %buf, i64 19)
  ret void
}
".to_string();
    module.add_definition(print_hex_ir);
    
    let ir_path = "hex_test.ll";
    let bin_path = "./hex_test_bin";
    
    backend.emit_to_file(&module, ir_path).expect("Failed to emit IR");
    
    // Should now succeed
    let success = compile_to_binary(ir_path, bin_path).expect("Compilation failed");
    assert!(success);
    
    let output = Command::new(bin_path)
        .output()
        .expect("Failed to execute binary");
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    // 255 is 0x00000000000000ff
    assert!(stdout.contains("0x00000000000000ff"));
    
    let _ = fs::remove_file(ir_path);
    let _ = fs::remove_file(bin_path);
}
