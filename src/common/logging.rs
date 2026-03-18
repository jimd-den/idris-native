//! # Trace Logging Utility (Common Utility)
//!
//! This module provides the `trace_span!` macro and an RAII `LogSpan` 
//! guard for automated entry/exit logging across the compiler.
//!
//! # Strategic Architecture
//! By leveraging RAII (Resource Acquisition Is Initialization), we ensure 
//! that an "EXIT" log is always generated when a function returns, 
//! even in the presence of early returns or panics. This satisfies 
//! the DRY principle by eliminating redundant, copy-pasted log calls.
//!
//! # Literate Documentation
//! Tracing function execution is critical for debugging complex compiler 
//! pipelines. `trace_span!` automates this by creating a guard object 
//! that logs "ENTER" upon creation and "EXIT" upon destruction (Drop).

use crate::adapters::diagnostics;

/// An RAII guard that logs when it is created and when it is dropped.
pub struct LogSpan {
    tag: &'static str,
    name: &'static str,
}

impl LogSpan {
    /// Creates a new `LogSpan` and logs the "ENTER" message.
    pub fn new(tag: &'static str, name: &'static str) -> Self {
        diagnostics::log(tag, &format!("ENTER {}", name));
        Self { tag, name }
    }
}

impl Drop for LogSpan {
    /// Logs the "EXIT" message when the span goes out of scope.
    fn drop(&mut self) {
        diagnostics::log(self.tag, &format!("EXIT {}", self.name));
    }
}

/// A macro that creates a `LogSpan` for the current scope.
///
/// Example:
/// ```
/// use idris_native::trace_span;
/// {
///     let _span = trace_span!("PARSER", "parse_expr");
///     // ... logic ...
/// } // EXIT parse_expr is logged here.
/// ```
#[macro_export]
macro_rules! trace_span {
    ($tag:expr, $name:expr) => {
        $crate::common::logging::LogSpan::new($tag, $name)
    };
}
