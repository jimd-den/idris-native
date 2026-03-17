//! # Compiler (Use Case)
//!
//! This module serves as the primary pipeline orchestrator for the 
//! Idris 2 compiler, coordinating parsing, checking, and lowering.
//!
//! # Strategic Architecture
//! The `compiler` use-case is the "brain" of the application logic, 
//! bridging the high-level syntax with the core QTT checker and the 
//! eventual code generation backends.
//!
//! # Screaming Architecture
//! By placing the compiler pipeline at the top level, we explicitly 
//! "scream" the primary purpose of this project: building a native 
//! Idris 2 compiler.
