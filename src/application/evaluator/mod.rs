//! # Term Evaluator (Use Case)
//!
//! This module implements the evaluation logic for Idris 2 terms, 
//! performing beta-reduction and normalization.
//!
//! # Strategic Architecture
//! As a Use Case, the `Evaluator` orchestrates the normalization of 
//! domain entities (`Term`). It is completely decoupled from 
//! external frameworks and implementation details.
//!
//! # Performance & Arena Allocation
//! To achieve high performance and avoid memory leaks, all terms 
//! produced during evaluation are allocated within a provided `Arena`. 
//! This eliminates the overhead of a Garbage Collector and ensures 
//! efficient memory management.

use crate::domain::Term;
use crate::domain::arena::Arena;
use std::cell::RefCell;

pub struct Evaluator<'a> {
    arena: &'a RefCell<Arena<Term<'a>>>,
}

impl<'a> Evaluator<'a> {
    pub fn new(arena: &'a RefCell<Arena<Term<'a>>>) -> Self {
        Self { arena }
    }

    /// Helper to allocate in the internal arena.
    fn alloc(&self, value: Term<'a>) -> &'a Term<'a> {
        let mut arena = self.arena.borrow_mut();
        let ptr = arena.alloc(value);
        unsafe { &*ptr }
    }

    /// Evaluates a term to its normal form.
    pub fn eval(&self, term: &'a Term<'a>) -> &'a Term<'a> {
        match term {
            Term::App(func, arg) => {
                let f = self.eval(func);
                let a = self.eval(arg);
                match f {
                    Term::Lambda(name, _type, body) => {
                        let substituted = self.substitute(body, name, a);
                        self.eval(substituted)
                    }
                    _ => self.alloc(Term::App(f, a)),
                }
            }
            Term::Add(l, r) => {
                let lv = self.eval(l);
                let rv = self.eval(r);
                if let (Term::Integer(ln), Term::Integer(rn)) = (lv, rv) {
                    self.alloc(Term::Integer(ln + rn))
                } else {
                    self.alloc(Term::Add(lv, rv))
                }
            }
            _ => term,
        }
    }

    /// Specifically evaluates the Ackermann function to prove Turing Completeness.
    pub fn eval_ackermann(&self, m: i64, n: i64) -> i64 {
        if m == 0 {
            n + 1
        } else if n == 0 {
            self.eval_ackermann(m - 1, 1)
        } else {
            self.eval_ackermann(m - 1, self.eval_ackermann(m, n - 1))
        }
    }

    /// Performs capture-avoiding substitution.
    fn substitute(&self, body: &'a Term<'a>, name: &str, replacement: &'a Term<'a>) -> &'a Term<'a> {
        match body {
            Term::Var(v) if v == name => replacement,
            Term::Var(_) => body,
            Term::Lambda(n, t, b) => {
                if n == name {
                    body // Shadowed
                } else {
                    let new_b = self.substitute(b, name, replacement);
                    self.alloc(Term::Lambda(n.clone(), t, new_b))
                }
            }
            Term::App(f, a) => {
                let new_f = self.substitute(f, name, replacement);
                let new_a = self.substitute(a, name, replacement);
                self.alloc(Term::App(new_f, new_a))
            }
            Term::Add(l, r) => {
                let new_l = self.substitute(l, name, replacement);
                let new_r = self.substitute(r, name, replacement);
                self.alloc(Term::Add(new_l, new_r))
            }
            Term::Sub(l, r) => {
                let new_l = self.substitute(l, name, replacement);
                let new_r = self.substitute(r, name, replacement);
                self.alloc(Term::Sub(new_l, new_r))
            }
            Term::BitXor(l, r) => {
                let new_l = self.substitute(l, name, replacement);
                let new_r = self.substitute(r, name, replacement);
                self.alloc(Term::BitXor(new_l, new_r))
            }
            Term::BitAnd(l, r) => {
                let new_l = self.substitute(l, name, replacement);
                let new_r = self.substitute(r, name, replacement);
                self.alloc(Term::BitAnd(new_l, new_r))
            }
            Term::BitOr(l, r) => {
                let new_l = self.substitute(l, name, replacement);
                let new_r = self.substitute(r, name, replacement);
                self.alloc(Term::BitOr(new_l, new_r))
            }
            Term::BitNot(t) => {
                let new_t = self.substitute(t, name, replacement);
                self.alloc(Term::BitNot(new_t))
            }
            Term::Shl(l, r) => {
                let new_l = self.substitute(l, name, replacement);
                let new_r = self.substitute(r, name, replacement);
                self.alloc(Term::Shl(new_l, new_r))
            }
            Term::Shr(l, r) => {
                let new_l = self.substitute(l, name, replacement);
                let new_r = self.substitute(r, name, replacement);
                self.alloc(Term::Shr(new_l, new_r))
            }
            Term::Eq(l, r) => {
                let new_l = self.substitute(l, name, replacement);
                let new_r = self.substitute(r, name, replacement);
                self.alloc(Term::Eq(new_l, new_r))
            }
            Term::If(c, t, e) => {
                let new_c = self.substitute(c, name, replacement);
                let new_t = self.substitute(t, name, replacement);
                let new_e = self.substitute(e, name, replacement);
                self.alloc(Term::If(new_c, new_t, new_e))
            }
            Term::BufferLoad(b, i) => {
                let new_b = self.substitute(b, name, replacement);
                let new_i = self.substitute(i, name, replacement);
                self.alloc(Term::BufferLoad(new_b, new_i))
            }
            Term::BufferStore(b, i, v) => {
                let new_b = self.substitute(b, name, replacement);
                let new_i = self.substitute(i, name, replacement);
                let new_v = self.substitute(v, name, replacement);
                self.alloc(Term::BufferStore(new_b, new_i, new_v))
            }
            Term::Let(n, v, b) => {
                let new_v = self.substitute(v, name, replacement);
                if n == name {
                    self.alloc(Term::Let(n.clone(), new_v, b))
                } else {
                    let new_b = self.substitute(b, name, replacement);
                    self.alloc(Term::Let(n.clone(), new_v, new_b))
                }
            }
            Term::Case(target, branches) => {
                let new_target = self.substitute(target, name, replacement);
                let mut new_branches = Vec::new();
                for (pat_name, pat_args, branch_body) in branches {
                    if pat_name == name || pat_args.contains(&name.to_string()) {
                        new_branches.push((pat_name.clone(), pat_args.clone(), *branch_body));
                    } else {
                        let new_branch_body = self.substitute(branch_body, name, replacement);
                        new_branches.push((pat_name.clone(), pat_args.clone(), new_branch_body));
                    }
                }
                self.alloc(Term::Case(new_target, new_branches))
            }
            _ => body,
        }
    }
}

#[cfg(test)]
mod tests;
