//! # QTT Checker Tests
//!
//! This module contains tests for the Idris Native QTT checker's 
//! elaboration and resource tracking logic.
//!
//! # Strategic Architecture
//! These tests verify that the `qtt_checker` use-case correctly 
//! orchestrates domain logic (Entities) to perform type checking.

use super::QttChecker;
use crate::domain::multiplicity::Multiplicity;

#[test]
fn test_elaborate_maybe_adt() {
    let checker = QttChecker::new();
    // We expect the elaboration of a simple ADT like 'Maybe' to succeed.
    let success = checker.elaborate_adt("Maybe");
    assert!(success);
}

#[test]
fn test_elaborate_show_interface() {
    let checker = QttChecker::new();
    // We expect the elaboration of a simple interface like 'Show' to succeed.
    let success = checker.elaborate_interface("Show");
    assert!(success);
}

#[test]
fn test_check_multiplicity_usage() {
    let checker = QttChecker::new();
    
    // We expect a term with multiplicity 'One' to be valid for exactly one use.
    // This is a foundational test for our QTT and zero-GC performance goals.
    let success = checker.check_usage(Multiplicity::One, 1);
    assert!(success);
    
    // Using a 'One' multiplicity term twice should fail.
    let fail = checker.check_usage(Multiplicity::One, 2);
    assert!(!fail);
}
