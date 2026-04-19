//! Sparse matrix type aliases — see `specs/calc/v0.1.md` §4.2.
//!
//! The engine's sparse matrix backend is `sprs`. Choosing a concrete
//! backend here (rather than abstracting over a trait) is deliberate:
//! - The calc spec requires **deterministic ordering** inside matrix ops
//!   (§7.2). Proliferating backends multiplies the audit surface.
//! - Every conforming reference implementation will ship `sprs` anyway;
//!   alternative backends (faer, UMFPACK) are plugged in at the
//!   **solver** layer, not the storage layer.
//!
//! If a future spec revision requires a different storage layout (e.g.,
//! block-sparse for hybrid I-O analysis, §3.2 open question), the
//! migration will be explicit, versioned, and `cfg`-gated — not silent.

/// Technosphere / biosphere / characterization matrix.
pub type SparseMatrix = sprs::CsMat<f64>;

/// Functional unit / intermediate vector.
pub type SparseVector = sprs::CsVec<f64>;

/// Dense vector — results (`s`, `g`, `h`) and sensitivity columns.
pub type DenseVector = Vec<f64>;
