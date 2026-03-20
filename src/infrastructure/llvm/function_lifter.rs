//! # Function Lifter (Infrastructure/LLVM)
//!
//! This module implements lambda lifting and manages top-level LLVM 
//! function definitions.

use crate::domain::Term;
use std::collections::{HashMap, HashSet};

/// ISO 8601: 2026-03-19T20:55:29Z
/// Pattern: Strategy Pattern (Workable iʃɛ́)
/// Why: Decoupling lambda-to-function lowering from the core IR emission 
/// allows us to swap lifting strategies (e.g., closure vs. lifted params) 
/// without affecting the main builder.
pub struct FunctionLifter {
    /// Maps function names to their LLVM definitions.
    pub function_definitions: HashMap<String, String>,
    /// Maps function names to their captured (lambda-lifted) variable names.
    pub lifted_captures: HashMap<String, Vec<String>>,
    /// Counter for generating unique lambda function names.
    pub fn_counter: usize,
}

impl FunctionLifter {
    /// S-02: Side-effect free construction.
    pub fn new() -> Self {
        Self {
            function_definitions: HashMap::new(),
            lifted_captures: HashMap::new(),
            fn_counter: 0,
        }
    }

    /// Generates a unique name for a lifted lambda function.
    pub fn new_fn_name(&mut self) -> String {
        let name = format!("lambda_fn_{}", self.fn_counter);
        self.fn_counter += 1;
        name
    }

    /// Lowers a function definition to an LLVM IR function.
    pub fn lower_def(
        &mut self,
        name: &str,
        args: &[String],
        body_reg: &str,
        instructions: Vec<String>,
        env: &HashMap<String, String>,
        captures: &[String],
        bit_width: u32,
        sanitized_name: &str,
    ) -> String {
        let ty = format!("i{}", bit_width);
        let mut arg_str = String::new();

        // Captured variables come first as extra parameters
        for cap in captures {
            if !arg_str.is_empty() { arg_str.push_str(", "); }
            let sanitized = cap.replace("\"", ""); // Simplified for parameter name
            arg_str.push_str(&format!("{} %{}", ty, sanitized));
        }

        // Then the declared parameters
        for arg in args {
            if !arg_str.is_empty() { arg_str.push_str(", "); }
            let sanitized = arg.replace("\"", "");
            arg_str.push_str(&format!("{} %{}", ty, sanitized));
        }

        let mut fn_def = String::new();
        fn_def.push_str(&format!("define {} @{}({}) {{\n", ty, sanitized_name, arg_str));
        for instr in instructions {
            fn_def.push_str(&instr);
        }
        fn_def.push_str(&format!("  ret {} {}\n}}\n", ty, body_reg));

        self.function_definitions.insert(sanitized_name.to_string(), fn_def);

        // Record captures so call sites can supply the extra arguments
        if !captures.is_empty() {
            self.lifted_captures.insert(name.to_string(), captures.to_vec());
        }

        String::from("void")
    }

    /// Collects free variables in a term that are not in `bound`.
    pub fn collect_free_vars<'b>(term: &'b Term, bound: &HashSet<&str>, out: &mut Vec<String>) {
        match term {
            Term::Var(name) if !bound.contains(name.as_str()) => {
                out.push(name.clone());
            }
            Term::App(f, a) => {
                Self::collect_free_vars(f, bound, out);
                Self::collect_free_vars(a, bound, out);
            }
            Term::Add(l, r) | Term::Sub(l, r) | Term::Mul(l, r) | Term::Div(l, r) |
            Term::Append(l, r) | Term::BitXor(l, r) | Term::BitAnd(l, r) |
            Term::BitOr(l, r) | Term::Shl(l, r) | Term::Shr(l, r) |
            Term::Eq(l, r) | Term::Lt(l, r) | Term::Gt(l, r) |
            Term::BufferLoad(l, r) => {
                Self::collect_free_vars(l, bound, out);
                Self::collect_free_vars(r, bound, out);
            }
            Term::If(c, t, e) | Term::BufferStore(c, t, e) => {
                Self::collect_free_vars(c, bound, out);
                Self::collect_free_vars(t, bound, out);
                Self::collect_free_vars(e, bound, out);
            }
            Term::Let(n, v, b) | Term::LetRec(n, v, b) => {
                Self::collect_free_vars(v, bound, out);
                let mut inner = bound.clone();
                inner.insert(n.as_str());
                Self::collect_free_vars(b, &inner, out);
            }
            Term::Lambda(n, _, b) => {
                let mut inner = bound.clone();
                inner.insert(n.as_str());
                Self::collect_free_vars(b, &inner, out);
            }
            Term::Case(target, branches) => {
                Self::collect_free_vars(target, bound, out);
                for (_, args, body) in branches {
                    let mut inner = bound.clone();
                    for a in args { inner.insert(a.as_str()); }
                    Self::collect_free_vars(body, &inner, out);
                }
            }
            Term::BitNot(t) | Term::Bind(_, t) => {
                Self::collect_free_vars(t, bound, out);
            }
            Term::Do(stmts) | Term::Mutual(stmts) => {
                for s in stmts { Self::collect_free_vars(s, bound, out); }
            }
            Term::Where(body, defs) => {
                Self::collect_free_vars(body, bound, out);
                for d in defs { Self::collect_free_vars(d, bound, out); }
            }
            Term::Def(_, args, body) => {
                let mut inner = bound.clone();
                for a in args { inner.insert(a.as_str()); }
                Self::collect_free_vars(body, &inner, out);
            }
            _ => {} // Literals, types, etc. have no free variables
        }
    }
}
