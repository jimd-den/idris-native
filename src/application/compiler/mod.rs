//! # Compiler Use Case (Brain of the Application)
//!
//! This module orchestrates the end-to-end compilation pipeline.

use crate::domain::Term;
use crate::domain::arena::Arena;
use crate::adapters::syntax_parser::{lex, Parser};
use crate::application::qtt_checker::QttChecker;
use crate::adapters::diagnostics;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

/// An abstraction for the code generation and toolchain backend.
pub trait Backend {
    fn lower_term(&self, term: &Term, env: &HashMap<String, String>) -> String;
    fn lower_program(&self, declarations: &[Term]) -> String;
    fn compile_to_binary(&self, ir: String, output_path: &str) -> std::io::Result<bool>;
}

/// The Compiler orchestrator.
pub struct Compiler<'a> {
    backend: &'a dyn Backend,
    qtt_enabled: bool,
}

impl<'a> Compiler<'a> {
    pub fn new(backend: &'a dyn Backend) -> Self {
        diagnostics::log("COMPILER", "INITIALIZE");
        Self { backend, qtt_enabled: true }
    }

    pub fn with_qtt(mut self, enabled: bool) -> Self {
        self.qtt_enabled = enabled;
        self
    }

    /// Executes the full compilation pipeline for an Idris 2 source file.
    pub fn compile_file(&self, filepath: &str) -> Result<String, String> {
        diagnostics::log("COMPILER", &format!("ENTER compile_file(filepath: {})", filepath));

        let output_path = format!("./{}_bin", filepath.replace(".idr", ""));
        let declarations = self.load_program_from_file(filepath)?;
        self.compile_declarations(&declarations, &output_path, filepath)
    }

    /// Executes the compilation pipeline for a string of Idris 2 source code.
    pub fn compile_str(&self, source: &str, output_path: &str, filename: &str) -> Result<String, String> {
        diagnostics::log("COMPILER", "ENTER compile_str");

        let declarations = self.load_program_from_source(source, filename)?;
        self.compile_declarations(&declarations, output_path, filename)
    }

    fn compile_declarations(&self, declarations: &[Term], output_path: &str, _filename: &str) -> Result<String, String> {
        // 2. QTT Validation
        if self.qtt_enabled {
            let checker = QttChecker::new();
            for decl in declarations {
                if !checker.check_term(decl) {
                    let err = "QTT Structural Error: Boundary violation detected.".to_string();
                    diagnostics::log("COMPILER", &format!("ERROR: {}", err));
                    return Err(err);
                }
            }
            diagnostics::log("COMPILER", "QTT validation successful.");
        }

        // 3. Lowering and Compilation
        let ir = self.backend.lower_program(declarations);

        diagnostics::log("COMPILER", "INVOKE backend.compile_to_binary");
        match self.backend.compile_to_binary(ir, output_path) {
            Ok(true) => {
                diagnostics::log("COMPILER", &format!("EXIT compile_str -> Success: {}", output_path));
                Ok(output_path.to_string())
            },
            _ => {
                let err = "Compilation failed in backend.".to_string();
                diagnostics::log("COMPILER", &format!("ERROR: {}", err));
                Err(err)
            }
        }
    }

    fn load_program_from_file(&self, filepath: &str) -> Result<Vec<Term<'static>>, String> {
        let path = Path::new(filepath);
        let source = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read file: {}", e))?;

        let mut visited = HashSet::new();
        self.load_program_recursive(&source, path, &mut visited)
    }

    fn load_program_from_source(&self, source: &str, filename: &str) -> Result<Vec<Term<'static>>, String> {
        let path = Path::new(filename);
        let mut visited = HashSet::new();
        self.load_program_recursive(source, path, &mut visited)
    }

    fn load_program_recursive(&self, source: &str, path: &Path, visited: &mut HashSet<PathBuf>) -> Result<Vec<Term<'static>>, String> {
        let canonical = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
        if !visited.insert(canonical) {
            return Ok(Vec::new());
        }

        let declarations = self.parse_source(source, path.to_string_lossy().as_ref())?;
        let mut resolved = Vec::new();

        for decl in &declarations {
            if let Term::Import(module_name) = decl {
                if let Some(import_path) = self.resolve_import_path(module_name, path.parent()) {
                    let import_source = fs::read_to_string(&import_path)
                        .map_err(|e| format!("Failed to read import {}: {}", import_path.display(), e))?;
                    let import_decls = self.load_program_recursive(&import_source, &import_path, visited)?;
                    resolved.extend(import_decls);
                }
            }
        }

        resolved.extend(declarations);
        Ok(resolved)
    }

    fn parse_source(&self, source: &str, filename: &str) -> Result<Vec<Term<'static>>, String> {
        let arena = Box::leak(Box::new(Arena::new()));
        let tokens = match lex(source) {
            Ok(t) => t,
            Err(e) => {
                diagnostics::report_error(&e, source, filename);
                return Err("Lexing failed".to_string());
            }
        };

        let mut parser = Parser::new(tokens, arena);
        match parser.parse_program() {
            Ok(decls) => Ok(decls),
            Err(e) => {
                diagnostics::report_error(&e, source, filename);
                Err("Parsing failed".to_string())
            }
        }
    }

    fn resolve_import_path(&self, module_name: &str, current_dir: Option<&Path>) -> Option<PathBuf> {
        let relative_module_path = PathBuf::from(module_name.replace('.', "/")).with_extension("idr");
        let module_file = PathBuf::from(format!("{}.idr", module_name));

        let mut roots = Vec::new();
        if let Some(dir) = current_dir {
            roots.push(dir.to_path_buf());
        }
        roots.push(PathBuf::from("."));
        roots.push(PathBuf::from("idris2_ref/libs/base"));
        roots.push(PathBuf::from("idris2_ref/libs/contrib"));
        roots.push(PathBuf::from("idris2_ref/libs/linear"));
        roots.push(PathBuf::from("idris2_ref/libs/network"));
        roots.push(PathBuf::from("idris2_ref/libs/prelude"));
        roots.push(PathBuf::from("idris2_ref/samples"));
        roots.push(PathBuf::from("idris2_ref/samples/FFI-readline/src"));

        for root in roots {
            let candidates = [root.join(&relative_module_path), root.join(&module_file)];
            for candidate in candidates {
                if candidate.exists() {
                    return Some(candidate);
                }
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    pub mod string_input_tests;
    pub mod buffer_lowering_tests;
    pub mod sha256_lowering_tests;
    pub mod string_concat_tests;
}
