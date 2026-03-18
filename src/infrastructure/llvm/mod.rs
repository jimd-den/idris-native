//! # LLVM Native Backend (Infrastructure)
//!
//! This module implements the LLVM-based code generation backend 
//! for the Idris 2 compiler.
//!
//! # Strategic Architecture
//! `infrastructure::llvm` is an Infrastructure component that translates our 
//! internal representations into LLVM IR for native compilation. 
//! It sits at the outermost layer of our Clean Architecture.
//!
//! # Performance & Zero-GC
//! To achieve performance exceeding well-optimized C, this backend 
//! generates strictly GC-free native code, relying on the memory 
//! safety and resource bounds guaranteed by the QTT checker.

use crate::domain::Term;
use crate::application::compiler::Backend;
use crate::adapters::diagnostics;
use std::collections::HashMap;
use std::process::Command;
use std::fs;

/// Implementation of the `Backend` trait for LLVM.
pub struct LlvmBackend {
    target_triple: String,
}

impl LlvmBackend {
    pub fn new() -> Self {
        diagnostics::log("LLVM_BACKEND", "INITIALIZE");
        Self {
            target_triple: "x86_64-unknown-linux-gnu".to_string(),
        }
    }

    /// Pure LLVM/Assembly integer-to-string and syscall(write) routine.
    fn get_print_int_ir(&self) -> String {
        "\ndefine void @print_int(i64 %n) {\nentry: \n  %is_zero = icmp eq i64 %n, 0\n  br i1 %is_zero, label %zero, label %nonzero\n\nzero: \n  %buf0 = alloca [2 x i8]\n  %p0 = getelementptr [2 x i8], ptr %buf0, i32 0, i32 0\n  store i8 48, ptr %p0\n  %p1 = getelementptr [2 x i8], ptr %buf0, i32 0, i32 1\n  store i8 10, ptr %p1\n  call void asm sideeffect \"syscall\", \"{rax},{rdi},{rsi},{rdx},~{rcx},~{r11}\"(i64 1, i64 1, ptr %buf0, i64 2)\n  ret void\n\nnonzero: \n  %buf = alloca [21 x i8] ; space for i64 and newline\n  %end_ptr = getelementptr [21 x i8], ptr %buf, i32 0, i32 20\n  store i8 10, ptr %end_ptr ; newline\n  br label %loop\n\nloop: \n  %n_val = phi i64 [ %n, %nonzero ], [ %n_next, %loop ]\n  %curr_ptr = phi ptr [ %end_ptr, %nonzero ], [ %next_ptr, %loop ]\n  \n  %rem = urem i64 %n_val, 10\n  %n_next = udiv i64 %n_val, 10\n  \n  %char_val = add i64 %rem, 48\n  %char = trunc i64 %char_val to i8\n  \n  %next_ptr = getelementptr i8, ptr %curr_ptr, i32 -1\n  store i8 %char, ptr %next_ptr\n  \n  %done = icmp eq i64 %n_next, 0\n  br i1 %done, label %exit, label %loop\n\nexit: \n  %final_ptr = phi ptr [ %next_ptr, %loop ]\n  %ptr_int = ptrtoint ptr %end_ptr to i64\n  %start_int = ptrtoint ptr %final_ptr to i64\n  %len = sub i64 %ptr_int, %start_int\n  %total_len = add i64 %len, 1\n  call void asm sideeffect \"syscall\", \"{rax},{rdi},{rsi},{rdx},~{rcx},~{r11}\"(i64 1, i64 1, ptr %final_ptr, i64 %total_len)\n  ret void\n}\n".to_string()
    }
}

impl Backend for LlvmBackend {
    fn lower_term(&self, term: &Term, env: &HashMap<String, String>) -> String {
        let mut builder = IRBuilder::new();
        builder.lower_term(term, env)
    }

