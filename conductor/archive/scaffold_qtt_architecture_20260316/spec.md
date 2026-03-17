# Specification: Scaffold the QTT Screaming Architecture

## Objective
Establish the foundational directory structure and module scaffolding for the Idris 2 compiler using a QTT-driven Screaming Architecture. The design emphasizes zero-GC runtime management, explicit domain representation, and extreme performance using data-oriented design (Arena allocation).

## Architecture Layout
The codebase must be structured as follows:

```text
src/
├── core_terms/         # Entities: Core AST, QTT Resources/Multiplicities (Arena Allocated)
├── qtt_checker/        # Use Cases: Quantitative Type Theory Elaboration & Resource Tracking
├── evaluator/          # Use Cases: Compile-time normalization & reduction
├── compiler/           # Use Cases: Pipeline orchestrator (Parse -> Check -> Lower)
├── syntax_parser/      # Adapters: Translating source code text into core_terms
├── repl_session/       # Adapters: Managing interactive REPL state and inputs
├── diagnostics/        # Adapters: Translating QTT/Parsing errors into Idris-style output
├── cli_driver/         # Frameworks/Drivers: Command-line entrypoint and argument routing
└── llvm_native/        # Frameworks/Drivers: LLVM IR generation & AOT/JIT (Strictly GC-free)
```

## Requirements
1. **Module Scaffolding:** Create all required directories under `src/` with corresponding `mod.rs` or `lib.rs` files.
2. **Clean Architecture Adherence:** Document the boundaries and dependency rules within the module headers.
3. **Rust Best Practices:** Ensure the base project compiles cleanly and passes all initial linting checks (`cargo check` / `cargo clippy`).
