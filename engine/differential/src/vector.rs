//! Test-vector data model.
//!
//! A `TestVector` is the atomic unit of conformance: a fully-specified
//! study paired with the impact result every conforming engine should
//! produce for it, plus the tolerance class under which equality is
//! checked.

use arko_core::study::Study;
use serde::{Deserialize, Serialize};

/// Conformance level the vector belongs to (spec §14.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConformanceLevel {
    /// Static inventory + characterization. ≥1,000 vectors at v1.0.
    L1Basic,
    /// L1 + allocation + parameters. ≥5,000 vectors at v1.0.
    L2Full,
    /// L2 + uncertainty + incremental + determinism.
    L3Elite,
}

impl ConformanceLevel {
    /// Human-readable label for reports.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::L1Basic => "L1-basic",
            Self::L2Full => "L2-full",
            Self::L3Elite => "L3-elite",
        }
    }
}

/// Tolerance class applied to the vector (spec §8.1 / §8.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToleranceClass {
    /// `ε_abs = 1e-12`, `ε_rel = 1e-9` — for vectors derived from a
    /// **reference** implementation (Brightway 2.5 at pinned commit
    /// in v0.1). The tightest class.
    ReferenceParity,
    /// `ε_abs = 1e-9`, `ε_rel = 1e-6` — for cross-implementation
    /// vectors where the reference value came from a different engine
    /// (OpenLCA, SimaPro). Looser because of rounded method tables
    /// in those engines.
    CrossImpl,
}

impl ToleranceClass {
    #[must_use]
    pub const fn eps_abs(self) -> f64 {
        match self {
            Self::ReferenceParity => 1e-12,
            Self::CrossImpl => 1e-9,
        }
    }

    #[must_use]
    pub const fn eps_rel(self) -> f64 {
        match self {
            Self::ReferenceParity => 1e-9,
            Self::CrossImpl => 1e-6,
        }
    }

    /// Per-component tolerance `max(ε_abs, ε_rel * |want|)` from §8.1.
    #[must_use]
    pub fn tolerance_for(self, want: f64) -> f64 {
        f64::max(self.eps_abs(), self.eps_rel() * want.abs())
    }
}

/// One conformance test vector.
///
/// Not `PartialEq` — `Study` isn't, and comparing two vectors
/// structurally isn't meaningful (they're identified by `id`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestVector {
    /// Stable identifier — file name for on-disk vectors, code symbol
    /// for in-Rust seed vectors. Used as the `vector_id` in reports.
    pub id: String,
    /// Conformance level per §14.2.
    pub level: ConformanceLevel,
    /// One-line description surfaced in reports.
    pub description: String,
    /// The fully-specified study to solve.
    pub study: Study,
    /// Expected impact vector `h`. Length must equal
    /// `study.n_impacts()` (enforced by the runner).
    pub expected_h: Vec<f64>,
    /// Tolerance class for per-component parity checks.
    pub tolerance_class: ToleranceClass,
    /// Optional authorship / provenance note that rides along with
    /// the vector — useful when divergence investigations reach the
    /// original author years later.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tolerance_classes_match_spec() {
        assert_eq!(ToleranceClass::ReferenceParity.eps_abs(), 1e-12);
        assert_eq!(ToleranceClass::ReferenceParity.eps_rel(), 1e-9);
        assert_eq!(ToleranceClass::CrossImpl.eps_abs(), 1e-9);
        assert_eq!(ToleranceClass::CrossImpl.eps_rel(), 1e-6);
    }

    #[test]
    fn tolerance_for_uses_absolute_floor_on_small_values() {
        // |want| = 0 → the relative term is 0, so we fall back to the
        // absolute floor.
        let t = ToleranceClass::ReferenceParity.tolerance_for(0.0);
        assert_eq!(t, 1e-12);
    }

    #[test]
    fn tolerance_for_scales_with_large_values() {
        // |want| = 1000 → relative term dominates: 1e-9 * 1000 ≈ 1e-6.
        // (Exact floating-point: 1.0000000000000002e-6 — not bit-equal
        //  to 1e-6, which is expected.)
        let t = ToleranceClass::ReferenceParity.tolerance_for(1000.0);
        assert!((t - 1e-6).abs() < 1e-18);
    }

    #[test]
    fn levels_have_total_order() {
        assert!(ConformanceLevel::L1Basic < ConformanceLevel::L2Full);
        assert!(ConformanceLevel::L2Full < ConformanceLevel::L3Elite);
    }
}
