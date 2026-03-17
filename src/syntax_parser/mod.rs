//! # Syntax Parser (Adapter)
//!
//! This module implements the Idris 2 syntax parser, translating 
//! source code text into our internal `core_terms` (Entities).
//!
//! # Strategic Architecture
//! As an Adapter, the `syntax_parser` bridges the external world 
//! (source files) with our domain logic. It depends on `core_terms` 
//! but remains decoupled from the specific implementation details 
//! of the QTT checker.
//!
//! # Performance
//! To beat C performance, the parser must be extremely fast and 
//! avoid unnecessary allocations, leveraging the `Arena` from 
//! `core_terms` for the generated AST nodes.
