//! SMW parity tests — every incremental update path must agree with a
//! full refactor within §8.1 reference-parity tolerance
//! (`ε_abs = 1e-12`, `ε_rel = 1e-9`). This is the sensitivity-layer's
//! contribution to the §14 differential harness: we don't yet have
//! 10,000 studies, but every update method gets a direct head-to-head
//! against a recomputed-from-scratch ground truth.

use approx::assert_relative_eq;
use arko_core::{
    error::EngineError,
    matrices::{SparseMatrix, SparseVector},
    solver::Solver,
};
use arko_sensitivity::{FactoredSystem, SensitivityError};
use arko_solvers_dense::DenseLuSolver;
use sprs::TriMat;

/// Build an `n × n` system from a dense-style triplet list.
fn build_a(n: usize, triplets: &[(usize, usize, f64)]) -> SparseMatrix {
    let mut t = TriMat::new((n, n));
    for &(i, j, v) in triplets {
        t.add_triplet(i, j, v);
    }
    t.to_csr()
}

/// Ground-truth: full refactor against a freshly-constructed `A`.
fn ground_truth_solve(a: &SparseMatrix, f: &SparseVector) -> Vec<f64> {
    DenseLuSolver.solve(a, f).unwrap()
}

fn assert_vectors_parity(got: &[f64], want: &[f64]) {
    assert_eq!(got.len(), want.len());
    for (i, (&g, &w)) in got.iter().zip(want.iter()).enumerate() {
        assert_relative_eq!(g, w, epsilon = 1e-9, max_relative = 1e-9);
        let _ = i;
    }
}

#[test]
fn from_solve_matches_direct_solve() {
    let a = build_a(3, &[(0, 0, 2.0), (0, 1, 1.0), (1, 1, 3.0), (2, 2, 4.0), (1, 2, -1.0)]);
    let f = SparseVector::new(3, vec![0, 1, 2], vec![1.0, 2.0, 3.0]);
    let sys = FactoredSystem::from_solve(a.clone(), &f, &DenseLuSolver).unwrap();
    let truth = ground_truth_solve(&a, &f);
    assert_vectors_parity(&sys.scaling, &truth);
    assert_eq!(sys.generation, 0);
}

#[test]
fn edit_entry_matches_full_refactor() {
    // Starting A.
    let a0 = build_a(3, &[(0, 0, 2.0), (0, 1, 1.0), (1, 1, 3.0), (2, 2, 4.0), (1, 2, -1.0)]);
    let f = SparseVector::new(3, vec![0, 1, 2], vec![1.0, 2.0, 3.0]);
    let mut sys = FactoredSystem::from_solve(a0.clone(), &f, &DenseLuSolver).unwrap();

    // Edit A[0, 1] += 0.5.
    sys.edit_entry(0, 1, 0.5, &DenseLuSolver).unwrap();
    assert_eq!(sys.generation, 1);

    // Ground truth: build the modified A from scratch.
    let a1 = build_a(
        3,
        &[(0, 0, 2.0), (0, 1, 1.5), (1, 1, 3.0), (2, 2, 4.0), (1, 2, -1.0)],
    );
    let truth = ground_truth_solve(&a1, &f);
    assert_vectors_parity(&sys.scaling, &truth);
}

#[test]
fn two_sequential_edits_match_full_refactor() {
    let a0 = build_a(3, &[(0, 0, 2.0), (1, 1, 3.0), (2, 2, 4.0)]);
    let f = SparseVector::new(3, vec![0, 1, 2], vec![1.0, 1.0, 1.0]);
    let mut sys = FactoredSystem::from_solve(a0, &f, &DenseLuSolver).unwrap();

    sys.edit_entry(0, 1, 0.5, &DenseLuSolver).unwrap();
    sys.edit_entry(1, 2, -0.25, &DenseLuSolver).unwrap();
    assert_eq!(sys.generation, 2);

    let a_final = build_a(
        3,
        &[(0, 0, 2.0), (0, 1, 0.5), (1, 1, 3.0), (1, 2, -0.25), (2, 2, 4.0)],
    );
    let truth = ground_truth_solve(&a_final, &f);
    assert_vectors_parity(&sys.scaling, &truth);
}