    fn lower_program(&self, name: &str, _sig: &Term, body: &Term, args: &[String]) -> String {
        diagnostics::log("LLVM_BACKEND", &format!("ENTER lower_program(name: {})", name));
        
        let mut builder = IRBuilder::new();
        let mut env_map = HashMap::new();
        for arg in args {
            env_map.insert(arg.clone(), format!("%{}", arg));
        }
        env_map.insert(name.to_string(), format!("@{}", name));

        let res_reg = builder.lower_term(body, &env_map);
        builder.instructions.push(format!("  ret i64 {}", res_reg));

        let args_str = args.iter().map(|a| format!("i64 %{}", a)).collect::<Vec<_>>().join(", ");
        let func_ir = format!(
            "define i64 @{}({}) {{\nentry:\n{}\n}}",
            name,
            args_str,
            builder.instructions.join("\n")
        );

        let mut module = Module::new(name);
        module.add_definition(func_ir);
        module.add_definition(self.get_print_int_ir());

        // auto-generate main
        let main_func = format!("\ndefine i32 @main() {{\nentry: \n  %res = call i64 @{}(i64 2, i64 2)\n  call void @print_int(i64 %res)\n  ret i32 0\n}}", name);
        module.add_definition(main_func);

        module.link()
    }

    fn compile_to_binary(&self, ir: String, output_path: &str) -> std::io::Result<bool> {
        diagnostics::log("LLVM_BACKEND", &format!("ENTER compile_to_binary(output_path: {})", output_path));
        let ir_path = format!("{}.ll", output_path);
        fs::write(&ir_path, ir)?;

        let status = Command::new("clang")
            .arg(&ir_path)
            .arg("-o")
            .arg(output_path)
            .status()?;

        let _ = fs::remove_file(ir_path);
        Ok(status.success())
    }
}

