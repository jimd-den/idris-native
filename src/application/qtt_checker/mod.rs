//! # QTT Checker (Use Case)
//!
//! This module implements the Quantitative Type Theory (QTT) elaboration 
//! and resource tracking logic for the Idris 2 compiler.

use crate::domain::multiplicity::Multiplicity;
use crate::domain::Term;
use std::cmp::max;

pub struct QttChecker {
    // We will add state here as needed.
}

impl QttChecker {
    pub fn new() -> Self {
        Self {}
    }

    /// Recursively validates the term against QTT constraints.
    pub fn check_term(&self, term: &Term) -> bool {
        match term {
            Term::Add(lhs, rhs) | Term::Sub(lhs, rhs) | Term::Eq(lhs, rhs) | Term::App(lhs, rhs) |
            Term::BitXor(lhs, rhs) | Term::BitAnd(lhs, rhs) | Term::BitOr(lhs, rhs) |
            Term::Shl(lhs, rhs) | Term::Shr(lhs, rhs) => {
                self.check_term(lhs) && self.check_term(rhs)
            }
            Term::BitNot(body) => self.check_term(body),
            Term::BufferLoad(buffer, index) => {
                if !self.check_term(buffer) || !self.check_term(index) {
                    return false;
                }
                self.check_buffer_bounds(buffer, index)
            }
            Term::BufferStore(buffer, index, value) => {
                if !self.check_term(buffer) || !self.check_term(index) || !self.check_term(value) {
                    return false;
                }
                self.check_buffer_bounds(buffer, index)
            }
            Term::If(cond, then_br, else_br) => {
                self.check_term(cond) && self.check_term(then_br) && self.check_term(else_br)
            }
            Term::Lambda(_, _, body) => self.check_term(body),
            Term::Pi(_, _, _, body) => self.check_term(body),
            Term::LetRec(_, _, body) => self.check_term(body),
            Term::Let(_, val, body) => {
                self.check_term(val) && self.check_term(body)
            }
            Term::Case(target, branches) => {
                if !self.check_term(target) { return false; }
                for (_, _, body) in branches {
                    if !self.check_term(body) { return false; }
                }
                true
            }
            Term::Var(_) | Term::Integer(_) | Term::IntegerType | Term::I32Type | Term::I8Type | Term::Bits64Type | Term::IOType | Term::Buffer(_) => true,
        }
    }

    /// Safely checks if a buffer access is within bounds.
    fn check_buffer_bounds(&self, buffer: &Term, index: &Term) -> bool {
        match (buffer, index) {
            (Term::Buffer(size), Term::Integer(idx)) => {
                if *idx < 0 || *idx >= (*size as i64) {
                    return false;
                }
            }
            _ => (),
        }
        true
    }

    /// Validates that a specific variable name satisfies its multiplicity constraint.
    pub fn check_multiplicity(&self, name: &str, quantity: Multiplicity, body: &Term) -> bool {
        let usage = self.count_usage(name, body) as usize;
        match quantity {
            Multiplicity::Zero => usage == 0,
            Multiplicity::One => usage == 1,
            Multiplicity::Many => true,
        }
    }

    /// Sums the usage of a name across two terms.
    fn count_binary(&self, name: &str, l: &Term, r: &Term) -> i64 {
        self.count_usage(name, l) + self.count_usage(name, r)
    }

    fn count_usage(&self, name: &str, term: &Term) -> i64 {
        match term {
            Term::Var(v) if v == name => 1,
            Term::Var(_) | Term::Integer(_) | Term::IntegerType | Term::I32Type | Term::I8Type | Term::Buffer(_) => 0,
            
            Term::Add(l, r) | Term::Sub(l, r) | Term::Eq(l, r) | Term::App(l, r) |
            Term::BitXor(l, r) | Term::BitAnd(l, r) | Term::BitOr(l, r) |
            Term::Shl(l, r) | Term::Shr(l, r) | Term::BufferLoad(l, r) => {
                self.count_binary(name, l, r)
            }
            
            Term::BitNot(b) => self.count_usage(name, b),
            
            Term::BufferStore(b, i, v) => {
                self.count_usage(name, b) + self.count_usage(name, i) + self.count_usage(name, v)
            }
            
            Term::If(c, t, e) => {
                self.count_usage(name, c) + max(self.count_usage(name, t), self.count_usage(name, e))
            }
            
            Term::Lambda(n, _, b) | Term::Pi(n, _, _, b) | Term::LetRec(n, _, b) | Term::Let(n, _, b) => {
                if n == name {
                    0 // Shadowed
                } else {
                    self.count_usage(name, b)
                }
            }
            
            Term::Case(target, branches) => {
                let target_usage = self.count_usage(name, target);
                let mut max_branch_usage = 0;
                for (pat_name, pat_args, body) in branches {
                    if pat_name != name && !pat_args.contains(&name.to_string()) {
                        let u = self.count_usage(name, body);
                        if u > max_branch_usage { max_branch_usage = u; }
                    }
                }
                target_usage + max_branch_usage
            }
            
            Term::Bits64Type | Term::IOType => 0,
        }
    }

    /// Checks if a term's usage matches its QTT multiplicity.
    pub fn check_usage(&self, multiplicity: Multiplicity, count: usize) -> bool {
        match multiplicity {
            Multiplicity::Zero => count == 0,
            Multiplicity::One => count == 1,
            Multiplicity::Many => true,
        }
    }

    /// Elaborates an ADT definition.
    pub fn elaborate_adt(&self, _name: &str) -> bool {
        true
    }

    /// Elaborates an interface definition.
    pub fn elaborate_interface(&self, _name: &str) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    pub mod buffer_qtt_tests;
    pub mod multiplicity_tests;
}