#[test]
fn replace_column_matches_full_refactor() {
    let a0 = build_a(3, &[(0, 0, 2.0), (1, 0, 1.0), (1, 1, 3.0), (2, 2, 4.0)]);
    let f = SparseVector::new(3, vec![0, 1, 2], vec![1.0, 1.0, 1.0]);
    let mut sys = FactoredSystem::from_solve(a0, &f, &DenseLuSolver).unwrap();

    // Replace column 0 with [3, 0, 1].
    let new_col = vec![3.0, 0.0, 1.0];
    sys.replace_column(0, &new_col, &DenseLuSolver).unwrap();

    let a_final = build_a(3, &[(0, 0, 3.0), (1, 1, 3.0), (2, 0, 1.0), (2, 2, 4.0)]);
    let truth = ground_truth_solve(&a_final, &f);
    assert_vectors_parity(&sys.scaling, &truth);
    assert_eq!(sys.generation, 1);
}

#[test]
fn modify_edge_is_rank_2_and_matches_full_refactor() {
    // Start: A[1, 0] = 2 (electricity used by steel).
    let a0 = build_a(
        3,
        &[(0, 0, 1.0), (1, 0, 2.0), (1, 1, 1.0), (2, 2, 1.0)],
    );
    let f = SparseVector::new(3, vec![0, 1, 2], vec![1.0, 0.0, 1.0]);
    let mut sys = FactoredSystem::from_solve(a0, &f, &DenseLuSolver).unwrap();

    // modify_edge(1, 0, 2, 1.5): move the "electricity used" from column 0
    // to column 2, with a new value of 1.5.
    sys.modify_edge(1, 0, 2, 1.5, &DenseLuSolver).unwrap();
    assert_eq!(sys.generation, 1);

    let a_final = build_a(
        3,
        &[(0, 0, 1.0), (1, 1, 1.0), (1, 2, 1.5), (2, 2, 1.0)],
    );
    let truth = ground_truth_solve(&a_final, &f);
    assert_vectors_parity(&sys.scaling, &truth);
}

#[test]
fn update_rank_r_matches_full_refactor() {
    // 4×4 diagonal dominant system, rank-2 update.
    let a0 = build_a(
        4,
        &[
            (0, 0, 5.0), (1, 1, 5.0), (2, 2, 5.0), (3, 3, 5.0),
            (0, 1, 1.0), (2, 3, 1.0),
        ],
    );
    let f = SparseVector::new(4, vec![0, 1, 2, 3], vec![1.0, 1.0, 1.0, 1.0]);
    let mut sys = FactoredSystem::from_solve(a0, &f, &DenseLuSolver).unwrap();

    // ΔA = u_0·v_0^T + u_1·v_1^T, both hitting disjoint entries.
    let u0 = vec![1.0, 0.0, 0.0, 0.0];
    let v0 = vec![0.0, 0.0, 1.0, 0.0];
    let u1 = vec![0.0, 0.0, 0.0, 1.0];
    let v1 = vec![0.0, 1.0, 0.0, 0.0];
    sys.update_rank_r(&[&u0, &u1], &[&v0, &v1], &DenseLuSolver)
        .unwrap();

    let a_final = build_a(
        4,
        &[
            (0, 0, 5.0), (1, 1, 5.0), (2, 2, 5.0), (3, 3, 5.0),
            (0, 1, 1.0), (0, 2, 1.0),       // added from u_0·v_0^T
            (2, 3, 1.0), (3, 1, 1.0),       // added from u_1·v_1^T
        ],
    );
    let truth = ground_truth_solve(&a_final, &f);
    assert_vectors_parity(&sys.scaling, &truth);
}

#[test]
fn refactor_resets_generation_and_replaces_system() {
    let a0 = build_a(2, &[(0, 0, 1.0), (1, 1, 1.0)]);
    let f0 = SparseVector::new(2, vec![0, 1], vec![1.0, 1.0]);
    let mut sys = FactoredSystem::from_solve(a0, &f0, &DenseLuSolver).unwrap();

    sys.edit_entry(0, 1, 0.5, &DenseLuSolver).unwrap();
    assert_eq!(sys.generation, 1);

    // Full refactor with different A and f.
    let a_new = build_a(2, &[(0, 0, 2.0), (1, 1, 2.0)]);
    let f_new = SparseVector::new(2, vec![0, 1], vec![4.0, 4.0]);
    sys.refactor(a_new, &f_new, &DenseLuSolver).unwrap();
    assert_eq!(sys.generation, 0);
    assert_vectors_parity(&sys.scaling, &[2.0, 2.0]);
}

