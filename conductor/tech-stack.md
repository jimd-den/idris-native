# Technology Stack

## Core Language
- **Rust (2024 Edition)**: The project's primary language, chosen for its performance, safety, and excellent toolchain support.

## Backend & Tooling
- **LLVM**: Used for generating high-performance native machine code, WebAssembly (WASM), and bare-metal targets.
- **Cargo**: Rust's build system and package manager.

## Architecture & Memory Management
- **Clean Screaming Architecture**: The codebase follows a rigid four-layer Clean Architecture:
  - **Domain (Entities)**: Core AST and primitive types.
  - **Application (Use Cases)**: Orchestration logic (Checker, Evaluator, Compiler).
  - **Adapters**: Translation between internal and external formats (Parser, Diagnostics).
  - **Infrastructure/Drivers**: External tools and entry points (LLVM, CLI).
- **Zero-GC / Quantitative Type Theory**: Memory is managed deterministically via compile-time linearity and resource tracking (QTT). The runtime explicitly relies on `qtt_checker` bounds rather than a garbage collector.
- **Arena Allocation**: An internal `Arena` is used for high-performance, deterministic memory management of AST nodes and evaluation results.

## Libraries & Dependencies
- **Standard Library**: Minimal dependencies are preferred to maintain high security and performance.
