//! # Common Utilities (Frameworks & Drivers)
//!
//! This module contains shared utilities and helper functions that 
//! are used across multiple layers of the compiler.

pub mod cursor;
pub mod logging;
pub mod test_helpers;
pub mod errors;

#[cfg(test)]
mod tests {
    pub mod cursor_tests;
    pub mod logging_tests;
    pub mod helper_tests;
}
