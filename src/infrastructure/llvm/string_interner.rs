//! # String Interner (Infrastructure/LLVM)
//!
//! This module manages string literal interning and unique label generation
//! for LLVM IR string constants.

use std::collections::HashMap;

/// ISO 8601: 2026-03-19T20:55:29Z
/// Pattern: String Interner Pattern (Workable iʃɛ́)
/// Why: Decoupling string literal management from the core IR emission logic
/// ensures that labels are consistent and memory is used efficiently.
pub struct StringInterner {
    /// Maps raw string values to their unique LLVM labels.
    pub string_literals: HashMap<String, String>,
}

impl StringInterner {
    /// S-02: Side-effect free construction.
    pub fn new() -> Self {
        Self {
            string_literals: HashMap::new(),
        }
    }

    /// Interns a string and returns its unique LLVM label.
    pub fn intern(&mut self, s: &str) -> String {
        if let Some(label) = self.string_literals.get(s) {
            label.clone()
        } else {
            let label = self.new_string_label();
            self.string_literals.insert(s.to_string(), label.clone());
            label
        }
    }

    /// Generates a unique placeholder label for a new string literal.
    fn new_string_label(&mut self) -> String {
        format!("str_{}", self.string_literals.len())
    }

    /// Escapes a string for LLVM IR string literals (hexadecimal escaping).
    /// Requirement: Universal Readability - semantic and readable by domain experts.
    pub fn escape_string(&self, s: &str) -> String {
        let mut escaped = String::new();
        for b in s.as_bytes() {
            match b {
                b'\"' => escaped.push_str("\\22"),
                b'\\' => escaped.push_str("\\5C"),
                b if *b < 32 || *b > 126 => {
                    escaped.push_str(&format!("\\{:02X}", b));
                }
                _ => escaped.push(*b as char),
            }
        }
        escaped
    }
}
