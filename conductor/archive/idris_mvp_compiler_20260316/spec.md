# Specification: Idris Native MVP Compiler & REPL

## Overview
This track aims to implement a Turing-complete Idris 2 compiler and interactive REPL (Read-Eval-Print Loop) that exceeds the performance of well-optimized C while remaining flexible for multiple architectures. This will be achieved through a custom QTT-driven Screaming Architecture in Rust and a pure LLVM backend.

## Functional Requirements
- **Compiler (Idris 2 Syntax)**:
    - Support for Core Primitives: Integers, Strings, Booleans, etc.
    - Full support for Algebraic Data Types (ADTs) and pattern matching.
    - Implementation of basic Type Classes (Interfaces).
- **Zero-C Runtime**:
    - Strictly **pure LLVM** generation with no dependency on C runtimes or external C libraries.
    - Implement all runtime primitives (memory management, basic I/O) directly in the generated LLVM IR or as minimal, hand-optimized assembly/LLVM modules.
- **REPL (Interactive)**:
    - **Basic Evaluation**: Evaluate expressions directly in the REPL.
    - **Type Inspection (:t)**: Provide the inferred type of any term or definition.
    - **File Loading (:l)**: Load and reload definitions from Idris source files (`.idr`).
- **Flexible Architectures**:
    - Support for Mainstream Native (x86_64, AArch64).
    - WebAssembly (WASM) target support.
    - Support for Embedded/No-OS environments for bare-metal execution.

## Non-Functional Requirements
- **Performance**:
    - **LLVM Optimization**: Leverage advanced LLVM optimization passes (O3+) to exceed C runtime performance.
    - **Compile-time Speed**: Optimize the Rust-based compiler frontend and QTT checker for rapid feedback.
    - **Runtime O(1) Primitives**: Ensure core operations (like memory allocation/deallocation via QTT) are O(1) where possible.
- **Methodology**: Strict adherence to Red/Green/Refactor TDD (Test-Driven Development).
- **Architecture**: Rigid adherence to the QTT Screaming Architecture (Entities, Use Cases, Adapters, Infra).

## Acceptance Criteria
- [ ] A sample "Hello World" Idris 2 program compiles to a native binary with no C runtime dependency.
- [ ] A sample program using ADTs and pattern matching executes correctly.
- [ ] The REPL successfully evaluates expressions and displays their types.
- [ ] The compiler generates valid WASM for a basic Idris 2 program.
- [ ] Automated benchmarks show the generated code performing at or better than equivalent C code.

## Out of Scope
- Full Idris 2 library support (only core primitives and MVP features).
- Advanced IDE integrations (only CLI/REPL).
- Complex build systems (relying on `cli_driver` for now).
