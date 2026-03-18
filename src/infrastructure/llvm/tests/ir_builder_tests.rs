//! # IR Builder Tests
//!
//! These tests verify that the `IRBuilder` correctly generates LLVM IR 
//! for various `Term` nodes.

use crate::infrastructure::llvm::ir_builder::IRBuilder;
use crate::domain::Term;
use std::collections::HashMap;

#[test]
fn test_ir_builder_integer() {
    let mut builder = IRBuilder::new();
    let env = HashMap::new();
    let term = Term::Integer(42);
    
    let res = builder.lower_term(&term, &env);
    assert_eq!(res, "i64 42");
    assert!(builder.instructions.is_empty());
}

#[test]
fn test_ir_builder_add() {
    let mut builder = IRBuilder::new();
    let env = HashMap::new();
    let term = Term::Add(&Term::Integer(1), &Term::Integer(2));
    
    let res = builder.lower_term(&term, &env);
    assert_eq!(res, "%r1");
    assert_eq!(builder.instructions.len(), 1);
    assert!(builder.instructions[0].contains("add i64 i64 1, i64 2"));
}
