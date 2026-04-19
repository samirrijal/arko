//! §14.3 property tests exercised against the reference engine.
//!
//! Each test takes a seed study, runs it through one property check,
//! and asserts the property holds. Failing any of these means the
//! reference engine violates an invariant every conforming engine is
//! required to preserve.

use arko_differential::{
    check_block_diagonal_independence, check_idempotent_recompute, check_scaling_identity,
    check_sherman_morrison_parity, seed_vectors,
};
use arko_solvers_dense::DenseLuSolver;

#[test]
fn scaling_identity_holds_on_every_seed_vector() {
    for v in seed_vectors() {
        check_scaling_identity(&v.study, &DenseLuSolver)
            .unwrap_or_else(|e| panic!("scaling identity failed on vector {}: {e}", v.id));
    }
}

#[test]
fn idempotent_recompute_holds_on_every_seed_vector() {
    for v in seed_vectors() {
        check_idempotent_recompute(&v.study, &DenseLuSolver)
            .unwrap_or_else(|e| panic!("idempotent recompute failed on vector {}: {e}", v.id));
    }
}

#[test]
fn block_diagonal_independence_holds_for_two_seed_studies() {
    // Pair two studies that share the same impact list (GWP100) — required
    // by the property. `l1_identity_single_impact` and
    // `l1_coupled_two_process` both carry the single gwp100 row.
    let vs = seed_vectors();
    let s1 = &vs.iter().find(|v| v.id == "l1_identity_single_impact").unwrap().study;
    let s2 = &vs.iter().find(|v| v.id == "l1_coupled_two_process").unwrap().study;
    check_block_diagonal_independence(s1, s2, &DenseLuSolver)
        .expect("block-diagonal independence must hold for two independent studies");
}

#[test]
fn sherman_morrison_parity_holds_for_a_small_rank_1_edit() {
    // Pick a seed whose A is non-trivial — the coupled 2-process study.
    let vs = seed_vectors();
    let v = vs
        .iter()
        .find(|v| v.id == "l1_coupled_two_process")
        .expect("coupled seed must exist");

    // Rank-1 edit: ΔA[0, 1] -= 0.1  (tightens the coupling from -0.5
    // to -0.6). u = -0.1·e_0, v = e_1.
    let n = v.study.n_processes();
    let mut u = vec![0.0_f64; n];
    u[0] = -0.1;
    let mut v_vec = vec![0.0_f64; n];
    v_vec[1] = 1.0;

    check_sherman_morrison_parity(&v.study, &u, &v_vec, &DenseLuSolver)
        .expect("SMW rank-1 update must match full refactor");
}
