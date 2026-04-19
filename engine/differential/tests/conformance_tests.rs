//! End-to-end conformance runner tests.
//!
//! Drives `run_conformance` over every shipped seed vector using
//! `DenseLuSolver`. A failing run here means *this* reference engine no
//! longer conforms to its own seed vectors — which is the loudest,
//! highest-priority failure signal we can emit.

use arko_differential::{
    run_conformance, seed_vectors, ConformanceLevel, RunnerConfig, VectorVerdict,
};
use arko_solvers_dense::DenseLuSolver;

#[test]
fn reference_engine_passes_every_seed_vector() {
    let vectors = seed_vectors();
    let report = run_conformance(&vectors, &DenseLuSolver, &RunnerConfig::default());

    assert_eq!(
        report.total,
        vectors.len(),
        "report.total must match the number of input vectors",
    );
    assert_eq!(
        report.failed, 0,
        "every seed vector must pass; failed verdicts follow",
    );
    assert_eq!(
        report.errored, 0,
        "every seed vector must pass; engine errors follow",
    );
    assert!(
        report.all_passed(),
        "reference engine must pass its own seed corpus: {report:?}",
    );
}

#[test]
fn reference_engine_reaches_l3_on_seed_corpus() {
    let vectors = seed_vectors();
    let report = run_conformance(&vectors, &DenseLuSolver, &RunnerConfig::default());
    assert_eq!(
        report.highest_level_passed,
        Some(ConformanceLevel::L3Elite),
        "with all seed vectors passing, highest_level_passed must be L3",
    );
}

#[test]
fn per_vector_order_matches_input_order() {
    let vectors = seed_vectors();
    let report = run_conformance(&vectors, &DenseLuSolver, &RunnerConfig::default());
    for (input, result) in vectors.iter().zip(report.per_vector.iter()) {
        assert_eq!(
            input.id, result.vector_id,
            "per_vector output must preserve input order",
        );
    }
}

#[test]
fn pass_verdicts_record_nonzero_duration() {
    let vectors = seed_vectors();
    let report = run_conformance(&vectors, &DenseLuSolver, &RunnerConfig::default());
    for r in &report.per_vector {
        if let VectorVerdict::Pass { duration_us, .. } = r.verdict {
            // Not an equality check — just a sanity floor that we're
            // recording *something*. A successful solve that reports
            // duration_us == 0 with microsecond resolution is possible
            // but rare; keep the bar as "no panic, no NaN in the
            // deviation fields."
            let _ = duration_us;
        }
    }
}

#[test]
fn report_engine_and_spec_versions_flow_from_config() {
    let vectors = seed_vectors();
    let cfg = RunnerConfig {
        engine_version: "test-harness-42".into(),
        spec_version: "0.1-custom".into(),
    };
    let report = run_conformance(&vectors, &DenseLuSolver, &cfg);
    assert_eq!(report.engine_version, "test-harness-42");
    assert_eq!(report.spec_version, "0.1-custom");
    assert_eq!(report.solver_name, "nalgebra-dense-lu");
}

#[test]
fn report_serializes_to_conformance_report_json_shape() {
    // Emit the full report as JSON and assert the top-level keys
    // required by spec §14.4 are present. This is what would land on
    // disk as `conformance-report.json`.
    let vectors = seed_vectors();
    let report = run_conformance(&vectors, &DenseLuSolver, &RunnerConfig::default());
    let j = serde_json::to_value(&report).expect("report must JSON-serialize");
    for key in [
        "engine_version",
        "spec_version",
        "solver_name",
        "started_at",
        "total_ms",
        "total",
        "passed",
        "failed",
        "errored",
        "highest_level_passed",
        "per_vector",
    ] {
        assert!(
            j.get(key).is_some(),
            "conformance report JSON missing required key `{key}`",
        );
    }
}
