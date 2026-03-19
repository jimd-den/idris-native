#[cfg(test)]
mod tests {
    use crate::application::compiler::{Compiler, Backend};
    use crate::infrastructure::llvm::LlvmBackend;
    use crate::domain::Term;
    use std::collections::HashMap;

    #[test]
    fn test_string_concatenation_lowering() {
        let backend = LlvmBackend::new();
        let compiler = Compiler::new(&backend);
        
        let source = r#"
main : String
main = "Hello, " ++ "World!"
"#;
        
        // 1. Parse
        let mut arena = crate::domain::arena::Arena::new();
        let tokens = crate::adapters::syntax_parser::lex(source).unwrap();
        let mut parser = crate::adapters::syntax_parser::Parser::new(tokens, &mut arena);
        let declarations = parser.parse_program().unwrap();

        // 2. Lower
        let ir = backend.lower_program(&declarations);
        println!("Generated IR:\n{}", ir);
        
        // Verify that @concat is called in the IR
        assert!(ir.contains("call i64 @concat(i64 %"));
        // Verify the runtime declaration is present
        assert!(ir.contains("define i64 @concat(i64 %s1_int, i64 %s2_int)"));
    }
}
