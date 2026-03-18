//! # Compiler Use Case (Brain of the Application)
//!
//! This module orchestrates the end-to-end compilation pipeline.
//!
//! # Strategic Architecture
//! In accordance with Clean Architecture, the `Compiler` is a Use Case 
//! that depends only on the Entities (`Term`) and an abstract `Backend` 
//! interface. Implementation details (LLVM, File I/O) are injected.
//!
//! # Screaming Architecture
//! This module "screams" its intent: parsing Idris source, validating 
//! QTT multiplicity, and generating binary output.

use crate::domain::Term;
use crate::domain::arena::Arena;
use crate::adapters::syntax_parser::{lex, Parser};
use crate::application::qtt_checker::QttChecker;
use crate::adapters::diagnostics;
use std::collections::HashMap;
use std::fs;

/// An abstraction for the code generation and toolchain backend.
/// 
/// Why this exists:
/// Dependency Inversion. The Use Case layer must not depend on 
/// low-level LLVM details. This trait allows us to swap LLVM 
/// for other backends (WASM, C, etc.) without changing core logic.
pub trait Backend {
    /// Lowers a high-level `Term` to a string of IR (LLVM IR, etc.).
    fn lower_term(&self, term: &Term, env: &HashMap<String, String>) -> String;
    
    /// Lowers an entire program (signature + body) to a full IR module.
    fn lower_program(&self, name: &str, sig: &Term, body: &Term, args: &[String]) -> String;

    /// Compiles the generated IR into a native binary at the specified path.
    fn compile_to_binary(&self, ir: String, output_path: &str) -> std::io::Result<bool>;
}

/// The Compiler orchestrator.
pub struct Compiler<'a> {
    backend: &'a dyn Backend,
}

impl<'a> Compiler<'a> {
    pub fn new(backend: &'a dyn Backend) -> Self {
        diagnostics::log("COMPILER", "INITIALIZE");
        Self { backend }
    }

    /// Executes the full compilation pipeline for an Idris 2 source file.
    /// 
    /// # Logic Flow
    /// 1. Lex/Parse -> Term AST
    /// 2. QttChecker -> Validate Multiplicity/Bounds
    /// 3. Backend -> Lower to IR
    /// 4. Backend -> Compile to Binary
    pub fn compile_file(&self, filepath: &str) -> Result<String, String> {
        diagnostics::log("COMPILER", &format!("ENTER compile_file(filepath: {})", filepath));

        // 1. Read and Parse
        let source = fs::read_to_string(filepath)
            .map_err(|e| format!("Failed to read file: {}", e))?;
        
        let mut arena = Arena::new();
        let tokens = lex(&source);
        let mut parser = Parser::new(tokens, &mut arena);
        let (name, _sig, body, args) = parser.parse_program();
        diagnostics::log("COMPILER", &format!("PARSED definition: {}", name));

        // 2. QTT Validation
        let checker = QttChecker::new();
        for arg in &args {
            if !checker.check_multiplicity(arg, 1, body) {
                let err = format!("QTT Multiplicity Error: Linear variable '{}' used incorrectly.", arg);
                diagnostics::log("COMPILER", &format!("ERROR: {}", err));
                return Err(err);
            }
        }
        if !checker.check_term(body) {
            let err = "QTT Structural Error: Boundary violation detected.".to_string();
            diagnostics::log("COMPILER", &format!("ERROR: {}", err));
            return Err(err);
        }
        diagnostics::log("COMPILER", "QTT validation successful.");

        // 3. Lowering and Compilation
        // For MVP, we pass the signature and body to the backend.
        let bin_path = format!("./{}_bin", filepath.replace(".idr", ""));
        let ir = self.backend.lower_program(&name, _sig, body, &args);
        
        diagnostics::log("COMPILER", "INVOKE backend.compile_to_binary");
        match self.backend.compile_to_binary(ir, &bin_path) {
            Ok(true) => {
                diagnostics::log("COMPILER", &format!("EXIT compile_file -> Success: {}", bin_path));
                Ok(bin_path)
            },
            _ => {
                let err = "Compilation failed in backend.".to_string();
                diagnostics::log("COMPILER", &format!("ERROR: {}", err));
                Err(err)
            }
        }
    }
}
