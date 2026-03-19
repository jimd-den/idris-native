# Implementation Plan: Full Idris 2 Language & Non-QTT Mode (`stubs_and_qtt_20260317`)

## Phase 1: Structured Error Pipeline (F, G) [checkpoint: c4c17d1]
- [x] **Task: Define `CompilerError` & Remove Diagnostics Mock**
    - [x] **Red:** Write tests expecting `Result::Err` instead of panics for syntax errors.
    - [x] **Green:** Implement structured `LexError`, `ParseError`, and `QttError`. Remove the hardcoded timestamp in `diagnostics::log` and implement Idris 2 style caret/hint rendering.
- [x] **Task: Refactor Scanner & Parser Error Handling**
    - [x] **Red:** Write tests for unknown characters in the Scanner and invalid syntax in the Parser to catch panics.
    - [x] **Green:** Refactor `consume`, `parse_*` methods, and `Scanner::scan_token` to return `Result<T, CompilerError>` without crashing.
- [x] **Task: Conductor - User Manual Verification 'Phase 1' (Protocol in workflow.md)**

## Phase 2: Non-QTT Mode & QTT Refinement (1, 2, 5) [checkpoint: 1b178d2]
- [x] **Task: Implement `--no-qtt` Flag & Configuration**
    - [x] **Red:** Write tests checking the CLI parser correctly accepts `--no-qtt` and passes it to the `Compiler`.
    - [x] **Green:** Wire the non-QTT flag through `main.rs`, `cli_driver`, and `QttChecker`.
- [x] **Task: Fix QTT Counting Bugs**
    - [x] **Red:** Write failing tests for recursive self-calls double-counting linear variables.
    - [x] **Green:** Fix `count_usage` to correctly scope recursive application arguments.
- [x] **Task: Wire `Pi` Multiplicity Annotations**
    - [x] **Red:** Add parser tests for `(0 x : T)` and `(1 x : T)` ensuring they map to the correct AST nodes.
    - [x] **Green:** Complete the parsing logic and verify it routes correctly to `QttChecker`.
- [x] **Task: Conductor - User Manual Verification 'Phase 2' (Protocol in workflow.md)**

## Phase 3: Backend Stubs & Missing Targets (A, B, E, I) [checkpoint: fb62e92]
- [x] **Task: Implement Missing `LlvmBackend` Methods**
    - [x] **Red:** Uncomment and fix `tests_broken.rs` for `gen_integer_ir`, `gen_print_ir`, `set_target`, etc.
    - [x] **Green:** Implement the missing methods in `LlvmBackend` and `IRBuilder::set_bit_width`.
- [x] **Task: Handle All `Term` Variants in Lowering**
    - [x] **Red:** Write tests passing `Lambda`, `Pi`, `LetRec` to `lower_term` expecting valid IR or safe placeholders, not panics.
    - [x] **Green:** Implement exhaustive pattern matching in `IRBuilder::lower_term`.
- [x] **Task: Dynamic `main()` arguments & Target Triples**
    - [x] **Red:** Write tests checking `lower_program` generates dynamic `main()` calls instead of hardcoded `(2, 2)`.
    - [x] **Green:** Refactor `lower_program` to respect program signatures. Implement stub target switching for WASM/Bare-Metal.
- [x] **Task: Conductor - User Manual Verification 'Phase 3' (Protocol in workflow.md)**

## Phase 4: Idris 2 Language Expansion (Entities & Adapters) [checkpoint: 997f1ba]
- [x] **Task: Expand AST (`Term`) for Full Idris 2**
    - [x] **Red:** Write tests ensuring the AST can represent modules, imports, Data constructors, interfaces, records, strings, and floats.
    - [x] **Green:** Add required `Term` variants.
- [x] **Task: Expand Parser to Full Language**
    - [x] **Red:** Create test files containing full Idris 2 syntax (e.g., `module Main`, `import Data.Buffer`, `data`, `interface`).
    - [x] **Green:** Implement parsing logic for these constructs in `Parser`.
- [ ] **Task: Conductor - User Manual Verification 'Phase 4' (Protocol in workflow.md)**

## Phase 5: REPL Integration & Reference Verification (C)
- [x] **Task: Wire `ReplSession::eval` to Compiler**
    - [x] **Red:** Write tests ensuring the REPL executes logic instead of mock string matching.
    - [x] **Green:** Refactor `ReplSession` to invoke the `Evaluator` / `Compiler`.
- [~] **Task: Execute Reference Examples in Non-QTT Mode**
    - [ ] **Red:** Create a test runner script that targets `idris2_ref/samples/` using the `--no-qtt` flag.
    - [ ] **Green:** Fix remaining compiler/lowering gaps until the selected standard samples compile and execute cleanly.
- [ ] **Task: Conductor - User Manual Verification 'Phase 5' (Protocol in workflow.md)**