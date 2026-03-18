//! # Cursor Logic Tests
//!
//! These tests verify that the generic `Cursor<T>` correctly handles 
//! sliding window operations over a sequence of items of type `T`.

use crate::common::cursor::Cursor;

#[test]
fn test_cursor_basic_navigation() {
    let items = vec![1, 2, 3];
    let mut cursor = Cursor::new(items);
    
    assert_eq!(cursor.peek(), Some(&1));
    assert_eq!(cursor.advance(), Some(&1));
    assert_eq!(cursor.peek(), Some(&2));
    assert_eq!(cursor.advance(), Some(&2));
    assert_eq!(cursor.peek(), Some(&3));
    assert_eq!(cursor.advance(), Some(&3));
    assert_eq!(cursor.peek(), None);
    assert!(cursor.is_at_end());
}

#[test]
fn test_cursor_check_and_match() {
    let items = vec!['a', 'b', 'c'];
    let mut cursor = Cursor::new(items);
    
    assert!(cursor.check(&'a'));
    assert!(!cursor.check(&'b'));
    
    assert!(cursor.match_item(&'a')); // Should advance
    assert_eq!(cursor.peek(), Some(&'b'));
    
    assert!(!cursor.match_item(&'z')); // Should not advance
    assert_eq!(cursor.peek(), Some(&'b'));
}

#[test]
fn test_cursor_peek_next() {
    let items = vec![10, 20, 30];
    let cursor = Cursor::new(items);
    
    assert_eq!(cursor.peek(), Some(&10));
    assert_eq!(cursor.peek_next(), Some(&20));
}
