//! # Term Entity (Domain Layer)
//!
//! This module defines the foundational `Term` data structure, which represents 
//! the core Abstract Syntax Tree (AST) of the Idris 2 language.

use crate::domain::multiplicity::Multiplicity;

/// The core representation of an Idris 2 term.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Term<'a> {
    /// A variable reference.
    Var(String),
    /// A lambda abstraction: \x:type. body
    Lambda(String, &'a Term<'a>, &'a Term<'a>),
    /// A function application: func arg
    App(&'a Term<'a>, &'a Term<'a>),
    /// A Pi type (dependent function type): (q x:type) -> body
    /// Includes QTT Multiplicity.
    Pi(String, Multiplicity, &'a Term<'a>, &'a Term<'a>),
    /// A core integer constant.
    Integer(i64),
    /// Basic types.
    IntegerType,
    I32Type,
    I8Type,
    Bits64Type,
    IOType,
    /// Arithmetic operations.
    Add(&'a Term<'a>, &'a Term<'a>),
    Sub(&'a Term<'a>, &'a Term<'a>),
    /// Bitwise operations.
    BitXor(&'a Term<'a>, &'a Term<'a>),
    BitAnd(&'a Term<'a>, &'a Term<'a>),
    BitOr(&'a Term<'a>, &'a Term<'a>),
    BitNot(&'a Term<'a>),
    Shl(&'a Term<'a>, &'a Term<'a>),
    Shr(&'a Term<'a>, &'a Term<'a>),
    /// Comparison.
    Eq(&'a Term<'a>, &'a Term<'a>),
    /// Control flow.
    If(&'a Term<'a>, &'a Term<'a>, &'a Term<'a>),
    /// bindings.
    LetRec(String, &'a Term<'a>, &'a Term<'a>),
    Let(String, &'a Term<'a>, &'a Term<'a>),
    /// Buffer primitives.
    Buffer(usize),
    BufferLoad(&'a Term<'a>, &'a Term<'a>),
    BufferStore(&'a Term<'a>, &'a Term<'a>, &'a Term<'a>),
    /// Pattern matching.
    Case(&'a Term<'a>, Vec<(String, Vec<String>, &'a Term<'a>)>),
}

/// Represents an Algebraic Data Type (ADT) definition.
pub struct AdtDefinition<'a> {
    pub name: String,
    pub params: Vec<String>,
    pub constructors: Vec<(String, Vec<&'a Term<'a>>)>,
}
