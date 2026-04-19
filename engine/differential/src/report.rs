//! Conformance report structure — spec §14.4.
//!
//! A `ConformanceReport` is the deliverable of a conformance run. It
//! is serializable to `conformance-report.json`, reviewable by hand,
//! and (critically) reproducible: given the same vector set + engine
//! binary, the same report must be produced bit-identically (§7.1).

use crate::vector::{ConformanceLevel, ToleranceClass};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Top-level report emitted by [`crate::run_conformance`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConformanceReport {
    /// Engine version (matches `Provenance.engine_version`).
    pub engine_version: String,
    /// Spec version the engine claims to implement.
    pub spec_version: String,
    /// Solver used for this run (e.g., `"nalgebra-dense-lu"`).
    pub solver_name: String,
    /// When the run started.
    pub started_at: DateTime<Utc>,
    /// Total duration of the run in milliseconds.
    pub total_ms: u64,

    // Summary counts.
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub errored: usize,

    /// Highest level at which **every** vector of that level passed.
    /// `None` if even L1 has failures.
    pub highest_level_passed: Option<ConformanceLevel>,

    /// Per-vector detail. One entry per input vector, in input order.
    pub per_vector: Vec<VectorResult>,
}

impl ConformanceReport {
    /// `true` iff `failed + errored == 0`.
    #[must_use]
    pub fn all_passed(&self) -> bool {
        self.failed == 0 && self.errored == 0
    }
}

/// Per-vector result included in a report.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VectorResult {
    pub vector_id: String,
    pub level: ConformanceLevel,
    pub tolerance_class: ToleranceClass,
    pub verdict: VectorVerdict,
}

/// Outcome for a single vector.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum VectorVerdict {
    /// Every component of `h` matched `expected_h` within tolerance.
    Pass {
        duration_us: u64,
        /// Worst observed `|got - want|` across components.
        max_abs_deviation: f64,
        /// Worst observed `|got - want| / |want|` (0 when `|want|` is 0).
        max_rel_deviation: f64,
    },

    /// Numeric parity failed at at least one component. Reports the
    /// *first* component that exceeded tolerance — investigation
    /// tooling can dig deeper from the raw data.
    Fail {
        index: usize,
        got: f64,
        want: f64,
        observed_deviation: f64,
        tolerance_applied: f64,
        duration_us: u64,
    },

    /// Shape mismatch between `expected_h.len()` and
    /// `study.n_impacts()`. Vector-authorship bug.
    ShapeMismatch {
        expected_impacts: usize,
        study_impacts: usize,
    },

    /// The engine returned an `EngineError` (e.g., E_SINGULAR).
    EngineError {
        code: String,
        message: String,
    },
}

impl VectorVerdict {
    #[must_use]
    pub const fn is_pass(&self) -> bool {
        matches!(self, Self::Pass { .. })
    }

    #[must_use]
    pub const fn is_fail(&self) -> bool {
        matches!(self, Self::Fail { .. } | Self::ShapeMismatch { .. })
    }

    #[must_use]
    pub const fn is_error(&self) -> bool {
        matches!(self, Self::EngineError { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verdict_classification_is_mutually_exclusive() {
        let pass = VectorVerdict::Pass {
            duration_us: 1,
            max_abs_deviation: 0.0,
            max_rel_deviation: 0.0,
        };
        assert!(pass.is_pass());
        assert!(!pass.is_fail());
        assert!(!pass.is_error());

        let fail = VectorVerdict::Fail {
            index: 0,
            got: 1.0,
            want: 2.0,
            observed_deviation: 1.0,
            tolerance_applied: 1e-9,
            duration_us: 1,
        };
        assert!(fail.is_fail());
        assert!(!fail.is_pass());

        let err = VectorVerdict::EngineError {
            code: "E_SINGULAR".into(),
            message: "x".into(),
        };
        assert!(err.is_error());
        assert!(!err.is_pass());
    }

    #[test]
    fn report_roundtrips_through_json() {
        let r = ConformanceReport {
            engine_version: "0.0.1+test".into(),
            spec_version: "0.1".into(),
            solver_name: "test".into(),
            started_at: chrono::Utc::now(),
            total_ms: 0,
            total: 1,
            passed: 1,
            failed: 0,
            errored: 0,
            highest_level_passed: Some(ConformanceLevel::L1Basic),
            per_vector: vec![VectorResult {
                vector_id: "v1".into(),
                level: ConformanceLevel::L1Basic,
                tolerance_class: ToleranceClass::ReferenceParity,
                verdict: VectorVerdict::Pass {
                    duration_us: 10,
                    max_abs_deviation: 0.0,
                    max_rel_deviation: 0.0,
                },
            }],
        };
        let j = serde_json::to_string(&r).unwrap();
        let back: ConformanceReport = serde_json::from_str(&j).unwrap();
        assert_eq!(r, back);
    }
}
