//! Conformance runner — turn a set of `TestVector`s + a `Solver` into
//! a `ConformanceReport`.

use crate::{
    report::{ConformanceReport, VectorResult, VectorVerdict},
    vector::{ConformanceLevel, TestVector},
};
use arko_core::{pipeline::compute, solver::Solver};
use chrono::Utc;
use std::time::Instant;

/// Configuration for a run.
#[derive(Debug, Clone)]
pub struct RunnerConfig {
    /// String recorded in `ConformanceReport.engine_version`. Callers
    /// typically pass `env!("CARGO_PKG_VERSION")` + a git SHA.
    pub engine_version: String,
    /// String recorded in `ConformanceReport.spec_version`. Callers
    /// typically pass `arko_core::SPEC_VERSION`.
    pub spec_version: String,
}

impl Default for RunnerConfig {
    fn default() -> Self {
        Self {
            engine_version: "0.0.1".to_string(),
            spec_version: arko_core::SPEC_VERSION.to_string(),
        }
    }
}

/// Drive `solver` through every vector in `vectors` and build a report.
pub fn run_conformance<S: Solver>(
    vectors: &[TestVector],
    solver: &S,
    config: &RunnerConfig,
) -> ConformanceReport {
    let started_at = Utc::now();
    let start = Instant::now();

    let per_vector: Vec<VectorResult> = vectors
        .iter()
        .map(|v| VectorResult {
            vector_id: v.id.clone(),
            level: v.level,
            tolerance_class: v.tolerance_class,
            verdict: run_single_vector(v, solver),
        })
        .collect();

    let total = per_vector.len();
    let passed = per_vector.iter().filter(|r| r.verdict.is_pass()).count();
    let failed = per_vector.iter().filter(|r| r.verdict.is_fail()).count();
    let errored = per_vector.iter().filter(|r| r.verdict.is_error()).count();

    let highest_level_passed = compute_highest_level(&per_vector);

    ConformanceReport {
        engine_version: config.engine_version.clone(),
        spec_version: config.spec_version.clone(),
        solver_name: solver.name().to_string(),
        started_at,
        total_ms: u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX),
        total,
        passed,
        failed,
        errored,
        highest_level_passed,
        per_vector,
    }
}

/// Run a single vector. Pure — no I/O.
#[must_use]
pub fn run_single_vector<S: Solver>(vector: &TestVector, solver: &S) -> VectorVerdict {
    // Guard: the expected impact vector must match the study's shape.
    let expected_len = vector.expected_h.len();
    let study_len = vector.study.n_impacts();
    if expected_len != study_len {
        return VectorVerdict::ShapeMismatch {
            expected_impacts: expected_len,
            study_impacts: study_len,
        };
    }

    let start = Instant::now();
    let computed = match compute(&vector.study, solver) {
        Ok(c) => c,
        Err(e) => {
            return VectorVerdict::EngineError {
                code: e.code().to_string(),
                message: e.to_string(),
            };
        }
    };
    let duration_us = u64::try_from(start.elapsed().as_micros()).unwrap_or(u64::MAX);

    let mut max_abs = 0.0_f64;
    let mut max_rel = 0.0_f64;

    for (i, (&got, &want)) in computed
        .impact
        .iter()
        .zip(vector.expected_h.iter())
        .enumerate()
    {
        let abs_dev = (got - want).abs();
        let rel_dev = if want.abs() > f64::EPSILON {
            abs_dev / want.abs()
        } else {
            0.0
        };
        max_abs = f64::max(max_abs, abs_dev);
        max_rel = f64::max(max_rel, rel_dev);

        let tol = vector.tolerance_class.tolerance_for(want);
        if abs_dev > tol {
            return VectorVerdict::Fail {
                index: i,
                got,
                want,
                observed_deviation: abs_dev,
                tolerance_applied: tol,
                duration_us,
            };
        }
    }

    VectorVerdict::Pass {
        duration_us,
        max_abs_deviation: max_abs,
        max_rel_deviation: max_rel,
    }
}

/// Highest level at which every vector of that level (and all lower
/// levels) passed.
fn compute_highest_level(per_vector: &[VectorResult]) -> Option<ConformanceLevel> {
    let levels = [
        ConformanceLevel::L1Basic,
        ConformanceLevel::L2Full,
        ConformanceLevel::L3Elite,
    ];

    let mut highest: Option<ConformanceLevel> = None;
    for &lvl in &levels {
        let relevant: Vec<&VectorResult> = per_vector.iter().filter(|r| r.level <= lvl).collect();
        // If there are no vectors at or below this level, skip.
        if relevant.is_empty() {
            continue;
        }
        // A level is only claimed if at least one vector exists AT that
        // exact level — otherwise an L1+L2-only run would silently earn
        // an L3Elite badge on the strength of absent L3 coverage.
        let has_vector_at_this_level = per_vector.iter().any(|r| r.level == lvl);
        if !has_vector_at_this_level {
            break;
        }
        if relevant.iter().all(|r| r.verdict.is_pass()) {
            highest = Some(lvl);
        } else {
            break;
        }
    }
    highest
}

#[cfg(test)]
mod tests {
    use super::*;

    // Runner-level tests that don't require a real solver live here.
    // Full end-to-end coverage lives in tests/conformance_tests.rs.

    #[test]
    fn runner_config_default_uses_spec_version_const() {
        let c = RunnerConfig::default();
        assert_eq!(c.spec_version, arko_core::SPEC_VERSION);
    }

    #[test]
    fn highest_level_none_when_no_vectors() {
        assert!(compute_highest_level(&[]).is_none());
    }

    #[test]
    fn highest_level_l2_when_l1_and_l2_pass_but_no_l3_vectors() {
        let vr = |level, pass: bool| VectorResult {
            vector_id: "x".into(),
            level,
            tolerance_class: crate::vector::ToleranceClass::ReferenceParity,
            verdict: if pass {
                VectorVerdict::Pass {
                    duration_us: 0,
                    max_abs_deviation: 0.0,
                    max_rel_deviation: 0.0,
                }
            } else {
                VectorVerdict::EngineError {
                    code: "x".into(),
                    message: "x".into(),
                }
            },
        };
        let results = vec![
            vr(ConformanceLevel::L1Basic, true),
            vr(ConformanceLevel::L2Full, true),
        ];
        assert_eq!(
            compute_highest_level(&results),
            Some(ConformanceLevel::L2Full)
        );
    }

    #[test]
    fn highest_level_none_when_l1_fails() {
        let vr = VectorResult {
            vector_id: "x".into(),
            level: ConformanceLevel::L1Basic,
            tolerance_class: crate::vector::ToleranceClass::ReferenceParity,
            verdict: VectorVerdict::EngineError {
                code: "E_SINGULAR".into(),
                message: "x".into(),
            },
        };
        assert_eq!(compute_highest_level(&[vr]), None);
    }
}
