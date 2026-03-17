# Technology Stack

## Core Language
- **Rust (2024 Edition)**: The project's primary language, chosen for its performance, safety, and excellent toolchain support.

## Backend & Tooling
- **LLVM**: Used for generating high-performance native machine code, WebAssembly (WASM), and bare-metal targets.
- **Cargo**: Rust's build system and package manager.

## Architecture & Memory Management
- **QTT Screaming Architecture**: The codebase is organized by primary domain capabilities (`core_terms`, `qtt_checker`, `llvm_native`, etc.) ensuring a pure mapping of Idris compiler use cases.
- **Zero-GC / Quantitative Type Theory**: Memory is managed deterministically via compile-time linearity and resource tracking (QTT). The runtime explicitly relies on `qtt_checker` bounds rather than a garbage collector.

## Libraries & Dependencies
- **Standard Library**: Minimal dependencies are preferred to maintain high security and performance.
