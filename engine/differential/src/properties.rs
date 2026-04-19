//! Property tests — spec §14.3.
//!
//! These are *universal* properties every conforming engine must
//! satisfy, regardless of the specific vector it is solving. Unlike the
//! per-vector runner (→ `runner.rs`), the property tests take a **study
//! generator or a specific study** and assert invariants hold.
//!
//! Four of the five §14.3 properties are implemented here:
//!
//! 1. **Scaling identity** — after `compute`, `A·s` must equal `f`
//!    componentwise within §8.1 reference-parity tolerance.
//! 2. **Idempotent recompute** — computing the same study twice must
//!    produce bit-identical `(s, g, h)` per §7.1 determinism.
//! 3. **Block-diagonal independence** — stacking two independent studies
//!    into one block-diagonal study must yield the concatenation of the
//!    individual solutions.
//! 4. **Sherman-Morrison parity** — an incremental update via
//!    `arko-sensitivity` must match a full refactor to within §8.1.
//!
//! The fifth §14.3 property — **parameter-rewrite equivalence** — is
//! deferred to v0.2 because it requires parameter-expression
//! evaluation wired into `pipeline::compute`, which is not yet in
//! `arko-core`. See `engine/CHANGELOG.md` Pending.

use arko_core::{
    error::EngineError,
    matrices::{SparseMatrix, SparseVector},
    pipeline::compute,
    solver::Solver,
    study::Study,
};
use arko_sensitivity::FactoredSystem;
use sprs::TriMat;

/// Property-test outcomes. A `Pass` variant is implicit (we return
/// `Result<(), PropertyFailure>` with `Ok(())` meaning pass).
#[derive(Debug, thiserror::Error)]
pub enum PropertyFailure {
    /// The property's pre-condition failed before we could check the
    /// invariant itself — e.g., the engine returned an error on a
    /// study the property expected to be solvable.
    #[error("property precondition failed: {0}")]
    Precondition(String),

    /// The engine returned an `EngineError` during property evaluation.
    #[error("engine error during property check: {0}")]
    Engine(#[from] EngineError),

    /// The numeric invariant was violated. `max_dev` is the worst
    /// componentwise absolute deviation observed; `tol` is the
    /// tolerance the property applied.
    #[error(
        "numeric invariant violated at component {index}: got {got}, want {want}, \
         |dev| = {max_dev}, tol = {tol}"
    )]
    NumericViolation {
        index: usize,
        got: f64,
        want: f64,
        max_dev: f64,
        tol: f64,
    },

    /// A structural invariant (shape, ordering) was violated.
    #[error("structural invariant violated: {0}")]
    Structural(String),
}

/// `EPS_ABS` for property checks — matches `ReferenceParity` (§8.1).
const EPS_ABS: f64 = 1e-12;
/// `EPS_REL` for property checks — matches `ReferenceParity` (§8.1).
const EPS_REL: f64 = 1e-9;

/// Per-component tolerance: `max(EPS_ABS, EPS_REL * |want|)`.
fn tol_for(want: f64) -> f64 {
    f64::max(EPS_ABS, EPS_REL * want.abs())
}

/// **§14.3 Property 1** — after `compute`, `A·s` must equal `f` within
/// reference-parity tolerance.
///
/// This is the correctness of the solve itself, independent of B or C.
pub fn check_scaling_identity<S: Solver>(study: &Study, solver: &S) -> Result<(), PropertyFailure> {
    let computed = compute(study, solver).map_err(PropertyFailure::Engine)?;

    // Dense f from the sparse functional unit.
    let n = study.n_processes();
    let mut f_dense = vec![0.0_f64; n];
    for (&i, &v) in study
        .functional_unit
        .indices()
        .iter()
        .zip(study.functional_unit.data().iter())
    {
        f_dense[i] = v;
    }

    // Compute A·s in place (row-major).
    let mut as_vec = vec![0.0_f64; n];
    for (row_idx, row) in study.technosphere.outer_iterator().enumerate() {
        let mut acc = 0.0_f64;
        for (col_idx, &val) in row.iter() {
            acc += val * computed.scaling[col_idx];
        }
        as_vec[row_idx] = acc;
    }

    for (i, (got, want)) in as_vec.iter().zip(f_dense.iter()).enumerate() {
        let dev = (got - want).abs();
        let tol = tol_for(*want);
        if dev > tol {
            return Err(PropertyFailure::NumericViolation {
                index: i,
                got: *got,
                want: *want,
                max_dev: dev,
                tol,
            });
        }
    }
    Ok(())
}

