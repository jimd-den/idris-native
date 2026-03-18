//! # Hypothesis Tests: QTT Usage
//!
//! This module contains targeted tests to prove that `QttChecker` 
//! incorrectly sums usages across mutual exclusive branches.

use crate::domain::{Term, arena::Arena};
use crate::application::qtt_checker::QttChecker;

#[test]
fn test_hypothesis_qtt_if_branches() {
    let mut arena: Arena<Term> = Arena::new();
    let checker = QttChecker::new();
    
    unsafe {
        // (1 x : Integer) -> if cond then x else x
        let x_var = &*arena.alloc(Term::Var("x".to_string()));
        let cond = &*arena.alloc(Term::Var("cond".to_string()));
        let body = &*arena.alloc(Term::If(cond, x_var, x_var));
        
        // PROOF: If this is false, our count_usage is too strict for If.
        assert!(checker.check_multiplicity("x", 1, body), 
                "Hypothesis Proof: QttChecker should allow linear variable in mutual exclusive branches.");
    }
}
