//! # REPL Session Tests
//!
//! This module contains tests for the Idris Native REPL's expression 
//! evaluation and session management.
//!
//! # Strategic Architecture
//! These tests verify that the `repl_session` adapter correctly 
//! orchestrates the compiler frontend to provide a functional REPL.

pub mod real_eval_tests;

use super::ReplSession;

#[test]
fn test_eval_integer() {
    let session = ReplSession::new();
    let result = session.eval("42");
    // We expect the result of evaluating an integer literal to be its string representation.
    assert_eq!(result, "42");
}

#[test]
fn test_eval_string() {
    let session = ReplSession::new();
    let result = session.eval("\"Hello, Idris!\"");
    // We expect the result of evaluating a string literal to be its string representation (including quotes).
    assert_eq!(result, "\"Hello, Idris!\"");
}

#[test]
fn test_type_inspection_integer() {
    let session = ReplSession::new();
    let result = session.eval(":t 42");
    // We expect the type of an integer literal to be 'Integer'.
    assert_eq!(result, "Integer");
}

#[test]
fn test_type_inspection_string() {
    let session = ReplSession::new();
    let result = session.eval(":t \"Hello\"");
    // We expect the type of a string literal to be 'String'.
    assert_eq!(result, "String");
}

#[test]
fn test_file_loading() {
    let session = ReplSession::new();
    let result = session.eval(":l test.idr");
    // We expect the result of loading a file to be a success message.
    assert_eq!(result, "Loaded file: test.idr");
}
