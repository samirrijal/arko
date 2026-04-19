//! The three-equation calculation pipeline — see `specs/calc/v0.1.md` §4.3.
//!
//! ```text
//!   A · s = f                 [Eq. 1]  — solver delegated
//!   g     = B · s             [Eq. 2]  — sparse · dense, done here
//!   h     = C · g             [Eq. 3]  — sparse · dense, done here
//! ```
//!
//! Along with the core math this module also computes:
//! - the **effective license restriction** (§11.2) by joining the tiers
//!   of every process whose scaling factor exceeds the presence threshold,
//! - the **contributing processes** list recorded in provenance (§12).
//!
//! Provenance itself (engine version, hash, solver name) is assembled by
//! the caller because it depends on facts the pipeline does not know —
//! git SHA, the user identity, the generation counter.

use crate::{
    error::EngineError,
    matrices::{DenseVector, SparseMatrix},
    provenance::EffectiveRestriction,
    solver::Solver,
    study::Study,
};

/// Presence threshold for contribution to a result — spec §11.2.
///
/// A process whose `|s[j]| ≤ EPS_PRESENCE` is considered to have not
/// contributed and is excluded from both `contributing_processes` and
/// the license-tier join.
pub const EPS_PRESENCE: f64 = 1e-9;

/// Full result of a pipeline run.
#[derive(Debug, Clone, PartialEq)]
pub struct Computed {
    /// Scaling vector `s` of length `n = study.n_processes()`.
    pub scaling: DenseVector,
    /// Inventory vector `g = B · s` of length `m = study.n_flows()`.
    pub inventory: DenseVector,
    /// Impact vector `h = C · g` of length `k = study.n_impacts()`.
    pub impact: DenseVector,
    /// License restriction joined from all contributing processes.
    pub effective_restriction: EffectiveRestriction,
    /// Indices (into `study.processes`) of processes with `|s[j]| > EPS_PRESENCE`.
    pub contributing_processes: Vec<u32>,
}

/// Execute the full three-equation pipeline against `study` using `solver`.
///
/// This does **not** perform the full §6.1 validation order — full
/// validation belongs in `arko-validation` which callers should run
/// first. What this function does enforce is the minimum set of shape
/// invariants required to not produce garbage: A square, `f` length-n,
/// B is `m × n`, C is `k × m`.
pub fn compute<S: Solver>(study: &Study, solver: &S) -> Result<Computed, EngineError> {
    let n = study.n_processes();
    let m = study.n_flows();
    let k = study.n_impacts();

    let (a_rows, a_cols) = study.technosphere.shape();
    if a_rows != n || a_cols != n {
        return Err(EngineError::ShapeMismatch(format!(
            "technosphere A is {a_rows}×{a_cols}, expected {n}×{n}"
        )));
    }
    if study.functional_unit.dim() != n {
        return Err(EngineError::ShapeMismatch(format!(
            "functional unit length {} != technosphere dim {n}",
            study.functional_unit.dim()
        )));
    }
    let (b_rows, b_cols) = study.biosphere.shape();
    if b_rows != m || b_cols != n {
        return Err(EngineError::ShapeMismatch(format!(
            "biosphere B is {b_rows}×{b_cols}, expected {m}×{n}"
        )));
    }
    let (c_rows, c_cols) = study.characterization.shape();
    if c_rows != k || c_cols != m {
        return Err(EngineError::ShapeMismatch(format!(
            "characterization C is {c_rows}×{c_cols}, expected {k}×{m}"
        )));
    }

    // Eq. 1 — delegate to solver.
    let scaling = solver.solve(&study.technosphere, &study.functional_unit)?;

    // Eq. 2 — g = B · s.
    let inventory = sparse_mat_vec(&study.biosphere, &scaling);

    // Eq. 3 — h = C · g.
    let impact = sparse_mat_vec(&study.characterization, &inventory);

    // §11.2 — effective restriction + contributing processes.
    let mut restriction = EffectiveRestriction::permissive();
    let mut contributing: Vec<u32> = Vec::new();
    for (j, &s_j) in scaling.iter().enumerate() {
        if s_j.abs() > EPS_PRESENCE {
            contributing.push(
                u32::try_from(j).map_err(|_| EngineError::Internal("process index > u32::MAX".into()))?,
            );
            let tier_idx = study.processes[j].license_tier.0 as usize;
            if let Some(tier) = study.license_tiers.get(tier_idx) {
                restriction.join(tier);
            }
        }
    }

    Ok(Computed {
        scaling,
        inventory,
        impact,
        effective_restriction: restriction,
        contributing_processes: contributing,
    })
}

/// Dense-output `y = M · x` for a sparse matrix `M` (CSR) and dense `x`.
///
/// Summation order is deterministic: outer-iterator row-major, inner
/// iterator in sparse index order. This satisfies the determinism
/// contract of spec §7.2 for this reduction.
fn sparse_mat_vec(m: &SparseMatrix, x: &[f64]) -> DenseVector {
    let rows = m.rows();
    let mut y = vec![0.0_f64; rows];
    for (row_idx, row_view) in m.outer_iterator().enumerate() {
        let mut acc = 0.0_f64;
        for (col_idx, &val) in row_view.iter() {
            acc += val * x[col_idx];
        }
        y[row_idx] = acc;
    }
    y
}