/// **§14.3 Property 2** — computing the same study twice produces
/// bit-identical `(s, g, h)`. Required by §7.1 determinism.
///
/// We assert strict equality (not approximate) because determinism
/// means *identical floats*, not "within a tolerance."
pub fn check_idempotent_recompute<S: Solver>(
    study: &Study,
    solver: &S,
) -> Result<(), PropertyFailure> {
    let a = compute(study, solver).map_err(PropertyFailure::Engine)?;
    let b = compute(study, solver).map_err(PropertyFailure::Engine)?;

    for (i, (x, y)) in a.scaling.iter().zip(b.scaling.iter()).enumerate() {
        if x.to_bits() != y.to_bits() {
            return Err(PropertyFailure::NumericViolation {
                index: i,
                got: *y,
                want: *x,
                max_dev: (x - y).abs(),
                tol: 0.0,
            });
        }
    }
    for (i, (x, y)) in a.inventory.iter().zip(b.inventory.iter()).enumerate() {
        if x.to_bits() != y.to_bits() {
            return Err(PropertyFailure::NumericViolation {
                index: i,
                got: *y,
                want: *x,
                max_dev: (x - y).abs(),
                tol: 0.0,
            });
        }
    }
    for (i, (x, y)) in a.impact.iter().zip(b.impact.iter()).enumerate() {
        if x.to_bits() != y.to_bits() {
            return Err(PropertyFailure::NumericViolation {
                index: i,
                got: *y,
                want: *x,
                max_dev: (x - y).abs(),
                tol: 0.0,
            });
        }
    }
    Ok(())
}

/// **§14.3 Property 3** — block-diagonal independence.
///
/// Given two studies `s1` and `s2` that share no processes or flows,
/// stacking them into a single block-diagonal study must produce a
/// scaling vector equal to `concat(s1.scaling, s2.scaling)`.
///
/// Both input studies must be independently solvable and must share the
/// same `ImpactMeta` list (so the resulting `h` is meaningful). We do
/// not require that `C` participate — this property is about the
/// technosphere block structure.
pub fn check_block_diagonal_independence<S: Solver>(
    s1: &Study,
    s2: &Study,
    solver: &S,
) -> Result<(), PropertyFailure> {
    let c1 = compute(s1, solver).map_err(PropertyFailure::Engine)?;
    let c2 = compute(s2, solver).map_err(PropertyFailure::Engine)?;

    // Build the block-diagonal study.
    let combined = block_diagonal(s1, s2).map_err(PropertyFailure::Structural)?;
    let c_combined = compute(&combined, solver).map_err(PropertyFailure::Engine)?;

    let n1 = s1.n_processes();
    let n2 = s2.n_processes();
    let expected_n = n1 + n2;
    if c_combined.scaling.len() != expected_n {
        return Err(PropertyFailure::Structural(format!(
            "combined scaling length {} != {n1} + {n2} = {expected_n}",
            c_combined.scaling.len()
        )));
    }

    // Compare the two halves.
    for (i, (got, want)) in c_combined
        .scaling
        .iter()
        .take(n1)
        .zip(c1.scaling.iter())
        .enumerate()
    {
        let dev = (got - want).abs();
        let tol = tol_for(*want);
        if dev > tol {
            return Err(PropertyFailure::NumericViolation {
                index: i,
                got: *got,
                want: *want,
                max_dev: dev,
                tol,
            });
        }
    }
    for (j, (got, want)) in c_combined
        .scaling
        .iter()
        .skip(n1)
        .zip(c2.scaling.iter())
        .enumerate()
    {
        let dev = (got - want).abs();
        let tol = tol_for(*want);
        if dev > tol {
            return Err(PropertyFailure::NumericViolation {
                index: n1 + j,
                got: *got,
                want: *want,
                max_dev: dev,
                tol,
            });
        }
    }
    Ok(())
}

