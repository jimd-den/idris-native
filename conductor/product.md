# Initial Concept
idris2 native llvm compiler using clean screaming architecture

# Product Definition

## Vision
To provide a fast, natively-compiled Idris 2 compiler and runtime environment that leverages LLVM to generate highly efficient machine code, enabling advanced functional programming with dependent types for systems-level development.

## Target Users
- **Functional Programmers**: Seeking Idris's power with native performance.
- **Compiler Developers**: Exploring dependent-type system implementations.
- **Researchers/Theorists**: Using Idris for formal verification and proof assistants.

## Primary Goals
- **Performance/Portability**: Delivering a fast, dependency-minimized native binary.
- **Systems Programming**: Enabling Idris for low-level software where performance is critical.
- **Developer Productivity**: Enhancing the compiler's speed and providing clear error messages.

## Key Features
- **Native REPL**: A lightweight, responsive interactive environment for Idris development.
- **Native Backends**: Direct generation of native code via LLVM for maximum efficiency.
- **Clean Architecture**: Adopting a screaming architecture in Rust to ensure the compiler is maintainable and robust.

## Constraints & Principles
- **Code Purity (Rust)**: Leveraging Rust's safety and performance while maintaining clean, idiomatic code.
- **Idris 2 Compatibility**: Ensuring the compiler adheres to the Idris 2 language specification.
- **Resource Efficiency**: Aiming for minimal memory usage and rapid startup times.
