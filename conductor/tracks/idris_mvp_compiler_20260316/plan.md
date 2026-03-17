# Implementation Plan: Idris Native MVP Compiler & REPL

## Phase 1: Foundation & Core REPL [checkpoint: b7ce1c5]
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

## Phase 2: QTT & Frontend Elaboration [checkpoint: d38a227]
- [x] Task: Implement algebraic data types (ADTs) in the QTT checker.
    - [x] Write tests for ADT definition and pattern matching rules.
    - [x] Implement ADT elaboration logic in `qtt_checker`.
- [x] Task: Implement basic Type Classes (Interfaces).
    - [x] Write tests for interface definition and implementation resolution.
    - [x] Implement type class elaboration logic with QTT multiplicities.
- [x] Task: Conductor - User Manual Verification 'Phase 2: QTT & Frontend Elaboration' (Protocol in workflow.md)

## Phase 3: Pure LLVM Backend & Code Generation [checkpoint: b2f679f]
- [x] Task: Implement code generation for core primitives using pure LLVM IR.
    - [x] Write tests for native code generation without C runtime dependencies.
    - [x] Implement LLVM IR generation for integers, strings, etc., in `llvm_native`.
- [x] Task: Implement zero-GC memory management via QTT in the backend.
    - [x] Write tests verifying deterministic memory deallocation in generated code.
    - [x] Implement the resource-tracking-to-LLVM-IR lowering.
- [x] Task: Implement runtime I/O primitives (e.g., `print`) in pure LLVM/Assembly.
    - [x] Write tests for native console output.
    - [x] Implement low-level I/O routines.
- [x] Task: Conductor - User Manual Verification 'Phase 3: Pure LLVM Backend & Code Generation' (Protocol in workflow.md)

## Phase 4: Cross-Platform & Verification [checkpoint: cf18eef]
- [x] Task: Enable WebAssembly (WASM) target support.
    - [x] Write tests verifying WASM generation and execution.
    - [x] Configure `llvm_native` for WASM emission.
- [x] Task: Verify Embedded/No-OS target support.
    - [x] Write tests for bare-metal execution (e.g., via QEMU or similar).
    - [x] Implement necessary low-level startup routines for no-OS.
- [x] Task: Performance Benchmarking & Optimization.
    - [x] Write comparative benchmarks against well-optimized C code.
    - [x] Apply LLVM optimization passes and refine IR generation to meet performance goals.
- [x] Task: Conductor - User Manual Verification 'Phase 4: Cross-Platform & Verification' (Protocol in workflow.md)
