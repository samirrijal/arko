//! Cross-solver parity tests — a smoke-grade down-payment on the
//! differential test harness of `specs/calc/v0.1.md` §14.
//!
//! We run the same inputs through `DenseLuSolver` and `SparseLuSolver`
//! and assert the solutions agree within the §8.1 reference-parity
//! tolerance. Passing this on every commit is what lets us trust that
//! swapping solver backends does not silently change results.

use approx::assert_relative_eq;
use arko_core::{matrices::SparseMatrix, Solver, SparseVector};
use arko_solvers_dense::DenseLuSolver;
use arko_solvers_sparse::SparseLuSolver;
use sprs::TriMat;

fn minimal_spec_16() -> (SparseMatrix, SparseVector) {
    // The §16 worked example: steel + electricity, demand 1 kg steel.
    let mut a = TriMat::new((2, 2));
    a.add_triplet(0, 0, 1.0);
    a.add_triplet(0, 1, -0.5);
    a.add_triplet(1, 1, 1.0);
    let a_csr: SparseMatrix = a.to_csr();
    let f = SparseVector::new(2, vec![0], vec![1.0]);
    (a_csr, f)
}

fn banded_5x5() -> (SparseMatrix, SparseVector) {
    // A small banded system: identity diagonal with negative
    // super-diagonals so A is well-conditioned but non-trivial.
    //
    //   1.0 -0.1  0    0    0
    //   0    1.0 -0.2  0    0
    //   0    0    1.0 -0.1  0
    //   0    0    0    1.0 -0.3
    //   0    0    0    0    1.0
    let mut a = TriMat::new((5, 5));
    for i in 0..5 {
        a.add_triplet(i, i, 1.0);
    }
    a.add_triplet(0, 1, -0.1);
    a.add_triplet(1, 2, -0.2);
    a.add_triplet(2, 3, -0.1);
    a.add_triplet(3, 4, -0.3);
    let a_csr: SparseMatrix = a.to_csr();
    let f = SparseVector::new(5, vec![0], vec![1.0]);
    (a_csr, f)
}

fn assert_solvers_agree(a: &SparseMatrix, f: &SparseVector) {
    let dense = DenseLuSolver.solve(a, f).expect("dense solve");
    let sparse = SparseLuSolver.solve(a, f).expect("sparse solve");
    assert_eq!(
        dense.len(),
        sparse.len(),
        "solvers produced different-length solutions"
    );
    for i in 0..dense.len() {
        // §8.1 reference-parity tolerance.
        assert_relative_eq!(dense[i], sparse[i], epsilon = 1e-12, max_relative = 1e-9);
    }
}

#[test]
fn parity_on_spec_16() {
    let (a, f) = minimal_spec_16();
    assert_solvers_agree(&a, &f);
}

#[test]
fn parity_on_banded_5x5() {
    let (a, f) = banded_5x5();
    assert_solvers_agree(&a, &f);
}

#[test]
fn parity_on_20x20_random_diagonally_dominant() {
    // Build a 20x20 diagonally-dominant sparse matrix deterministically
    // (no RNG — hand-computed values keep the test bit-reproducible).
    let n = 20;
    let mut a = TriMat::new((n, n));
    for i in 0..n {
        // Diagonal large enough to dominate the off-diagonals we add.
        a.add_triplet(i, i, 10.0 + (i as f64) * 0.1);
    }
    // Scattered off-diagonals (at most 2 per row to keep sparsity real).
    let edges: &[(usize, usize, f64)] = &[
        (0, 1, -0.5),
        (0, 5, -0.3),
        (1, 2, -0.7),
        (2, 3, -0.2),
        (3, 7, -0.1),
        (4, 6, -0.4),
        (5, 10, -0.6),
        (6, 11, -0.2),
        (7, 13, -0.3),
        (8, 14, -0.5),
        (9, 15, -0.1),
        (10, 12, -0.7),
        (11, 17, -0.2),
        (12, 18, -0.3),
        (13, 19, -0.1),
    ];
    for &(r, c, v) in edges {
        a.add_triplet(r, c, v);
    }
    let a_csr: SparseMatrix = a.to_csr();

    // Demand: 1.0 on process 0, 0.5 on process 10 — non-trivial.
    let f = SparseVector::new(n, vec![0, 10], vec![1.0, 0.5]);
    assert_solvers_agree(&a_csr, &f);
}
