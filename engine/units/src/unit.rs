//! `ParsedUnit` — the canonical analyzed form of a unit string,
//! plus the conversion / commensurability API callers actually use.

use crate::{
    dimension::Dimension,
    parser::{parse_expression, UnitParseError},
};
use serde::{Deserialize, Serialize};

/// A parsed, dimension-annotated unit ready for compatibility checks
/// and conversions.
///
/// Invariant: `scale_to_si > 0`. A ParsedUnit with `scale_to_si == 0`
/// is never produced (every atom in `atom.rs` has positive scale).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParsedUnit {
    /// Original input, trimmed and whitespace-normalized.
    pub source: String,
    /// 7D SI-base-dimension exponent vector.
    pub dimension: Dimension,
    /// Multiplier to convert a value in this unit to the SI base unit
    /// for `dimension` (kg for mass, m for length, J for energy, …).
    pub scale_to_si: f64,
    /// Optional semantic tag. `kg CO2-eq` and `kg` share `dimension`
    /// but have different tags; they are **not** interconvertible.
    pub tag: Option<String>,
}

/// Higher-level errors exposed by the unit layer.
#[derive(Debug, thiserror::Error, Clone, PartialEq)]
pub enum UnitError {
    #[error(transparent)]
    Parse(#[from] UnitParseError),

    #[error(
        "incompatible dimensions: `{from}` has dimension `{from_dim}`, \
         `{to}` has dimension `{to_dim}`"
    )]
    IncompatibleDimension {
        from: String,
        to: String,
        from_dim: String,
        to_dim: String,
    },

    #[error(
        "incompatible semantic tags: `{from}` tag = {from_tag:?}, \
         `{to}` tag = {to_tag:?}; conversion requires a characterization factor"
    )]
    IncompatibleTag {
        from: String,
        to: String,
        from_tag: Option<String>,
        to_tag: Option<String>,
    },
}

impl ParsedUnit {
    /// Parse a unit string. Accepts UCUM-subset syntax (see `parser`).
    pub fn parse(s: &str) -> Result<Self, UnitError> {
        let expr = parse_expression(s)?;
        Ok(Self {
            source: s.trim().to_string(),
            dimension: expr.dimension,
            scale_to_si: expr.scale,
            tag: expr.tag,
        })
    }

    /// The dimensionless unit `1`.
    #[must_use]
    pub fn dimensionless() -> Self {
        Self {
            source: "1".to_string(),
            dimension: Dimension::DIMENSIONLESS,
            scale_to_si: 1.0,
            tag: None,
        }
    }

    /// `true` iff this unit has no SI-base dimension and no tag.
    #[must_use]
    pub fn is_dimensionless(&self) -> bool {
        self.dimension.is_dimensionless() && self.tag.is_none()
    }
}

/// Are two units interconvertible by a scalar multiply?
///
/// Returns `true` iff both the 7D dimension and the semantic tag match
/// exactly. Tag matching is case-sensitive string equality on the
/// parser-normalized form.
#[must_use]
pub fn commensurable(a: &ParsedUnit, b: &ParsedUnit) -> bool {
    a.dimension == b.dimension && a.tag == b.tag
}

/// Conversion factor `v_b = factor · v_a` where `v_a` is in unit `a`
/// and `v_b` is in unit `b`. Returns `Err` if the units are not
/// commensurable (different dimensions, or same dimension but
/// different tags).
pub fn conversion_factor(a: &ParsedUnit, b: &ParsedUnit) -> Result<f64, UnitError> {
    if a.dimension != b.dimension {
        return Err(UnitError::IncompatibleDimension {
            from: a.source.clone(),
            to: b.source.clone(),
            from_dim: a.dimension.to_string(),
            to_dim: b.dimension.to_string(),
        });
    }
    if a.tag != b.tag {
        return Err(UnitError::IncompatibleTag {
            from: a.source.clone(),
            to: b.source.clone(),
            from_tag: a.tag.clone(),
            to_tag: b.tag.clone(),
        });
    }
    // Both scale to the same SI base unit; factor = a_scale / b_scale.
    Ok(a.scale_to_si / b.scale_to_si)
}

/// Convert a scalar value from unit `from` to unit `to`. Composition
/// of `conversion_factor` + multiplication — provided as a convenience
/// since callers rarely want the raw factor.
pub fn convert(value: f64, from: &ParsedUnit, to: &ParsedUnit) -> Result<f64, UnitError> {
    Ok(conversion_factor(from, to)? * value)
}

/// Convenience: parse both sides and then convert.
pub fn convert_str(value: f64, from: &str, to: &str) -> Result<f64, UnitError> {
    let a = ParsedUnit::parse(from)?;
    let b = ParsedUnit::parse(to)?;
    convert(value, &a, &b)
}

