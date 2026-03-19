//! # Real REPL Evaluation Tests
//!
//! These tests verify that the `ReplSession` correctly invokes the 
//! full compiler/evaluator pipeline for expressions (Category C).

use crate::adapters::repl_session::ReplSession;

#[test]
fn test_repl_real_eval_addition() {
    let mut session = ReplSession::new();
    
    // This currently returns the input string if it looks like digits.
    // We want it to actually evaluate 1 + 1 using the real pipeline.
    let result = session.eval("1 + 1");
    
    assert_eq!(result, "2");
}
