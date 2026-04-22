//! Dense LU solver for the Arko calculation engine.
//!
//! Implements the [`arko_core::Solver`] contract using `nalgebra`'s dense
//! LU decomposition with partial pivoting. Intended for **small**
//! studies (`n < 100`) per `specs/calc/v0.1.md` §6.3. Larger studies
//! should use the sparse backend (`arko-solvers-sparse`).
//!
//! # Two surfaces
//!
//! - [`Solver`] — the core single-shot trait. Convenient when you have
//!   one `(A, f)` pair and don't intend to reuse the factorization.
//! - [`FactoredSolver`] — factor `A` once, [`solve`][DenseLuFactorization::solve]
//!   many right-hand sides. The performance lever for sensitivity
//!   sweeps, Monte Carlo, and any workflow where `A` is reused across
//!   varying `f`. Spec §10 (incremental recalc) names this surface
//!   directly.
//!
//! `Solver::solve` is implemented in terms of `FactoredSolver` —
//! `solve(A, f) ≡ factorize(A)?.solve(f)` — so the two paths share the
//! same numerical kernel. Code that wants caching reaches for
//! `FactoredSolver`; code that wants a one-shot solve reaches for
//! `Solver`.
//!
//! # Numerical contract
//! - LU with partial pivoting (nalgebra's `LU`).
//! - Deterministic: single-threaded linear algebra, stable pivot order.
//! - Returns `EngineError::Singular` if the factorization detects a
//!   zero pivot (nalgebra's `lu.solve` returns `None`). For
//!   `FactoredSolver`, singularity surfaces at `factorize` time when
//!   the first solve is attempted; nalgebra's `LU` constructor itself
//!   never errors, so the singularity check happens during the first
//!   `DenseLuFactorization::solve` call by inspecting the result.

use arko_core::{
    error::EngineError,
    matrices::{DenseVector, SparseMatrix, SparseVector},
    solver::Solver,
};
use nalgebra::{DMatrix, DVector, Dyn, LU};

/// Dense LU solver, stateless.
///
/// Construction is free (no allocation, no warmup). The decomposition
/// itself happens inside [`Solver::solve`] (one-shot) or
/// [`FactoredSolver::factorize`] (cached).
#[derive(Debug, Clone, Copy, Default)]
pub struct DenseLuSolver;

/// A precomputed LU factorization of a square `A` matrix.
///
/// Stores `P · A = L · U` once; each call to [`solve`][Self::solve] is
/// a triangular solve only — no re-decomposition. This is the
/// performance lever for any workflow that reuses `A` across many
/// right-hand sides (sensitivity sweeps, Monte Carlo, the §10
/// incremental recalc path).
///
/// Construct via [`FactoredSolver::factorize`]; the type itself has no
/// public constructor because the only meaningful way to populate the
/// factorization is through a backend.
///
/// # Numerical contract
///
/// - Same partial-pivot LU as the single-shot path
///   ([`Solver::solve`]); `factorize(A)?.solve(f)` is bitwise
///   identical to `DenseLuSolver::solve(A, f)` for the same `(A, f)`.
/// - Singularity is detected at [`solve`][Self::solve] time, not
///   [`factorize`][FactoredSolver::factorize] time — nalgebra's `LU`
///   constructor does not error; the zero-pivot signal surfaces when
///   the triangular back-substitution is attempted. Concretely: a
///   factorization of a singular `A` constructs successfully, then
///   every `solve` call against it returns `EngineError::Singular`.
/// - Dimension mismatches between the cached `n` and the supplied `f`
///   surface as `EngineError::ShapeMismatch` per the same contract as
///   [`Solver::solve`].
#[derive(Debug, Clone)]
pub struct DenseLuFactorization {
    /// The factored A as nalgebra's packed LU + permutation.
    lu: LU<f64, Dyn, Dyn>,
    /// Cached square dimension of the original `A`. Carried explicitly
    /// so RHS-dim checks in `solve` don't have to reconstruct it from
    /// the LU type at every call.
    n: usize,
}

impl DenseLuFactorization {
    /// Solve `A · s = f` against the cached factorization.
    ///
    /// Returns `EngineError::ShapeMismatch` if `f.dim() != n`, where
    /// `n` is the dimension of the originally factored `A`. Returns
    /// `EngineError::Singular` if the cached factorization is rank-
    /// deficient (see type-level docs — singularity surfaces at solve
    /// time, not factorize time).
    pub fn solve(&self, f: &SparseVector) -> Result<DenseVector, EngineError> {
        if f.dim() != self.n {
            return Err(EngineError::ShapeMismatch(format!(
                "functional_unit length {} != factored dim {}",
                f.dim(),
                self.n
            )));
        }
        let mut rhs = DVector::<f64>::zeros(self.n);
        for (i, &v) in f.iter() {
            rhs[i] = v;
        }
        let s = self.lu.solve(&rhs).ok_or(EngineError::Singular)?;
        Ok(s.as_slice().to_vec())
    }

