//! The `Study` — see `specs/calc/v0.1.md` §4.1.
//!
//! A study is the unit of calculation: everything the engine needs to
//! produce a result. Construction is deliberately **not** validated by
//! this module; validation (§6.1 order) lives in `arko-validation` so
//! that study construction in tests and fixtures is cheap.

use crate::{
    license::LicenseTier,
    matrices::{SparseMatrix, SparseVector},
    meta::{FlowMeta, ImpactMeta, ProcessMeta},
    parameters::Parameter,
    sign::SignConvention,
};
use serde::{Deserialize, Serialize};

/// Reference to an impact assessment method. The method itself lives in
/// a method registry (sibling crate `arko-methods`); we only store the
/// identifier + version here so studies are portable across method
/// registry snapshots.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MethodRef {
    pub id: String,
    pub version: String,
}

/// A complete calculable LCA dataset.
///
/// Shape invariants (enforced at validation, not construction):
/// - `technosphere` is `n × n` where `n = processes.len()`.
/// - `biosphere` is `m × n` where `m = flows.len()`.
/// - `characterization` is `k × m` where `k = impacts.len()`.
/// - `functional_unit` has length `n`.
/// - Every `ProcessMeta::license_tier` indexes a valid `license_tiers` entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Study {
    pub technosphere: SparseMatrix,
    pub biosphere: SparseMatrix,
    pub characterization: SparseMatrix,
    pub functional_unit: SparseVector,

    pub processes: Vec<ProcessMeta>,
    pub flows: Vec<FlowMeta>,
    pub impacts: Vec<ImpactMeta>,
    pub parameters: Vec<Parameter>,
    pub license_tiers: Vec<LicenseTier>,

    pub method: MethodRef,
    pub sign_convention: SignConvention,
}

impl Study {
    /// Number of technosphere processes (`n` in the spec).
    pub fn n_processes(&self) -> usize {
        self.processes.len()
    }

    /// Number of elementary flows (`m` in the spec).
    pub fn n_flows(&self) -> usize {
        self.flows.len()
    }

    /// Number of impact categories (`k` in the spec).
    pub fn n_impacts(&self) -> usize {
        self.impacts.len()
    }

    /// BLAKE3 hash of the canonical serialization — used as `study_hash`
    /// in `Provenance` (§12). The canonical form here is `serde_json` in
    /// a deterministic key order; a richer canonicalization (e.g., CBOR
    /// with fixed field ordering) is a v0.2 upgrade.
    pub fn canonical_hash(&self) -> String {
        let bytes = serde_json::to_vec(self).expect("Study is always JSON-serializable");
        blake3::hash(&bytes).to_hex().to_string()
    }
}
