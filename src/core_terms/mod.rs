//! # Core Terms (Entities)
//!
//! This module defines the core terms and data structures for the Idris 2 
//! compiler, including the Abstract Syntax Tree (AST) and QTT-related types.
//!
//! # Strategic Architecture
//! This module is placed at the top level to explicitly announce its role 
//! as the source of truth for all compiler entities.
//!
//! # Performance & Purity
//! To achieve performance exceeding well-optimized C, all terms in this 
//! module are allocated within our custom internal `Arena`, ensuring 
//! high cache locality and O(1) bulk deallocation.
//!
//! # QTT & Zero-GC
//! By leveraging Quantitative Type Theory (QTT) for memory management, we 
//! avoid the overhead of a garbage collector. Our `Arena` provides 
//! the physical memory pool for these QTT-checked terms.

pub mod arena;
pub mod multiplicity;

/// The core representation of an Idris 2 term.
/// 
/// Why this exists:
/// This is the foundational entity of our compiler. Every Idris 2 term 
/// (expressions, types, interfaces) is eventually represented as a `Term` 
/// within the `Arena`.
#[derive(Debug, Clone)]
pub enum Term<'a> {
    /// A variable reference (de Bruijn name/index for efficiency).
    Var(String),
    /// A lambda abstraction: \x:type. body
    Lambda(String, &'a Term<'a>, &'a Term<'a>),
    /// A function application: func arg
    App(&'a Term<'a>, &'a Term<'a>),
    /// A Pi type (dependent function type): (x:type) -> body
    Pi(String, &'a Term<'a>, &'a Term<'a>),
    /// A core integer constant.
    Integer(i64),
    /// The base 'Integer' type.
    IntegerType,
    /// The base 'I32' type.
    I32Type,
    /// The base 'I8' type.
    I8Type,
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
}

#[cfg(test)]
mod tests {
    pub mod arena_tests;
    pub mod term_tests;
    pub mod sha256_primitives_tests;
}
