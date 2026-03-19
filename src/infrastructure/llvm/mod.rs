//! # LLVM Backend (Infrastructure)
//!
//! This module implements the LLVM backend for the Idris Native compiler.

pub mod ir_builder;
pub mod toolchain;

pub use ir_builder::IRBuilder;
use crate::application::compiler::Backend;
use crate::domain::Term;
use crate::domain::multiplicity::Multiplicity;
use std::collections::HashMap;
use std::io;
use std::fs;

pub struct LlvmBackend {
    target_triple: String,
    opt_level: u32,
}

impl LlvmBackend {
    /// S-02: Side-effect free construction.
    pub fn new() -> Self {
        Self {
            target_triple: "x86_64-pc-linux-gnu".to_string(),
            opt_level: 0,
        }
    }

    /// Sets the optimization level for the backend.
    pub fn set_opt_level(&mut self, level: u32) {
        self.opt_level = level;
    }

    /// Returns the current optimization level.
    pub fn get_opt_level(&self) -> u32 {
        self.opt_level
    }

    /// Sets the target triple for the backend.
    pub fn set_target(&mut self, target: &str) {
        self.target_triple = target.to_string();
    }

    /// Returns the current target triple.
    pub fn get_target(&self) -> String {
        self.target_triple.clone()
    }

    /// Generates IR for an integer literal.
    pub fn gen_integer_ir(&self, val: i64) -> String {
        format!("i64 {}", val)
    }

    /// Generates IR for deallocating a resource based on its multiplicity.
    pub fn gen_dealloc_ir(&self, mult: Multiplicity) -> String {
        match mult {
            Multiplicity::One => "  call void @free(i8* %ptr)\n".to_string(),
            _ => String::new(),
        }
    }

    /// Generates IR for a print statement based on target triple.
    pub fn gen_print_ir(&self, msg: &str) -> (String, String) {
        if self.target_triple.contains("wasm32") {
            let decl = "declare void @__wasm_print(i8*)".to_string();
            let body = format!("  call void @__wasm_print(i8* getelementptr inbounds ([{} x i8], [{} x i8]* @.str, i64 0, i64 0))", msg.len() + 1, msg.len() + 1);
            (decl, body)
        } else if self.target_triple.contains("arm") || self.target_triple.contains("aarch64") {
            let decl = "declare void @__bare_metal_print(i8*)".to_string();
            let body = format!("  call void @__bare_metal_print(i8* getelementptr inbounds ([{} x i8], [{} x i8]* @.str, i64 0, i64 0))", msg.len() + 1, msg.len() + 1);
            (decl, body)
        } else {
            let decl = "declare i32 @puts(i8*)".to_string();
            let body = format!("  call i32 @puts(i8* getelementptr inbounds ([{} x i8], [{} x i8]* @.str, i64 0, i64 0))", msg.len() + 1, msg.len() + 1);
            (decl, body)
        }
    }

    /// Emits the module IR to a file.
    pub fn emit_to_file(&self, ir: &str, path: &str) -> io::Result<()> {
        fs::write(path, ir)
    }

    /// KISS-03: Multi-line raw string literal for boilerplate IR.
    fn get_runtime_ir(&self) -> String {
        r#"
declare i32 @puts(i8*)
declare i8* @malloc(i64)
declare void @free(i8*)
declare i64 @strlen(i8*)
declare i8* @memcpy(i8*, i8*, i64)

@str_buffer = global [1024 x i8] zeroinitializer

define i64 @getLine() {
  %ptr = getelementptr [1024 x i8], [1024 x i8]* @str_buffer, i64 0, i64 0
  ; Simplified: return the global buffer pointer
  %res = ptrtoint i8* %ptr to i64
  ret i64 %res
}

define void @putStr(i64 %s_int) {
  %s = inttoptr i64 %s_int to i8*
  %void = call i32 @puts(i8* %s)
  ret void
}

define void @putStrLn(i64 %s_int) {
  %s = inttoptr i64 %s_int to i8*
  %void = call i32 @puts(i8* %s)
  ret void
}

define i64 @concat(i64 %s1_int, i64 %s2_int) {
entry:
  %s1 = inttoptr i64 %s1_int to i8*
  %s2 = inttoptr i64 %s2_int to i8*
  %len1 = call i64 @strlen(i8* %s1)
  %len2 = call i64 @strlen(i8* %s2)
  %total_len = add i64 %len1, %len2
  %alloc_len = add i64 %total_len, 1
  %new_str = call i8* @malloc(i64 %alloc_len)
  
  ; Copy first string
  %void1 = call i8* @memcpy(i8* %new_str, i8* %s1, i64 %len1)
  
  ; Copy second string
  %dest2 = getelementptr i8, i8* %new_str, i64 %len1
  %void2 = call i8* @memcpy(i8* %dest2, i8* %s2, i64 %len2)
  
  ; Null terminator
  %term_ptr = getelementptr i8, i8* %new_str, i64 %total_len
  store i8 0, i8* %term_ptr
  
  %res = ptrtoint i8* %new_str to i64
  ret i64 %res
}

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

    fn lower_program(&self, declarations: &[Term]) -> String {
        let mut builder = IRBuilder::new();
        let env = HashMap::new();
        
        // Pre-pass: register all ADTs (including nested ones)
        fn register_adts<'a>(decls: &[Term<'a>], builder: &mut IRBuilder) {
            for decl in decls {
                match decl {
                    Term::Data(_, _, _) => { builder.lower_term(decl, &HashMap::new()); }
                    Term::Where(_, local_decls) => { register_adts(local_decls, builder); }
                    Term::Def(_, _, body) => { register_adts_in_term(body, builder); }
                    _ => {}
                }
            }
        }

