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
    bit_width: u32,
}

impl IRBuilder {
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
            next_reg: 1, // Start at 1 to avoid colliding with named arguments
            label_counter: 0,
            current_block: "entry".to_string(),
            bit_width: 64, // Default to i64
        }
    }

    pub fn set_bit_width(&mut self, width: u32) {
        self.bit_width = width;
    }

    fn get_type_str(&self) -> String {
        format!("i{}", self.bit_width)
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
        let ty = self.get_type_str();
        match term {
            Term::Integer(val) => format!("{}", val),
            Term::Var(name) => {
                env.get(name).cloned().unwrap_or_else(|| format!("@{}", name)) // Assume global if not in local env
            }
            Term::Add(lhs, rhs) => {
                let l_reg = self.lower_term(lhs, env);
                let r_reg = self.lower_term(rhs, env);
                let res = self.fresh_reg();
                self.instructions.push(format!("  {} = add {} {}, {}", res, ty, l_reg, r_reg));
                res
            }
            Term::Sub(lhs, rhs) => {
                let l_reg = self.lower_term(lhs, env);
                let r_reg = self.lower_term(rhs, env);
                let res = self.fresh_reg();
                self.instructions.push(format!("  {} = sub {} {}, {}", res, ty, l_reg, r_reg));
                res
            }
            Term::Eq(lhs, rhs) => {
                let l_reg = self.lower_term(lhs, env);
                let r_reg = self.lower_term(rhs, env);
                let res = self.fresh_reg();
                self.instructions.push(format!("  {} = icmp eq {} {}, {}", res, ty, l_reg, r_reg));
                res
            }
            Term::BitXor(lhs, rhs) => {
                let l_reg = self.lower_term(lhs, env);
                let r_reg = self.lower_term(rhs, env);
                let res = self.fresh_reg();
                self.instructions.push(format!("  {} = xor {} {}, {}", res, ty, l_reg, r_reg));
                res
            }
            Term::BitAnd(lhs, rhs) => {
                let l_reg = self.lower_term(lhs, env);
                let r_reg = self.lower_term(rhs, env);
                let res = self.fresh_reg();
                self.instructions.push(format!("  {} = and {} {}, {}", res, ty, l_reg, r_reg));
                res
            }
            Term::BitOr(lhs, rhs) => {
                let l_reg = self.lower_term(lhs, env);
                let r_reg = self.lower_term(rhs, env);
                let res = self.fresh_reg();
                self.instructions.push(format!("  {} = or {} {}, {}", res, ty, l_reg, r_reg));
                res
            }
            Term::BitNot(body) => {
                let reg = self.lower_term(body, env);
                let res = self.fresh_reg();
                // LLVM doesn't have 'not', use 'xor -1'
                self.instructions.push(format!("  {} = xor {} {}, -1", res, ty, reg));
                res
            }
            Term::Shl(lhs, rhs) => {
                let l_reg = self.lower_term(lhs, env);
                let r_reg = self.lower_term(rhs, env);
                let res = self.fresh_reg();
                self.instructions.push(format!("  {} = shl {} {}, {}", res, ty, l_reg, r_reg));
                res
            }
            Term::Shr(lhs, rhs) => {
                let l_reg = self.lower_term(lhs, env);
                let r_reg = self.lower_term(rhs, env);
                let res = self.fresh_reg();
                self.instructions.push(format!("  {} = lshr {} {}, {}", res, ty, l_reg, r_reg));
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
                self.instructions.push(format!("  {} = phi {} [ {}, %{} ], [ {}, %{} ]", res, ty, then_reg, final_then_block, else_reg, final_else_block));
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
                let args_str = arg_regs.iter().map(|a| format!("{} {}", ty, a)).collect::<Vec<_>>().join(", ");
                self.instructions.push(format!("  {} = call {} {}( {} )", res, ty, func_name, args_str));
                res
            }
            Term::Buffer(size) => {
                let res = self.fresh_reg();
                // For MVP, buffers are always arrays of the current bit_width
                self.instructions.push(format!("  {} = alloca [{} x {}]", res, size, ty));
                res
            }
            Term::BufferLoad(buffer, index) => {
                let buf_reg = self.lower_term(buffer, env);
                let idx_reg = self.lower_term(index, env);
                
                // We need the size for the GEP instruction. 
                // For MVP, we'll try to find it in the term if it's a direct Buffer(size)
                // Otherwise we might need a more robust type system.
                let size = match buffer {
                    Term::Buffer(s) => *s,
                    _ => 64, // Fallback for now
                };
                
                let ptr_reg = self.fresh_reg();
                self.instructions.push(format!("  {} = getelementptr [{} x {}], ptr {}, i32 0, i64 {}", ptr_reg, size, ty, buf_reg, idx_reg));
                
                let res = self.fresh_reg();
                self.instructions.push(format!("  {} = load {}, ptr {}", res, ty, ptr_reg));
                res
            }
            Term::BufferStore(buffer, index, value) => {
                let buf_reg = self.lower_term(buffer, env);
                let idx_reg = self.lower_term(index, env);
                let val_reg = self.lower_term(value, env);
                
                let size = match buffer {
                    Term::Buffer(s) => *s,
                    _ => 64,
                };
                
                let ptr_reg = self.fresh_reg();
                self.instructions.push(format!("  {} = getelementptr [{} x {}], ptr {}, i32 0, i64 {}", ptr_reg, size, ty, buf_reg, idx_reg));
                
                self.instructions.push(format!("  store {} {}, ptr {}", ty, val_reg, ptr_reg));
                buf_reg // Return the buffer register (standard for some functional store patterns)
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

#[cfg(test)]
mod tests {
    pub mod sha256_lowering_tests;
    pub mod buffer_lowering_tests;
}
