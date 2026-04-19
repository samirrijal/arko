//! Incremental updates — rank-1, rank-r, and the three convenience
//! entry points required by §10.1 (single-parameter edit, column
//! replacement, edge add/remove).

use crate::factored::{dense_to_sparse_vector, dot, FactoredSystem, SensitivityError};
use arko_core::{
    error::EngineError,
    matrices::{DenseVector, SparseMatrix},
    solver::Solver,
};
use nalgebra::{DMatrix, DVector};
use sprs::TriMat;

/// Threshold for detecting a singular SMW update: `|det(M)| < SMW_EPS`
/// → `EngineError::Singular`. Tight enough to not false-positive on
/// §8.1 reference-parity tolerance problems, loose enough that a
/// truly-singular update surfaces instead of silently NaN-propagating.
pub const SMW_EPS: f64 = 1e-12;

impl FactoredSystem {
    /// Rank-1 Sherman-Morrison update.
    ///
    /// Applies `ΔA = u · v^T` to `A` and updates `s` in place.
    /// Increments the staleness generation on success.
    ///
    /// # Errors
    ///
    /// - `EngineError::ShapeMismatch` if `u.len() != n` or `v.len() != n`.
    /// - `EngineError::Singular` if `|1 + v^T·A⁻¹·u| < SMW_EPS`.
    pub fn update_rank_1<S: Solver>(
        &mut self,
        u: &[f64],
        v: &[f64],
        solver: &S,
    ) -> Result<(), SensitivityError> {
        let n = self.n();
        if u.len() != n || v.len() != n {
            return Err(EngineError::ShapeMismatch(format!(
                "rank-1 update: expected length {n}, got u.len()={}, v.len()={}",
                u.len(),
                v.len()
            ))
            .into());
        }

        // x = A⁻¹·u
        let u_sv = dense_to_sparse_vector(u);
        let x = solver.solve(&self.a, &u_sv)?;

        // m = 1 + v^T·x  (scalar)
        let m = 1.0 + dot(v, &x);
        if m.abs() < SMW_EPS {
            return Err(EngineError::Singular.into());
        }

        // coeff = (v^T·s) / m
        let vs = dot(v, &self.scaling);
        let coeff = vs / m;

        // s_new = s - coeff·x
        for i in 0..n {
            self.scaling[i] -= coeff * x[i];
        }

        // A_new = A + u·v^T  (sparse rebuild, deterministic triplet order)
        self.a = add_rank_r_to_sparse(&self.a, &[u], &[v]);
        self.generation += 1;
        Ok(())
    }

    /// Rank-r Sherman-Morrison-Woodbury update for `r >= 1`.
    ///
    /// Applies `ΔA = Σ_k u_k · v_k^T` to `A` and updates `s` in place.
    /// For `r = 0` this is a no-op that does not bump the generation
    /// counter. For `r = 1` it delegates to `update_rank_1`.
    ///
    /// # Errors
    ///
    /// - `EngineError::ShapeMismatch` if `u.len() != v.len()` or any
    ///   vector has length `!= n`.
    /// - `EngineError::Singular` if the `r × r` capacitance matrix
    ///   `M = I + V^T·A⁻¹·U` is numerically singular.
    pub fn update_rank_r<S: Solver>(
        &mut self,
        u: &[&[f64]],
        v: &[&[f64]],
        solver: &S,
    ) -> Result<(), SensitivityError> {
        let r = u.len();
        if r != v.len() {
            return Err(EngineError::ShapeMismatch(format!(
                "rank-r update: {r} u-vectors but {} v-vectors",
                v.len()
            ))
            .into());
        }
        if r == 0 {
            return Ok(());
        }
        if r == 1 {
            return self.update_rank_1(u[0], v[0], solver);
        }

        let n = self.n();
        for (k, col) in u.iter().enumerate() {
            if col.len() != n {
                return Err(EngineError::ShapeMismatch(format!(
                    "rank-r update: u[{k}].len() = {}, expected {n}",
                    col.len()
                ))
                .into());
            }
        }
        for (k, col) in v.iter().enumerate() {
            if col.len() != n {
                return Err(EngineError::ShapeMismatch(format!(
                    "rank-r update: v[{k}].len() = {}, expected {n}",
                    col.len()
                ))
                .into());
            }
        }

        // X = [x_0 … x_{r-1}], each x_k = A⁻¹·u_k.
        let mut x_cols: Vec<DenseVector> = Vec::with_capacity(r);
        for u_k in u {
            let u_sv = dense_to_sparse_vector(u_k);
            let x_k = solver.solve(&self.a, &u_sv)?;
            x_cols.push(x_k);
        }

        // M = I + V^T·X  (r × r)
        let mut m = DMatrix::<f64>::identity(r, r);
        for i in 0..r {
            for j in 0..r {
                m[(i, j)] += dot(v[i], &x_cols[j]);
            }
        }

        // y = V^T·s  (length r)
        let y = DVector::<f64>::from_iterator(r, v.iter().map(|v_k| dot(v_k, &self.scaling)));

        // z = M⁻¹·y. Check determinant before trusting LU::solve — LU
        // on a near-singular matrix will happily return a nonsense
        // answer rather than None.
        let lu = m.clone().lu();
        if lu.determinant().abs() < SMW_EPS {
            return Err(EngineError::Singular.into());
        }
        let z = lu.solve(&y).ok_or(EngineError::Singular)?;

        // s_new = s - X·z
        for i in 0..n {
            let mut delta = 0.0;
            for k in 0..r {
                delta += z[k] * x_cols[k][i];
            }
            self.scaling[i] -= delta;
        }

        // A_new = A + Σ u_k·v_k^T
        self.a = add_rank_r_to_sparse(&self.a, u, v);
        self.generation += 1;
        Ok(())
    }

