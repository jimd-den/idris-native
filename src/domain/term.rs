//! # Term Entity (Domain Layer)
//!
//! This module defines the foundational `Term` data structure, which represents 
//! the core Abstract Syntax Tree (AST) of the Idris 2 language.

use crate::domain::multiplicity::Multiplicity;

/// The core representation of an Idris 2 term.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Term<'a> {
    // --- Foundation ---
    /// A variable reference.
    Var(String),
    /// A lambda abstraction: \x:type. body
    Lambda(String, &'a Term<'a>, &'a Term<'a>),
    /// A function application: func arg
    App(&'a Term<'a>, &'a Term<'a>),
    /// A Pi type (dependent function type): (q x:type) -> body
    Pi(String, Multiplicity, &'a Term<'a>, &'a Term<'a>),
    /// A universe level (Type n).
    Universe(usize),

    // --- Literals & Basic Types ---
    /// A core integer constant.
    Integer(i64),
    /// A floating point constant.
    Float(u64), // Using u64 bits for Eq/Hash compatibility in AST
    /// A string literal.
    String(String),
    /// A character literal.
    Char(char),
    
    /// Basic types.
    IntegerType,
    FloatType,
    StringType,
    CharType,
    I32Type,
    I8Type,
    Bits64Type,
    IOType,
    TypeType,

    // --- Operations ---
    Add(&'a Term<'a>, &'a Term<'a>),
    Sub(&'a Term<'a>, &'a Term<'a>),
    Mul(&'a Term<'a>, &'a Term<'a>),
    Div(&'a Term<'a>, &'a Term<'a>),
    Append(&'a Term<'a>, &'a Term<'a>),
    
    BitXor(&'a Term<'a>, &'a Term<'a>),
    BitAnd(&'a Term<'a>, &'a Term<'a>),
    BitOr(&'a Term<'a>, &'a Term<'a>),
    BitNot(&'a Term<'a>),
    Shl(&'a Term<'a>, &'a Term<'a>),
    Shr(&'a Term<'a>, &'a Term<'a>),
    
    Eq(&'a Term<'a>, &'a Term<'a>),
    Lt(&'a Term<'a>, &'a Term<'a>),
    Gt(&'a Term<'a>, &'a Term<'a>),

    // --- Control Flow & Bindings ---
    If(&'a Term<'a>, &'a Term<'a>, &'a Term<'a>),
    /// A let binding: let x = val in body
    Let(String, &'a Term<'a>, &'a Term<'a>),
    /// Recursive definition.
    LetRec(String, &'a Term<'a>, &'a Term<'a>),
    /// Pattern matching.
    Case(&'a Term<'a>, Vec<(String, Vec<String>, &'a Term<'a>)>),
    
    // --- IO & Do Notation ---
    /// A do block: do { stmt1; stmt2; ... }
    Do(Vec<Term<'a>>),
    /// A bind in a do block: x <- action
    Bind(String, &'a Term<'a>),

    // --- High-Level Declarations ---
    /// A module declaration: module Name
    Module(String),
    /// An import statement: import Name
    Import(String),
    /// Data type definition.
    Data(String, Vec<String>, Vec<Constructor<'a>>),
    /// Interface definition.
    Interface(String, Vec<String>, Vec<Term<'a>>),
    /// Implementation of an interface.
    Implementation(String, String, Vec<Term<'a>>),
    /// A record definition.
    Record(String, Vec<(String, &'a Term<'a>)>),
    
    // --- Blocks ---
    /// A mutual block for mutually recursive definitions.
    Mutual(Vec<Term<'a>>),
    /// A where clause attaching local definitions to a term.
    Where(&'a Term<'a>, Vec<Term<'a>>),

    /// A top-level function definition: name args = body
    Def(String, Vec<String>, &'a Term<'a>),
    
    // --- Primitives ---
    Buffer(usize),
    BufferLoad(&'a Term<'a>, &'a Term<'a>),
    BufferStore(&'a Term<'a>, &'a Term<'a>, &'a Term<'a>),
}

/// Represents a constructor in a data type definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Constructor<'a> {
    pub name: String,
    pub fields: Vec<&'a Term<'a>>,
}

/// Represents an Algebraic Data Type (ADT) definition (Deprecated in favor of Term::Data).
pub struct AdtDefinition<'a> {
    pub name: String,
    pub params: Vec<String>,
    pub constructors: Vec<(String, Vec<&'a Term<'a>>)>,
}
