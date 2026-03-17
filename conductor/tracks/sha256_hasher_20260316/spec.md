# Specification: SHA-256 Hasher in Idris Native

## Overview
This track implements a SHA-256 cryptographic hasher directly in Idris 2. This acts as a stress test for the compiler's performance, memory safety (zero leaks via QTT), and its ability to handle low-level systems programming tasks without relying on a C runtime.

## Functional Requirements
- **Bitwise Operations:** Add support to the AST and compiler for operations such as `Xor`, `And`, `Or`, `Not`, `ShiftLeft`, and `ShiftRight`.
- **Word Types:** Expand the primitive type system to support multiple word sizes, specifically `i32` and `i8` alongside `i64`.
- **Arrays/Buffers:** Introduce fixed-size array/buffer primitives to store message blocks (512-bit) and hash states (256-bit). These must be managed deterministically without a garbage collector, enforced by the QTT checker.
- **SHA-256 Algorithm:** Implement the core SHA-256 algorithm in Idris 2 source syntax, parsing it and compiling it to a native binary.

## Non-Functional Requirements
- **Zero-C Runtime:** The implementation must continue to rely strictly on pure LLVM and syscalls for output and memory manipulation, maintaining the "no C" constraint.
- **Performance:** The resulting SHA-256 implementation should have performance characteristics comparable to a hand-optimized C implementation, owing to the lack of GC and use of LLVM optimization passes.

## Acceptance Criteria
- [ ] Bitwise operators (`^`, `&`, `|`, `~`, `<<`, `>>`) and Word Types (`i32`, `i8`) are successfully parsed, lowered to LLVM IR, and executed.
- [ ] A fixed-size Array/Buffer primitive is added, allowing zero-GC indexed loads and stores.
- [ ] An Idris source file (`sha256.idr`) containing the SHA-256 algorithm compiles and executes.
- [ ] The generated binary correctly computes the SHA-256 hash of a test string (e.g., "hello world") and prints the resulting hex string without utilizing libc.