        fn register_adts_in_term<'a>(term: &Term<'a>, builder: &mut IRBuilder) {
            match term {
                Term::Where(body, local_decls) => {
                    register_adts(local_decls, builder);
                    register_adts_in_term(body, builder);
                }
                Term::Let(_, val, body) => {
                    register_adts_in_term(val, builder);
                    register_adts_in_term(body, builder);
                }
                Term::LetRec(_, val, body) => {
                    register_adts_in_term(val, builder);
                    register_adts_in_term(body, builder);
                }
                Term::Case(target, branches) => {
                    register_adts_in_term(target, builder);
                    for (_, _, body) in branches {
                        register_adts_in_term(body, builder);
                    }
                }
                Term::App(f, a) => {
                    register_adts_in_term(f, builder);
                    register_adts_in_term(a, builder);
                }
                Term::Lambda(_, _, body) => {
                    register_adts_in_term(body, builder);
                }
                Term::Do(stmts) => {
                    for stmt in stmts {
                        register_adts_in_term(stmt, builder);
                    }
                }
                _ => {}
            }
        }

        register_adts(declarations, &mut builder);

        let mut main_name = "main".to_string();
        let mut main_args_count = 0;
        let mut main_defined = false;

        for decl in declarations {
            if let Term::Def(name, args, _) = decl {
                builder.lower_term(decl, &env);
                if name == "main" {
                    main_name = name.clone();
                    main_args_count = args.len();
                    main_defined = true;
                } else if !main_defined && main_args_count == 0 {
                    main_name = name.clone();
                    main_args_count = args.len();
                }
            }
        }
        
        let mut ir = String::new();
        ir.push_str("target triple = \"");
        ir.push_str(&self.target_triple);
        ir.push_str("\"\n");
        
        // Define string literals
        for (value, label) in &builder.string_literals {
            let escaped = builder.escape_string(value);
            ir.push_str(&format!("@{} = private unnamed_addr constant [{} x i8] c\"{}\\00\", align 1\n", 
                label, value.len() + 1, escaped));
        }

        ir.push_str(&self.get_runtime_ir());

        for def in builder.function_definitions.values() {
            ir.push_str(def);
            ir.push_str("\n");
        }


        // Add a main wrapper for the MVP to make it executable if main is not defined
        if !main_defined {
            ir.push_str("\ndefine i32 @main() {\n");
            let mut call_args = String::new();
            for _ in 0..main_args_count {
                if !call_args.is_empty() { call_args.push_str(", "); }
                call_args.push_str("i64 2");
            }
            ir.push_str("  %res = call i64 @");
            ir.push_str(&builder.sanitize_id(&main_name));
            ir.push_str("(");
            ir.push_str(&call_args);
            ir.push_str(")\n");
            ir.push_str("  call void @print_int(i64 %res)\n");
            ir.push_str("  ret i32 0\n}\n");
        }

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
    pub mod tests_restored;
    pub mod all_variants_tests;
    pub mod dynamic_main_tests;
    pub mod wasm_tests;
    pub mod bare_metal_tests;
    pub mod sanitizer_tests;
    pub mod string_literal_tests;
    pub mod adt_tests;
    pub mod pattern_matching_tests;
}

#[cfg(feature = "broken_tests")]
mod module_tests;
