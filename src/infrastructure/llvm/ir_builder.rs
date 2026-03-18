//! # LLVM IR Builder (Infrastructure)
//!
//! This module implements the `IRBuilder`, which is responsible for 
//! translating high-level `Term` nodes into LLVM IR instructions.
//!
//! # Strategic Architecture
//! By isolating the IR construction logic, we keep the `LlvmBackend` 
//! clean and focused on orchestrating the overall code generation 
//! and toolchain integration.

use crate::domain::Term;
use std::collections::HashMap;

/// The IRBuilder translates our `Term` AST directly into LLVM IR.
pub struct IRBuilder {
    pub instructions: Vec<String>,
    next_reg: usize,
    label_counter: usize,
}

impl IRBuilder {
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
            next_reg: 1,
            label_counter: 0,
        }
    }

    fn new_reg(&mut self) -> String {
        let reg = format!("%r{}", self.next_reg);
        self.next_reg += 1;
        reg
    }

    fn new_label(&mut self, prefix: &str) -> String {
        let label = format!("{}_{}", prefix, self.label_counter);
        self.label_counter += 1;
        label
    }

    /// Lowers a high-level term into LLVM IR.
    pub fn lower_term(&mut self, term: &Term, env: &HashMap<String, String>) -> String {
        match term {
            Term::Integer(n) => format!("i64 {}", n),
            Term::Var(name) => {
                env.get(name).cloned().unwrap_or_else(|| format!("%{}", name))
            }
            Term::Add(l, r) => {
                let lv = self.lower_term(l, env);
                let rv = self.lower_term(r, env);
                let res = self.new_reg();
                self.instructions.push(format!("  {} = add i64 {}, {}", res, lv, rv));
                res
            }
            Term::Eq(l, r) => {
                let lv = self.lower_term(l, env);
                let rv = self.lower_term(r, env);
                let cmp = self.new_reg();
                self.instructions.push(format!("  {} = icmp eq i64 {}, {}", cmp, lv, rv));
                let res = self.new_reg();
                self.instructions.push(format!("  {} = zext i1 {} to i64", res, cmp));
                res
            }
            Term::If(c, t, e) => {
                let cond_val = self.lower_term(c, env);
                let cond_bool = self.new_reg();
                self.instructions.push(format!("  {} = icmp ne i64 {}, 0", cond_bool, cond_val));
                
                let then_label = self.new_label("then");
                let else_label = self.new_label("else");
                let merge_label = self.new_label("if_merge");
                
                self.instructions.push(format!("  br i1 {}, label %{}, label %{}", cond_bool, then_label, else_label));
                
                self.instructions.push(format!("{}:", then_label));
                let then_val = self.lower_term(t, env);
                self.instructions.push(format!("  br label %{}", merge_label));
                
                self.instructions.push(format!("{}:", else_label));
                let else_val = self.lower_term(e, env);
                self.instructions.push(format!("  br label %{}", merge_label));
                
                self.instructions.push(format!("{}:", merge_label));
                let res = self.new_reg();
                self.instructions.push(format!("  {} = phi i64 [ {}, %{} ], [ {}, %{} ]", res, then_val, then_label, else_val, else_label));
                res
            }
            // L-01: Handle previously unhandled variants to avoid panic.
            Term::Lambda(_name, _type, _body) => {
                // Placeholder: For MVP, lambdas require closure conversion.
                // We return a null-like IR value for now.
                "i64 0".to_string()
            }
            Term::Pi(_, _, _, _) | Term::IntegerType | Term::I32Type | Term::I8Type | Term::Bits64Type | Term::IOType => {
                "i64 0".to_string()
            }
            _ => {
                // Return a safe placeholder instead of panicking.
                "i64 0".to_string()
            }
        }
    }
}
