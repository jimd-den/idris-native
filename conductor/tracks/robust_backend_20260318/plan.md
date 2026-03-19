# Implementation Plan: Robust LLVM Backend (`robust_backend_20260318`)

## Phase 1: Identifier Sanitization & Unique Naming
- [ ] **Task: Implement Identifier Sanitizer**
    - [ ] **Red:** Write tests in `src/infrastructure/llvm/tests/sanitizer_tests.rs` for various Idris names (`?foo`, `data.Buffer`, `_pat`).
    - [ ] **Green:** Create a `sanitize_id` helper in `LlvmBackend` or `IRBuilder` that escapes special chars and wraps names in quotes.
- [ ] **Task: Unique Placeholder Generation**
    - [ ] **Red:** Create a test case that triggers duplicate `%_pat` definitions.
    - [ ] **Green:** Refactor `IRBuilder` to use a counter for all placeholder/pattern names.
- [ ] **Task: Conductor - User Manual Verification 'Phase 1' (Protocol in workflow.md)**

## Phase 2: Refined String & IO Primitives
- [ ] **Task: Robust Global String Literals**
    - [ ] **Red:** Write tests verifying string IR generation for multiple unique and duplicate strings.
    - [ ] **Green:** Refine `IRBuilder::string_literals` to correctly handle escaping and global definition placement.
- [ ] **Task: Implement `@concat` and IO Stubs**
    - [ ] **Red:** Add an integration test for string concatenation (`"a" ++ "b"`).
    - [ ] **Green:** Provide a more realistic `@concat` implementation in the IR (using `malloc` and `memcpy`).
- [ ] **Task: Conductor - User Manual Verification 'Phase 2' (Protocol in workflow.md)**

## Phase 3: ADT Registry & Struct Lowering
- [ ] **Task: Implement ADT Registry**
    - [ ] **Red:** Write tests ensuring `data` declarations populate a type environment in the backend.
    - [ ] **Green:** Add `type_env` to `IRBuilder` to track ADT layouts (tags, field offsets).
- [ ] **Task: Lower Constructors to Structs**
    - [ ] **Red:** Create a test for `Term::Data` and `Term::App` (constructor call) that expects struct allocation.
    - [ ] **Green:** Implement IR generation for allocating and initializing LLVM structs for constructors.
- [ ] **Task: Conductor - User Manual Verification 'Phase 3' (Protocol in workflow.md)**

## Phase 4: Robust Pattern Matching (`switch`)
- [ ] **Task: Refactor Case Lowering to `switch`**
    - [ ] **Red:** Write tests for `Term::Case` matching on literal integers, expecting `switch` IR.
    - [ ] **Green:** Implement `switch` instruction generation in `IRBuilder::lower_term`.
- [ ] **Task: Implement Tag-Based Matching for ADTs**
    - [ ] **Red:** Create a test matching on ADT constructors (e.g., `Maybe` -> `Just` vs `Nothing`).
    - [ ] **Green:** Extend `Case` lowering to extract the tag from the struct and `switch` on it.
- [ ] **Task: Conductor - User Manual Verification 'Phase 4' (Protocol in workflow.md)**

## Phase 5: Final Integration & Sample Verification
- [ ] **Task: Compile Sample Suite**
    - [ ] **Red:** Run the `test_all_samples.sh` script and expect failures for `BTree`, `Vect`, etc.
    - [ ] **Green:** Iterate on backend fixes until all selected samples from `idris2_ref/samples/` compile successfully.
- [ ] **Task: Conductor - User Manual Verification 'Phase 5' (Protocol in workflow.md)**
