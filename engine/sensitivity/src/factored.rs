//! The `FactoredSystem` — a cached `(A, f, s)` triple that supports
//! incremental updates via Sherman-Morrison-Woodbury.
//!
//! Construction is via `from_solve`, which runs the base solve once.
//! Every subsequent `update_*` call returns a new `s` in-place, bumps
//! the staleness generation (§10.2), and leaves `A` updated to reflect
//! the modification.

use arko_core::{
    error::EngineError,
    matrices::{DenseVector, SparseMatrix, SparseVector},
    solver::Solver,
};
use serde::{Deserialize, Serialize};

/// Errors specific to the sensitivity layer. Non-sensitivity errors
/// (e.g., shape mismatch from the solver) propagate as `EngineError`.
#[derive(Debug, thiserror::Error)]
pub enum SensitivityError {
    #[error("engine error: {0}")]
    Engine(#[from] EngineError),
}

/// A cached factorization-aware system: `A·s = f`, plus the generation
/// counter from §10.2.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FactoredSystem {
    /// The current technosphere matrix. Updated in place by each
    /// incremental `update_*` call.
    pub a: SparseMatrix,
    /// The current functional unit, in dense form for fast dot products.
    pub f_dense: DenseVector,
    /// The current solution: `A · scaling = f_dense`.
    pub scaling: DenseVector,
    /// Staleness counter per §10.2. `0` immediately after a full solve
    /// or refactor; increments on each incremental update.
    pub generation: u64,
}

impl FactoredSystem {
    /// Run the base solve and return a `FactoredSystem` ready for
    /// incremental updates. `generation` starts at 0.
    pub fn from_solve<S: Solver>(
        a: SparseMatrix,
        f: &SparseVector,
        solver: &S,
    ) -> Result<Self, SensitivityError> {
        let scaling = solver.solve(&a, f)?;
        let f_dense = sparse_to_dense(f, a.rows());
        Ok(Self {
            a,
            f_dense,
            scaling,
            generation: 0,
        })
    }

    /// Full refactorization path per §10.2 — replaces `A` and `f`,
    /// re-solves from scratch, and **resets** the staleness counter
    /// to 0. Callers **MUST** record this reset in result provenance.
    pub fn refactor<S: Solver>(
        &mut self,
        a: SparseMatrix,
        f: &SparseVector,
        solver: &S,
    ) -> Result<(), SensitivityError> {
        let scaling = solver.solve(&a, f)?;
        let f_dense = sparse_to_dense(f, a.rows());
        self.a = a;
        self.f_dense = f_dense;
        self.scaling = scaling;
        self.generation = 0;
        Ok(())
    }

    /// Current dimension (`n`).
    pub fn n(&self) -> usize {
        self.a.rows()
    }
}

/// Expand a sparse vector to a dense one of the given length.
pub(crate) fn sparse_to_dense(sv: &SparseVector, n: usize) -> DenseVector {
    let mut out = vec![0.0_f64; n];
    for (&i, &v) in sv.indices().iter().zip(sv.data().iter()) {
        out[i] = v;
    }
    out
}

/// Dot product of two equal-length slices. Caller ensures shape match.
pub(crate) fn dot(a: &[f64], b: &[f64]) -> f64 {
    debug_assert_eq!(a.len(), b.len());
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

/// Build a SparseVector of length `n` from a dense slice.
pub(crate) fn dense_to_sparse_vector(dense: &[f64]) -> SparseVector {
    let mut idx: Vec<usize> = Vec::new();
    let mut data: Vec<f64> = Vec::new();
    for (i, &v) in dense.iter().enumerate() {
        if v != 0.0 {
            idx.push(i);
            data.push(v);
        }
    }
    SparseVector::new(dense.len(), idx, data)
}
