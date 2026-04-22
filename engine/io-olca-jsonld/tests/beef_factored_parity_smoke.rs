//! Cross-implementation parity smoke for `FactoredSolver` on the USDA
//! LCA Commons beef cattle finishing bundle — closes the
//! factor-once-solve-many path.
//!
//! # Why this test exists
//!
//! [`beef_multi_process_parity_smoke`][crate::beef_multi_process_parity_smoke]
//! closed the **single-shot** LU path: one `(A, f)` pair, one solve,
//! Arko vs `numpy.linalg.solve`. That test pinned the numerics of the
//! one-shot kernel.
//!
//! `FactoredSolver` introduces a second surface — factor `A` once, then
//! solve many right-hand sides reusing the cached LU. The numerical
//! kernel is shared with the one-shot path by construction
//! (`Solver::solve` delegates to `factorize().solve()`), but the
//! observable behaviour of "factor once, solve N times" needs its own
//! parity coverage so that future refactors of either surface can't
//! silently diverge them.
//!
//! # Independence posture
//!
//! Transitive parity (no new Python script needed):
//!
//! - **Single-shot path vs numpy:** already proven by
//!   [`beef_multi_process_parity_smoke`][crate::beef_multi_process_parity_smoke]
//!   (Arko `DenseLuSolver::solve` ≈ `numpy.linalg.solve` on the same
//!   beef A).
//! - **Factored path vs single-shot:** this test. For each `e_k`
//!   column of the 5×5 identity, solve `A · s_k = e_k` both ways and
//!   compare per-column with [`ToleranceClass::CrossImpl`].
//! - **Factored path vs numpy (transitive):** if factored ≈
//!   single-shot and single-shot ≈ numpy, then factored ≈ numpy.
//!   No third reference computation is needed.
//!
//! Reconstructing each column of `A⁻¹` via 5 solves is the canonical
//! workload that motivates `FactoredSolver` in the first place
//! (sensitivity sweeps, Monte Carlo, the §10 incremental recalc
//! path) — so this is the right exemplar shape for the parity smoke.
//!
//! # Residual sanity check
//!
//! In addition to the per-column factored-vs-single-shot comparison,
//! the test verifies `A · s_k ≈ e_k` for each reconstructed column.
//! The residual check is independent of whichever solver produced
//! `s_k`; if both surfaces silently regressed in the same way (e.g., a
//! shared upstream nalgebra bug) the residual would still surface it.
//!
//! # Tolerance class
//!
//! - **Per-column factored-vs-single-shot:** absolute equality at
//!   `1e-15` (epsilon of `f64`). The two paths share one numerical
//!   kernel — there is no algorithmic source of divergence. Any
//!   nonzero gap means the surfaces have actually diverged.
//! - **Per-column residual `A · s_k = e_k`:**
//!   [`ToleranceClass::CrossImpl`] (`ε_abs = 1e-9`, `ε_rel = 1e-6`).
//!   This is the same class the single-shot smoke uses for its h
//!   comparison; tightening would over-claim against the LU pivot
//!   ordering's intrinsic floating-point rounding.
//!
//! # How to run
//!
//! ```bash
//! USDA_BEEF_BUNDLE=/path/to/USDA_Beef cargo test -p arko-io-olca-jsonld \
//!     --test beef_factored_parity_smoke -- --ignored --nocapture
//! ```

use std::collections::BTreeMap;
use std::path::PathBuf;

use arko_core::{
    matrices::{SparseMatrix, SparseVector},
    solver::Solver,
};
use arko_io_ilcd_linker::FlowType;
use arko_io_olca_jsonld::{olca_to_typed_column, OlcaBundle};
use arko_solvers_dense::{DenseLuSolver, FactoredSolver};
use sprs::TriMat;

const BUNDLE_ENV_VAR: &str = "USDA_BEEF_BUNDLE";

/// Tolerance for per-column factored-vs-single-shot equality. The two
/// surfaces share one numerical kernel by construction (`Solver::solve`
/// delegates to `factorize().solve()`), so any divergence at all is a
/// regression — not a numerical artefact. Absolute equality at the
/// `f64` epsilon level.
const FACTORED_VS_SINGLE_SHOT_EPSILON: f64 = 1e-15;

