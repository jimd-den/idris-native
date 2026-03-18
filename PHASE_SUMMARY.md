# Task Summary: Extend Term AST with Word Types and Bitwise Operators

- **Changes made to `src/core_terms/mod.rs`**: Added `I32Type`, `I8Type`, `BitXor`, `BitAnd`, `BitOr`, `BitNot`, `Shl`, and `Shr` variants to the `Term` enum.
- **Changes made to `src/evaluator/mod.rs`**: Updated `eval`, `eval_owned`, and `substitute` to handle the new variants recursively.
- **Changes made to `src/qtt_checker/mod.rs`**: Updated `check_term` to handle the new variants in QTT validation.
- **Added `src/core_terms/tests/sha256_primitives_tests.rs`**: New tests verifying the creation and structure of the new primitives.
- **Changes made to `src/syntax_parser/mod.rs`**: 
    - Updated `lex` to explicitly handle bitwise operators (`^`, `&`, `|`, `~`) and shift operators (`<<`, `>>`).
    - Refactored `Parser` to implement proper operator precedence levels (unary, shift, bitwise and/xor/or).
    - Updated `parse_primary` to recognize `i32`, `i8`, and `Integer` as type terms.
- **Added `src/syntax_parser/tests/sha256_syntax_tests.rs`**: New tests verifying the lexing and parsing of the new bitwise syntax and word types.
- **Changes made to `src/compiler/mod.rs`**: 
    - Updated `IRBuilder` to support dynamic bit-width (`bit_width` field).
    - Implemented lowering for bitwise XOR (`xor`), AND (`and`), OR (`or`), and NOT (`xor -1`).
    - Implemented lowering for shift operators (`shl`, `lshr`).
    - Refactored `lower_term` to use the configured bit-width for all instructions and function calls.
- **Added `src/compiler/tests/sha256_lowering_tests.rs`**: New tests verifying the correct lowering of bitwise operations and different word types (`i32`, `i8`) to LLVM IR.
- **Why**: Accurate lowering to LLVM IR is critical for performance and correctness. SHA-256 requires precise bitwise manipulation on 32-bit words, which is now supported.
- **Why**: A robust parser is essential for translating Idris 2 source code into the internal AST correctly, respecting operator precedence for complex cryptographic algorithms.
- **Why**: These primitives are foundational for implementing low-level bitwise manipulation algorithms like SHA-256 natively in the compiler.
