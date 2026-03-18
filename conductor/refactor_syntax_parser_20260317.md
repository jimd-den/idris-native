# Refactoring Plan: Syntax Parser Decomposition

## Objective
Decompose the monolithic and fragile `Parser` "god class" into specialized, decoupled components to improve maintainability, testability, and reliability.

## Structural Violations Identified
1. **Monolithic Responsibility**: The `Parser` is currently handling lexical analysis (implied), AST construction, and program-level orchestration.
2. **Fragile State**: Manual token indexing and unwrap-heavy logic make it prone to panics and infinite loops.
3. **God Class Pattern**: One single implementation handles all Idris 2 syntax rules, making it difficult to extend without side effects.

## Proposed Components

### 1. `Scanner` (Entities/Infrastructure)
- **Role**: Pure lexical analysis.
- **Responsibility**: Converts a stream of characters into a robust `Token` stream (not just `String`s).
- **Benefit**: Removes string comparison overhead from the parser and provides better error location tracking.

### 2. `Parser` (Adapter)
- **Role**: Recursive Descent AST construction.
- **Responsibility**: Translates `Token` stream into `Term` AST.
- **Sub-components**:
    - `ExpressionParser`: Handles operators, precedence, and primary terms.
    - `TypeParser`: Specializes in type signatures and Pi-binders.
    - `DeclParser`: Handles top-level declarations (data, function definitions).

### 3. `ProgramParser` (Use Case/Adapter)
- **Role**: High-level orchestration.
- **Responsibility**: Maps signatures to definitions and ensures logical program flow.

## Implementation Steps

### Step 1: Define `Token` and `Scanner`
- Create `src/syntax_parser/scanner.rs`.
- Implement `Token` enum (Identifier, Keyword, Operator, Literal).

### Step 2: Implement Decomposed `Parser`
- Create `src/syntax_parser/parser.rs`.
- Build specialized methods for expression levels (Comparison, Bitwise, Arithmetic, etc.).

### Step 3: Implement `ProgramBuilder`
- Ensure signatures and definitions are correctly paired.

## Verification
- Port all existing `sha256_syntax_tests.rs` to the new architecture.
- Verify end-to-end compilation of `ackermann.idr` and `sha256.idr`.
