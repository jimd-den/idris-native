# Implementation Plan: Full Idris 2 Language & Non-QTT Mode (`stubs_and_qtt_20260317`)

## Phase 1: Structured Error Pipeline (F, G)
- [ ] **Task: Define `CompilerError` & Remove Diagnostics Mock**
    - [ ] **Red:** Write tests expecting `Result::Err` instead of panics for syntax errors.
    - [ ] **Green:** Implement structured `LexError`, `ParseError`, and `QttError`. Remove the hardcoded timestamp in `diagnostics::log` and implement Idris 2 style caret/hint rendering.
- [ ] **Task: Refactor Scanner & Parser Error Handling**
    - [ ] **Red:** Write tests for unknown characters in the Scanner and invalid syntax in the Parser to catch panics.
    - [ ] **Green:** Refactor `consume`, `parse_*` methods, and `Scanner::scan_token` to return `Result<T, CompilerError>` without crashing.
- [ ] **Task: Conductor - User Manual Verification 'Phase 1' (Protocol in workflow.md)**

## Phase 2: Non-QTT Mode & QTT Refinement (1, 2, 5)
- [ ] **Task: Implement `--no-qtt` Flag & Configuration**
    - [ ] **Red:** Write tests checking the CLI parser correctly accepts `--no-qtt` and passes it to the `Compiler`.
    - [ ] **Green:** Wire the non-QTT flag through `main.rs`, `cli_driver`, and `QttChecker`.
- [ ] **Task: Fix QTT Counting Bugs**
    - [ ] **Red:** Write failing tests for recursive self-calls double-counting linear variables.
    - [ ] **Green:** Fix `count_usage` to correctly scope recursive application arguments.
- [ ] **Task: Wire `Pi` Multiplicity Annotations**
    - [ ] **Red:** Add parser tests for `(0 x : T)` and `(1 x : T)` ensuring they map to the correct AST nodes.
    - [ ] **Green:** Complete the parsing logic and verify it routes correctly to `QttChecker`.
- [ ] **Task: Conductor - User Manual Verification 'Phase 2' (Protocol in workflow.md)**

## Phase 3: Backend Stubs & Missing Targets (A, B, E, I)
- [ ] **Task: Implement Missing `LlvmBackend` Methods**
    - [ ] **Red:** Uncomment and fix `tests_broken.rs` for `gen_integer_ir`, `gen_print_ir`, `set_target`, etc.
    - [ ] **Green:** Implement the missing methods in `LlvmBackend` and `IRBuilder::set_bit_width`.
- [ ] **Task: Handle All `Term` Variants in Lowering**
    - [ ] **Red:** Write tests passing `Lambda`, `Pi`, `LetRec` to `lower_term` expecting valid IR or safe placeholders, not panics.
    - [ ] **Green:** Implement exhaustive pattern matching in `IRBuilder::lower_term`.
- [ ] **Task: Dynamic `main()` arguments & Target Triples**
    - [ ] **Red:** Write tests checking `lower_program` generates dynamic `main()` calls instead of hardcoded `(2, 2)`.
    - [ ] **Green:** Refactor `lower_program` to respect program signatures. Implement stub target switching for WASM/Bare-Metal.
- [ ] **Task: Conductor - User Manual Verification 'Phase 3' (Protocol in workflow.md)**

## Phase 4: Idris 2 Language Expansion (Entities & Adapters)
- [ ] **Task: Expand AST (`Term`) for Full Idris 2**
    - [ ] **Red:** Write tests ensuring the AST can represent modules, imports, Data constructors, interfaces, records, strings, and floats.
    - [ ] **Green:** Add required `Term` variants.
- [ ] **Task: Expand Parser to Full Language**
    - [ ] **Red:** Create test files containing full Idris 2 syntax (e.g., `module Main`, `import Data.Buffer`, `data`, `interface`).
    - [ ] **Green:** Implement parsing logic for these constructs in `Parser`.
- [ ] **Task: Conductor - User Manual Verification 'Phase 4' (Protocol in workflow.md)**

## Phase 5: REPL Integration & Reference Verification (C)
- [ ] **Task: Wire `ReplSession::eval` to Compiler**
    - [ ] **Red:** Write tests ensuring the REPL executes logic instead of mock string matching.
    - [ ] **Green:** Refactor `ReplSession` to invoke the `Evaluator` / `Compiler`.
- [ ] **Task: Execute Reference Examples in Non-QTT Mode**
    - [ ] **Red:** Create a test runner script that targets `idris2_ref/samples/` using the `--no-qtt` flag.
    - [ ] **Green:** Fix remaining compiler/lowering gaps until the selected standard samples compile and execute cleanly.
- [ ] **Task: Conductor - User Manual Verification 'Phase 5' (Protocol in workflow.md)**