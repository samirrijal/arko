//! Arko conformance harness — see `specs/calc/v0.1.md` §14.
//!
//! This crate owns the machinery that lets **any** Arko engine build —
//! first-party or third-party — demonstrate it conforms to the calc
//! specification. The spec calls for ≥10,000 test vectors by v1.0; we
//! ship v0.0.1 with the *framework* plus a handful of seed vectors that
//! exercise the full pipeline. Growing the corpus to 10,000 is a
//! populate-over-time operation; the loader, runner, and reporter do
//! not need to change as it grows.
//!
//! # Moving parts
//!
//! - [`vector`] — `TestVector`: the `(study, method, expected_h,
//!   tolerance_class)` tuple from §14.1.
//! - [`runner`] — `run_conformance`: drive a `Solver` through a set
//!   of vectors and collect pass/fail/error verdicts.
//! - [`report`] — `ConformanceReport`: the §14.4 required output
//!   structure, serializable to `conformance-report.json`.
//! - [`properties`] — the §14.3 property tests (scaling identity,
//!   idempotent recompute, block-diagonal independence, parameter
//!   rewrite, Sherman-Morrison parity).
//! - [`loader`] — deserialize a directory of on-disk JSON vectors.
//! - [`seed`] — in-Rust seed vectors so the crate ships with working
//!   test material even before the on-disk corpus is populated.

pub mod loader;
pub mod properties;
pub mod report;
pub mod runner;
pub mod seed;
pub mod vector;

pub use loader::{load_vector_directory, VectorLoadError};
pub use properties::{
    check_block_diagonal_independence, check_idempotent_recompute, check_scaling_identity,
    check_sherman_morrison_parity, PropertyFailure,
};
pub use report::{ConformanceReport, VectorResult, VectorVerdict};
pub use runner::{run_conformance, run_single_vector, RunnerConfig};
pub use seed::seed_vectors;
pub use vector::{ConformanceLevel, TestVector, ToleranceClass};
