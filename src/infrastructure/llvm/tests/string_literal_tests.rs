#[cfg(test)]
mod tests {
    use crate::infrastructure::llvm::ir_builder::IRBuilder;
    use crate::domain::Term;
    use std::collections::HashMap;

    #[test]
    fn test_unique_string_literals() {
        let mut builder = IRBuilder::new();
        let env = HashMap::new();
        
        builder.lower_term(&Term::String("foo".to_string()), &env);
        builder.lower_term(&Term::String("bar".to_string()), &env);
        
        assert_eq!(builder.string_literals.len(), 2);
        assert!(builder.string_literals.contains_key("foo"));
        assert!(builder.string_literals.contains_key("bar"));
    }

    #[test]
    fn test_duplicate_string_literals() {
        let mut builder = IRBuilder::new();
        let env = HashMap::new();
        
        builder.lower_term(&Term::String("foo".to_string()), &env);
        builder.lower_term(&Term::String("foo".to_string()), &env);
        
        assert_eq!(builder.string_literals.len(), 1);
        assert!(builder.string_literals.contains_key("foo"));
    }

    #[test]
    fn test_string_literal_escaping() {
        let mut builder = IRBuilder::new();
        let env = HashMap::new();
        
        // Test with characters that need escaping in LLVM IR
        let s = "hello\nworld\"".to_string();
        builder.lower_term(&Term::String(s.clone()), &env);
        
        let label = builder.string_literals.get(&s).expect("String should be registered");
        let escaped = builder.escape_string(&s);
        assert_eq!(escaped, "hello\\0Aworld\\22");
    }

    #[test]
    fn test_string_literal_labels() {
        let mut builder = IRBuilder::new();
        let env = HashMap::new();
        
        builder.lower_term(&Term::String("foo".to_string()), &env);
        builder.lower_term(&Term::String("bar".to_string()), &env);
        
        let l1 = builder.string_literals.get("foo").unwrap();
        let l2 = builder.string_literals.get("bar").unwrap();
        assert_ne!(l1, l2);
    }
}
