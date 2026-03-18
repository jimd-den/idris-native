# Specification: Full Idris 2 Language & Non-QTT Mode (`stubs_and_qtt_20260317`)

## Overview
This track massively expands the scope of the Idris Native compiler to support the full Idris 2 language specification. It introduces a "Non-QTT Mode" to evaluate and run the official reference examples found in the `idris2_ref/` folder without strict linear resource constraints, while simultaneously remediating the existing architectural stubs and gaps.

## Functional Requirements
- **Full Idris 2 Language Parsing & AST Expansion**:
  - Implement full parsing and AST nodes for: Module declarations, imports, Data ADTs with constructors, Type classes/interfaces, pattern matching on constructors, String types/literals, IO monad / `do` notation, mutual blocks, `where` clauses, and records.
- **Non-QTT Mode & Reference Execution**:
  - Introduce a non-QTT fallback mode (e.g., a `--no-qtt` compiler flag or default inference) that bypasses strict linearity checks.
  - Establish an integration test pipeline that compiles and successfully executes the official Idris 2 examples from `idris2_ref/samples/`.
- **Structured Error Pipeline (F)**:
  - Replace `panic!` calls in `Scanner` and `Parser` with a `Result`-based `CompilerError` system.
  - Implement Idris 2 style error diagnostics (carets, line numbers, hints).
- **Backend Stubs & Target Completion (A, B, E)**:
  - Implement missing `LlvmBackend` methods (`gen_integer_ir`, `gen_print_ir`, etc.).
  - Handle all new and existing `Term` variants in `lower_term` to prevent runtime panics.
  - Add basic target switching for `wasm32` and `arm-none-eabi`.
- **QTT Refinement (for QTT Mode)**:
  - Fix linear variable counting in recursive self-calls and wire `Pi` multiplicity annotations.
- **REPL Integration (C)**:
  - Wire `ReplSession::eval` to the real compiler pipeline.

## Non-Functional Requirements
- **TDD & Robustness**: Every new language feature must be driven by tests.
- **Literate Documentation**: Maintain detailed architectural intent comments.

## Acceptance Criteria
1. The compiler successfully parses and compiles the standard `idris2_ref/samples/` examples in non-QTT mode.
2. The REPL accurately evaluates expressions using the actual AST and evaluator.
3. Syntax errors are reported with structured diagnostics, never panics.
4. All previously identified stubs (Backend, QttChecker) are implemented.

## Out of Scope
- Full standard library (`Prelude`) implementation (we will rely on basic primitives and mock the minimal required prelude for the examples).