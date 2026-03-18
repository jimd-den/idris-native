# Implementation Plan: SHA-256 Hasher (Red/Green TDD)

## Phase 1: Bitwise Primitives and Word Types [checkpoint: 0ff855f]
- [x] Task: TDD Red/Green - Extend `Term` AST with Word Types (`i32`, `i8`) and Bitwise Operators (`Xor`, `And`, `Or`, `Not`, `ShiftLeft`, `ShiftRight`). Write failing tests first.
- [x] Task: TDD Red/Green - Update the Parser to handle the new syntax and operators. Test with malformed and correct syntax strings.
- [x] Task: TDD Red/Green - Implement lowering for the new types and operators in `IRBuilder`. Write tests verifying bitwise operations lower to correct LLVM IR and execute correctly.
- [x] Task: Conductor - User Manual Verification 'Phase 1: Bitwise Primitives and Word Types' (Protocol in workflow.md)

## Phase 2: Array/Buffer Primitives
- [x] Task: TDD Red/Green - Add a fixed-size `Buffer` primitive to `Term` AST alongside `BufferLoad` and `BufferStore`. Write failing structural tests.
- [x] Task: TDD Red/Green - Enforce QTT bounds on buffers in `QttChecker` to prevent memory leaks or use-after-free. Write failing boundary tests first.
- [x] Task: TDD Red/Green - Implement lowering of buffer operations to LLVM `alloca`, `getelementptr`, `load`, and `store` instructions. Verify with native execution tests.
- [ ] Task: Conductor - User Manual Verification 'Phase 2: Array/Buffer Primitives' (Protocol in workflow.md)

## Phase 3: SHA-256 Idris Implementation
- [ ] Task: TDD Red/Green - Write `sha256.idr` incrementally. Test individual block hashing operations and verify state updates via integration tests.
- [ ] Task: TDD Red/Green - Update `cli_driver` string printing routines to support hex output (for hash strings). Write failing I/O routine tests first.
- [ ] Task: Compile and execute `sha256.idr` end-to-end to verify correct final hash output matches expected test vectors.
- [ ] Task: Conductor - User Manual Verification 'Phase 3: SHA-256 Idris Implementation' (Protocol in workflow.md)