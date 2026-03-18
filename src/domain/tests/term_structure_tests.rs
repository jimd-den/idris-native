//! # Term Structure Tests
//!
//! These tests verify that the `Term` enum is correctly structured 
//! and that exhaustive matching covers all expected variants.

use crate::domain::Term;

#[test]
fn test_exhaustive_match_on_terms() {
    let term = Term::Integer(42);
    
    match &term {
        Term::Var(name) => assert_eq!(name, ""),
        Term::Lambda(name, _type, _body) => assert_eq!(name, ""),
        Term::App(_func, _arg) => (),
        Term::Pi(name, _mult, _type, _body) => assert_eq!(name, ""),
        Term::Integer(val) => assert_eq!(*val, 42),
        Term::IntegerType => (),
        Term::I32Type => (),
        Term::I8Type => (),
        Term::Bits64Type => (),
        Term::IOType => (),
        Term::Add(_l, _r) => (),
        Term::Sub(_l, _r) => (),
        Term::BitXor(_l, _r) => (),
        Term::BitAnd(_l, _r) => (),
        Term::BitOr(_l, _r) => (),
        Term::BitNot(_t) => (),
        Term::Shl(_l, _r) => (),
        Term::Shr(_l, _r) => (),
        Term::Eq(_l, _r) => (),
        Term::If(_c, _t, _f) => (),
        Term::LetRec(name, _val, _body) => assert_eq!(name, ""),
        Term::Let(name, _val, _body) => assert_eq!(name, ""),
        Term::Buffer(size) => assert_eq!(*size, 0),
        Term::BufferLoad(_b, _i) => (),
        Term::BufferStore(_b, _i, _v) => (),
        Term::Case(_t, _branches) => (),
    }
}
