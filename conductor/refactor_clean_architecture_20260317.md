# Refactoring Plan: Clean Architecture & Observability

## Objective
Align the Idris Native compiler with strict Clean Architecture principles, ensuring clear separation of concerns, dependency inversion, and comprehensive observability.

## Violations Identified
1. **Leaky Layers**: LLVM-specific IR generation (Infrastructure) is currently inside the `compiler` Use Case layer.
2. **Orchestration Placement**: The `cli_driver` (Infrastructure) is orchestrating the compilation pipeline, which is Application Logic belonging in the Use Case layer.
3. **Lack of Telemetry**: Missing ISO 8601 timestamps and detailed function logging required by the project mandate.
4. **Direct Coupling**: The Use Case layer is directly coupled to the LLVM backend instead of an abstraction.

## Proposed Changes

### 1. Refactor Use Case Layer (`src/compiler/mod.rs`)
- **Introduce `Backend` Trait**: Define an abstraction for code generation and binary compilation.
- **Implement `Compiler` Service**: Create an orchestrator that manages the lifecycle:
    - Load source via `syntax_parser`.
    - Validate via `qtt_checker`.
    - Lower via `Backend` abstraction.
    - Compile to binary via `Backend` abstraction.

### 2. Refactor Infrastructure Layer (`src/llvm_native/mod.rs`)
- **Relocate `IRBuilder`**: Move LLVM IR generation logic from `compiler` to `llvm_native`.
- **Implement `Backend` Trait**: Make `LlvmBackend` implement the code generation and toolchain integration logic.
- **Centralize Toolchain Integration**: Keep `clang` invocation inside this layer.

### 3. Thin out Infrastructure Layer (`src/cli_driver/mod.rs`)
- Remove all pipeline logic.
- The `run()` method will only parse arguments and invoke the `Compiler` Use Case.

### 4. Implement Observability & Telemetry
- Add a helper for ISO 8601 timestamp logging.
- Add granular logs to every public function:
    - `[2026-03-17T10:00:00Z] ENTER function_name(args: ...)`
    - `[2026-03-17T10:00:01Z] EXIT function_name -> return_value`

### 5. Literate Documentation
- Add whitepaper-style headers to all files explaining the Clean Architecture role of the component.

## Verification & Testing
- **Unit Tests**: Update all tests to reflect moved logic.
- **Integration Tests**: Verify the `ackermann.idr` and `sha256.idr` still compile and run correctly via the new architecture.
- **Telemetry Audit**: Verify logs appear in stdout with correct timestamps and context.
