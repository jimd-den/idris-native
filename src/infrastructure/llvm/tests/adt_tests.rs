#[cfg(test)]
mod tests {
    use crate::infrastructure::llvm::ir_builder::IRBuilder;
    use crate::domain::Term;
    use crate::domain::term::Constructor;
    use std::collections::HashMap;

    #[test]
    fn test_adt_registry_population() {
        let mut builder = IRBuilder::new();
        let env = HashMap::new();
        
        // data Maybe a = Nothing | Just a
        let nothing = Constructor { name: "Nothing".to_string(), fields: vec![] };
        let just = Constructor { name: "Just".to_string(), fields: vec![&Term::IntegerType] };
        let maybe = Term::Data("Maybe".to_string(), vec!["a".to_string()], vec![nothing, just]);
        
        builder.lower_term(&maybe, &env);
        
        // Verify that the type environment is populated
        // We'll need to expose some way to check this, or test via side effects
        // For now, let's assume IRBuilder will have a type_env field
        assert!(builder.type_env.contains_key("Nothing"));
        assert!(builder.type_env.contains_key("Just"));
        
        let nothing_layout = builder.type_env.get("Nothing").unwrap();
        assert_eq!(nothing_layout.tag, 0);
        assert_eq!(nothing_layout.field_count, 0);
        
        let just_layout = builder.type_env.get("Just").unwrap();
        assert_eq!(just_layout.tag, 1);
        assert_eq!(just_layout.field_count, 1);
    }

    #[test]
    fn test_constructor_lowering() {
        let mut builder = IRBuilder::new();
        let env = HashMap::new();
        
        // data Maybe a = Nothing | Just a
        let nothing = Constructor { name: "Nothing".to_string(), fields: vec![] };
        let just = Constructor { name: "Just".to_string(), fields: vec![&Term::IntegerType] };
        let maybe = Term::Data("Maybe".to_string(), vec!["a".to_string()], vec![nothing, just]);
        
        builder.lower_term(&maybe, &env);
        
        // App(Var("Just"), Integer(42))
        let just_app = Term::App(&Term::Var("Just".to_string()), &Term::Integer(42));
        
        let res = builder.lower_term(&just_app, &env);
        
        let ir = builder.instructions.join("");
        println!("Generated IR for Constructor:\n{}", ir);
        
        // Should call global constructor
        assert!(ir.contains("call i64 @\"Just\"(i64 42)"));
    }

    #[test]
    fn test_nullary_constructor_lowering() {
        let mut builder = IRBuilder::new();
        let env = HashMap::new();
        
        // data Maybe a = Nothing | Just a
        let nothing = Constructor { name: "Nothing".to_string(), fields: vec![] };
        let just = Constructor { name: "Just".to_string(), fields: vec![&Term::IntegerType] };
        let maybe = Term::Data("Maybe".to_string(), vec!["a".to_string()], vec![nothing, just]);
        
        builder.lower_term(&maybe, &env);
        
        // Var("Nothing")
        let nothing_var = Term::Var("Nothing".to_string());
        
        // For nullary constructors, we need to decide if they are just the tag (as i64) 
        // or a struct with 0 fields.
        // Currently IRBuilder::lower_term(Term::Var) doesn't check type_env.
        // Let's implement it to return the tag if it's a nullary constructor.
        
        let res = builder.lower_term(&nothing_var, &env);
        assert_eq!(res, "0"); // Tag 0 for Nothing
    }
}