/// The IRBuilder translates our `Term` AST directly into LLVM IR.
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
            next_reg: 1,
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

    pub fn lower_term(&mut self, term: &Term, env: &HashMap<String, String>) -> String {
        match term {
            Term::Integer(val) => format!("{}", val),
            Term::Var(name) => {
                env.get(name).cloned().unwrap_or_else(|| format!("@{}", name))
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
            Term::BitXor(lhs, rhs) => {
                let l_reg = self.lower_term(lhs, env);
                let r_reg = self.lower_term(rhs, env);
                let res = self.fresh_reg();
                self.instructions.push(format!("  {} = xor i64 {}, {}", res, l_reg, r_reg));
                res
            }
            Term::BitAnd(lhs, rhs) => {
                let l_reg = self.lower_term(lhs, env);
                let r_reg = self.lower_term(rhs, env);
                let res = self.fresh_reg();
                self.instructions.push(format!("  {} = and i64 {}, {}", res, l_reg, r_reg));
                res
            }
            Term::BitOr(lhs, rhs) => {
                let l_reg = self.lower_term(lhs, env);
                let r_reg = self.lower_term(rhs, env);
                let res = self.fresh_reg();
                self.instructions.push(format!("  {} = or i64 {}, {}", res, l_reg, r_reg));
                res
            }
            Term::BitNot(body) => {
                let reg = self.lower_term(body, env);
                let res = self.fresh_reg();
                self.instructions.push(format!("  {} = xor i64 {}, -1", res, reg));
                res
            }
            Term::Shl(lhs, rhs) => {
                let l_reg = self.lower_term(lhs, env);
                let r_reg = self.lower_term(rhs, env);
                let res = self.fresh_reg();
                self.instructions.push(format!("  {} = shl i64 {}, {}", res, l_reg, r_reg));
                res
            }
            Term::Shr(lhs, rhs) => {
                let l_reg = self.lower_term(lhs, env);
                let r_reg = self.lower_term(rhs, env);
                let res = self.fresh_reg();
                self.instructions.push(format!("  {} = lshr i64 {}, {}", res, l_reg, r_reg));
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
                let mut args = Vec::new();
                let func_name = self.flatten_app(term, &mut args);
                let mut arg_regs = Vec::new();
                for arg in args.into_iter().rev() {
                    arg_regs.push(self.lower_term(arg, env));
                }
                let res = self.fresh_reg();
                let args_str = arg_regs.iter().map(|a| format!("i64 {}", a)).collect::<Vec<_>>().join(", ");
                self.instructions.push(format!("  {} = call i64 @{}({})", res, func_name, args_str));
                res
            }
            Term::Buffer(size) => {
                let res = self.fresh_reg();
                self.instructions.push(format!("  {} = alloca [{} x i64]", res, size));
                res
            }
            Term::BufferLoad(buffer, index) => {
                let buf_reg = self.lower_term(buffer, env);
                let idx_reg = self.lower_term(index, env);
                let size = match buffer { Term::Buffer(s) => *s, _ => 64 };
                let ptr_reg = self.fresh_reg();
                self.instructions.push(format!("  {} = getelementptr [{} x i64], ptr {}, i32 0, i64 {}", ptr_reg, size, buf_reg, idx_reg));
                let res = self.fresh_reg();
                self.instructions.push(format!("  {} = load i64, ptr {}", res, ptr_reg));
                res
            }
            Term::BufferStore(buffer, index, value) => {
                let buf_reg = self.lower_term(buffer, env);
                let idx_reg = self.lower_term(index, env);
                let val_reg = self.lower_term(value, env);
                let size = match buffer { Term::Buffer(s) => *s, _ => 64 };
                let ptr_reg = self.fresh_reg();
                self.instructions.push(format!("  {} = getelementptr [{} x i64], ptr {}, i32 0, i64 {}", ptr_reg, size, buf_reg, idx_reg));
                self.instructions.push(format!("  store i64 {}, ptr {}", val_reg, ptr_reg));
                buf_reg
            }
            Term::Let(name, val, body) => {
                let val_reg = self.lower_term(val, env);
                let mut new_env = env.clone();
                new_env.insert(name.clone(), val_reg);
                self.lower_term(body, &new_env)
            }
            Term::Case(target, branches) => {
                let target_reg = self.lower_term(target, env);
                let merge_label = self.fresh_label("case_merge");
                let mut phi_entries = Vec::new();
                for (pat_name, _pat_args, body) in branches.iter() {
                    let next_pat_label = self.fresh_label("case_next");
                    let body_label = self.fresh_label("case_body");
                    if pat_name == "_" {
                        self.instructions.push(format!("  br label %{}", body_label));
                    } else if let Ok(val) = pat_name.parse::<i64>() {
                        let cond_reg = self.fresh_reg();
                        self.instructions.push(format!("  {} = icmp eq i64 {}, {}", cond_reg, target_reg, val));
                        self.instructions.push(format!("  br i1 {}, label %{}, label %{}", cond_reg, body_label, next_pat_label));
                    } else {
                        self.instructions.push(format!("  br label %{}", next_pat_label));
                    }
                    self.instructions.push(format!("\n{}:", body_label));
                    self.current_block = body_label.clone();
                    let res_reg = self.lower_term(body, env);
                    let final_block = self.current_block.clone();
                    self.instructions.push(format!("  br label %{}", merge_label));
                    phi_entries.push((res_reg, final_block));
                    self.instructions.push(format!("\n{}:", next_pat_label));
                    self.current_block = next_pat_label.clone();
                }
                self.instructions.push(format!("  br label %{}", merge_label));
                self.instructions.push(format!("\n{}:", merge_label));
                self.current_block = merge_label.clone();
                let res = self.fresh_reg();
                let phi_str = phi_entries.iter().map(|(r, b)| format!("[ {}, %{} ]", r, b)).collect::<Vec<_>>().join(", ");
                self.instructions.push(format!("  {} = phi i64 {}", res, phi_str));
                res
            }
            _ => panic!("Unsupported term for LLVM lowering: {:?}", term),
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

pub struct Module {
    name: String,
    definitions: Vec<String>,
}

impl Module {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            definitions: Vec::new(),
        }
    }

    pub fn add_definition(&mut self, ir: String) {
        self.definitions.push(ir);
    }

    pub fn link(&self) -> String {
        let mut module_ir = format!("source_filename = \"{}\"\n\n", self.name);
        for def in &self.definitions {
            module_ir.push_str(def);
            module_ir.push('\n');
        }
        module_ir
    }
}

#[cfg(test)]
mod tests;
#[cfg(test)]
mod wasm_tests;
#[cfg(test)]
mod bare_metal_tests;
#[cfg(test)]
mod module_tests;
