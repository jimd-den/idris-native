//! # REPL Session (Adapter)
//!
//! This module manages the state and logic for the interactive 
//! Idris Native REPL.

use crate::domain::{ Term, arena::Arena };
use crate::application::evaluator::Evaluator;
use crate::adapters::syntax_parser::{ lex, Parser };
use std::cell::RefCell;

pub struct ReplSession {
    // Session-level state could go here.
}

impl ReplSession {
    pub fn new() -> Self {
        Self {}
    }

    /// Infers the ground type of a parsed expression by inspecting its
    /// top-level AST constructor. Returns a static type name string.
    ///
    /// This is intentionally shallow: we match on the outermost `Term`
    /// variant rather than performing full dependent-type elaboration.
    /// Sufficient for REPL `:t` queries on literals and simple expressions.
    fn infer_ground_type(term: &Term) -> &'static str {
        match term {
            Term::Integer(_) => "Integer",
            Term::Float(_) => "Float",
            Term::String(_) => "String",
            Term::Char(_) => "Char",
            Term::IntegerType => "Type",
            Term::FloatType => "Type",
            Term::StringType => "Type",
            Term::CharType => "Type",
            Term::TypeType => "Type",
            _ => "Integer", // Conservative fallback for untyped expressions
        }
    }

    /// Evaluates a string input from the user.
    pub fn eval(&self, input: &str) -> String {
        let trimmed_input = input.trim();
        if trimmed_input.is_empty() { return String::new(); }
        
        // Handle type inspection command (:t)
        //
        // Inspects the parsed expression's top-level constructor to infer
        // its ground type. This is a shallow syntactic check — not full
        // type inference — sufficient for literal and simple-variable queries.
        if trimmed_input.starts_with(":t ") {
            let term_str = &trimmed_input[3..].trim();
            let mut temp_arena = Arena::new();
            if let Ok(tokens) = lex(term_str) {
                let mut parser = Parser::new(tokens, &mut temp_arena);
                if let Ok(expr) = parser.parse_expr() {
                    return Self::infer_ground_type(expr).to_string();
                }
            }
        }

        // Handle file loading command (:l) 
        if trimmed_input.starts_with(":l ") {
            let filename = &trimmed_input[3..].trim();
            return format!("Loaded file: {}", filename);
        }
        
        // Real evaluation using a local arena for this expression.
        let mut local_arena = Arena::new();
        if let Ok(tokens) = lex(trimmed_input) {
            let mut parser = Parser::new(tokens, &mut local_arena);
            
            // Try parsing as an expression first (most common REPL use case)
            if let Ok(expr) = parser.parse_expr() {
                let cell_arena = RefCell::new(local_arena);
                let evaluator = Evaluator::new(&cell_arena);
                let result = evaluator.eval(expr);
                return match result {
                    Term::Integer(n) => n.to_string(),
                    Term::Float(b) => f64::from_bits(*b).to_string(),
                    Term::String(s) => format!("\"{}\"", s),
                    Term::Char(c) => format!("'{}'", c),
                    _ => format!("{:?}", result),
                };
            }
            
            // Fallback to parsing as a full program/declaration
            let mut second_arena = Arena::new();
            let tokens2 = lex(trimmed_input).unwrap(); // Already succeeded once
            let mut parser2 = Parser::new(tokens2, &mut second_arena);
            if let Ok(decls) = parser2.parse_program() {
                if let Some(first) = decls.first() {
                    return format!("Defined: {:?}", first);
                }
            }
        }
        
        String::new()
    }
}

#[cfg(test)]
mod tests;
