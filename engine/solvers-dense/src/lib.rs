//! Dense LU solver for the Arko calculation engine.
//!
//! Implements the `arko_core::Solver` contract using `nalgebra`'s dense
//! LU decomposition with partial pivoting. Intended for **small**
//! studies (`n < 100`) per `specs/calc/v0.1.md` §6.3. Larger studies
//! should use the sparse backend (`arko-solvers-sparse`, TBD).
//!
//! # Numerical contract
//! - LU with partial pivoting (nalgebra's `LU`).
//! - Deterministic: single-threaded linear algebra, stable pivot order.
//! - Returns `EngineError::Singular` if the factorization detects a
//!   zero pivot (nalgebra's `lu.solve` returns `None`).

use arko_core::{
    error::EngineError,
    matrices::{DenseVector, SparseMatrix, SparseVector},
    solver::Solver,
};
use nalgebra::{DMatrix, DVector};

/// Dense LU solver, stateless.
///
/// Construction is free (no allocation, no warmup). The decomposition
/// itself happens inside `solve` and is not cached — callers that need
/// caching should factor once and call `solve_factored` in a future
/// incremental-recalc API (spec §10).
#[derive(Debug, Clone, Copy, Default)]
pub struct DenseLuSolver;

impl Solver for DenseLuSolver {
    fn name(&self) -> &'static str {
        "nalgebra-dense-lu"
    }

    fn config(&self) -> serde_json::Value {
        serde_json::json!({
            "backend": "nalgebra",
            "decomposition": "lu",
            "pivoting": "partial",
            "threads": 1,
        })
    }

    fn solve(&self, a: &SparseMatrix, f: &SparseVector) -> Result<DenseVector, EngineError> {
        let (rows, cols) = a.shape();
        if rows != cols {
            return Err(EngineError::ShapeMismatch(format!(
                "technosphere must be square; got {rows}×{cols}"
            )));
        }
        if f.dim() != rows {
            return Err(EngineError::ShapeMismatch(format!(
                "functional_unit length {} != technosphere dim {rows}",
                f.dim()
            )));
        }

        let mut dense = DMatrix::<f64>::zeros(rows, cols);
        for (row_idx, row_view) in a.outer_iterator().enumerate() {
            for (col_idx, &val) in row_view.iter() {
                dense[(row_idx, col_idx)] = val;
            }
        }

        let mut rhs = DVector::<f64>::zeros(rows);
        for (i, &v) in f.iter() {
            rhs[i] = v;
        }

        let lu = dense.lu();
        let s = lu.solve(&rhs).ok_or(EngineError::Singular)?;

        Ok(s.as_slice().to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    use sprs::TriMat;

    fn identity_2x2() -> SparseMatrix {
        let mut t = TriMat::new((2, 2));
        t.add_triplet(0, 0, 1.0);
        t.add_triplet(1, 1, 1.0);
        t.to_csr()
    }

    #[test]
    fn solves_identity() {
        let a = identity_2x2();
        let f = SparseVector::new(2, vec![0, 1], vec![3.0, 4.0]);
        let s = DenseLuSolver.solve(&a, &f).unwrap();
        assert_eq!(s.len(), 2);
        assert_relative_eq!(s[0], 3.0, epsilon = 1e-12);
        assert_relative_eq!(s[1], 4.0, epsilon = 1e-12);
    }

    #[test]
    fn rejects_non_square() {
        let mut t = TriMat::new((2, 3));
        t.add_triplet(0, 0, 1.0);
        let a: SparseMatrix = t.to_csr();
        let f = SparseVector::new(2, vec![0], vec![1.0]);
        let err = DenseLuSolver.solve(&a, &f).unwrap_err();
        assert_eq!(err.code(), "E_SHAPE_MISMATCH");
    }

    #[test]
    fn rejects_dim_mismatch_on_rhs() {
        let a = identity_2x2();
        let f = SparseVector::new(3, vec![0], vec![1.0]);
        let err = DenseLuSolver.solve(&a, &f).unwrap_err();
        assert_eq!(err.code(), "E_SHAPE_MISMATCH");
    }

    #[test]
    fn detects_singular() {
        // Zero matrix — singular.
        let a: SparseMatrix = TriMat::<f64>::new((2, 2)).to_csr();
        let f = SparseVector::new(2, vec![0], vec![1.0]);
        let err = DenseLuSolver.solve(&a, &f).unwrap_err();
        assert_eq!(err.code(), "E_SINGULAR");
    }

    #[test]
    fn provenance_config_is_nontrivial() {
        let cfg = DenseLuSolver.config();
        assert!(cfg.is_object(), "config() must be a JSON object");
        assert_eq!(cfg["backend"], "nalgebra");
    }
}