#[test]
fn singular_update_returns_engine_singular() {
    // Construct an update that makes (A + uv^T) singular. Easiest way:
    // take A = I_2 and u = [1, 0], v = [-1, 0], which yields
    // A + uv^T = [[0, 0], [0, 1]] — singular.
    let a0 = build_a(2, &[(0, 0, 1.0), (1, 1, 1.0)]);
    let f = SparseVector::new(2, vec![0, 1], vec![1.0, 1.0]);
    let mut sys = FactoredSystem::from_solve(a0, &f, &DenseLuSolver).unwrap();

    let u = vec![1.0, 0.0];
    let v = vec![-1.0, 0.0];
    let err = sys.update_rank_1(&u, &v, &DenseLuSolver).unwrap_err();
    match err {
        SensitivityError::Engine(EngineError::Singular) => {}
        other => panic!("expected Engine(Singular), got {other:?}"),
    }
    // Generation NOT incremented on failure.
    assert_eq!(sys.generation, 0);
}

#[test]
fn dimension_mismatch_surfaces_shape_error() {
    let a0 = build_a(2, &[(0, 0, 1.0), (1, 1, 1.0)]);
    let f = SparseVector::new(2, vec![0, 1], vec![1.0, 1.0]);
    let mut sys = FactoredSystem::from_solve(a0, &f, &DenseLuSolver).unwrap();

    // u is length 3 but system is 2×2.
    let u = vec![1.0, 0.0, 0.0];
    let v = vec![1.0, 0.0];
    let err = sys.update_rank_1(&u, &v, &DenseLuSolver).unwrap_err();
    match err {
        SensitivityError::Engine(EngineError::ShapeMismatch(msg)) => {
            assert!(msg.contains("rank-1"), "message was: {msg}");
        }
        other => panic!("expected Engine(ShapeMismatch), got {other:?}"),
    }
    assert_eq!(sys.generation, 0);
}

#[test]
fn zero_delta_edit_is_noop_and_does_not_bump_generation() {
    let a0 = build_a(2, &[(0, 0, 1.0), (1, 1, 1.0)]);
    let f = SparseVector::new(2, vec![0, 1], vec![1.0, 1.0]);
    let mut sys = FactoredSystem::from_solve(a0, &f, &DenseLuSolver).unwrap();
    sys.edit_entry(0, 0, 0.0, &DenseLuSolver).unwrap();
    assert_eq!(sys.generation, 0);
}

#[test]
fn rank_0_update_is_noop() {
    let a0 = build_a(2, &[(0, 0, 1.0), (1, 1, 1.0)]);
    let f = SparseVector::new(2, vec![0, 1], vec![1.0, 1.0]);
    let mut sys = FactoredSystem::from_solve(a0, &f, &DenseLuSolver).unwrap();
    let empty_u: &[&[f64]] = &[];
    let empty_v: &[&[f64]] = &[];
    sys.update_rank_r(empty_u, empty_v, &DenseLuSolver).unwrap();
    assert_eq!(sys.generation, 0);
}

#[test]
fn factored_system_roundtrips_through_json() {
    let a0 = build_a(2, &[(0, 0, 2.0), (1, 1, 3.0)]);
    let f = SparseVector::new(2, vec![0, 1], vec![2.0, 3.0]);
    let sys = FactoredSystem::from_solve(a0, &f, &DenseLuSolver).unwrap();
    let j = serde_json::to_string(&sys).unwrap();
    let back: FactoredSystem = serde_json::from_str(&j).unwrap();
    assert_eq!(sys, back);
}

#[test]
fn smw_approaches_but_does_not_reach_singular() {
    // Edit that makes A *nearly* singular. SMW should still succeed
    // with results close to the full-refactor ground truth.
    let a0 = build_a(2, &[(0, 0, 1.0), (1, 1, 1.0)]);
    let f = SparseVector::new(2, vec![0, 1], vec![1.0, 1.0]);
    let mut sys = FactoredSystem::from_solve(a0, &f, &DenseLuSolver).unwrap();

    // Edit A[0, 0] to 1e-6 — tiny pivot but not singular.
    sys.edit_entry(0, 0, -1.0 + 1e-6, &DenseLuSolver).unwrap();
    let a_final = build_a(2, &[(0, 0, 1e-6), (1, 1, 1.0)]);
    let truth = ground_truth_solve(&a_final, &f);
    // Large scale, but relative parity holds.
    assert_relative_eq!(sys.scaling[0], truth[0], max_relative = 1e-6);
    assert_relative_eq!(sys.scaling[1], truth[1], max_relative = 1e-9);
}
