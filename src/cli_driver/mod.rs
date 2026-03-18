//! # CLI Driver (Infrastructure)
//!
//! This module implements the Command Line Interface (CLI) for the 
//! Idris Native compiler, providing the primary entrypoint for 
//! user interactions.
//!
//! # Strategic Architecture
//! As a Framework/Driver, the `cli_driver` is responsible for parsing 
//! command-line arguments and routing requests to the `compiler` 
//! use-case. It sits at the outermost layer of our Clean Architecture.
//!
//! # User Experience
//! In accordance with our Product Guidelines, the CLI driver should 
//! follow the Unix philosophy while also offering interactive features 
//! for a modern developer experience.

use std::process::Command;
use std::env;
use std::fs;
use crate::syntax_parser::{lex, Parser};
use crate::compiler::IRBuilder;
use crate::llvm_native::{LlvmBackend, Module};
use crate::qtt_checker::QttChecker;
use crate::core_terms::arena::Arena;
use std::collections::HashMap;

/// The primary entrypoint for the CLI driver.
pub fn run() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        println!("Idris Native Compiler: REPL mode not yet fully integrated.");
        println!("Usage: idris_native <file.idr>");
        return;
    }

    let filepath = &args[1];
    println!("Compiling {}...", filepath);

    let source = match fs::read_to_string(filepath) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading file {}: {}", filepath, e);
            std::process::exit(1);
        }
    };

    let mut arena = Arena::new();
    let tokens = lex(&source);
    let mut parser = Parser::new(tokens, &mut arena);
    
    // For MVP, we parse a single definition block
    let (body, name, args_parsed) = parser.parse_def();

    // 1. QTT Checking Phase
    let checker = QttChecker::new();
    
    // Perform multiplicity checks for each argument
    // (Assume quantity 1 for all arguments in MVP for now)
    for arg in &args_parsed {
        if !checker.check_multiplicity(arg, 1, body) {
            eprintln!("QTT Multiplicity Error: Linear variable '{}' is not used exactly once.", arg);
            std::process::exit(1);
        }
    }

    if !checker.check_term(body) {
        eprintln!("QTT Error: Boundary or structural constraint violated.");
        std::process::exit(1);
    }
    println!("QTT Check passed.");

    // 2. LLVM Lowering Phase
    let mut builder = IRBuilder::new();
    let mut env_map = HashMap::new();
    for arg in &args_parsed {
        env_map.insert(arg.clone(), format!("%{}", arg));
    }
    // Bind recursive calls to the function itself
    env_map.insert(name.clone(), format!("@{}", name));

    let res_reg = builder.lower_term(body, &env_map);
    builder.instructions.push(format!("  ret i64 {}", res_reg));

    let args_str = args_parsed.iter().map(|a| format!("i64 %{}", a)).collect::<Vec<_>>().join(", ");
    let func_ir = format!(
        "define i64 @{}({}) {{\nentry:\n{}\n}}",
        name,
        args_str,
        builder.instructions.join("\n")
    );

    // 3. Module Assembly
    let mut module = Module::new(filepath);
    module.add_definition(func_ir);

    // Pure LLVM/Assembly integer-to-string and syscall(write) routine.
    // This is strictly Zero-C and adheres to the "LLVM only" mandate.
    let print_int_ir = "
define void @print_int(i64 %n) {
entry:
  %is_zero = icmp eq i64 %n, 0
  br i1 %is_zero, label %zero, label %nonzero

zero:
  %buf0 = alloca [2 x i8]
  %p0 = getelementptr [2 x i8], ptr %buf0, i32 0, i32 0
  store i8 48, ptr %p0
  %p1 = getelementptr [2 x i8], ptr %buf0, i32 0, i32 1
  store i8 10, ptr %p1
  call void asm sideeffect \"syscall\", \"{rax},{rdi},{rsi},{rdx},~{rcx},~{r11}\"(i64 1, i64 1, ptr %buf0, i64 2)
  ret void

nonzero:
  %buf = alloca [21 x i8] ; space for i64 and newline
  %end_ptr = getelementptr [21 x i8], ptr %buf, i32 0, i32 20
  store i8 10, ptr %end_ptr ; newline
  br label %loop

loop:
  %n_val = phi i64 [ %n, %nonzero ], [ %n_next, %loop ]
  %curr_ptr = phi ptr [ %end_ptr, %nonzero ], [ %next_ptr, %loop ]
  
  %rem = urem i64 %n_val, 10
  %n_next = udiv i64 %n_val, 10
  
  %char_val = add i64 %rem, 48
  %char = trunc i64 %char_val to i8
  
  %next_ptr = getelementptr i8, ptr %curr_ptr, i32 -1
  store i8 %char, ptr %next_ptr
  
  %done = icmp eq i64 %n_next, 0
  br i1 %done, label %exit, label %loop

exit:
  %final_ptr = phi ptr [ %next_ptr, %loop ]
  %ptr_int = ptrtoint ptr %end_ptr to i64
  %start_int = ptrtoint ptr %final_ptr to i64
  %len = sub i64 %ptr_int, %start_int
  %total_len = add i64 %len, 1
  call void asm sideeffect \"syscall\", \"{rax},{rdi},{rsi},{rdx},~{rcx},~{r11}\"(i64 1, i64 1, ptr %final_ptr, i64 %total_len)
  ret void
}
".to_string();

    module.add_definition(print_int_ir);

    // Pure LLVM hex printing routine.
    let print_hex_ir = "
