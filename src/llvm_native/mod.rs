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
    target_triple: String,
    opt_level: u32,
}

impl LlvmBackend {
    pub fn new() -> Self {
        Self {
            target_triple: "x86_64-unknown-linux-gnu".to_string(), // Default
            opt_level: 0, // Default to no optimization
        }
    }

    /// Sets the LLVM optimization level (0-3).
    pub fn set_opt_level(&mut self, level: u32) {
        self.opt_level = level;
    }

    /// Gets the current optimization level.
    pub fn get_opt_level(&self) -> u32 {
        self.opt_level
    }

    /// Sets the target triple for code generation.
    /// 
    /// Why this exists:
    /// Flexibility is a core requirement. By allowing the target triple 
    /// to be set, we can generate code for diverse architectures like 
    /// WASM or bare-metal ARM.
    pub fn set_target(&mut self, triple: &str) {
        self.target_triple = triple.to_string();
    }

    /// Gets the current target triple.
    pub fn get_target(&self) -> &str {
        &self.target_triple
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
    pub fn gen_print_ir(&self, text: &str) -> String {
        if self.target_triple == "wasm32-unknown-unknown" {
            // WASM uses a different I/O routine (imported from the host).
            format!(
                "declare void @__wasm_print(ptr, i32)\n\
                 @str = internal constant [{len} x i8] c\"{text}\"\n\
                 call void @__wasm_print(ptr @str, i32 {len})",
                len = text.len(),
                text = text
            )
        } else if self.target_triple == "arm-none-eabi" {
            // Bare-metal uses a low-level routine (implemented by user).
            format!(
                "declare void @__bare_metal_print(ptr, i32)\n\
                 @str = internal constant [{len} x i8] c\"{text}\"\n\
                 call void @__bare_metal_print(ptr @str, i32 {len})",
                len = text.len(),
                text = text
            )
        } else {
            // Native default uses 'puts' from standard C lib.
            format!(
                "declare i32 @puts(ptr)\n\
                 @str = internal constant [{len} x i8] c\"{text}\\00\"\n\
                 call i32 @puts(ptr @str)",
                len = text.len() + 1,
                text = text
            )
        }
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod wasm_tests;

#[cfg(test)]
mod bare_metal_tests;
