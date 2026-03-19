# Implementation Plan: Robust LLVM Backend (`robust_backend_20260318`)

## Phase 1: Identifier Sanitization & Unique Naming
- [x] **Task: Implement Identifier Sanitizer**
    - [ ] **Red:** Write tests in `src/infrastructure/llvm/tests/sanitizer_tests.rs` for various Idris names (`?foo`, `data.Buffer`, `_pat`).
    - [ ] **Green:** Create a `sanitize_id` helper in `LlvmBackend` or `IRBuilder` that escapes special chars and wraps names in quotes.
- [x] **Task: Unique Placeholder Generation**
    - [ ] **Red:** Create a test case that triggers duplicate `%_pat` definitions.
    - [ ] **Green:** Refactor `IRBuilder` to use a counter for all placeholder/pattern names.
- [x] **Task: Conductor - User Manual Verification 'Phase 1' (Protocol in workflow.md)**

## Phase 2: Refined String & IO Primitives
- [x] **Task: Robust Global String Literals**
    - [ ] **Red:** Write tests verifying string IR generation for multiple unique and duplicate strings.
    - [ ] **Green:** Refine `IRBuilder::string_literals` to correctly handle escaping and global definition placement.
- [x] **Task: Implement @concat and IO Stubs**
    - [ ] **Red:** Add an integration test for string concatenation (`"a" ++ "b"`).
    - [ ] **Green:** Provide a more realistic `@concat` implementation in the IR (using `malloc` and `memcpy`).
- [x] **Task: Conductor - User Manual Verification 'Phase 2' (Protocol in workflow.md)**

## Phase 3: ADT Registry & Struct Lowering
- [x] **Task: Implement ADT Registry**
    - [ ] **Red:** Write tests ensuring `data` declarations populate a type environment in the backend.
    - [ ] **Green:** Add `type_env` to `IRBuilder` to track ADT layouts (tags, field offsets).
- [x] **Task: Lower Constructors to Structs**
    - [ ] **Red:** Create a test for `Term::Data` and `Term::App` (constructor call) that expects struct allocation.
    - [ ] **Green:** Implement IR generation for allocating and initializing LLVM structs for constructors.
- [x] **Task: Conductor - User Manual Verification 'Phase 3' (Protocol in workflow.md)**

## Phase 4: Robust Pattern Matching (`switch`)
- [x] **Task: Refactor Case Lowering to `switch`**
    - [ ] **Red:** Write tests for `Term::Case` matching on literal integers, expecting `switch` IR.
    - [ ] **Green:** Implement `switch` instruction generation in `IRBuilder::lower_term`.
- [x] **Task: Implement Tag-Based Matching for ADTs**
    - [ ] **Red:** Create a test matching on ADT constructors (e.g., `Maybe` -> `Just` vs `Nothing`).
    - [ ] **Green:** Extend `Case` lowering to extract the tag from the struct and `switch` on it.
- [x] **Task: Conductor - User Manual Verification 'Phase 4' (Protocol in workflow.md)**

## Phase 5: Final Integration & Sample Verification
- [x] **Task: Compile Sample Suite**
    - [ ] **Red:** Run the `test_all_samples.sh` script and expect failures for `BTree`, `Vect`, etc.
    - [ ] **Green:** Iterate on backend fixes until all selected samples from `idris2_ref/samples/` compile successfully.
- [x] **Task: Conductor - User Manual Verification 'Phase 5' (Protocol in workflow.md)**
