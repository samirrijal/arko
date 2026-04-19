//! Cross-surface parse tests — exercise the public `ParsedUnit::parse`
//! entry point on a variety of real-world LCA unit strings.
//!
//! The inline tests in `parser.rs` cover the grammar; this file covers
//! "does it recognize the strings LCA data actually contains."

use arko_units::{Dimension, ParsedUnit};

#[test]
fn all_reference_unit_strings_used_in_repo_parse() {
    // Every unit string that appears in tests / seed vectors today,
    // plus a few canonical neighbors.
    let cases = [
        "kg",
        "kg CO2-eq",
        "g",
        "t",
        "m",
        "m2",
        "m3",
        "L",
        "MJ",
        "kWh",
        "kg.km",
        "t.km",
        "mol",
        "mmol",
        "1",
        "s",
        "h",
        "d",
        "yr",
    ];
    for c in cases {
        ParsedUnit::parse(c).unwrap_or_else(|e| panic!("failed to parse `{c}`: {e}"));
    }
}

#[test]
fn tagged_lca_units_preserve_their_tags() {
    let cases = [
        ("kg CO2-eq", "CO2-eq"),
        ("kg CH4-eq", "CH4-eq"),
        ("kg N-eq", "N-eq"),
        ("kg P-eq", "P-eq"),
        ("kg SO2-eq", "SO2-eq"),
        ("kg PM2.5-eq", "PM2.5-eq"),
    ];
    for (input, expected_tag) in cases {
        let p = ParsedUnit::parse(input).unwrap();
        assert_eq!(
            p.tag.as_deref(),
            Some(expected_tag),
            "tag not preserved for `{input}`"
        );
        assert_eq!(p.dimension, Dimension::mass());
    }
}

#[test]
fn tag_with_internal_whitespace_is_collapsed() {
    // "kg CO2 eq" (no hyphen) — trailing tag with multiple spaces
    // gets whitespace-collapsed so ParsedUnit equality behaves.
    let p = ParsedUnit::parse("kg   CO2   eq").unwrap();
    assert_eq!(p.dimension, Dimension::mass());
    assert_eq!(p.tag.as_deref(), Some("CO2 eq"));
}

#[test]
fn whitespace_only_tag_is_absent() {
    let p = ParsedUnit::parse("kg   ").unwrap();
    assert_eq!(p.tag, None);
}

#[test]
fn exponent_variants_are_equivalent() {
    // m^3 == m3 — two spelling of the same dimension and scale.
    let a = ParsedUnit::parse("m^3").unwrap();
    let b = ParsedUnit::parse("m3").unwrap();
    assert_eq!(a.dimension, b.dimension);
    assert!((a.scale_to_si - b.scale_to_si).abs() < 1e-15);
}

#[test]
fn energy_chain_consistent_between_prefixed_atom_and_base() {
    let mj = ParsedUnit::parse("MJ").unwrap();
    let mwh = ParsedUnit::parse("MWh").unwrap();
    let j = ParsedUnit::parse("J").unwrap();
    let gj = ParsedUnit::parse("GJ").unwrap();
    assert_eq!(mj.dimension, j.dimension);
    assert_eq!(mwh.dimension, j.dimension);
    assert_eq!(gj.dimension, j.dimension);
    assert!((mj.scale_to_si - 1e6).abs() < 1e-3);
    assert!((gj.scale_to_si - 1e9).abs() < 1e-1);
    assert!((mwh.scale_to_si - 3.6e9).abs() < 1e-2);
}

#[test]
fn annum_alias_matches_yr() {
    let a = ParsedUnit::parse("a").unwrap();
    let yr = ParsedUnit::parse("yr").unwrap();
    assert_eq!(a.dimension, yr.dimension);
    assert!((a.scale_to_si - yr.scale_to_si).abs() < 1e-9);
}

#[test]
fn micrometer_vs_millimeter() {
    let um = ParsedUnit::parse("um").unwrap();
    let mm = ParsedUnit::parse("mm").unwrap();
    let m = ParsedUnit::parse("m").unwrap();
    assert_eq!(um.dimension, m.dimension);
    assert!((um.scale_to_si - 1e-6).abs() < 1e-12);
    assert!((mm.scale_to_si - 1e-3).abs() < 1e-9);
}

#[test]
fn source_string_is_trimmed() {
    let p = ParsedUnit::parse("   kg  ").unwrap();
    assert_eq!(p.source, "kg");
}

#[test]
fn parse_failure_exposes_offending_atom_in_error() {
    let err = ParsedUnit::parse("m.banana").unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("banana"),
        "error should mention the offending identifier: got `{msg}`"
    );
}