/// **§14.3 Property 4** — Sherman-Morrison-Woodbury parity.
///
/// Given a base study and a rank-1 update `ΔA = u·v^T`, compare the
/// `arko-sensitivity` `update_rank_1` path against a full refactor of
/// `A + u·v^T` from scratch. They must agree within §8.1.
pub fn check_sherman_morrison_parity<S: Solver>(
    study: &Study,
    u: &[f64],
    v: &[f64],
    solver: &S,
) -> Result<(), PropertyFailure> {
    let n = study.n_processes();
    if u.len() != n || v.len() != n {
        return Err(PropertyFailure::Structural(format!(
            "SMW parity: u.len()={} or v.len()={} != n={n}",
            u.len(),
            v.len()
        )));
    }

    // Incremental path: factor, then SMW-update.
    let mut sys =
        FactoredSystem::from_solve(study.technosphere.clone(), &study.functional_unit, solver)
            .map_err(|e| match e {
                arko_sensitivity::SensitivityError::Engine(ee) => PropertyFailure::Engine(ee),
            })?;
    sys.update_rank_1(u, v, solver).map_err(|e| match e {
        arko_sensitivity::SensitivityError::Engine(ee) => PropertyFailure::Engine(ee),
    })?;
    let incremental = sys.scaling.clone();

    // Refactor path: build A + u·v^T as a fresh sparse matrix and solve.
    let refactored_a = add_rank_1_dense(&study.technosphere, u, v);
    let fresh = solver
        .solve(&refactored_a, &study.functional_unit)
        .map_err(PropertyFailure::Engine)?;

    for (i, (got, want)) in incremental.iter().zip(fresh.iter()).enumerate() {
        let dev = (got - want).abs();
        let tol = tol_for(*want);
        if dev > tol {
            return Err(PropertyFailure::NumericViolation {
                index: i,
                got: *got,
                want: *want,
                max_dev: dev,
                tol,
            });
        }
    }
    Ok(())
}

// --- helpers ----------------------------------------------------------