    /// Convenience: edit a single entry `A[i, j] += delta`.
    /// Equivalent to a rank-1 update with `u = delta·e_i` and `v = e_j`.
    pub fn edit_entry<S: Solver>(
        &mut self,
        i: usize,
        j: usize,
        delta: f64,
        solver: &S,
    ) -> Result<(), SensitivityError> {
        let n = self.n();
        if i >= n || j >= n {
            return Err(EngineError::ShapeMismatch(format!(
                "edit_entry: ({i}, {j}) out of range for {n}×{n} matrix"
            ))
            .into());
        }
        if delta == 0.0 {
            return Ok(()); // No-op, do not bump generation.
        }
        let mut u = vec![0.0; n];
        u[i] = delta;
        let mut v = vec![0.0; n];
        v[j] = 1.0;
        self.update_rank_1(&u, &v, solver)
    }

    /// Convenience: replace column `j` of `A` with `new_col`.
    /// Equivalent to a rank-1 update with `u = new_col - A[:, j]` and
    /// `v = e_j`.
    pub fn replace_column<S: Solver>(
        &mut self,
        j: usize,
        new_col: &[f64],
        solver: &S,
    ) -> Result<(), SensitivityError> {
        let n = self.n();
        if j >= n {
            return Err(EngineError::ShapeMismatch(format!(
                "replace_column: column {j} out of range for {n}×{n} matrix"
            ))
            .into());
        }
        if new_col.len() != n {
            return Err(EngineError::ShapeMismatch(format!(
                "replace_column: new_col.len()={} != n={n}",
                new_col.len()
            ))
            .into());
        }
        let current = extract_column(&self.a, j);
        let u: Vec<f64> = new_col.iter().zip(current.iter()).map(|(n, c)| n - c).collect();
        if u.iter().all(|&x| x == 0.0) {
            return Ok(()); // Nothing changed.
        }
        let mut v = vec![0.0; n];
        v[j] = 1.0;
        self.update_rank_1(&u, &v, solver)
    }

    /// Convenience: modify an edge in the technosphere graph.
    ///
    /// In matrix terms, swap entry `A[i, j_from]` → 0 and
    /// `A[i, j_to]` → old `A[i, j_from]` (or whatever `new_value`
    /// specifies). This is a rank-2 update per spec §10.1.
    ///
    /// Arguments:
    /// - `i`: row (flow index).
    /// - `j_from`: column to zero out.
    /// - `j_to`: column to populate.
    /// - `value`: the value to place at `A[i, j_to]`.
    pub fn modify_edge<S: Solver>(
        &mut self,
        i: usize,
        j_from: usize,
        j_to: usize,
        value: f64,
        solver: &S,
    ) -> Result<(), SensitivityError> {
        let n = self.n();
        if i >= n || j_from >= n || j_to >= n {
            return Err(EngineError::ShapeMismatch(format!(
                "modify_edge: ({i}, {j_from}, {j_to}) out of range for {n}×{n}"
            ))
            .into());
        }
        if j_from == j_to {
            // Collapses to a single-entry edit.
            let old = get_entry(&self.a, i, j_from);
            return self.edit_entry(i, j_from, value - old, solver);
        }
        let old_from = get_entry(&self.a, i, j_from);
        let old_to = get_entry(&self.a, i, j_to);

        // Update 1: remove A[i, j_from].   ΔA = -old_from · e_i · e_{j_from}^T
        let mut u1 = vec![0.0; n];
        u1[i] = -old_from;
        let mut v1 = vec![0.0; n];
        v1[j_from] = 1.0;

        // Update 2: set A[i, j_to] = value.  ΔA = (value - old_to) · e_i · e_{j_to}^T
        let mut u2 = vec![0.0; n];
        u2[i] = value - old_to;
        let mut v2 = vec![0.0; n];
        v2[j_to] = 1.0;

        self.update_rank_r(&[&u1, &u2], &[&v1, &v2], solver)
    }
}