/// Absolute tolerance for the residual check `A · s_k ≈ e_k`.
/// Matches the single-shot smoke's `ToleranceClass::CrossImpl`
/// `eps_abs` so the two smokes report comparable numerical headroom.
const RESIDUAL_EPS_ABS: f64 = 1e-9;
/// Relative tolerance for the residual check (matches `CrossImpl`
/// `eps_rel`).
const RESIDUAL_EPS_REL: f64 = 1e-6;

#[test]
#[ignore = "requires USDA_BEEF_BUNDLE env var pointing at an unpacked beef bundle"]
fn beef_factored_parity_smoke() {
    let bundle_root = std::env::var(BUNDLE_ENV_VAR).unwrap_or_else(|_| {
        panic!(
            "{BUNDLE_ENV_VAR} not set; point it at an unpacked USDA beef bundle \
             (see engine/io-olca-jsonld/tests/beef_bundle_smoke.rs for layout)"
        )
    });
    let bundle_root = PathBuf::from(bundle_root);

    let bundle = OlcaBundle::open(&bundle_root).expect("open bundle");
    let uuids: Vec<String> = bundle.process_uuids();
    let n = uuids.len();
    assert_eq!(n, 5, "beef bundle should have exactly 5 processes");
    let col: BTreeMap<String, usize> = uuids
        .iter()
        .enumerate()
        .map(|(i, u)| (u.clone(), i))
        .collect();

    // Build A using the same pipeline the existing smoke uses, so the
    // matrix factored here is bit-identical to the matrix solved in
    // `beef_multi_process_parity_smoke`. We don't need B / impacts /
    // method here — A alone is what `FactoredSolver` operates on.
    let columns: Vec<_> = uuids
        .iter()
        .map(|u| {
            let p = bundle.load_process(u).expect("load process");
            let c = olca_to_typed_column(&p, &bundle).expect("adapt");
            (p, c)
        })
        .collect();

    let mut ref_flow_uuid = vec![String::new(); n];
    let mut a = TriMat::new((n, n));

    for (j, (_p, c)) in columns.iter().enumerate() {
        let ref_ex = c
            .exchanges
            .iter()
            .find(|e| e.is_reference_flow)
            .expect("reference exchange present");
        ref_flow_uuid[j] = ref_ex.flow_uuid.clone();
        let sign = if ref_ex.direction.is_input() { -1.0 } else { 1.0 };
        a.add_triplet(j, j, sign * ref_ex.amount);
    }
    for (j, (p, c)) in columns.iter().enumerate() {
        for typed in &c.exchanges {
            if typed.is_reference_flow {
                continue;
            }
            let olca = p
                .exchanges
                .iter()
                .find(|e| e.internal_id == typed.data_set_internal_id)
                .expect("typed-to-olca internal_id join");
            if matches!(typed.flow_type, FlowType::Product | FlowType::Waste) {
                if let Some(ref dp) = olca.default_provider_uuid {
                    if let Some(&prov_col) = col.get(dp) {
                        assert_eq!(
                            typed.flow_uuid, ref_flow_uuid[prov_col],
                            "exchange flow must match provider's ref product"
                        );
                        let sign = if typed.direction.is_input() { -1.0 } else { 1.0 };
                        a.add_triplet(prov_col, j, sign * typed.amount);
                    }
                }
            }
            // Elementary exchanges land in B, not A — irrelevant for this test.
        }
    }
    let technosphere: SparseMatrix = a.to_csr();

    // Factor once. The `cargo test --nocapture` output reports timings
    // separately for the factorization and each solve so the
    // amortisation is observable in the log.
    let t_factor_start = std::time::Instant::now();
    let factored = DenseLuSolver
        .factorize(&technosphere)
        .expect("factorize beef A");
    let factor_us = t_factor_start.elapsed().as_micros();
    assert_eq!(factored.dim(), n, "cached dim must match A's row count");

    // For each column k of I_n, solve A · s_k = e_k two ways and
    // compare. The factored surface uses the cached LU (one
    // factorisation, n triangular solves); the single-shot surface
    // re-factorizes for every solve (n factorisations + n triangular
    // solves) — same numerical answer, dramatically different work.
    let mut total_factored_solve_us: u128 = 0;
    let mut total_single_shot_us: u128 = 0;
    let mut max_factored_vs_single: f64 = 0.0;
    let mut max_residual_abs: f64 = 0.0;
    let mut a_inv_columns: Vec<Vec<f64>> = Vec::with_capacity(n);

    for k in 0..n {
        let e_k = SparseVector::new(n, vec![k], vec![1.0]);

        let t = std::time::Instant::now();
        let s_factored = factored
            .solve(&e_k)
            .unwrap_or_else(|err| panic!("factored solve failed for e_{k}: {err}"));
        total_factored_solve_us += t.elapsed().as_micros();

        let t = std::time::Instant::now();
        let s_single = DenseLuSolver
            .solve(&technosphere, &e_k)
            .unwrap_or_else(|err| panic!("single-shot solve failed for e_{k}: {err}"));
        total_single_shot_us += t.elapsed().as_micros();

        assert_eq!(s_factored.len(), n);
        assert_eq!(s_single.len(), n);

        // Per-element factored-vs-single-shot equality. Surfaces
        // share a kernel; any nonzero gap is a regression.
        for i in 0..n {
            let dev = (s_factored[i] - s_single[i]).abs();
            if dev > max_factored_vs_single {
                max_factored_vs_single = dev;
            }
            assert!(
                dev <= FACTORED_VS_SINGLE_SHOT_EPSILON,
                "factored vs single-shot diverged at row {i} of e_{k} solve: \
                 |dev|={dev:e} (allowed {FACTORED_VS_SINGLE_SHOT_EPSILON:e})"
            );
        }

        // Residual check: A · s_factored ≈ e_k. Done by hand against
        // the sparse A so this assertion is independent of whichever
        // solver produced s_factored.
        for row in 0..n {
            let row_view = technosphere.outer_view(row).unwrap();
            let mut acc = 0.0;
            for (col_idx, &val) in row_view.iter() {
                acc += val * s_factored[col_idx];
            }
            let target = if row == k { 1.0 } else { 0.0 };
            let abs = (acc - target).abs();
            let rel = abs / target.abs().max(1.0);
            if abs > max_residual_abs {
                max_residual_abs = abs;
            }
            assert!(
                abs <= RESIDUAL_EPS_ABS || rel <= RESIDUAL_EPS_REL,
                "A · s_{k} != e_{k} at row {row}: got {acc}, want {target}, \
                 |dev|={abs:e}, rel={rel:e} (CrossImpl tolerance)"
            );
        }

        a_inv_columns.push(s_factored);
    }

    println!("--- beef_factored_parity_smoke ---");
    println!("bundle:                      {}", bundle_root.display());
    println!("processes (n):               {n}");
    println!("factor time:                 {factor_us} µs");
    println!(
        "factored solve total ({n}):     {total_factored_solve_us} µs ({} µs avg)",
        total_factored_solve_us / n as u128
    );
    println!(
        "single-shot solve total ({n}):  {total_single_shot_us} µs ({} µs avg)",
        total_single_shot_us / n as u128
    );
    let amortisation = if total_factored_solve_us > 0 {
        total_single_shot_us as f64 / total_factored_solve_us.max(1) as f64
    } else {
        f64::NAN
    };
    println!("speedup factor (single/factored): {amortisation:.2}x");
    println!("max |dev| factored vs single-shot:  {max_factored_vs_single:e}");
    println!("max |dev| residual A·s_k = e_k:     {max_residual_abs:e}");
    println!(
        "tolerance factored-vs-single-shot:  {FACTORED_VS_SINGLE_SHOT_EPSILON:e} (absolute)"
    );
    println!(
        "tolerance residual:                 eps_abs={RESIDUAL_EPS_ABS:e}, eps_rel={RESIDUAL_EPS_REL:e}"
    );
    println!("VERDICT:                            PASS");
    println!("--- end beef_factored_parity_smoke ---");

    // Final structural sanity: the reconstructed A⁻¹ has n columns of
    // n entries each. Catches a silent length regression in
    // `DenseLuFactorization::solve` that wouldn't show up in the
    // per-element comparisons above (they assert per-row, but only
    // for `0..n`).
    assert_eq!(a_inv_columns.len(), n);
    for col in &a_inv_columns {
        assert_eq!(col.len(), n);
    }
}