/// Build the block-diagonal combination of two studies' technosphere
/// matrices, stacking `f`, B, C, and all metadata in declaration order.
///
/// Returns a plain string error on shape inconsistencies; the caller
/// wraps it into `PropertyFailure::Structural`.
fn block_diagonal(s1: &Study, s2: &Study) -> Result<Study, String> {
    let n1 = s1.n_processes();
    let n2 = s2.n_processes();
    let m1 = s1.n_flows();
    let m2 = s2.n_flows();
    let k1 = s1.n_impacts();
    let k2 = s2.n_impacts();

    if k1 != k2 {
        return Err(format!("impact dimensions differ: {k1} vs {k2}"));
    }

    // A_combined (n1+n2) × (n1+n2).
    let mut a_triplets: Vec<(usize, usize, f64)> = Vec::new();
    for (row, row_view) in s1.technosphere.outer_iterator().enumerate() {
        for (col, &v) in row_view.iter() {
            a_triplets.push((row, col, v));
        }
    }
    for (row, row_view) in s2.technosphere.outer_iterator().enumerate() {
        for (col, &v) in row_view.iter() {
            a_triplets.push((row + n1, col + n1, v));
        }
    }
    a_triplets.sort_by(|a, b| (a.0, a.1).cmp(&(b.0, b.1)));
    let mut a_tri = TriMat::new((n1 + n2, n1 + n2));
    for (i, j, v) in a_triplets {
        a_tri.add_triplet(i, j, v);
    }
    let technosphere: SparseMatrix = a_tri.to_csr();

    // B_combined (m1+m2) × (n1+n2).
    let mut b_triplets: Vec<(usize, usize, f64)> = Vec::new();
    for (row, row_view) in s1.biosphere.outer_iterator().enumerate() {
        for (col, &v) in row_view.iter() {
            b_triplets.push((row, col, v));
        }
    }
    for (row, row_view) in s2.biosphere.outer_iterator().enumerate() {
        for (col, &v) in row_view.iter() {
            b_triplets.push((row + m1, col + n1, v));
        }
    }
    b_triplets.sort_by(|a, b| (a.0, a.1).cmp(&(b.0, b.1)));
    let mut b_tri = TriMat::new((m1 + m2, n1 + n2));
    for (i, j, v) in b_triplets {
        b_tri.add_triplet(i, j, v);
    }
    let biosphere: SparseMatrix = b_tri.to_csr();

    // C_combined: k × (m1+m2). We *horizontally concatenate* the two C
    // blocks, putting s1's C in columns 0..m1 and s2's C in columns
    // m1..(m1+m2). Rows stay at k (impact dimension is shared).
    let mut c_triplets: Vec<(usize, usize, f64)> = Vec::new();
    for (row, row_view) in s1.characterization.outer_iterator().enumerate() {
        for (col, &v) in row_view.iter() {
            c_triplets.push((row, col, v));
        }
    }
    for (row, row_view) in s2.characterization.outer_iterator().enumerate() {
        for (col, &v) in row_view.iter() {
            c_triplets.push((row, col + m1, v));
        }
    }
    c_triplets.sort_by(|a, b| (a.0, a.1).cmp(&(b.0, b.1)));
    let mut c_tri = TriMat::new((k1, m1 + m2));
    for (i, j, v) in c_triplets {
        c_tri.add_triplet(i, j, v);
    }
    let characterization: SparseMatrix = c_tri.to_csr();

    // f_combined — concat f1 and f2 by shifting s2's indices by n1.
    let mut fu_idx: Vec<usize> = Vec::new();
    let mut fu_data: Vec<f64> = Vec::new();
    for (&i, &v) in s1
        .functional_unit
        .indices()
        .iter()
        .zip(s1.functional_unit.data().iter())
    {
        fu_idx.push(i);
        fu_data.push(v);
    }
    for (&i, &v) in s2
        .functional_unit
        .indices()
        .iter()
        .zip(s2.functional_unit.data().iter())
    {
        fu_idx.push(i + n1);
        fu_data.push(v);
    }
    let functional_unit = SparseVector::new(n1 + n2, fu_idx, fu_data);

    // Metadata: concat in declaration order. License tiers are merged
    // and s2's process refs are offset by s1's tier count.
    let tier_offset = s1.license_tiers.len() as u32;
    let mut processes = s1.processes.clone();
    for p in &s2.processes {
        let mut p = p.clone();
        p.license_tier = arko_core::meta::LicenseTierRef(p.license_tier.0 + tier_offset);
        processes.push(p);
    }

    let mut flows = s1.flows.clone();
    flows.extend(s2.flows.iter().cloned());

    let impacts = s1.impacts.clone(); // k1 == k2 checked above

    let mut license_tiers = s1.license_tiers.clone();
    license_tiers.extend(s2.license_tiers.iter().cloned());

    let mut parameters = s1.parameters.clone();
    parameters.extend(s2.parameters.iter().cloned());

    Ok(Study {
        technosphere,
        biosphere,
        characterization,
        functional_unit,
        processes,
        flows,
        impacts,
        parameters,
        license_tiers,
        method: s1.method.clone(),
        sign_convention: s1.sign_convention,
    })
}

/// Rebuild `A + u·v^T` as a fresh CSR matrix. Mirrors the internal
/// helper in `arko-sensitivity` but duplicated here so we do not rely
/// on a private API from a sibling crate.
fn add_rank_1_dense(a: &SparseMatrix, u: &[f64], v: &[f64]) -> SparseMatrix {
    let (rows, cols) = a.shape();
    let mut triplets: Vec<(usize, usize, f64)> = Vec::new();

    for (row_idx, row) in a.outer_iterator().enumerate() {
        for (col_idx, &val) in row.iter() {
            triplets.push((row_idx, col_idx, val));
        }
    }
    for (i, &ui) in u.iter().enumerate() {
        if ui == 0.0 {
            continue;
        }
        for (j, &vj) in v.iter().enumerate() {
            if vj == 0.0 {
                continue;
            }
            triplets.push((i, j, ui * vj));
        }
    }
    triplets.sort_by(|a, b| (a.0, a.1).cmp(&(b.0, b.1)));

    let mut tri = TriMat::new((rows, cols));
    for (i, j, val) in triplets {
        tri.add_triplet(i, j, val);
    }
    tri.to_csr()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tol_for_uses_absolute_floor_at_zero() {
        assert_eq!(tol_for(0.0), EPS_ABS);
    }

    #[test]
    fn tol_for_scales_with_want() {
        // |want| = 1000 → rel term 1e-9 * 1000 = 1e-6 dominates.
        assert!((tol_for(1000.0) - 1e-6).abs() < 1e-18);
    }
}
