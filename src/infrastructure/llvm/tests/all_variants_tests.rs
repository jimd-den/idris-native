//! # Term Variant Lowering Tests
//!
//! These tests verify that all `Term` variants can be passed to `lower_term` 
//! without causing panics, ensuring the backend is robust.

use crate::infrastructure::llvm::IRBuilder;
use crate::domain::{Term, Multiplicity};
use std::collections::HashMap;

#[test]
fn test_lower_lambda() {
    let mut builder = IRBuilder::new();
    let env = HashMap::new();
    let ty = Term::IntegerType;
    let body = Term::Var("x".to_string());
    let lambda = Term::Lambda("x".to_string(), &ty, &body);
    
    // Should not panic
    let res = builder.lower_term(&lambda, &env);
    assert!(!res.is_empty());
}

#[test]
fn test_lower_pi() {
    let mut builder = IRBuilder::new();
    let env = HashMap::new();
    let ty = Term::IntegerType;
    let pi = Term::Pi("x".to_string(), Multiplicity::One, &ty, &ty);
    
    // Should not panic
    let res = builder.lower_term(&pi, &env);
    assert!(!res.is_empty());
}

#[test]
fn test_lower_letrec() {
    let mut builder = IRBuilder::new();
    let env = HashMap::new();
    let val = Term::Integer(42);
    let body = Term::Var("f".to_string());
    let letrec = Term::LetRec("f".to_string(), &val, &body);
    
    // Should not panic
    let res = builder.lower_term(&letrec, &env);
    assert!(!res.is_empty());
}

#[test]
fn test_lower_basic_types() {
    let mut builder = IRBuilder::new();
    let env = HashMap::new();
    
    builder.lower_term(&Term::IntegerType, &env);
    builder.lower_term(&Term::I32Type, &env);
    builder.lower_term(&Term::I8Type, &env);
    builder.lower_term(&Term::Bits64Type, &env);
    builder.lower_term(&Term::IOType, &env);
}
