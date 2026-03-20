//! # Type Registry (Infrastructure/LLVM)
//!
//! This module tracks ADT constructor layouts and known global functions
//! to facilitate correct name resolution and memory offsets.

use std::collections::{HashMap, HashSet};

/// Represents the layout of a constructor in memory.
#[derive(Debug, Clone)]
pub struct ConstructorLayout {
    pub tag: u32,
    pub field_count: usize,
}

/// ISO 8601: 2026-03-19T20:55:29Z
/// Pattern: Registry Pattern (Workable iʃɛ́)
/// Why: Centralizing metadata about types and global symbols prevents 
/// duplication and provides a single source of truth for the backend.
pub struct TypeRegistry {
    /// Maps constructor names to their memory layout (tag and field count).
    pub type_env: HashMap<String, ConstructorLayout>,
    /// Tracks names of functions known to be defined globally.
    pub known_functions: HashSet<String>,
}

impl TypeRegistry {
    /// S-02: Side-effect free construction.
    pub fn new() -> Self {
        let mut type_env = HashMap::new();
        // Built-in List constructors
        type_env.insert("Nil".to_string(), ConstructorLayout { tag: 0, field_count: 0 });
        type_env.insert("::".to_string(), ConstructorLayout { tag: 1, field_count: 2 });
        // Built-in Nat constructors
        type_env.insert("Z".to_string(), ConstructorLayout { tag: 0, field_count: 0 });
        type_env.insert("S".to_string(), ConstructorLayout { tag: 1, field_count: 1 });

        Self {
            type_env,
            known_functions: HashSet::new(),
        }
    }

    /// Registers a new constructor layout.
    pub fn register_constructor(&mut self, name: String, layout: ConstructorLayout) {
        self.type_env.insert(name, layout);
    }

    /// Registers a global function name.
    pub fn register_function(&mut self, name: String) {
        self.known_functions.insert(name);
    }

    /// Resolves a high-level name to its LLVM global symbol (e.g., @name).
    /// Requirement: Universal Readability - semantic and readable by domain experts.
    pub fn resolve_global_name(&self, name: &str, function_definitions: &HashMap<String, String>) -> Option<String> {
        let sanitized = self.sanitize_id(name);
        let direct_match = self.known_functions.contains(name) || function_definitions.contains_key(&sanitized);
        if direct_match {
            return Some(format!("@{}", sanitized));
        }

        if let Some((_, short_name)) = name.rsplit_once('.') {
            let short_sanitized = self.sanitize_id(short_name);
            let short_match = self.known_functions.contains(short_name)
                || function_definitions.contains_key(&short_sanitized);
            if short_match {
                return Some(format!("@{}", short_sanitized));
            }
        }

        None
    }

    /// Sanitizes an Idris identifier for LLVM, escaping special characters
    /// and wrapping it in quotes to prevent collisions.
    pub fn sanitize_id(&self, id: &str) -> String {
        let mut sanitized = String::new();
        let mut input = id;
        
        // Handle holes
        if id.starts_with('?') {
            sanitized.push_str("_hole_");
            input = &id[1..];
        }

        for c in input.chars() {
            match c {
                '.' | '-' | ' ' | '(' | ')' | '[' | ']' | ',' | '?' => sanitized.push('_'),
                _ => sanitized.push(c),
            }
        }

        format!("\"{}\"", sanitized)
    }
}
