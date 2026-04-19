//! Provenance — see `specs/calc/v0.1.md` §12.
//!
//! Every result carries a provenance record. Results without provenance
//! **MUST NOT** be exportable or publishable (spec §12).

use crate::warning::Warning;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// The full provenance record attached to every result.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Provenance {
    /// `semver + "+" + git-sha`, e.g., `"0.0.1+a1b2c3d"`.
    pub engine_version: String,

    /// Specification version implemented by the engine that produced
    /// this result. See `SPEC_VERSION`.
    pub spec_version: String,

    /// BLAKE3 hash of the canonically-serialized study.
    pub study_hash: String,

    pub method_id: String,
    pub method_version: String,

    /// Human-readable identifier of the solver used. Examples:
    /// `"nalgebra-dense"`, `"faer-sparse-lu"`, `"umfpack-0.5"`.
    pub solver_used: String,

    /// Opaque, solver-specific configuration echoed back for
    /// reproducibility (pivot thresholds, fill-in strategies, etc.).
    #[serde(default)]
    pub solver_config: serde_json::Value,

    /// Staleness counter at the moment of computation. See §10.2.
    pub generation: u64,

    pub computed_at: DateTime<Utc>,

    /// Attribution — who triggered the calculation. Stored as opaque
    /// string so the engine has no opinion on the identity system.
    pub computed_by: String,

    /// The effective license restriction computed per §11.2.
    pub restrictions: EffectiveRestriction,

    /// Indices of processes that contributed non-trivially (`s[j] > 1e-9`)
    /// to the result — the `contributing_processes` field of §12.
    #[serde(default)]
    pub contributing_processes: Vec<u32>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<Warning>,
}

/// The join of all license tiers touched by the solve. See §11.2.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct EffectiveRestriction {
    pub allow_publish: bool,
    pub allow_share: bool,
    pub allow_export: bool,

    /// Contributing tier sources — e.g., `["ecoinvent-3.11", "custom-user"]`.
    #[serde(default)]
    pub sources: Vec<String>,
}

impl EffectiveRestriction {
    /// The maximally-permissive initial value, to be narrowed by `join`ing
    /// each tier touched during a graph walk.
    pub fn permissive() -> Self {
        Self {
            allow_publish: true,
            allow_share: true,
            allow_export: true,
            sources: Vec::new(),
        }
    }

    /// Narrow `self` with the restrictions of `tier`. This is the
    /// boolean-AND join from §11.2.
    pub fn join(&mut self, tier: &crate::license::LicenseTier) {
        self.allow_publish &= tier.allow_publish;
        self.allow_share &= tier.allow_share;
        self.allow_export &= tier.allow_export;
        if !self.sources.iter().any(|s| s == &tier.source) {
            self.sources.push(tier.source.clone());
        }
    }
}
