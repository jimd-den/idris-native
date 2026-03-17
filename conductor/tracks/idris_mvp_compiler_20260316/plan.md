# Implementation Plan: Idris Native MVP Compiler & REPL

## Phase 1: Foundation & Core REPL
- [x] Task: Implement basic expression evaluation in the REPL.
    - [x] Write tests for parsing and evaluating core primitives (integers, strings).
    - [x] Implement the evaluation logic in `repl_session`.
- [x] Task: Implement type inspection (`:t`) in the REPL.
    - [x] Write tests for displaying types of core terms.
    - [x] Implement the type inspection logic in `repl_session` and `diagnostics`.
- [x] Task: Implement file loading (`:l`) in the REPL.
    - [x] Write tests for loading definitions from a source file.
    - [x] Implement file loading and hot-reloading logic.
- [x] Task: Conductor - User Manual Verification 'Phase 1: Foundation & Core REPL' (Protocol in workflow.md)

## Phase 2: QTT & Frontend Elaboration
- [ ] Task: Implement algebraic data types (ADTs) in the QTT checker.
    - [ ] Write tests for ADT definition and pattern matching rules.
    - [ ] Implement ADT elaboration logic in `qtt_checker`.
- [ ] Task: Implement basic Type Classes (Interfaces).
    - [ ] Write tests for interface definition and implementation resolution.
    - [ ] Implement type class elaboration logic.
- [ ] Task: Conductor - User Manual Verification 'Phase 2: QTT & Frontend Elaboration' (Protocol in workflow.md)

## Phase 3: Pure LLVM Backend & Code Generation
- [ ] Task: Implement code generation for core primitives using pure LLVM IR.
    - [ ] Write tests for native code generation without C runtime dependencies.
    - [ ] Implement LLVM IR generation for integers, strings, etc., in `llvm_native`.
- [ ] Task: Implement zero-GC memory management via QTT in the backend.
    - [ ] Write tests verifying deterministic memory deallocation in generated code.
    - [ ] Implement the resource-tracking-to-LLVM-IR lowering.
- [ ] Task: Implement runtime I/O primitives (e.g., `print`) in pure LLVM/Assembly.
    - [ ] Write tests for native console output.
    - [ ] Implement low-level I/O routines.
- [ ] Task: Conductor - User Manual Verification 'Phase 3: Pure LLVM Backend & Code Generation' (Protocol in workflow.md)

## Phase 4: Cross-Platform & Verification
- [ ] Task: Enable WebAssembly (WASM) target support.
    - [ ] Write tests verifying WASM generation and execution.
    - [ ] Configure `llvm_native` for WASM emission.
- [ ] Task: Verify Embedded/No-OS target support.
    - [ ] Write tests for bare-metal execution (e.g., via QEMU or similar).
    - [ ] Implement necessary low-level startup routines for no-OS.
- [ ] Task: Performance Benchmarking & Optimization.
    - [ ] Write comparative benchmarks against well-optimized C code.
    - [ ] Apply LLVM optimization passes and refine IR generation to meet performance goals.
- [ ] Task: Conductor - User Manual Verification 'Phase 4: Cross-Platform & Verification' (Protocol in workflow.md)