    /// Square dimension of the originally factored matrix `A`.
    #[must_use]
    pub fn dim(&self) -> usize {
        self.n
    }
}

/// Backends that support **factor once, solve many** for `A · s = f`.
///
/// The associated [`Factorization`][Self::Factorization] type is the
/// backend-specific cached form; it carries its own `solve` method
/// (see [`DenseLuFactorization::solve`] for the dense LU case). This
/// lets the type system enforce "you cannot solve before factoring"
/// without runtime checks.
///
/// # Why a per-backend trait, not core
///
/// The factored form is irreducibly backend-specific: dense LU stores
/// nalgebra's packed `LU` + permutation; sparse LU would store
/// SuperLU/UMFPACK column structure; iterative methods store
/// preconditioners. Hiding all of those behind a single core trait
/// would force `Box<dyn Any>` or aggressive generics; co-locating
/// `FactoredSolver` with each backend keeps the factorization type
/// concrete and the API free of gymnastics. The single-shot
/// [`Solver`] contract still lives in core because `pipeline::compute`
/// needs to name *one* polymorphic solve point.
pub trait FactoredSolver {
    /// The cached form of a factored matrix produced by this backend.
    /// Carries its own `solve` method (per-backend signature, no trait
    /// bound enforced at this layer).
    type Factorization;

    /// Compute and cache the factorization of `A`. The returned value
    /// may be reused across many subsequent `solve` calls without
    /// repeating the decomposition.
    ///
    /// Implementations **MUST** return `EngineError::ShapeMismatch` if
    /// `A` is not square. Singularity detection is permitted to defer
    /// to solve time (see [`DenseLuFactorization`] type docs); some
    /// backends factor singular matrices successfully and only fail
    /// on the back-substitution step.
    fn factorize(&self, a: &SparseMatrix) -> Result<Self::Factorization, EngineError>;
}

impl FactoredSolver for DenseLuSolver {
    type Factorization = DenseLuFactorization;

