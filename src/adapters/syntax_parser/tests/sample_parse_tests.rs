//! # Sample Parse Smoke Tests (TDD Red Phase)
//!
//! These tests verify that every sample `.idr` file in `idris2_ref/samples/`
//! can be parsed without error. This is the acceptance criterion for the
//! parser extension work.
//!
//! # Test-First TDD
//! We start with these tests failing (RED), then extend the scanner and
//! parser until they all pass (GREEN).

use crate::adapters::syntax_parser::{lex, Parser};
use crate::domain::arena::Arena;

/// Helper: reads a sample file and attempts to parse it.
/// Returns Ok(declaration_count) or the error.
fn parse_sample(filename: &str) -> Result<usize, String> {
    let path = format!("idris2_ref/samples/{}", filename);
    let source = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read {}: {}", path, e))?;
    let tokens = lex(&source)
        .map_err(|e| format!("Lex error in {}: {:?}", filename, e))?;
    let mut arena = Arena::new();
    let mut parser = Parser::new(tokens, &mut arena);
    let decls = parser.parse_program()
        .map_err(|e| format!("Parse error in {}: {:?}", filename, e))?;
    Ok(decls.len())
}

// ── Tier 1: Simple files that should already work ──

#[test]
fn test_parse_sample_prims() {
    let count = parse_sample("Prims.idr").expect("Prims.idr should parse");
    assert!(count > 0, "Expected at least one declaration");
}

// ── Tier 2: Files needing minor extensions ──

#[test]
fn test_parse_sample_io() {
    let count = parse_sample("io.idr").expect("io.idr should parse");
    assert!(count > 0);
}

#[test]
fn test_parse_sample_bmain() {
    let count = parse_sample("bmain.idr").expect("bmain.idr should parse");
    assert!(count > 0);
}

#[test]
fn test_parse_sample_holes() {
    let count = parse_sample("holes.idr").expect("holes.idr should parse");
    assert!(count > 0);
}

// ── Tier 3: Files needing where clauses, patterns, etc. ──

#[test]
fn test_parse_sample_wheres() {
    let count = parse_sample("wheres.idr").expect("wheres.idr should parse");
    assert!(count > 0);
}

#[test]
fn test_parse_sample_void() {
    let count = parse_sample("Void.idr").expect("Void.idr should parse");
    assert!(count > 0);
}

#[test]
fn test_parse_sample_interp_e() {
    let count = parse_sample("InterpE.idr").expect("InterpE.idr should parse");
    assert!(count > 0);
}

#[test]
fn test_parse_sample_fctypes() {
    let count = parse_sample("fctypes.idr").expect("fctypes.idr should parse");
    assert!(count > 0);
}

// ── Tier 4: Files needing GADT, with views, records, etc. ──

#[test]
fn test_parse_sample_vect() {
    let count = parse_sample("Vect.idr").expect("Vect.idr should parse");
    assert!(count > 0);
}

#[test]
fn test_parse_sample_btree() {
    let count = parse_sample("BTree.idr").expect("BTree.idr should parse");
    assert!(count > 0);
}

#[test]
fn test_parse_sample_with() {
    let count = parse_sample("With.idr").expect("With.idr should parse");
    assert!(count > 0);
}

#[test]
fn test_parse_sample_interp() {
    let count = parse_sample("Interp.idr").expect("Interp.idr should parse");
    assert!(count > 0);
}

#[test]
fn test_parse_sample_proofs() {
    let count = parse_sample("Proofs.idr").expect("Proofs.idr should parse");
    assert!(count > 0);
}

#[test]
fn test_parse_sample_deprec() {
    let count = parse_sample("deprec.idr").expect("deprec.idr should parse");
    assert!(count > 0);
}

#[test]
fn test_parse_sample_multiplicity() {
    let count = parse_sample("multiplicity.idr").expect("multiplicity.idr should parse");
    assert!(count > 0);
}

#[test]
fn test_parse_sample_params() {
    let count = parse_sample("params.idr").expect("params.idr should parse");
    assert!(count > 0);
}

#[test]
fn test_parse_sample_listcomp() {
    let count = parse_sample("listcomp.idr").expect("listcomp.idr should parse");
    assert!(count > 0);
}

#[test]
fn test_parse_sample_my_ord() {
    let count = parse_sample("MyOrd.idr").expect("MyOrd.idr should parse");
    assert!(count > 0);
}

#[test]
fn test_parse_sample_named_semi() {
    let count = parse_sample("NamedSemi.idr").expect("NamedSemi.idr should parse");
    assert!(count > 0);
}
