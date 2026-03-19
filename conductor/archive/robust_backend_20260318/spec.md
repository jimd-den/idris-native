# Specification: Robust LLVM Backend (`robust_backend_20260318`)

## Overview
This track refactors and extends the LLVM backend to support complex Idris 2 constructs that currently cause compilation failures. It focuses on robust ADT lowering using LLVM structs, efficient pattern matching via the `switch` instruction, and comprehensive identifier sanitization.

## Functional Requirements
- **ADT Lowering (LLVM Structs)**:
  - Implement a registry for Data Type definitions.
  - Lower constructors to LLVM struct types containing a tag and a union/struct of fields.
  - Implement memory allocation for constructors (stack-based for now, or via `malloc`).
- **Robust Pattern Matching (LLVM Switch)**:
  - Refactor `Term::Case` lowering to use the LLVM `switch` instruction for literal integer and tag-based matching.
  - Support nested pattern matching through recursive lowering.
  - Correctly handle wildcard (`_`) branches as the `default` case.
- **Identifier Sanitization**:
  - Implement a sanitization layer that escapes special characters (e.g., `?` -> `hole_`, `.` -> `_`).
  - Use LLVM quoted identifiers (`"..."`) for all Idris-originated names to prevent collisions with LLVM keywords.
- **Fix Argument Redefinition**:
  - Ensure that dummy pattern arguments have unique names (e.g., `_pat_1`, `_pat_2`) to prevent LLVM redefinition errors.
- **Functional IO & Strings**:
  - Properly lower `Term::String` to global constant pointers (already started, needs refinement).
  - Implement `@concat` and other IO primitives using the new robust pointer handling.

## Non-Functional Requirements
- **Runtime Performance**: Prioritize `switch` and efficient struct layouts for fast execution.
- **Robustness**: The backend must never generate IR that fails LLVM validation for standard Idris 2 constructs.
- **Literate Documentation**: Every new backend module must explain the mapping from Idris concepts to LLVM IR.

## Acceptance Criteria
1. `BTree.idr`, `wheres.idr`, `Vect.idr`, and `holes.idr` compile successfully without LLVM errors.
2. Generated IR passes `opt -verify` (or equivalent validation).
3. Pattern matching logic correctly handles both tag-based (ADTs) and value-based (Integers) dispatch.
4. Identifiers with special characters (like holes) are correctly preserved and linked.

## Out of Scope
- Advanced optimization passes.
- Garbage collection (remain strictly QTT/manual for now).
- Foreign Function Interface (FFI) implementation.
