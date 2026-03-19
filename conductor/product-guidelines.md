# Product Guidelines

## Communication Style
- **Verbose/Detailed**: Documentation, messages, and internal communications should be comprehensive and in-depth, providing full context for all technical decisions.
- **Literate Documentation**: Code should be treated as a whitepaper for stakeholders. Comments must explain the "why" and the underlying business/logic behind the code in plain English.

## User Experience & CLI Design
- **Hybrid Approach (Unix & Interactive)**: The CLI should adhere to the Unix philosophy (small, composable tools with standard outputs) while also offering modern, rich, and interactive features when appropriate for the best developer experience.
- **Idris-Compatible Errors**: The compiler must handle and display errors in a manner consistent with the Idris 2 language specification, ensuring familiarity for Idris developers.

## Development Principles
- **Clean Screaming Architecture**: Top-level modules reflect the four layers of Clean Architecture (`domain`, `application`, `adapters`, `infrastructure`, `drivers`, `common`). Dependencies point strictly inward.
- **Data-Oriented & Zero-Cost**: Entities must use data-oriented design (Struct-of-Arrays, Arena allocation) for cache locality. The backend (`infrastructure::llvm`) must be strictly zero-GC, leveraging QTT.
- **Technical Integrity**: All implementations must maintain high technical standards, balancing robustness with the KISS principle.
- **Literate Programming**: Every source file must contain extensive "Literate Programming" comments that explain the business/architectural intent in plain English for non-technical stakeholders.
