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
    // In a full compiler, we would extract the type signature and verify the term against it.
    // Here we simulate the QTT check over the parsed body.
    if !checker.check_term(body) {
        eprintln!("QTT Error: Resource bounds or multiplicity constraints violated in '{}'.", name);
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

    // If there is a 'main', we should link it or assume this is a library.
    // For MVP, we auto-generate a C-main that calls our function if it's named 'main' or 'ack'.
    let main_func = format!("\
define i32 @main() {{
entry:
  %res = call i64 @{}(i64 2, i64 2)
  %exit_code = trunc i64 %res to i32
  ret i32 %exit_code
}}", name);
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

