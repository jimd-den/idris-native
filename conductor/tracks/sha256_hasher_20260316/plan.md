# Implementation Plan: SHA-256 Hasher (Red/Green TDD)

## Phase 1: Bitwise Primitives and Word Types [checkpoint: 0ff855f]
- [x] Task: TDD Red/Green - Extend `Term` AST with Word Types (`i32`, `i8`) and Bitwise Operators (`Xor`, `And`, `Or`, `Not`, `ShiftLeft`, `ShiftRight`). Write failing tests first.
- [x] Task: TDD Red/Green - Align Parser with official Idris 2 syntax (operators like `xor`, `.&.`, `.|.`, and buffer routines like `setBits64`).
- [x] Task: TDD Red/Green - Update the Parser to handle the new syntax and operators. Test with malformed and correct syntax strings.
- [x] Task: TDD Red/Green - Implement lowering for the new types and operators in `IRBuilder`. Write tests verifying bitwise operations lower to correct LLVM IR and execute correctly.
- [x] Task: Conductor - User Manual Verification 'Phase 1: Bitwise Primitives and Word Types' (Protocol in workflow.md)

## Phase 2: Array/Buffer Primitives [checkpoint: 772600a]
- [x] Task: TDD Red/Green - Add a fixed-size `Buffer` primitive to `Term` AST alongside `BufferLoad` and `BufferStore`. Write failing structural tests.
- [x] Task: TDD Red/Green - Enforce QTT bounds on buffers in `QttChecker` to prevent memory leaks or use-after-free. Write failing boundary tests first.
- [x] Task: TDD Red/Green - Implement lowering of buffer operations to LLVM `alloca`, `getelementptr`, `load`, and `store` instructions. Verify with native execution tests.
- [x] Task: Conductor - User Manual Verification 'Phase 2: Array/Buffer Primitives' (Protocol in workflow.md)


## Phase 3: Mandatory QTT Enforcement & Sample Support
- [~] Task: Mandatory QTT Multiplicity Checking.
    - [x] Implement real multiplicity tracking (0, 1, Unrestricted) in `QttChecker`.
    - [x] Update `cli_driver` to halt if `QttChecker` returns false.
- [ ] Task: Official Idris 2 Function Syntax.
    - [x] Implement full parsing of type signatures, including `->`, `:`, and multiplicity annotations.
- [x] Task: Support for ADTs and Pattern Matching.
    - [x] Extend AST and Parser for `data` declarations and `case` expressions.
    - [x] Implement lowering for pattern matching.
- [~] Task: Pure LLVM Prelude and Sample Support.
    - [ ] Create a minimal `Prelude.idr` mapping to pure LLVM syscalls.
    - [ ] Compile and execute representative samples from `idris2_ref` (e.g., bitwise and buffer tests).
- [ ] Task: Conductor - User Manual Verification 'Phase 3: Mandatory QTT Enforcement & Sample Support' (Protocol in workflow.md)