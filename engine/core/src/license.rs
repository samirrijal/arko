//! License-tier model — see `specs/calc/v0.1.md` §11.
//!
//! The calc spec defines license tiers as first-class schema — every
//! process carries one, and the **join** of all tiers touched by a
//! solve is the effective restriction on the result (§11.2).
//!
//! This module defines the **storage types**. The evaluator — which
//! walks the calculation graph, computes effective restrictions, and
//! fires `DerivativeRule`s at publish time — lives in a sibling crate
//! (`arko-license`) because its logic will grow over time and we do not
//! want it in the hot path of the core types.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Per-process license tier. See spec §11.1.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LicenseTier {
    /// Canonical source identifier, e.g., `"ecoinvent-3.11"`, `"custom-user"`,
    /// `"gabi-2024-1"`. Matched against known-database rules by the evaluator.
    pub source: String,

    /// May the impact of processes carrying this tier be published externally
    /// (reports, EPDs, marketing)?
    pub allow_publish: bool,

    /// May the inventory itself be shared across workspaces (e.g., handed to
    /// a client workspace)?
    pub allow_share: bool,

    /// May the raw data be exported as open-format (ecospold2, EPDX)?
    pub allow_export: bool,

    /// Rules evaluated at result-publish time. See §11.3.
    #[serde(default)]
    pub derivative_rules: Vec<DerivativeRule>,

    /// Optional expiry — after this, any result derived from this tier
    /// **MUST** be re-consented before publishing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expiry: Option<DateTime<Utc>>,
}

/// A single publish-time rule. See §11.3.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DerivativeRule {
    /// Trigger condition. Currently only `scaling_ge` is supported: fire
    /// when the process's scaling factor in the solution vector exceeds
    /// this threshold.
    pub when: DerivativeTrigger,

    /// What to do when the rule fires.
    pub action: DerivativeAction,

    /// Message shown to the user / embedded in audit log.
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DerivativeTrigger {
    /// Fire when `s[j] >= threshold`.
    ScalingGe { threshold: f64 },
    /// Fire unconditionally on any presence in the solve.
    Always,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DerivativeAction {
    /// Block the publish/export action entirely.
    Block,
    /// Permit the action but attach a visible warning.
    Warn,
    /// Permit the action but watermark the output as restricted.
    Watermark,
}

impl LicenseTier {
    /// A fully-permissive tier, useful as a default for custom / user-authored
    /// processes that carry no third-party licensing obligations.
    pub fn permissive(source: impl Into<String>) -> Self {
        Self {
            source: source.into(),
            allow_publish: true,
            allow_share: true,
            allow_export: true,
            derivative_rules: Vec::new(),
            expiry: None,
        }
    }
}
