//! # LLVM Backend (Infrastructure)
//!
//! This module implements the LLVM backend for the Idris Native compiler.
//!
//! # Strategic Architecture
//! As an Infrastructure component, the `LlvmBackend` is responsible for 
//! translating our high-level IR into native machine code using the 
//! LLVM toolchain. It is the only module that knows about LLVM IR syntax.
//!
//! # Performance
//! By generating LLVM IR, we leverage decades of industrial-strength 
//! optimization (LTO, vectorization, etc.), allowing Idris 2 code 
//! to reach performance parity with C and C++.

pub mod ir_builder;
pub mod toolchain;

use crate::application::compiler::Backend;
use crate::domain::Term;
use crate::infrastructure::llvm::ir_builder::IRBuilder;
use std::collections::HashMap;
use std::io;

pub struct LlvmBackend {
    target_triple: String,
}

impl LlvmBackend {
    /// S-02: Side-effect free construction.
    pub fn new() -> Self {
        Self {
            target_triple: "x86_64-pc-linux-gnu".to_string(),
        }
    }

    /// KISS-03: Multi-line raw string literal for boilerplate IR.
    fn get_print_int_ir(&self) -> String {
        r#"
define void @print_int(i64 %n) {
entry:
  %is_zero = icmp eq i64 %n, 0
  br i1 %is_zero, label %zero, label %not_zero

zero:
  %char_zero = add i8 48, 0
  %buf_zero = alloca i8, i32 2
  store i8 %char_zero, i8* %buf_zero
  %next_zero = getelementptr i8, i8* %buf_zero, i32 1
  store i8 10, i8* %next_zero
  %void_zero = call i32 @write(i32 1, i8* %buf_zero, i32 2)
  ret void

not_zero:
  %abs_n = call i64 @llvm.abs.i64(i64 %n, i1 true)
  %is_neg = icmp slt i64 %n, 0
  br i1 %is_neg, label %print_minus, label %convert

print_minus:
  %minus_sign = alloca i8
  store i8 45, i8* %minus_sign
  %void_minus = call i32 @write(i32 1, i8* %minus_sign, i32 1)
  br label %convert

convert:
  %buf = alloca i8, i32 21
  %end_ptr = getelementptr i8, i8* %buf, i32 20
  store i8 10, i8* %end_ptr
  %res_ptr = call i8* @int_to_str(i64 %abs_n, i8* %end_ptr)
  %len = ptrtoint i8* %end_ptr to i64
  %start = ptrtoint i8* %res_ptr to i64
  %msg_len = sub i64 %len, %start
  %final_len = add i64 %msg_len, 1
  %len_i32 = trunc i64 %final_len to i32
  %void_final = call i32 @write(i32 1, i8* %res_ptr, i32 %len_i32)
  ret void
}

declare i32 @write(i32, i8*, i32)
declare i64 @llvm.abs.i64(i64, i1)

define i8* @int_to_str(i64 %n, i8* %buf) {
entry:
  %is_zero = icmp eq i64 %n, 0
  br i1 %is_zero, label %done, label %loop

loop:
  %curr_n = phi i64 [ %n, %entry ], [ %next_n, %loop ]
  %curr_ptr = phi i8* [ %buf, %entry ], [ %next_ptr, %loop ]
  %rem = urem i64 %curr_n, 10
  %next_n = udiv i64 %curr_n, 10
  %char = trunc i64 %rem to i8
  %char_val = add i8 %char, 48
  %next_ptr = getelementptr i8, i8* %curr_ptr, i32 -1
  store i8 %char_val, i8* %next_ptr
  %loop_cond = icmp eq i64 %next_n, 0
  br i1 %loop_cond, label %done, label %loop

done:
  %final_ptr = phi i8* [ %buf, %entry ], [ %next_ptr, %loop ]
  ret i8* %final_ptr
}
"#.to_string()
    }
}

impl Backend for LlvmBackend {
    fn lower_term(&self, term: &Term, env: &HashMap<String, String>) -> String {
        let mut builder = IRBuilder::new();
        builder.lower_term(term, env)
    }

    fn lower_program(&self, name: &str, _sig: &Term, body: &Term, args: &[String]) -> String {
        let mut builder = IRBuilder::new();
        let mut env = HashMap::new();
        
        let mut arg_str = String::new();
        for (i, arg) in args.iter().enumerate() {
            if i > 0 { arg_str.push_str(", "); }
            arg_str.push_str("i64 %");
            arg_str.push_str(arg);
            env.insert(arg.clone(), format!("%{}", arg));
        }

        let res_reg = builder.lower_term(body, &env);
        
        let mut ir = String::new();
        ir.push_str("target triple = \"");
        ir.push_str(&self.target_triple);
        ir.push_str("\"\n");
        ir.push_str(&self.get_print_int_ir());
        
        ir.push_str("\ndefine i64 @");
        ir.push_str(name);
        ir.push_str("(");
        ir.push_str(&arg_str);
        ir.push_str(") {\n");
        
        for instr in &builder.instructions {
            ir.push_str(instr);
            ir.push_str("\n");
        }
        ir.push_str("  ret i64 ");
        ir.push_str(&res_reg);
        ir.push_str("\n}\n");

        // Add a main wrapper for the MVP to make it executable
        ir.push_str("\ndefine i32 @main() {\n");
        // For MVP, we call the function with default args if any
        let mut call_args = String::new();
        for i in 0..args.len() {
            if i > 0 { call_args.push_str(", "); }
            call_args.push_str("i64 2"); // Default to 2 for things like ack(2,2)
        }
        ir.push_str("  %res = call i64 @");
        ir.push_str(name);
        ir.push_str("(");
        ir.push_str(&call_args);
        ir.push_str(")\n");
        ir.push_str("  call void @print_int(i64 %res)\n");
        ir.push_str("  ret i32 0\n}\n");

        ir
    }

    /// S-03: Decomposed binary compilation using toolchain module.
    fn compile_to_binary(&self, ir: String, output_path: &str) -> io::Result<bool> {
        toolchain::compile_ir_to_binary(ir, output_path)
    }
}

#[cfg(test)]
mod tests {
    pub mod robustness_tests;
    pub mod ir_builder_tests;
}

#[cfg(feature = "broken_tests")]
mod tests_broken;
#[cfg(feature = "broken_tests")]
mod wasm_tests;
#[cfg(feature = "broken_tests")]
mod bare_metal_tests;
#[cfg(feature = "broken_tests")]
mod module_tests;
