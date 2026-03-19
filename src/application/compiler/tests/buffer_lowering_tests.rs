//! # Buffer Lowering Tests
//!
//! This module verifies that the buffer primitives lower correctly 
//! to LLVM IR.

use crate::domain::{Term, arena::Arena};
use crate::infrastructure::llvm::IRBuilder;
use std::collections::HashMap;

#[test]
fn test_lower_buffer_creation() {
    let mut arena: Arena<Term> = Arena::new();
    let mut builder = IRBuilder::new();
    let env = HashMap::new();
    
    unsafe {
        let buffer_term = &*arena.alloc(Term::Buffer(64));
        let res = builder.lower_term(buffer_term, &env);
        
        // We expect an 'alloca [64 x i64]' or similar
        assert!(builder.instructions.iter().any(|instr| instr.contains("alloca [64 x i64]")));
        assert_eq!(res, "%1");
    }
}

#[test]
fn test_lower_buffer_store_load() {
    let mut arena: Arena<Term> = Arena::new();
    let mut builder = IRBuilder::new();
    let mut env = HashMap::new();
    
    unsafe {
        let buffer_ptr = arena.alloc(Term::Buffer(64));
        let buffer_term = &*buffer_ptr;
        let buf_reg = builder.lower_term(buffer_term, &env); // Emits %1 alloca
        env.insert("buf".to_string(), buf_reg.clone());
        
        let buf_var = &*arena.alloc(Term::Var("buf".to_string()));
        let index = &*arena.alloc(Term::Integer(0));
        let value = &*arena.alloc(Term::Integer(42));
        
        let store_term = &*arena.alloc(Term::BufferStore(buf_var, index, value));
        builder.lower_term(store_term, &env);
        
        // Trace:
        // %1 = alloca [64 x i64]
        // (lower_term(buf_var) returns %1)
        // (lower_term(index) returns 0)
        // (lower_term(value) returns 42)
        // %2 = getelementptr [64 x i64], ptr %1, i32 0, i64 0
        // store i64 42, ptr %2
        
        assert!(builder.instructions.iter().any(|instr| instr.contains("getelementptr [64 x i64]")));
        assert!(builder.instructions.iter().any(|instr| instr.contains("store i64 42")));
        
        let load_term = &*arena.alloc(Term::BufferLoad(buf_var, index));
        let load_res = builder.lower_term(load_term, &env);
        
        // %3 = getelementptr [64 x i64], ptr %1, i32 0, i64 0
        // %4 = load i64, ptr %3
        
        assert!(builder.instructions.iter().any(|instr| instr.contains("load i64")));
        assert_eq!(load_res, "%4");
    }
}
