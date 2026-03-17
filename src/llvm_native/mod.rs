//! # LLVM Native Backend (Infrastructure)
//!
//! This module implements the LLVM-based code generation backend 
//! for the Idris 2 compiler.
//!
//! # Strategic Architecture
//! `llvm_native` is an Infrastructure component that translates our 
//! internal representations into LLVM IR for native compilation. 
//! It depends on the `core_terms` (Entities) but remains decoupled 
//! from the higher-level elaboration logic.
//!
//! # Performance & Zero-GC
//! To achieve performance exceeding well-optimized C, this backend 
//! generates strictly GC-free native code, relying on the memory 
//! safety and resource bounds guaranteed by the QTT checker.
