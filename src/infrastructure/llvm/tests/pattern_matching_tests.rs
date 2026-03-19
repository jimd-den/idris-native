#[cfg(test)]
mod tests {
    use crate::infrastructure::llvm::ir_builder::IRBuilder;
    use crate::domain::Term;
    use std::collections::HashMap;

    #[test]
    fn test_case_integer_switch() {
        let mut builder = IRBuilder::new();
        let env = HashMap::new();
        
        // case 42 of 1 => 10 | 2 => 20 | _ => 0
        let branches = vec![
            ("1".to_string(), vec![], &Term::Integer(10)),
            ("2".to_string(), vec![], &Term::Integer(20)),
            ("_".to_string(), vec![], &Term::Integer(0)),
        ];
        let case_term = Term::Case(&Term::Integer(42), branches);
        
        builder.lower_term(&case_term, &env);
        
        let ir = builder.instructions.join("");
        println!("Generated IR for Case Switch:\n{}", ir);
        
        // Should use switch instead of icmp/br chain
        assert!(ir.contains("switch i64"));
        assert!(ir.contains("i64 1, label %"));
        assert!(ir.contains("i64 2, label %"));
        // Default label
        assert!(ir.contains("label %"));
    }

    #[test]
    fn test_case_adt_tag_switch() {
        let mut builder = IRBuilder::new();
        let env = HashMap::new();
        
        // data Maybe a = Nothing | Just a
        let nothing = crate::domain::term::Constructor { name: "Nothing".to_string(), fields: vec![] };
        let just = crate::domain::term::Constructor { name: "Just".to_string(), fields: vec![&Term::IntegerType] };
        let maybe = Term::Data("Maybe".to_string(), vec!["a".to_string()], vec![nothing, just]);
        builder.lower_term(&maybe, &env);

        // case val of Nothing => 0 | Just x => 1
        let branches = vec![
            ("Nothing".to_string(), vec![], &Term::Integer(0)),
            ("Just".to_string(), vec!["x".to_string()], &Term::Integer(1)),
        ];
        // Assume 'val' is a pointer to a struct (passed as i64)
        let case_term = Term::Case(&Term::Var("val".to_string()), branches);
        
        let mut env_with_val = env.clone();
        env_with_val.insert("val".to_string(), "%val_ptr".to_string());
        
        builder.lower_term(&case_term, &env_with_val);
        
        let ir = builder.instructions.join("");
        println!("Generated IR for ADT Case:\n{}", ir);
        
        // Should extract tag: getelementptr { i64, [N x i64] }, ... i32 0, i32 0
        assert!(ir.contains("getelementptr { i64, [") && ir.contains("i32 0, i32 0"));
        // Should load the tag
        assert!(ir.contains("load i64"));
        // Should switch on the tag
        assert!(ir.contains("switch i64"));
    }

    #[test]
    fn test_case_adt_field_extraction() {
        let mut builder = IRBuilder::new();
        let env = HashMap::new();
        
        // data Maybe a = Nothing | Just a
        let nothing = crate::domain::term::Constructor { name: "Nothing".to_string(), fields: vec![] };
        let just = crate::domain::term::Constructor { name: "Just".to_string(), fields: vec![&Term::IntegerType] };
        let maybe = Term::Data("Maybe".to_string(), vec!["a".to_string()], vec![nothing, just]);
        builder.lower_term(&maybe, &env);

        // case val of Nothing => 0 | Just x => x
        let nothing_body = Term::Integer(0);
        let just_body = Term::Var("x".to_string());
        let branches = vec![
            ("Nothing".to_string(), vec![], &nothing_body),
            ("Just".to_string(), vec!["x".to_string()], &just_body),
        ];
        let target = Term::Var("val".to_string());
        let case_term = Term::Case(&target, branches);
        
        let mut env_with_val = env.clone();
        env_with_val.insert("val".to_string(), "%val_ptr".to_string());
        
        builder.lower_term(&case_term, &env_with_val);
        
        let ir = builder.instructions.join("");
        println!("Generated IR for ADT Field Extraction:\n{}", ir);
        
        // Should extract field 'x' from Just
        // GEP {i64, [1 x i64]}, ... i32 0, i32 1, i32 0
        assert!(ir.contains("getelementptr { i64, [0 x i64] }") && ir.contains("i32 0, i32 1, i32 0"));
        // Should load the field
        assert!(ir.contains("load i64"));
    }
}
