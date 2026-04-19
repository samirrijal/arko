//! Engine warning taxonomy — see `specs/calc/v0.1.md` §13.2.
//!
//! Warnings accompany a result; unlike `EngineError`, they do not
//! prevent the engine from returning a value. Any user-facing surface
//! that displays results **MUST** surface warnings alongside them.

use serde::{Deserialize, Serialize};

/// Warning code — stable identifier matching spec §13.2.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WarningCode {
    #[serde(rename = "W_ILL_CONDITIONED")]
    IllConditioned,
    #[serde(rename = "W_NEAR_SINGULAR")]
    NearSingular,
    #[serde(rename = "W_SUBSTITUTION_USED")]
    SubstitutionUsed,
    #[serde(rename = "W_MC_NONCONVERGENT")]
    MonteCarloNonconvergent,
    #[serde(rename = "W_RESTRICTED_RESULT")]
    RestrictedResult,
    #[serde(rename = "W_OUTDATED_METHOD")]
    OutdatedMethod,
}

impl WarningCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::IllConditioned => "W_ILL_CONDITIONED",
            Self::NearSingular => "W_NEAR_SINGULAR",
            Self::SubstitutionUsed => "W_SUBSTITUTION_USED",
            Self::MonteCarloNonconvergent => "W_MC_NONCONVERGENT",
            Self::RestrictedResult => "W_RESTRICTED_RESULT",
            Self::OutdatedMethod => "W_OUTDATED_METHOD",
        }
    }
}

impl std::fmt::Display for WarningCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A warning attached to a result.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Warning {
    pub code: WarningCode,
    pub message: String,
    /// Optional numeric payload — e.g., the estimated condition number for
    /// `W_ILL_CONDITIONED`, the Monte Carlo standard error for `W_MC_NONCONVERGENT`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<f64>,
}

impl Warning {
    pub fn new(code: WarningCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            value: None,
        }
    }

    pub fn with_value(mut self, value: f64) -> Self {
        self.value = Some(value);
        self
    }
}
