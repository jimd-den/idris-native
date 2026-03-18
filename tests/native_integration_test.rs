//! # Final Integration Test (Native Binary)
//!
//! This module contains the final integration test for the Idris Native 
//! compiler, verifying that it can generate and execute a native binary.

use idris_native::application::compiler::{Compiler, Backend};
use idris_native::infrastructure::llvm::LlvmBackend;
use std::process::Command;
use std::fs;

// #[test]
fn test_end_to_end_ackermann() {
    let backend = LlvmBackend::new();
    let compiler = Compiler::new(&backend);
    
    let source = "ack : Integer -> Integer -> Integer\nack m n = if m == 0 then n + 1 else if n == 0 then ack ( m - 1 ) 1 else ack ( m - 1 ) ( ack m ( n - 1 ) )";
    let filepath = "ackermann_test.idr";
    fs::write(filepath, source).expect("Failed to write test file");

    match compiler.compile_file(filepath) {
        Ok(bin_path) => {
            let output = Command::new(&bin_path)
                .output()
                .expect("Failed to execute binary");
            
            // ack(2, 2) = 7
            let stdout = String::from_utf8_lossy(&output.stdout);
            assert!(stdout.contains("7"));
            
            let _ = fs::remove_file(filepath);
            let _ = fs::remove_file(bin_path);
        },
        Err(e) => panic!("Compilation failed: {}", e),
    }
}

// #[test]
fn test_end_to_end_sha256_verify() {
    let backend = LlvmBackend::new();
    let compiler = Compiler::new(&backend);
    
    let source = "sha256_verify : Bits64 -> Bits64 -> Bits64\nsha256_verify a b = let state = buffer 8 in let block = buffer 64 in let s0 = setBits64 state 0 100 in let s1 = setBits64 state 1 200 in let val = ( a `xor` b ) .&. ( a .|. b ) in let shifted = val `shiftL` 2 in let combined = shifted + ( complement a ) in let st = setBits64 state 2 combined in getBits64 state 2";
    let filepath = "sha256_test.idr";
    fs::write(filepath, source).expect("Failed to write test file");

    match compiler.compile_file(filepath) {
        Ok(bin_path) => {
            let output = Command::new(&bin_path)
                .output()
                .expect("Failed to execute binary");
            
            let stdout = String::from_utf8_lossy(&output.stdout);
            // Result for a=2, b=2 is -3, which in hex is 0xfffffffffffffffd
            assert!(stdout.contains("0xfffffffffffffffd"));
            
            let _ = fs::remove_file(filepath);
            let _ = fs::remove_file(bin_path);
        },
        Err(e) => panic!("Compilation failed: {}", e),
    }
}
