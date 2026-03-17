# Implementation Plan: Scaffold the QTT Screaming Architecture

## Phase 1: Entities & Use Cases Scaffolding [checkpoint: 0b06e41]
- [x] Task: Create `core_terms` module and establish internal, dependency-free Arena allocator.
    - [x] Write initial unit test verifying arena allocation basics.
    - [x] Implement minimal, zero-GC Arena to pass the tests.
- [x] Task: Create `qtt_checker`, `evaluator`, and `compiler` use-case modules.
    - [x] Write basic module linkage tests.
    - [x] Create folder structures and `mod.rs` files.
- [x] Task: Conductor - User Manual Verification 'Phase 1: Entities & Use Cases Scaffolding' (Protocol in workflow.md)

## Phase 2: Adapters & Infrastructure Scaffolding [checkpoint: c379bb2]
- [x] Task: Create `syntax_parser`, `repl_session`, and `diagnostics` adapter modules.
    - [x] Write tests ensuring dependency isolation (Adapters don't leak into Entities).
    - [x] Create folder structures and `mod.rs` files.
- [x] Task: Create `cli_driver` and `llvm_native` framework/driver modules.
    - [x] Ensure `main.rs` routes strictly through `cli_driver`.
    - [x] Create folder structures.
- [x] Task: Conductor - User Manual Verification 'Phase 2: Adapters & Infrastructure Scaffolding' (Protocol in workflow.md)
