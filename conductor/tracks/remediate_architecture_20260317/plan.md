# Implementation Plan: Architecture Remediation (v2 - Granular TDD)

## Phase 1: Structural Reorganization (In-place Migration)
- [ ] **Task: Restructure `domain/` layer**
    - [ ] Create `src/domain/` directory.
    - [ ] Move `src/core_terms/*` to `src/domain/`.
    - [ ] Update `src/lib.rs` and `src/domain/mod.rs` to reflect the change.
    - [ ] Verify build and existing tests pass.
- [ ] **Task: Restructure `application/` layer**
    - [ ] Create `src/application/` and subdirectories (`qtt_checker`, `evaluator`, `compiler`).
    - [ ] Move `src/qtt_checker/` to `src/application/qtt_checker/`.
    - [ ] Move `src/evaluator/` to `src/application/evaluator/`.
    - [ ] Move `src/compiler/` to `src/application/compiler/`.
    - [ ] Update `src/lib.rs` and submodule `mod.rs` files.
    - [ ] Verify build and existing tests pass.
- [ ] **Task: Restructure `adapters/` layer**
    - [ ] Create `src/adapters/` and subdirectories (`syntax_parser`, `diagnostics`, `repl_session`).
    - [ ] Move `src/syntax_parser/` to `src/adapters/syntax_parser/`.
    - [ ] Move `src/diagnostics/` to `src/adapters/diagnostics/`.
    - [ ] Move `src/repl_session/` to `src/adapters/repl_session/`.
    - [ ] Update `src/lib.rs` and submodule `mod.rs` files.
    - [ ] Verify build and existing tests pass.
- [ ] **Task: Restructure `infrastructure/` and `drivers/` layers**
    - [ ] Move `src/llvm_native/` to `src/infrastructure/llvm/`.
    - [ ] Move `src/cli_driver/` to `src/drivers/cli_driver/`.
    - [ ] Update `src/lib.rs` and submodule `mod.rs` files.
    - [ ] Verify build and existing tests pass.
- [ ] **Task: Conductor - User Manual Verification 'Phase 1' (Protocol in workflow.md)**

## Phase 2: Domain Layer Robustification (Red/Green TDD)
- [ ] **Task: Robustify `Arena` Allocator (KISS-01 - Revised)**
    - [ ] **Red:** Write stress tests for `Arena` with large allocations and boundary conditions in `src/domain/tests/arena_robust_tests.rs`.
    - [ ] **Green:** Refactor `Arena::alloc` to ensure safe, robust memory management without simplifying away the raw-pointer control.
- [ ] **Task: Modularize `Term` and `Multiplicity`**
    - [ ] **Red:** Write exhaustive match tests for `Term` variants in `src/domain/tests/term_structure_tests.rs`.
    - [ ] **Green:** Extract `Term` enum to `src/domain/term.rs` and `Multiplicity` to `src/domain/multiplicity.rs`. Implement Literate Programming comments.
- [ ] **Task: Conductor - User Manual Verification 'Phase 2' (Protocol in workflow.md)**

## Phase 3: Common Utilities & Literate Foundation
- [ ] **Task: Implement `common/cursor.rs` (DRY-03)**
    - [ ] **Red:** Write tests for generic `Cursor<T>` in `src/common/tests/cursor_tests.rs`.
    - [ ] **Green:** Implement small, composable functions for `advance`, `peek`, `match` in `src/common/cursor.rs`.
- [ ] **Task: Implement `common/logging.rs` (DRY-04)**
    - [ ] **Red:** Write tests for `trace_span!` macro in `src/common/tests/logging_tests.rs`.
    - [ ] **Green:** Implement `trace_span!` macro and RAII guard for deterministic entry/exit logging.
- [ ] **Task: Implement `common/test_helpers.rs` (DRY-05)**
    - [ ] **Red:** Create failing test requiring boiler-plate-free `arena_alloc` in `src/common/tests/helper_tests.rs`.
    - [ ] **Green:** Implement `arena_alloc()` helper.
- [ ] **Task: Conductor - User Manual Verification 'Phase 3' (Protocol in workflow.md)**

## Phase 4: Application Layer Refinement (Red/Green TDD)
- [ ] **Task: Refactor `QttChecker` Usage Logic (KISS-05, DRY-01, DRY-02)**
    - [ ] **Red:** Update `src/application/qtt_checker/tests/multiplicity_tests.rs` with failing `If` branch usage counts.
    - [ ] **Green:** Refactor `count_usage` to use `max` for branches. Extract `check_buffer_bounds` and `count_binary` into small, composable functions.
- [ ] **Task: Integrate `Evaluator` with `Arena` (CA-04, KISS-02)**
    - [ ] **Red:** Write tests in `src/application/evaluator/tests/memory_leak_tests.rs` that fail without Arena injection.
    - [ ] **Green:** Refactor `eval` and `substitute` to use `&mut Arena`, removing `Box::leak`.
- [ ] **Task: Decouple `Compiler` from I/O (S-04)**
    - [ ] **Red:** Write tests for `Compiler::compile_str()` in `src/application/compiler/tests/string_input_tests.rs`.
    - [ ] **Green:** Refactor `Compiler` to accept `&str` through a port-like interface.
- [ ] **Task: Conductor - User Manual Verification 'Phase 4' (Protocol in workflow.md)**

## Phase 5: Adapters & Infrastructure Refinement (Red/Green TDD)
- [ ] **Task: Refactor `SyntaxParser` to use `Cursor` (DRY-03, S-01)**
    - [ ] **Red:** Update parser/scanner tests to verify zero diagnostics dependency.
    - [ ] **Green:** Refactor `Scanner` and `Parser` to use `common::Cursor`. Replace explicit logging with `trace_span!`.
- [ ] **Task: Robustify `LlvmBackend` & IR Generation (S-02, S-03, KISS-03, L-01)**
    - [ ] **Red:** Write tests in `src/infrastructure/llvm/tests/robustness_tests.rs` for unhandled `Term` variants and side-effects.
    - [ ] **Green:** Split `LlvmBackend` into `IrLowerer` and `BinaryEmitter`. Decompose `compile_to_binary()` into `toolchain.rs`. Ensure exhaustive term handling in `lower_term()`.
- [ ] **Task: Extract `IRBuilder` (DRY-06, CA-03)**
    - [ ] **Red:** Write tests for `IRBuilder` in isolation in `src/infrastructure/llvm/tests/ir_builder_tests.rs`.
    - [ ] **Green:** Move `IRBuilder` to its own module. Document with literate comments.
- [ ] **Task: Conductor - User Manual Verification 'Phase 5' (Protocol in workflow.md)**

## Phase 6: Drivers & Final Integration (CA-02, D-01)
- [ ] **Task: Decouple `CliDriver`**
    - [ ] **Red:** Write tests for `CliDriver` using a Mock backend in `src/drivers/cli_driver/tests/mock_tests.rs`.
    - [ ] **Green:** Refactor `CliDriver` to accept a trait object. Update `main.rs` as the composition root.
- [ ] **Task: Final Literate Cleanup & Hypothesis Removal (KISS-06)**
    - [ ] Remove hypothesis tests. Add comprehensive Literate Programming comments to all files.
    - [ ] Final end-to-end integration test (`ackermann`, `sha256`).
- [ ] **Task: Conductor - User Manual Verification 'Phase 6' (Protocol in workflow.md)**
