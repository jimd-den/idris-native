//! # Evaluator Tests
//!
//! This module contains tests for the Idris Native term evaluator.

use crate::domain::arena::Arena;
use crate::domain::Term;
use crate::application::evaluator::Evaluator;

#[test]
fn test_eval_app_lambda() {
    let mut arena = Arena::new();
    let evaluator = Evaluator::new();
    
    // Create (\x. x) 42
    unsafe {
        let x_var = &*arena.alloc(Term::Var("x".to_string()));
        let int_type = &*arena.alloc(Term::IntegerType);
        let lambda = &*arena.alloc(Term::Lambda("x".to_string(), int_type, x_var));
        let forty_two = &*arena.alloc(Term::Integer(42));
        let app = &*arena.alloc(Term::App(lambda, forty_two));
        
        // Evaluate (\x. x) 42 -> 42
        let result = evaluator.eval(app);
        
        match result {
            Term::Integer(n) => assert_eq!(n, 42),
            _ => panic!("Expected Integer(42), got {:?}", result),
        }
    }
}

#[test]
fn test_ackermann() {
    let mut _arena: Arena<Term> = Arena::new();
    let evaluator = Evaluator::new();
    
    // ack m n = 
    //   if m == 0 then n + 1
    //   else if n == 0 then ack (m - 1) 1
    //   else ack (m - 1) (ack m (n - 1))
    
    let result = evaluator.eval_ackermann(1, 1);
    assert_eq!(result, 3);
    
    let result_2_2 = evaluator.eval_ackermann(2, 2);
    assert_eq!(result_2_2, 7);
}
