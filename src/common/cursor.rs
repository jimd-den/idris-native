//! # Generic Cursor (Common Utility)
//!
//! This module provides a generic `Cursor<T>` for traversing sequences 
//! of items. It is used by both the `Scanner` and the `Parser` to 
//! maintain a sliding window over characters and tokens, respectively.
//!
//! # Strategic Architecture
//! By extracting this logic into a common utility, we satisfy the DRY (Don't 
//! Repeat Yourself) principle and ensure consistent traversal logic across 
//! the entire codebase.
//!
//! # Literate Documentation
//! A cursor maintains a current position within a sequence. It provides 
//! lookahead (peek) and conditional advancement (match), which are 
//! essential primitives for building robust recursive descent parsers.

/// A generic cursor for traversing a sequence of items.
pub struct Cursor<T> {
    items: Vec<T>,
    current: usize,
}

impl<T: PartialEq> Cursor<T> {
    /// Creates a new cursor for the given sequence of items.
    pub fn new(items: Vec<T>) -> Self {
        Self { items, current: 0 }
    }

    /// Returns the item at the current position without advancing.
    pub fn peek(&self) -> Option<&T> {
        self.items.get(self.current)
    }

    /// Returns the item after the current position without advancing.
    pub fn peek_next(&self) -> Option<&T> {
        self.items.get(self.current + 1)
    }

    /// Advances the cursor and returns the item that was just passed.
    pub fn advance(&mut self) -> Option<&T> {
        if !self.is_at_end() {
            let item = self.items.get(self.current);
            self.current += 1;
            item
        } else {
            None
        }
    }

    /// Checks if the cursor has reached the end of the sequence.
    pub fn is_at_end(&self) -> bool {
        self.current >= self.items.len()
    }

    /// Checks if the current item matches the expected item.
    pub fn check(&self, expected: &T) -> bool {
        if self.is_at_end() {
            return false;
        }
        self.peek() == Some(expected)
    }

    /// Advances the cursor if the current item matches the expected item.
    /// Returns `true` if it matched and advanced, `false` otherwise.
    pub fn match_item(&mut self, expected: &T) -> bool {
        if self.check(expected) {
            self.advance();
            true
        } else {
            false
        }
    }

    /// Returns the current index of the cursor.
    pub fn current_pos(&self) -> usize {
        self.current
    }
}
