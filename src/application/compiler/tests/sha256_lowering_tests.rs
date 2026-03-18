//! # SHA-256 Lowering Tests
//!
//! This module verifies that the bitwise primitives and word types 
//! lower correctly to LLVM IR.

use crate::domain::{Term, arena::Arena};
use crate::application::compiler::IRBuilder;
use std::collections::HashMap;

#[test]
fn test_lower_bitwise_xor() {
    let mut arena: Arena<Term> = Arena::new();
    let mut builder = IRBuilder::new();
    let env = HashMap::new();
    
    unsafe {
        let a = &*arena.alloc(Term::Integer(1));
        let b = &*arena.alloc(Term::Integer(2));
        let xor_term = &*arena.alloc(Term::BitXor(a, b));
        
        let res = builder.lower_term(xor_term, &env);
        
        // We expect an 'xor i64' instruction
        assert!(builder.instructions.iter().any(|instr| instr.contains("xor i64 1, 2")));
        assert_eq!(res, "%1");
    }
}

#[test]
fn test_lower_bitwise_and() {
    let mut arena: Arena<Term> = Arena::new();
    let mut builder = IRBuilder::new();
    let env = HashMap::new();
    
    unsafe {
        let a = &*arena.alloc(Term::Integer(1));
        let b = &*arena.alloc(Term::Integer(2));
        let and_term = &*arena.alloc(Term::BitAnd(a, b));
        
        builder.lower_term(and_term, &env);
        
        assert!(builder.instructions.iter().any(|instr| instr.contains("and i64 1, 2")));
    }
}

#[test]
fn test_lower_bitwise_or() {
    let mut arena: Arena<Term> = Arena::new();
    let mut builder = IRBuilder::new();
    let env = HashMap::new();
    
    unsafe {
        let a = &*arena.alloc(Term::Integer(1));
        let b = &*arena.alloc(Term::Integer(2));
        let or_term = &*arena.alloc(Term::BitOr(a, b));
        
        builder.lower_term(or_term, &env);
        
        assert!(builder.instructions.iter().any(|instr| instr.contains("or i64 1, 2")));
    }
}

#[test]
fn test_lower_bitwise_not() {
    let mut arena: Arena<Term> = Arena::new();
    let mut builder = IRBuilder::new();
    let env = HashMap::new();
    
    unsafe {
        let a = &*arena.alloc(Term::Integer(1));
        let not_term = &*arena.alloc(Term::BitNot(a));
        
        builder.lower_term(not_term, &env);
        
        // Bitwise NOT in LLVM is xor with -1
        assert!(builder.instructions.iter().any(|instr| instr.contains("xor i64 1, -1")));
    }
}

#[test]
fn test_lower_shl() {
    let mut arena: Arena<Term> = Arena::new();
    let mut builder = IRBuilder::new();
    let env = HashMap::new();
    
    unsafe {
        let a = &*arena.alloc(Term::Integer(1));
        let b = &*arena.alloc(Term::Integer(2));
        let shl_term = &*arena.alloc(Term::Shl(a, b));
        
        builder.lower_term(shl_term, &env);
        
        assert!(builder.instructions.iter().any(|instr| instr.contains("shl i64 1, 2")));
    }
}

#[test]
fn test_lower_shr() {
    let mut arena: Arena<Term> = Arena::new();
    let mut builder = IRBuilder::new();
    let env = HashMap::new();
    
    unsafe {
        let a = &*arena.alloc(Term::Integer(1));
        let b = &*arena.alloc(Term::Integer(2));
        let shr_term = &*arena.alloc(Term::Shr(a, b));
        
        builder.lower_term(shr_term, &env);
        
        // We use logical shift right (lshr) for unsigned-like bitwise ops
        assert!(builder.instructions.iter().any(|instr| instr.contains("lshr i64 1, 2")));
    }
}

#[test]
fn test_lower_i32_addition() {
    let mut arena: Arena<Term> = Arena::new();
    let mut builder = IRBuilder::new();
    let env = HashMap::new();
    
    // In a real scenario, we'd have a way to specify types.
    // For this MVP, let's assume we can set the builder's bit-width.
    builder.set_bit_width(32);
    
    unsafe {
        let a = &*arena.alloc(Term::Integer(1));
        let b = &*arena.alloc(Term::Integer(2));
        let add_term = &*arena.alloc(Term::Add(a, b));
        
        builder.lower_term(add_term, &env);
        
        assert!(builder.instructions.iter().any(|instr| instr.contains("add i32 1, 2")));
    }
}

#[test]
fn test_lower_i8_subtraction() {
    let mut arena: Arena<Term> = Arena::new();
    let mut builder = IRBuilder::new();
    let env = HashMap::new();
    
    builder.set_bit_width(8);
    
    unsafe {
        let a = &*arena.alloc(Term::Integer(10));
        let b = &*arena.alloc(Term::Integer(3));
        let sub_term = &*arena.alloc(Term::Sub(a, b));
        
        builder.lower_term(sub_term, &env);
        
        assert!(builder.instructions.iter().any(|instr| instr.contains("sub i8 10, 3")));
    }
}

#[test]
fn test_lower_case_expression() {
    let mut arena: Arena<Term> = Arena::new();
    let mut builder = IRBuilder::new();
    let mut env = HashMap::new();
    
    unsafe {
        let x = &*arena.alloc(Term::Var("x".to_string()));
        env.insert("x".to_string(), "%x".to_string());
        
        let zero = &*arena.alloc(Term::Integer(0));
        let one = &*arena.alloc(Term::Integer(1));
        
        // case x of 0 => 1 | _ => x
        let branches = vec![
            ("0".to_string(), vec![], one),
            ("_".to_string(), vec![], x),
        ];
        let case_term = &*arena.alloc(Term::Case(x, branches));
        
        builder.lower_term(case_term, &env);
        
        // We expect comparison, branches, and a phi node
        assert!(builder.instructions.iter().any(|instr| instr.contains("icmp eq")));
        assert!(builder.instructions.iter().any(|instr| instr.contains("phi i64")));
    }
}

#[test]
fn test_lower_bits64_type() {
    let mut arena: Arena<Term> = Arena::new();
    let mut builder = IRBuilder::new();
    let env = HashMap::new();
    
    unsafe {
        // Bits64 should be treated as i64 in LLVM
        let bits_ptr = arena.alloc(Term::Bits64Type);
        let bits_type = &*bits_ptr;
        // Primitives like types don't emit instructions, but we check support
        let res = builder.lower_term(bits_type, &env);
        assert_eq!(res, "i64");
    }
}

#[test]
fn test_lower_io_action() {
    let mut arena: Arena<Term> = Arena::new();
    let mut builder = IRBuilder::new();
    let env = HashMap::new();
    
    unsafe {
        // Mock a putStrLn call
        let msg_ptr = arena.alloc(Term::Var("hello".to_string()));
        let msg = &*msg_ptr;
        let fn_name_ptr = arena.alloc(Term::Var("putStrLn".to_string()));
        let fn_name = &*fn_name_ptr;
        
        let put_str_ptr = arena.alloc(Term::App(fn_name, msg));
        let put_str = &*put_str_ptr;
        
        // This should eventually lower to a syscall or a call to our @print_int/hex
        // For now, let's just ensure it doesn't panic and we can recognize IO actions.
        builder.lower_term(put_str, &env);
    }
}
