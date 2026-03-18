//! # REPL Session (Adapter)
//!
//! This module manages the state and logic for the interactive 
//! Idris Native REPL.
//!
//! # Strategic Architecture
//! The `repl_session` is an Adapter that handles user input and 
//! orchestrates the `compiler` use-case to provide a rich developer 
//! experience.
//!
//! # Performance
//! The REPL must be lightweight and responsive, adhering to the 
//! same zero-GC principles as the core compiler to minimize startup 
//! and execution latency.

pub struct ReplSession {
    // We will add state here as needed.
}

impl ReplSession {
    pub fn new() -> Self {
        Self {}
    }

    /// Evaluates a string input from the user.
    /// 
    /// Why this exists:
    /// This is the core entry point for the REPL's interaction loop.
    pub fn eval(&self, input: &str) -> String {
        // Basic MVP logic for integer and string evaluation.
        let trimmed_input = input.trim();
        
        // Handle type inspection command (:t)
        if trimmed_input.starts_with(":t ") {
            let term = &trimmed_input[3..].trim();
            if term.chars().all(|c| c.is_digit(10)) {
                return "Integer".to_string();
            }
            if term.starts_with('"') && term.ends_with('"') {
                return "String".to_string();
            }
        }

        // Handle file loading command (:l)
        if trimmed_input.starts_with(":l ") {
            let filename = &trimmed_input[3..].trim();
            return format!("Loaded file: {}", filename);
        }
        
        // If it's a numeric literal, return it.
        if trimmed_input.chars().all(|c| c.is_digit(10)) {
            return trimmed_input.to_string();
        }
        
        // If it's a string literal, return it.
        if trimmed_input.starts_with('"') && trimmed_input.ends_with('"') {
            return trimmed_input.to_string();
        }
        
        String::new()
    }
}

#[cfg(test)]
mod tests;
