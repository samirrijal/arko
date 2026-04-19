//! Sparse LU solver for the Arko calculation engine.
//!
//! Implements the `arko_core::Solver` contract using `faer`'s sparse
//! LU with partial pivoting and a COLAMD-family fill-reducing ordering.
//! Intended for **medium** studies (`100 ≤ n < 10_000`) per
//! `specs/calc/v0.1.md` §6.3. For `n ≥ 10_000` the UMFPACK-bound solver
//! (`arko-solvers-umfpack`, TBD) generally wins.
//!
//! # Numerical contract
//! - Sparse LU with partial pivoting (faer's `sp_lu`).
//! - Deterministic: single-threaded by default, stable pivot order,
//!   and we sort triplets `(row, col, value)` into a canonical order
//!   before matrix construction to satisfy spec §7.2.
//! - Returns `EngineError::Singular` when the factorization detects a
//!   zero pivot.
//!
//! # Version pinning
//! Targets **faer 0.20**. The exact method names on `SparseColMat`
//! (`sp_lu`, `solve_in_place`) have shifted across recent faer
//! releases; if you upgrade faer, re-verify against the current docs.

use arko_core::{
    error::EngineError,
    matrices::{DenseVector, SparseMatrix, SparseVector},
    solver::Solver,
};
use faer::prelude::SpSolver;
use faer::sparse::SparseColMat;

#[derive(Debug, Clone, Copy, Default)]
pub struct SparseLuSolver;

impl Solver for SparseLuSolver {
    fn name(&self) -> &'static str {
        "faer-sparse-lu"
    }

    fn config(&self) -> serde_json::Value {
        serde_json::json!({
            "backend": "faer",
            "decomposition": "sparse_lu",
            "pivoting": "partial",
            "fill_reducing_ordering": "colamd",
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

        // Build a deterministic triplet list. We sort by (col, row) so
        // the matrix we hand to faer is in stable canonical order — this
        // makes the fill-reducing ordering and subsequent numerical
        // factorization reproducible, per spec §7.2.
        let mut triplets: Vec<(usize, usize, f64)> = Vec::with_capacity(a.nnz());
        for (row_idx, row_view) in a.outer_iterator().enumerate() {
            for (col_idx, &val) in row_view.iter() {
                triplets.push((row_idx, col_idx, val));
            }
        }
        triplets.sort_by(|x, y| x.1.cmp(&y.1).then(x.0.cmp(&y.0)));

        let mat = SparseColMat::<usize, f64>::try_new_from_triplets(rows, cols, &triplets)
            .map_err(|e| EngineError::Internal(format!("faer sparse build: {e:?}")))?;

        let mut rhs = faer::Col::<f64>::zeros(rows);
        for (i, &v) in f.iter() {
            rhs[i] = v;
        }

        let lu = mat.sp_lu().map_err(|_| EngineError::Singular)?;
        let mut sol = rhs.clone();
        lu.solve_in_place(sol.as_mut());

        Ok((0..rows).map(|i| sol[i]).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    use sprs::TriMat;

    fn identity(n: usize) -> SparseMatrix {
        let mut t = TriMat::new((n, n));
        for i in 0..n {
            t.add_triplet(i, i, 1.0);
        }
        t.to_csr()
    }

    #[test]
    fn solves_identity() {
        let a = identity(3);
        let f = SparseVector::new(3, vec![0, 1, 2], vec![1.0, 2.0, 3.0]);
        let s = SparseLuSolver.solve(&a, &f).unwrap();
        assert_relative_eq!(s[0], 1.0, epsilon = 1e-12);
        assert_relative_eq!(s[1], 2.0, epsilon = 1e-12);
        assert_relative_eq!(s[2], 3.0, epsilon = 1e-12);
    }

    #[test]
    fn rejects_non_square() {
        let mut t = TriMat::new((2, 3));
        t.add_triplet(0, 0, 1.0);
        let a: SparseMatrix = t.to_csr();
        let f = SparseVector::new(2, vec![0], vec![1.0]);
        assert_eq!(
            SparseLuSolver.solve(&a, &f).unwrap_err().code(),
            "E_SHAPE_MISMATCH"
        );
    }

    #[test]
    fn detects_singular() {
        let a: SparseMatrix = TriMat::<f64>::new((2, 2)).to_csr();
        let f = SparseVector::new(2, vec![0], vec![1.0]);
        assert_eq!(
            SparseLuSolver.solve(&a, &f).unwrap_err().code(),
            "E_SINGULAR"
        );
    }

    #[test]
    fn config_nontrivial() {
        let cfg = SparseLuSolver.config();
        assert_eq!(cfg["backend"], "faer");
    }
}
