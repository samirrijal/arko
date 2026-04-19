//! Engine error taxonomy — see `specs/calc/v0.1.md` §13.1.
//!
//! Every variant corresponds 1:1 to an `E_*` code in the spec. Variant
//! names are the `PascalCase` form of the code; the `Display` impl emits
//! the canonical `E_*` string so logs and API responses can be grepped
//! against the spec directly.

use thiserror::Error;

/// Fatal engine errors. An engine that returns one of these **MUST NOT**
/// also return a result.
#[derive(Debug, Error, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "code", content = "detail")]
pub enum EngineError {
    #[error("E_SHAPE_MISMATCH: {0}")]
    ShapeMismatch(String),

    #[error("E_PARAM_CYCLE: parameter dependency cycle involving {0}")]
    ParamCycle(String),

    #[error("E_PARAM_UNRESOLVED: parameter `{0}` references undefined `{1}`")]
    ParamUnresolved(String, String),

    #[error("E_PARAM_NONFINITE: parameter `{0}` evaluated to non-finite value")]
    ParamNonfinite(String),

    #[error("E_UNIT_INCOMPATIBLE: {0}")]
    UnitIncompatible(String),

    #[error("E_SINGULAR: technosphere matrix A is singular")]
    Singular,

    #[error("E_LICENSE_VIOLATION: {0}")]
    LicenseViolation(String),

    #[error("E_METHOD_UNKNOWN: impact method `{0}` is not registered")]
    MethodUnknown(String),

    #[error("E_ALLOCATION_INVALID: allocation factors sum to {sum} (expected 1.0 ± 1e-9)")]
    AllocationInvalid { sum: f64 },

    #[error("E_INTERNAL: {0}")]
    Internal(String),
}

impl EngineError {
    /// Canonical `E_*` code string — matches the spec §13.1 table.
    pub const fn code(&self) -> &'static str {
        match self {
            Self::ShapeMismatch(_) => "E_SHAPE_MISMATCH",
            Self::ParamCycle(_) => "E_PARAM_CYCLE",
            Self::ParamUnresolved(_, _) => "E_PARAM_UNRESOLVED",
            Self::ParamNonfinite(_) => "E_PARAM_NONFINITE",
            Self::UnitIncompatible(_) => "E_UNIT_INCOMPATIBLE",
            Self::Singular => "E_SINGULAR",
            Self::LicenseViolation(_) => "E_LICENSE_VIOLATION",
            Self::MethodUnknown(_) => "E_METHOD_UNKNOWN",
            Self::AllocationInvalid { .. } => "E_ALLOCATION_INVALID",
            Self::Internal(_) => "E_INTERNAL",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn code_matches_display_prefix() {
        let cases = [
            EngineError::ShapeMismatch("x".into()),
            EngineError::ParamCycle("p".into()),
            EngineError::Singular,
            EngineError::AllocationInvalid { sum: 0.99 },
        ];
        for err in cases {
            assert!(
                err.to_string().starts_with(err.code()),
                "Display prefix must equal code() for spec traceability",
            );
        }
    }

    #[test]
    fn roundtrip_json() {
        let err = EngineError::ParamUnresolved("density".into(), "rho".into());
        let s = serde_json::to_string(&err).unwrap();
        let back: EngineError = serde_json::from_str(&s).unwrap();
        assert_eq!(err, back);
    }
}
