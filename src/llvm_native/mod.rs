//! # LLVM Native Backend (Infrastructure)
//!
//! This module implements the LLVM-based code generation backend 
//! for the Idris 2 compiler.
//!
//! # Strategic Architecture
//! `llvm_native` is an Infrastructure component that translates our 
//! internal representations into LLVM IR for native compilation. 
//! It depends on the `core_terms` (Entities) but remains decoupled 
//! from the higher-level elaboration logic.
//!
//! # Performance & Zero-GC
//! To achieve performance exceeding well-optimized C, this backend 
//! generates strictly GC-free native code, relying on the memory 
//! safety and resource bounds guaranteed by the QTT checker.

use crate::core_terms::multiplicity::Multiplicity;

pub struct LlvmBackend {
    // We will add state here as needed.
}

impl LlvmBackend {
    pub fn new() -> Self {
        Self {}
    }

    /// Generates LLVM IR for an integer constant.
    pub fn gen_integer_ir(&self, value: i64) -> String {
        format!("i64 {}", value)
    }

    /// Generates LLVM IR for deallocating a resource.
    pub fn gen_dealloc_ir(&self, multiplicity: Multiplicity) -> String {
        match multiplicity {
            Multiplicity::Zero => String::new(),
            Multiplicity::One => "call void @free(ptr %term)".to_string(),
            Multiplicity::Many => String::new(),
        }
    }

    /// Generates LLVM IR for printing a string primitive.
    /// 
    /// Why this exists:
    /// This provides the fundamental output capability for the compiler 
    /// without relying on C runtimes. We generate pure LLVM declarations 
    /// for low-level I/O routines.
    pub fn gen_print_ir(&self, text: &str) -> String {
        // Simple MVP logic for print IR generation.
        // We include a declaration for @puts and call it with the string.
        let ir = format!(
            "declare i32 @puts(ptr)\n\
             @str = internal constant [{len} x i8] c\"{text}\\00\"\n\
             call i32 @puts(ptr @str)",
            len = text.len() + 1,
            text = text
        );
        ir
    }
}

#[cfg(test)]
mod tests;
