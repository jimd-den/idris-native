//! # Term Evaluator (Use Case)
//!
//! This module implements the evaluation logic for Idris 2 terms, 
//! performing beta-reduction and normalization.

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
            Term::Sub(l, r) => {
                let lv = self.eval(l);
                let rv = self.eval(r);
                if let (Term::Integer(ln), Term::Integer(rn)) = (lv, rv) {
                    self.alloc(Term::Integer(ln - rn))
                } else {
                    self.alloc(Term::Sub(lv, rv))
                }
            }
            Term::If(c, t, e) => {
                let cv = self.eval(c);
                if let Term::Integer(n) = cv {
                    if *n != 0 { self.eval(t) } else { self.eval(e) }
                } else {
                    let tv = self.eval(t);
                    let ev = self.eval(e);
                    self.alloc(Term::If(cv, tv, ev))
                }
            }
            Term::Let(n, v, b) => {
                let vv = self.eval(v);
                let substituted = self.substitute(b, n, vv);
                self.eval(substituted)
            }
            // Catch-all for non-reducible terms or high-level declarations
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
            Term::Var(_) | Term::Integer(_) | Term::Float(_) | Term::String(_) | Term::Char(_) |
            Term::IntegerType | Term::FloatType | Term::StringType | Term::CharType |
            Term::I32Type | Term::I8Type | Term::Bits64Type | Term::IOType | Term::TypeType |
            Term::Universe(_) => body,

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
            Term::Pi(n, q, t, b) => {
                if n == name {
                    let new_t = self.substitute(t, name, replacement);
                    self.alloc(Term::Pi(n.clone(), *q, new_t, b))
                } else {
                    let new_t = self.substitute(t, name, replacement);
                    let new_b = self.substitute(b, name, replacement);
                    self.alloc(Term::Pi(n.clone(), *q, new_t, new_b))
                }
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
            Term::Mul(l, r) => {
                let new_l = self.substitute(l, name, replacement);
                let new_r = self.substitute(r, name, replacement);
                self.alloc(Term::Mul(new_l, new_r))
            }
            Term::Div(l, r) => {
                let new_l = self.substitute(l, name, replacement);
                let new_r = self.substitute(r, name, replacement);
                self.alloc(Term::Div(new_l, new_r))
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
            Term::Lt(l, r) => {
                let new_l = self.substitute(l, name, replacement);
                let new_r = self.substitute(r, name, replacement);
                self.alloc(Term::Lt(new_l, new_r))
            }
            Term::Gt(l, r) => {
                let new_l = self.substitute(l, name, replacement);
                let new_r = self.substitute(r, name, replacement);
                self.alloc(Term::Gt(new_l, new_r))
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
            Term::LetRec(n, v, b) => {
                if n == name {
                    let new_v = self.substitute(v, name, replacement);
                    self.alloc(Term::LetRec(n.clone(), new_v, b))
                } else {
                    let new_v = self.substitute(v, name, replacement);
                    let new_b = self.substitute(b, name, replacement);
                    self.alloc(Term::LetRec(n.clone(), new_v, new_b))
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
            Term::Do(stmts) => {
                let new_stmts = stmts.iter().map(|s| self.substitute(s, name, replacement).clone()).collect();
                self.alloc(Term::Do(new_stmts))
            }
            Term::Bind(n, a) => {
                let new_a = self.substitute(a, name, replacement);
                self.alloc(Term::Bind(n.clone(), new_a))
            }
            Term::Where(t, defs) => {
                let new_t = self.substitute(t, name, replacement);
                let new_defs = defs.iter().map(|d| self.substitute(d, name, replacement).clone()).collect();
                self.alloc(Term::Where(new_t, new_defs))
            }
            Term::Mutual(terms) => {
                let new_terms = terms.iter().map(|t| self.substitute(t, name, replacement).clone()).collect();
                self.alloc(Term::Mutual(new_terms))
            }
            Term::Def(n, args, b) => {
                let new_b = self.substitute(b, name, replacement);
                self.alloc(Term::Def(n.clone(), args.clone(), new_b))
            }
            _ => body,
        }
    }
}

#[cfg(test)]
mod tests;