/// Rebuild `A + Σ u_k·v_k^T` as a fresh CSR matrix.
///
/// Determinism: triplets are sorted by `(row, col)` before handing to
/// sprs, so duplicate-entry accumulation proceeds in a fixed order
/// (spec §7.2).
fn add_rank_r_to_sparse(a: &SparseMatrix, u: &[&[f64]], v: &[&[f64]]) -> SparseMatrix {
    let (rows, cols) = a.shape();
    let mut triplets: Vec<(usize, usize, f64)> = Vec::new();

    for (row_idx, row) in a.outer_iterator().enumerate() {
        for (col_idx, &val) in row.iter() {
            triplets.push((row_idx, col_idx, val));
        }
    }

    for k in 0..u.len() {
        for (i, &ui) in u[k].iter().enumerate() {
            if ui == 0.0 {
                continue;
            }
            for (j, &vj) in v[k].iter().enumerate() {
                if vj == 0.0 {
                    continue;
                }
                triplets.push((i, j, ui * vj));
            }
        }
    }

    triplets.sort_by(|a, b| (a.0, a.1).cmp(&(b.0, b.1)));

    let mut tri = TriMat::new((rows, cols));
    for (i, j, val) in triplets {
        tri.add_triplet(i, j, val);
    }
    tri.to_csr()
}

fn extract_column(a: &SparseMatrix, j: usize) -> DenseVector {
    let rows = a.rows();
    let mut col = vec![0.0_f64; rows];
    for (row_idx, row) in a.outer_iterator().enumerate() {
        for (col_idx, &val) in row.iter() {
            if col_idx == j {
                col[row_idx] = val;
                break;
            }
        }
    }
    col
}

fn get_entry(a: &SparseMatrix, i: usize, j: usize) -> f64 {
    for (row_idx, row) in a.outer_iterator().enumerate() {
        if row_idx != i {
            continue;
        }
        for (col_idx, &val) in row.iter() {
            if col_idx == j {
                return val;
            }
        }
        return 0.0;
    }
    0.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use sprs::TriMat;

    #[test]
    fn rank_r_rebuild_preserves_existing_entries() {
        let mut t = TriMat::new((2, 2));
        t.add_triplet(0, 0, 3.0);
        t.add_triplet(1, 1, 4.0);
        let a: SparseMatrix = t.to_csr();

        let u = vec![0.0, 0.0];
        let v = vec![0.0, 0.0];
        let a2 = add_rank_r_to_sparse(&a, &[&u], &[&v]);
        assert_eq!(a2.shape(), (2, 2));
        // Zero update preserves entries.
        assert_eq!(get_entry(&a2, 0, 0), 3.0);
        assert_eq!(get_entry(&a2, 1, 1), 4.0);
    }

    #[test]
    fn rank_r_rebuild_adds_and_accumulates() {
        let mut t = TriMat::new((2, 2));
        t.add_triplet(0, 0, 1.0);
        let a: SparseMatrix = t.to_csr();

        // ΔA = e_0·e_0^T → A[0,0] += 1.
        let u = vec![1.0, 0.0];
        let v = vec![1.0, 0.0];
        let a2 = add_rank_r_to_sparse(&a, &[&u], &[&v]);
        assert_eq!(get_entry(&a2, 0, 0), 2.0);
    }

    #[test]
    fn extract_column_works_on_dense_column() {
        let mut t = TriMat::new((3, 3));
        t.add_triplet(0, 1, 10.0);
        t.add_triplet(2, 1, 30.0);
        let a: SparseMatrix = t.to_csr();
        assert_eq!(extract_column(&a, 1), vec![10.0, 0.0, 30.0]);
        assert_eq!(extract_column(&a, 0), vec![0.0, 0.0, 0.0]);
    }

    #[test]
    fn get_entry_returns_zero_for_missing() {
        let mut t = TriMat::new((2, 2));
        t.add_triplet(0, 1, 5.0);
        let a: SparseMatrix = t.to_csr();
        assert_eq!(get_entry(&a, 0, 1), 5.0);
        assert_eq!(get_entry(&a, 1, 0), 0.0);
    }
}