/// Core-facing helper: validate that two `arko_core::Unit` strings
/// are commensurable, returning `Ok(())` on success. This is the
/// hook `arko-validation` uses for the §6.1 step-3 consistency check.
pub fn check_compatibility(a: &arko_core::Unit, b: &arko_core::Unit) -> Result<(), UnitError> {
    let pa = ParsedUnit::parse(a.as_str())?;
    let pb = ParsedUnit::parse(b.as_str())?;
    if commensurable(&pa, &pb) {
        Ok(())
    } else if pa.dimension != pb.dimension {
        Err(UnitError::IncompatibleDimension {
            from: pa.source,
            to: pb.source,
            from_dim: pa.dimension.to_string(),
            to_dim: pb.dimension.to_string(),
        })
    } else {
        Err(UnitError::IncompatibleTag {
            from: pa.source,
            to: pb.source,
            from_tag: pa.tag,
            to_tag: pb.tag,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kg_to_g_factor_is_1000() {
        let a = ParsedUnit::parse("kg").unwrap();
        let b = ParsedUnit::parse("g").unwrap();
        let f = conversion_factor(&a, &b).unwrap();
        assert!((f - 1000.0).abs() < 1e-9);
    }

    #[test]
    fn g_to_kg_factor_is_0_001() {
        let a = ParsedUnit::parse("g").unwrap();
        let b = ParsedUnit::parse("kg").unwrap();
        let f = conversion_factor(&a, &b).unwrap();
        assert!((f - 1e-3).abs() < 1e-15);
    }

    #[test]
    fn kwh_to_mj_is_3_6() {
        let v = convert_str(1.0, "kWh", "MJ").unwrap();
        assert!((v - 3.6).abs() < 1e-12);
    }

    #[test]
    fn mj_to_kwh_is_one_over_3_6() {
        let v = convert_str(1.0, "MJ", "kWh").unwrap();
        assert!((v - (1.0 / 3.6)).abs() < 1e-12);
    }

    #[test]
    fn kg_vs_lbs_does_not_exist_yet() {
        // Pound isn't in the atom table. That's a deliberate v0.0.1
        // scope — the API surfaces it as an UnknownAtom parse error.
        let err = ParsedUnit::parse("lb").unwrap_err();
        assert!(matches!(err, UnitError::Parse(_)));
    }

    #[test]
    fn kg_to_m_is_incompatible_dim() {
        let a = ParsedUnit::parse("kg").unwrap();
        let b = ParsedUnit::parse("m").unwrap();
        let err = conversion_factor(&a, &b).unwrap_err();
        assert!(matches!(err, UnitError::IncompatibleDimension { .. }));
    }

    #[test]
    fn kg_to_kg_co2eq_is_tag_mismatch() {
        let a = ParsedUnit::parse("kg").unwrap();
        let b = ParsedUnit::parse("kg CO2-eq").unwrap();
        let err = conversion_factor(&a, &b).unwrap_err();
        assert!(matches!(err, UnitError::IncompatibleTag { .. }));
    }

    #[test]
    fn commensurable_reflexive_and_symmetric() {
        let a = ParsedUnit::parse("kWh").unwrap();
        let b = ParsedUnit::parse("MJ").unwrap();
        assert!(commensurable(&a, &a));
        assert!(commensurable(&a, &b));
        assert!(commensurable(&b, &a));
    }

    #[test]
    fn check_compatibility_with_core_unit_passes_for_equivalent_units() {
        let u1 = arko_core::Unit::new("kg");
        let u2 = arko_core::Unit::new("g");
        check_compatibility(&u1, &u2).unwrap();
    }

    #[test]
    fn check_compatibility_with_core_unit_fails_across_dimensions() {
        let u1 = arko_core::Unit::new("kg");
        let u2 = arko_core::Unit::new("m");
        let err = check_compatibility(&u1, &u2).unwrap_err();
        assert!(matches!(err, UnitError::IncompatibleDimension { .. }));
    }

    #[test]
    fn check_compatibility_with_core_unit_fails_on_tag_mismatch() {
        let u1 = arko_core::Unit::new("kg");
        let u2 = arko_core::Unit::new("kg CO2-eq");
        let err = check_compatibility(&u1, &u2).unwrap_err();
        assert!(matches!(err, UnitError::IncompatibleTag { .. }));
    }

    #[test]
    fn dimensionless_roundtrip_is_identity() {
        let a = ParsedUnit::parse("1").unwrap();
        assert!(a.is_dimensionless());
        assert!((convert(7.5, &a, &a).unwrap() - 7.5).abs() < 1e-15);
    }

    #[test]
    fn parsed_unit_json_roundtrip() {
        let a = ParsedUnit::parse("kg CO2-eq").unwrap();
        let j = serde_json::to_string(&a).unwrap();
        let b: ParsedUnit = serde_json::from_str(&j).unwrap();
        assert_eq!(a, b);
    }
}
