#[cfg(test)]
mod tests {
    use crate::infrastructure::llvm::ir_builder::IRBuilder;

    #[test]
    fn test_sanitize_id_simple() {
        let builder = IRBuilder::new();
        assert_eq!(builder.sanitize_id("foo"), "\"foo\"");
    }

    #[test]
    fn test_sanitize_id_with_special_chars() {
        let builder = IRBuilder::new();
        // Idris holes start with ?
        assert_eq!(builder.sanitize_id("?foo"), "\"_hole_foo\"");
        // Idris names can have dots
        assert_eq!(builder.sanitize_id("Data.Buffer"), "\"Data_Buffer\"");
        // Idris names can have other special chars
        assert_eq!(builder.sanitize_id("foo-bar"), "\"foo_bar\"");
    }

    #[test]
    fn test_sanitize_id_reserved_keywords() {
        let builder = IRBuilder::new();
        // Even if it's a valid LLVM identifier, we quote it to be safe
        assert_eq!(builder.sanitize_id("define"), "\"define\"");
    }

    #[test]
    fn test_unique_placeholders() {
        let mut builder = IRBuilder::new();
        let p1 = builder.new_placeholder();
        let p2 = builder.new_placeholder();
        assert_ne!(p1, p2);
        assert!(p1.contains("_pat_"));
        assert!(p2.contains("_pat_"));
    }
}
