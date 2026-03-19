# Phase 1 Summary: Identifier Sanitization & Unique Naming

## Task: Implement Identifier Sanitizer
- **Description:** Created a `sanitize_id` helper in `IRBuilder` that escapes special characters (e.g., `?` -> `_hole_`, `.` -> `_`) and wraps all Idris identifiers in LLVM-quoted format (`"..."`).
- **Why:** This prevents LLVM assembly errors when Idris names contain characters like dots or question marks, and ensures no collisions with LLVM reserved keywords.

## Task: Unique Placeholder Generation
- **Description:** Added a `pat_counter` to `IRBuilder` and implemented a `new_placeholder` method that generates unique names like `_pat_1`, `_pat_2`.
- **Why:** This prevents LLVM redefinition errors when multiple pattern-matching placeholders are used within the same scope.

## Task: Robust Global String Literals
- **Description:** Implemented `escape_string` in `IRBuilder` for LLVM-compatible hexadecimal escaping. Refined `string_literals` tracking to ensure deduplication and correct global definition emission in the backend.
- **Why:** LLVM IR requires non-printable characters and quotes to be escaped in string literals using a specific `\XX` hex format. Global strings must also be unique and shared across the module to save space and prevent symbol collisions.

## Task: Implement @concat and IO Stubs
- **Description:** Replaced the dummy `@concat` stub with a functional implementation using `malloc`, `memcpy`, and `strlen`. Updated the backend to only emit the `main` wrapper when necessary.
- **Why:** To support dynamic string concatenation (`++`) in compiled programs. The functional implementation ensures that new strings are correctly allocated and null-terminated.

## Task: Implement ADT Registry
- **Description:** Added a `type_env` to `IRBuilder` that stores `ConstructorLayout` (tag and field count) for each data constructor. Encountering `Term::Data` now populates this registry.
- **Why:** This registry is essential for lowering constructors to LLVM structs and for implementing tag-based pattern matching in subsequent phases.

## Task: Lower Constructors to Structs
- **Description:** Updated `IRBuilder` to lower constructor calls (`Term::App` with a constructor name) to LLVM struct allocations (`alloca`). The struct layout consists of an `i64` tag and an array of `i64` fields. Nullary constructors are lowered directly to their integer tag.
- **Why:** To represent Idris 2 data types in memory. Using LLVM structs allows for efficient storage and access of constructor data, enabling pattern matching.

## Task: Refactor Case Lowering to switch
- **Description:** Refactored `Term::Case` lowering to use the LLVM `switch` instruction instead of a chain of `icmp` and `br`. It correctly identifies the default branch (`_`) and generates unique labels for each case branch.
- **Why:** The `switch` instruction is more idiomatic for multi-way branches in LLVM and allows for better optimization by the LLVM backend (e.g., jump tables).

## Task: Implement Tag-Based Matching for ADTs
- **Description:** Extended `Term::Case` to support ADTs. If a constructor name is detected in the patterns, the backend generates IR to extract the `i64` tag from the struct pointer and switches on that tag value. It also correctly extracts fields from the struct and binds them to local variables within each branch.
- **Why:** This enables robust pattern matching on Idris 2 data types, allowing the compiler to correctly branch based on which constructor was used and access the data stored within those constructors.
