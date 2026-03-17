# Implementation Plan: Scaffold the QTT Screaming Architecture

## Phase 1: Entities & Use Cases Scaffolding
- [ ] Task: Create `core_terms` module and establish `bumpalo` arena memory types context.
    - [ ] Write initial unit test verifying arena allocation basics.
    - [ ] Implement scaffold code to pass the tests.
- [ ] Task: Create `qtt_checker`, `evaluator`, and `compiler` use-case modules.
    - [ ] Write basic module linkage tests.
    - [ ] Create folder structures and `mod.rs` files.
- [ ] Task: Conductor - User Manual Verification 'Phase 1: Entities & Use Cases Scaffolding' (Protocol in workflow.md)

## Phase 2: Adapters & Infrastructure Scaffolding
- [ ] Task: Create `syntax_parser`, `repl_session`, and `diagnostics` adapter modules.
    - [ ] Write tests ensuring dependency isolation (Adapters don't leak into Entities).
    - [ ] Create folder structures and `mod.rs` files.
- [ ] Task: Create `cli_driver` and `llvm_native` framework/driver modules.
    - [ ] Ensure `main.rs` routes strictly through `cli_driver`.
    - [ ] Create folder structures.
- [ ] Task: Conductor - User Manual Verification 'Phase 2: Adapters & Infrastructure Scaffolding' (Protocol in workflow.md)
