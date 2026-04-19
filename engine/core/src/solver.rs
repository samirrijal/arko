//! Solver trait — the contract between `arko-core` and any backend
//! implementing `specs/calc/v0.1.md` §6.3.
//!
//! The trait lives in core (not in a separate crate) because the calc
//! pipeline in `pipeline::compute` needs to name it, and because having
//! it here lets alternative solvers (`arko-solvers-dense`,
//! `arko-solvers-sparse`, etc.) be swapped with no changes to anything
//! upstream of the solver itself.

use crate::{
    error::EngineError,
    matrices::{DenseVector, SparseMatrix, SparseVector},
};

/// A direct-method linear solver for the technosphere system `A · s = f`.
///
/// Per spec §6.3, the primary solve **MUST** be a direct sparse (or dense,
/// for small `n`) method. Iterative solvers (`GMRES`, `BiCGStab`) are permitted
/// as fallbacks but **MUST** declare convergence tolerance through the
/// `config()` hook so provenance records are complete (§12).
pub trait Solver {
    /// Stable human-readable name, e.g. `"nalgebra-dense-lu"`.
    /// Emitted into `Provenance::solver_used`.
    fn name(&self) -> &'static str;

    /// Solver-specific configuration, echoed into
    /// `Provenance::solver_config` for reproducibility. Default is null.
    fn config(&self) -> serde_json::Value {
        serde_json::Value::Null
    }

    /// Solve `A · s = f` and return `s` as a dense vector.
    ///
    /// Implementations **MUST** return `EngineError::ShapeMismatch` if
    /// `A` is not square or `f.dim() != A.rows()`, and
    /// `EngineError::Singular` if a singular factor is encountered.
    fn solve(&self, a: &SparseMatrix, f: &SparseVector) -> Result<DenseVector, EngineError>;
}
