//! # Term Structure Tests
//!
//! These tests verify that the `Term` enum is correctly structured 
//! and that exhaustive matching covers all expected variants.

use crate::domain::{Term, multiplicity::Multiplicity};

#[test]
fn test_exhaustive_match_on_terms() {
    let term = Term::Integer(42);
    
    match &term {
        Term::Var(name) => assert_eq!(name, ""),
        Term::Lambda(name, _type, _body) => assert_eq!(name, ""),
        Term::App(_func, _arg) => (),
        Term::Pi(name, _mult, _type, _body) => assert_eq!(name, ""),
        Term::Universe(n) => assert_eq!(*n, 0),
        
        Term::Integer(val) => assert_eq!(*val, 42),
        Term::Float(val) => assert_eq!(*val, 0),
        Term::String(val) => assert_eq!(val, ""),
        Term::Char(val) => assert_eq!(*val, ' '),
        
        Term::IntegerType => (),
        Term::FloatType => (),
        Term::StringType => (),
        Term::CharType => (),
        Term::I32Type => (),
        Term::I8Type => (),
        Term::Bits64Type => (),
        Term::IOType => (),
        Term::TypeType => (),
        
        Term::Add(_l, _r) | Term::Sub(_l, _r) | Term::Mul(_l, _r) | Term::Div(_l, _r) | Term::Append(_l, _r) => (),
        Term::BitXor(_l, _r) | Term::BitAnd(_l, _r) | Term::BitOr(_l, _r) => (),
        Term::BitNot(_t) => (),
        Term::Shl(_l, _r) | Term::Shr(_l, _r) => (),
        Term::Eq(_l, _r) | Term::Lt(_l, _r) | Term::Gt(_l, _r) => (),
        
        Term::If(_c, _t, _f) => (),
        Term::Let(name, _val, _body) => assert_eq!(name, ""),
        Term::LetRec(name, _val, _body) => assert_eq!(name, ""),
        Term::Case(_t, _branches) => (),
        
        Term::Do(_stmts) => (),
        Term::Bind(name, _action) => assert_eq!(name, ""),
        
        Term::Module(name) => assert_eq!(name, ""),
        Term::Import(name) => assert_eq!(name, ""),
        Term::Data(name, _params, _constrs) => assert_eq!(name, ""),
        Term::Interface(name, _params, _methods) => assert_eq!(name, ""),
        Term::Implementation(iface, name, _impls) => {
            assert_eq!(iface, "");
            assert_eq!(name, "");
        },
        Term::Record(name, _fields) => assert_eq!(name, ""),
        
        Term::Mutual(_terms) => (),
        Term::Where(_term, _defs) => (),
        Term::Def(name, _args, _body) => assert_eq!(name, ""),
        
        Term::Buffer(size) => assert_eq!(*size, 0),
        Term::BufferLoad(_b, _i) => (),
        Term::BufferStore(_b, _i, _v) => (),
    }
}
