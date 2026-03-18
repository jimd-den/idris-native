# Specification: Architecture Remediation & Restructuring (`remediate_architecture_20260317`)

## Overview
This track implements a full remediation of the Idris Native Architecture Violation Report. It involves restructuring the project into a Clean Architecture ("Screaming Architecture") folder hierarchy and fixing 24 identified violations across Clean Architecture, SOLID, DRY, and KISS principles.

## Functional Requirements
- **Restructuring**: Move all source files into the new proposed structure: `domain/`, `application/`, `adapters/`, `infrastructure/`, `drivers/`, and `common/`.
- **Clean Architecture Fixes**:
  - `CA-01`: Decouple `diagnostics` (Adapter) from inner layers (`syntax_parser`, `llvm_native`, `compiler`).
  - `CA-02`: Inject `LlvmBackend` into `cli_driver` via the `Backend` abstraction at the composition root (`main.rs`).
  - `CA-03`: Encapsulate `IRBuilder` within the Infrastructure layer; remove leaks into the Use Case layer.
  - `CA-04`: Integrate `evaluator` as a first-class Use Case with Arena injection.
- **SOLID Fixes**:
  - `S-01`, `S-02`: Separate logging side-effects from `Scanner` and `LlvmBackend` construction.
  - `S-03`: Decompose `LlvmBackend::compile_to_binary()` into injectable units (I/O, subprocess, cleanup).
  - `S-04`: Inject source text into `Compiler` Use Case instead of performing file I/O directly.
  - `O-01`: Prepare `Term` enum for extension (e.g., via Visitor or Trait Objects) to avoid modifying all consumers.
  - `L-01`: Handle all `Term` variants in `IRBuilder::lower_term()` to prevent runtime panics.
  - `I-01`: Split `Backend` trait into `IrLowerer` and `BinaryEmitter`.
  - `D-01`: Invert `cli_driver`'s dependency on `LlvmBackend` using the `Backend` trait.
- **DRY Fixes**:
  - `DRY-01`: Extract `check_buffer_bounds` helper in `qtt_checker`.
  - `DRY-02`: Extract `count_binary` helper in `qtt_checker`.
  - `DRY-03`: Implement a generic `Cursor<T>` in `common/` for `Scanner` and `Parser`.
  - `DRY-04`: Replace copy-pasted trace logging with a `trace_span!` macro in `common/`.
  - `DRY-05`: Implement `arena_alloc()` test utility in `common/test_helpers.rs`.
  - `DRY-06`: Move `IRBuilder` to its own home in `infrastructure/llvm/ir_builder.rs`.
- **KISS Fixes**:
  - `KISS-01`: Robustify `Arena::alloc()` for deterministic, safe memory management.
  - `KISS-02`: Replace `Box::leak` in `evaluator` with `Arena` allocation.
  - `KISS-03`: Move large inline IR strings to multi-line raw literals or external files.
  - `KISS-04`: Refactor `parse_app` termination logic into a single predicate.
  - `KISS-05`: Correct `If` usage counting in `qtt_checker` (use `max` instead of `sum`).
  - `KISS-06`: Remove 'Hypothesis' bug-documenting tests; convert findings to issues.

## Non-Functional Requirements
- **Performance**: Ensure no performance regressions during restructuring.
- **Test Coverage**: Maintain >99% code coverage for all restructured modules.
- **Idiomatic Rust**: Adhere to Rust 2024 edition standards and Clean Architecture principles.
- **Literate Programming**: Every file must contain clear, detailed documentation explaining the business logic and architectural intent in plain English.

## Acceptance Criteria
1. The project builds successfully with `cargo build`.
2. All unit and integration tests pass after restructuring.
3. The folder structure matches the "Proposed Folder Structure" in the report.
4. Each of the 24 identified violations is demonstrably resolved.
5. All public functions are documented according to project guidelines.

## Out of Scope
- Adding new Idris language features (e.g., `Term::Float`).
- Optimizing LLVM IR generation beyond the identified architecture fixes.
- Modifying the existing `sha256` or `ackermann` logic unless required by restructuring.