    fn factorize(&self, a: &SparseMatrix) -> Result<DenseLuFactorization, EngineError> {
        let (rows, cols) = a.shape();
        if rows != cols {
            return Err(EngineError::ShapeMismatch(format!(
                "technosphere must be square; got {rows}×{cols}"
            )));
        }
        let mut dense = DMatrix::<f64>::zeros(rows, cols);
        for (row_idx, row_view) in a.outer_iterator().enumerate() {
            for (col_idx, &val) in row_view.iter() {
                dense[(row_idx, col_idx)] = val;
            }
        }
        let lu = dense.lu();
        Ok(DenseLuFactorization { lu, n: rows })
    }
}

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
        // Single-shot path delegates to the factored path so both
        // surfaces share one numerical kernel. The fast-path RHS
        // shape check is duplicated here to preserve the existing
        // error-message text for `f.dim() != A.rows()`, which existing
        // tests assert on.
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
        self.factorize(a)?.solve(f)
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

    /// A non-trivial 3×3 with known inverse — used by the
    /// factor-once-solve-many tests so the factored path is
    /// exercised against a matrix with real off-diagonal coupling
    /// (identity matrices can't catch a "we lost the permutation"
    /// bug because permutation of I is I).
    fn coupled_3x3() -> SparseMatrix {
        // A = [[2, 1, 0],
        //      [1, 3, 1],
        //      [0, 1, 2]]
        // det(A) = 2·(3·2 - 1·1) - 1·(1·2 - 0) = 10 - 2 = 8
        let mut t = TriMat::new((3, 3));
        t.add_triplet(0, 0, 2.0);
        t.add_triplet(0, 1, 1.0);
        t.add_triplet(1, 0, 1.0);
        t.add_triplet(1, 1, 3.0);
        t.add_triplet(1, 2, 1.0);
        t.add_triplet(2, 1, 1.0);
        t.add_triplet(2, 2, 2.0);
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

    // ------------------- FactoredSolver tests -------------------

    #[test]
    fn factorize_caches_dim() {
        let a = coupled_3x3();
        let f = DenseLuSolver.factorize(&a).unwrap();
        assert_eq!(f.dim(), 3);
    }

    #[test]
    fn factorize_then_solve_matches_single_shot_on_coupled_3x3() {
        // The same `(A, f)` must produce the same `s` whether we go
        // through the one-shot or the factored path — they share a
        // numerical kernel by construction (Solver::solve delegates
        // to FactoredSolver::factorize().solve()), so this is a
        // regression test against accidentally diverging the two
        // surfaces.
        let a = coupled_3x3();
        let f = SparseVector::new(3, vec![0, 1, 2], vec![1.0, 2.0, 3.0]);
        let single = DenseLuSolver.solve(&a, &f).unwrap();
        let cached = DenseLuSolver.factorize(&a).unwrap().solve(&f).unwrap();
        assert_eq!(single.len(), cached.len());
        for i in 0..single.len() {
            assert_relative_eq!(single[i], cached[i], epsilon = 1e-15);
        }
    }

    #[test]
    fn factorize_solves_many_rhs_without_refactoring() {
        // Factor once, solve N RHS — the API contract that justifies
        // the trait existing in the first place. We solve for each
        // column of the identity to reconstruct A⁻¹, then verify by
        // multiplying back: A · column_k must equal e_k.
        let a = coupled_3x3();
        let factored = DenseLuSolver.factorize(&a).unwrap();
        let mut a_inv_cols: Vec<DenseVector> = Vec::with_capacity(3);
        for k in 0..3 {
            let e_k = SparseVector::new(3, vec![k], vec![1.0]);
            let col = factored.solve(&e_k).unwrap();
            assert_eq!(col.len(), 3);
            a_inv_cols.push(col);
        }
        // Residual check: A · A⁻¹ ≈ I. Compute each (A · column_k) by
        // pulling the explicit dense form and multiplying.
        let mut a_dense = DMatrix::<f64>::zeros(3, 3);
        for (row_idx, row_view) in a.outer_iterator().enumerate() {
            for (col_idx, &val) in row_view.iter() {
                a_dense[(row_idx, col_idx)] = val;
            }
        }
        for k in 0..3 {
            let col_k = DVector::from_column_slice(&a_inv_cols[k]);
            let product = &a_dense * &col_k;
            for i in 0..3 {
                let expected = if i == k { 1.0 } else { 0.0 };
                assert_relative_eq!(product[i], expected, epsilon = 1e-12);
            }
        }
    }

    #[test]
    fn factorize_rejects_non_square() {
        let mut t = TriMat::new((2, 3));
        t.add_triplet(0, 0, 1.0);
        let a: SparseMatrix = t.to_csr();
        let err = DenseLuSolver.factorize(&a).unwrap_err();
        assert_eq!(err.code(), "E_SHAPE_MISMATCH");
    }

    #[test]
    fn factored_solve_rejects_dim_mismatch_on_rhs() {
        // Cached `n` is 2; supply a length-3 RHS — must surface as
        // ShapeMismatch with the cached-dim error message, not panic
        // inside nalgebra.
        let a = identity_2x2();
        let factored = DenseLuSolver.factorize(&a).unwrap();
        let f = SparseVector::new(3, vec![0], vec![1.0]);
        let err = factored.solve(&f).unwrap_err();
        assert_eq!(err.code(), "E_SHAPE_MISMATCH");
    }

    #[test]
    fn factored_solve_surfaces_singularity_at_solve_time() {
        // Per type-docs: singular A factorizes successfully (nalgebra
        // doesn't error in the LU constructor); the zero-pivot signal
        // surfaces during back-substitution. This test pins that
        // contract — accidentally moving the check to factorize time
        // would silently change the timing of the error and break
        // callers that expect to be able to construct a factorization
        // before knowing whether A is singular (e.g., for
        // diagnostic/inspection workflows).
        let a: SparseMatrix = TriMat::<f64>::new((2, 2)).to_csr();
        let factored = DenseLuSolver
            .factorize(&a)
            .expect("singular A still factorizes — singularity surfaces at solve time");
        let f = SparseVector::new(2, vec![0], vec![1.0]);
        let err = factored.solve(&f).unwrap_err();
        assert_eq!(err.code(), "E_SINGULAR");
    }

    #[test]
    fn factored_solve_is_deterministic_across_repeated_calls() {
        // Same factorization, same RHS, called twice — must produce
        // bitwise identical output. Caching state (the LU permutation,
        // the pivot order) MUST NOT mutate between solves.
        let a = coupled_3x3();
        let factored = DenseLuSolver.factorize(&a).unwrap();
        let f = SparseVector::new(3, vec![0, 1, 2], vec![1.5, -0.5, 2.25]);
        let s1 = factored.solve(&f).unwrap();
        let s2 = factored.solve(&f).unwrap();
        assert_eq!(s1, s2);
    }

    #[test]
    fn factored_solve_handles_distinct_rhs_independently() {
        // No state leaks between solves — solving for f1 then f2 must
        // give the same f2-result as solving for f2 directly.
        let a = coupled_3x3();
        let factored = DenseLuSolver.factorize(&a).unwrap();
        let f1 = SparseVector::new(3, vec![0, 1, 2], vec![1.0, 0.0, 0.0]);
        let f2 = SparseVector::new(3, vec![0, 1, 2], vec![0.0, 1.0, 0.0]);
        let _s1 = factored.solve(&f1).unwrap();
        let s2_after = factored.solve(&f2).unwrap();
        let s2_direct = DenseLuSolver.factorize(&a).unwrap().solve(&f2).unwrap();
        for i in 0..3 {
            assert_relative_eq!(s2_after[i], s2_direct[i], epsilon = 1e-15);
        }
    }
}
