//! # IR Builder Tests
//!
//! These tests verify that the `IRBuilder` correctly generates LLVM IR 
//! for various `Term` nodes.

use crate::infrastructure::llvm::IRBuilder;
use crate::domain::Term;
use std::collections::HashMap;

#[test]
fn test_ir_builder_integer() {
    let mut builder = IRBuilder::new();
    let env = HashMap::new();
    let term = Term::Integer(42);
    
    let res = builder.lower_term(&term, &env);
    assert_eq!(res, "42");
    assert!(builder.instructions.is_empty());
}

#[test]
fn test_ir_builder_add() {
    let mut builder = IRBuilder::new();
    let env = HashMap::new();
    let term = Term::Add(&Term::Integer(1), &Term::Integer(2));
    
    let res = builder.lower_term(&term, &env);
    assert_eq!(res, "%1");
    assert_eq!(builder.instructions.len(), 1);
    assert!(builder.instructions[0].contains("add i64 1, 2"));
}
