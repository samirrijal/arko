//! Arko unit parser and dimensional analyzer.
//!
//! This crate lifts `arko_core::Unit` — a bare string — into a
//! structured `ParsedUnit` with full SI-dimension analysis, scale-to-
//! base conversion, and semantic-tag tracking (so that `kg` and
//! `kg CO2-eq` do not silently compare equal).
//!
//! # Scope at v0.0.1
//!
//! - The seven SI base dimensions + standard derived atoms that
//!   appear in real LCA datasets: N, Pa, J, W, Wh, Hz, L, bar, t, h,
//!   min, d, yr.
//! - All standard SI prefixes, including unicode `μ` and the ASCII
//!   `u` variant.
//! - UCUM-subset expression grammar: `.`/`*` multiplication, `/`
//!   division (left-associative per UCUM), `^` exponentiation plus
//!   the `m2` implicit-positive exponent form.
//! - Semantic-tag parsing: anything after the first ASCII space is
//!   preserved and used to gate conversions.
//!
//! # Not covered at v0.0.1 (→ v0.2)
//!
//! - Affine conversions (°C ↔ K).
//! - UCUM curly-brace annotations like `kg{dry}`.
//! - Non-metric units outside the LCA domain (lb, ft, BTU, therm).
//! - Dimensional analysis across parameterized expressions — that
//!   belongs in `arko-parameters`, not here.

pub mod atom;
pub mod dimension;
pub mod parser;
pub mod unit;

pub use atom::{all_atoms, all_prefixes, find_atom, find_prefix, Atom, Prefix};
pub use dimension::Dimension;
pub use parser::{parse_expression, ParsedExpression, UnitParseError};
pub use unit::{
    check_compatibility, commensurable, conversion_factor, convert, convert_str, ParsedUnit,
    UnitError,
};
