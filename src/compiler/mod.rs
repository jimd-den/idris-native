use crate::core_terms::Term;
use std::collections::HashMap;

/// The IRBuilder translates our `Term` AST directly into LLVM IR.
/// 
/// Why this exists:
/// This is the core of our End-to-End Compiler Pipeline. It bridges the 
/// gap between the high-level Idris 2 semantics (which are QTT-checked 
/// and guaranteed memory-safe) and low-level LLVM representations.
pub struct IRBuilder {
    pub instructions: Vec<String>,
    next_reg: usize,
    label_counter: usize,
    current_block: String,
}

impl IRBuilder {
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
            next_reg: 1, // Start at 1 to avoid colliding with named arguments
            label_counter: 0,
            current_block: "entry".to_string(),
        }
    }

    pub fn fresh_reg(&mut self) -> String {
        let reg = format!("%{}", self.next_reg);
        self.next_reg += 1;
        reg
    }

    pub fn fresh_label(&mut self, prefix: &str) -> String {
        let label = format!("{}_{}", prefix, self.label_counter);
        self.label_counter += 1;
        label
    }

    /// Recursively lowers a `Term` to LLVM IR, returning the register/value string.
    pub fn lower_term(&mut self, term: &Term, env: &HashMap<String, String>) -> String {
        match term {
            Term::Integer(val) => format!("{}", val),
            Term::Var(name) => {
                env.get(name).cloned().unwrap_or_else(|| format!("@{}", name)) // Assume global if not in local env
            }
            Term::Add(lhs, rhs) => {
                let l_reg = self.lower_term(lhs, env);
                let r_reg = self.lower_term(rhs, env);
                let res = self.fresh_reg();
                self.instructions.push(format!("  {} = add i64 {}, {}", res, l_reg, r_reg));
                res
            }
            Term::Sub(lhs, rhs) => {
                let l_reg = self.lower_term(lhs, env);
                let r_reg = self.lower_term(rhs, env);
                let res = self.fresh_reg();
                self.instructions.push(format!("  {} = sub i64 {}, {}", res, l_reg, r_reg));
                res
            }
            Term::Eq(lhs, rhs) => {
                let l_reg = self.lower_term(lhs, env);
                let r_reg = self.lower_term(rhs, env);
                let res = self.fresh_reg();
                self.instructions.push(format!("  {} = icmp eq i64 {}, {}", res, l_reg, r_reg));
                res
            }
            Term::If(cond, then_br, else_br) => {
                let cond_reg = self.lower_term(cond, env);

                let then_label = self.fresh_label("then");
                let else_label = self.fresh_label("else");
                let merge_label = self.fresh_label("merge");

                self.instructions.push(format!("  br i1 {}, label %{}, label %{}", cond_reg, then_label, else_label));

                self.instructions.push(format!("\n{}:", then_label));
                self.current_block = then_label.clone();
                let then_reg = self.lower_term(then_br, env);
                let final_then_block = self.current_block.clone();
                self.instructions.push(format!("  br label %{}", merge_label));

                self.instructions.push(format!("\n{}:", else_label));
                self.current_block = else_label.clone();
                let else_reg = self.lower_term(else_br, env);
                let final_else_block = self.current_block.clone();
                self.instructions.push(format!("  br label %{}", merge_label));

                self.instructions.push(format!("\n{}:", merge_label));
                self.current_block = merge_label.clone();
                let res = self.fresh_reg();
                self.instructions.push(format!("  {} = phi i64 [ {}, %{} ], [ {}, %{} ]", res, then_reg, final_then_block, else_reg, final_else_block));
                res
            }
            Term::App(_, _) => {
                // Flatten curried application: App(App(f, a), b) -> f(a, b)
                let mut args = Vec::new();
                let func_name = self.flatten_app(term, &mut args);

                // Lower arguments
                let mut arg_regs = Vec::new();
                for arg in args.into_iter().rev() { // Reverse because flattening pushes innermost last
                    arg_regs.push(self.lower_term(arg, env));
                }

                let res = self.fresh_reg();
                let args_str = arg_regs.iter().map(|a| format!("i64 {}", a)).collect::<Vec<_>>().join(", ");
                self.instructions.push(format!("  {} = call i64 {}({})", res, func_name, args_str));
                res
            }
            _ => panic!("Unsupported term for MVP lowering: {:?}", term),
        }
    }

    fn flatten_app<'a>(&self, term: &'a Term<'a>, args: &mut Vec<&'a Term<'a>>) -> String {
        match term {
            Term::App(func, arg) => {
                args.push(arg);
                self.flatten_app(func, args)
            }
            Term::Var(name) => format!("@{}", name),
            _ => panic!("Expected function variable in application"),
        }
    }
}

