//! # Term Entity (Domain Layer)
//!
//! This module defines the foundational `Term` data structure, which represents 
//! the core Abstract Syntax Tree (AST) of the Idris 2 language.
//!
//! # Strategic Architecture
//! As an Entity, `Term` is at the center of our Screaming Architecture. 
//! It has zero dependencies on any other part of the system (other than 
//! the standard library).
//!
//! # Literate Documentation
//! In Idris 2, everything is a term. Whether it is a type, a value, or 
//! a function application, it is represented as a node in this AST. 
//! To achieve performance exceeding C, we ensure that these terms can 
//! be efficiently traversed and manipulated within the `Arena`.

/// The core representation of an Idris 2 term.
/// 
/// Why this exists:
/// This is the foundational entity of our compiler. Every Idris 2 term 
/// (expressions, types, interfaces) is eventually represented as a `Term` 
/// within the `Arena`.
#[derive(Debug, Clone)]
pub enum Term<'a> {
    /// A variable reference (using String for readability, intended for de Bruijn conversion).
    Var(String),
    /// A lambda abstraction: \x:type. body
    /// Represents an anonymous function.
    Lambda(String, &'a Term<'a>, &'a Term<'a>),
    /// A function application: func arg
    /// Represents applying a function to its argument.
    App(&'a Term<'a>, &'a Term<'a>),
    /// A Pi type (dependent function type): (x:type) -> body
    /// The core of Idris 2's dependent type system.
    Pi(String, &'a Term<'a>, &'a Term<'a>),
    /// A core integer constant (i64).
    Integer(i64),
    /// The base 'Integer' type (arbitrary precision in Idris, i64 in this MVP).
    IntegerType,
    /// The base 'I32' type.
    I32Type,
    /// The base 'I8' type.
    I8Type,
    /// The base 'Bits64' type.
    Bits64Type,
    /// The base 'IO' type, representing side-effecting operations.
    IOType,
    /// Arithmetic addition: a + b
    Add(&'a Term<'a>, &'a Term<'a>),
    /// Arithmetic subtraction: a - b
    Sub(&'a Term<'a>, &'a Term<'a>),
    /// Bitwise XOR: a ^ b
    BitXor(&'a Term<'a>, &'a Term<'a>),
    /// Bitwise AND: a & b
    BitAnd(&'a Term<'a>, &'a Term<'a>),
    /// Bitwise OR: a | b
    BitOr(&'a Term<'a>, &'a Term<'a>),
    /// Bitwise NOT: ~a
    BitNot(&'a Term<'a>),
    /// Bitwise shift left: a << b
    Shl(&'a Term<'a>, &'a Term<'a>),
    /// Bitwise shift right: a >> b
    Shr(&'a Term<'a>, &'a Term<'a>),
    /// Equality comparison: a == b
    Eq(&'a Term<'a>, &'a Term<'a>),
    /// Control flow: if cond then t else f
    If(&'a Term<'a>, &'a Term<'a>, &'a Term<'a>),
    /// Recursive definition (LetRec): let rec f x = body in expr
    LetRec(String, &'a Term<'a>, &'a Term<'a>),
    /// A let binding: let x = val in body
    Let(String, &'a Term<'a>, &'a Term<'a>),
    /// A fixed-size buffer primitive: Buffer(size)
    Buffer(usize),
    /// Load from buffer: BufferLoad(buffer, index)
    BufferLoad(&'a Term<'a>, &'a Term<'a>),
    /// Store into buffer: BufferStore(buffer, index, value)
    BufferStore(&'a Term<'a>, &'a Term<'a>, &'a Term<'a>),
    /// Pattern matching case expression: case target of { (pat, args, branch) }
    Case(&'a Term<'a>, Vec<(String, Vec<String>, &'a Term<'a>)>),
}

/// Represents an Algebraic Data Type (ADT) definition.
/// 
/// Why this exists:
/// ADTs are the primary way users define custom data structures in Idris 2.
pub struct AdtDefinition<'a> {
    pub name: String,
    pub params: Vec<String>,
    pub constructors: Vec<(String, Vec<&'a Term<'a>>)>,
}