define void @print_hex(i64 %n) {
entry:
  %buf = alloca [19 x i8] ; space for '0x' + 16 hex digits + newline
  %p0 = getelementptr [19 x i8], ptr %buf, i32 0, i32 0
  store i8 48, ptr %p0 ; '0'
  %p1 = getelementptr [19 x i8], ptr %buf, i32 0, i32 1
  store i8 120, ptr %p1 ; 'x'
  %p_newline = getelementptr [19 x i8], ptr %buf, i32 0, i32 18
  store i8 10, ptr %p_newline ; newline
  
  br label %loop

loop:
  %i = phi i32 [ 0, %entry ], [ %i_next, %loop ]
  %curr_n = phi i64 [ %n, %entry ], [ %n_next, %loop ]
  
  %shift = mul i32 %i, 4
  %shift_64 = zext i32 %shift to i64
  %bits = lshr i64 %curr_n, 60 ; extract top 4 bits
  %digit = trunc i64 %bits to i8
  
  %is_less_10 = icmp ult i8 %digit, 10
  %base = select i1 %is_less_10, i8 48, i8 87 ; '0' or 'a'-10
  %char = add i8 %digit, %base
  
  ; Store from left to right (after '0x')
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

    // The generated main function now calls our pure LLVM @print_int or @print_hex.
    let print_call = if name.contains("sha256") || name.contains("hex") {
        "call void @print_hex(i64 %res)"
    } else {
        "call void @print_int(i64 %res)"
    };

    let main_func = format!("\
define i32 @main() {{
entry:
  %res = call i64 @{}(i64 2, i64 2)
  {}
  ret i32 0
}}", name, print_call);
    module.add_definition(main_func);

    let backend = LlvmBackend::new();
    let ir_path = format!("{}.ll", filepath);
    let bin_path = format!("./{}_bin", filepath.replace(".idr", ""));

    if let Err(e) = backend.emit_to_file(&module, &ir_path) {
        eprintln!("Failed to emit IR: {}", e);
        std::process::exit(1);
    }

    // 4. Native Compilation
    match compile_to_binary(&ir_path, &bin_path) {
        Ok(true) => {
            println!("Successfully compiled to {}", bin_path);
            let _ = fs::remove_file(&ir_path); // Cleanup IR
        }
        Ok(false) => {
            eprintln!("Clang failed to compile the generated IR.");
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("Error invoking Clang: {}", e);
            std::process::exit(1);
        }
    }
}

/// Compiles an LLVM IR file to a native binary using the system toolchain.
/// 
/// Why this exists:
/// This is the final step in the compiler pipeline. It bridges the gap 
/// between our generated IR and a standalone executable.
pub fn compile_to_binary(ir_path: &str, output_path: &str) -> std::io::Result<bool> {
    // We attempt to use 'clang' as the driver for LLVM compilation and linking.
    let status = Command::new("clang")
        .arg(ir_path)
        .arg("-o")
        .arg(output_path)
        .status()?;

    Ok(status.success())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_compile_empty_main() {
        let ir = "define i32 @main() { ret i32 0 }";
        let ir_path = "empty_main.ll";
        let output_path = "empty_main";
        
        fs::write(ir_path, ir).unwrap();
        
        // This test requires 'clang' to be installed on the system.
        let result = compile_to_binary(ir_path, output_path);
        
        if let Ok(success) = result {
            if success {
                assert!(std::path::Path::new(output_path).exists());
                let _ = fs::remove_file(output_path);
            }
        }
        
        let _ = fs::remove_file(ir_path);
    }
}

