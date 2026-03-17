//! # REPL Session (Adapter)
//!
//! This module manages the state and logic for the interactive 
//! Idris Native REPL.
//!
//! # Strategic Architecture
//! The `repl_session` is an Adapter that handles user input and 
//! orchestrates the `compiler` use-case to provide a rich developer 
//! experience.
//!
//! # Performance
//! The REPL must be lightweight and responsive, adhering to the 
//! same zero-GC principles as the core compiler to minimize startup 
//! and execution latency.
