//! Sign convention — see `specs/calc/v0.1.md` §4.4.

use serde::{Deserialize, Serialize};

/// Sign convention declared by a study. An engine **MUST** accept either;
/// internally it **MAY** normalize to one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum SignConvention {
    /// Products produced are positive, consumed are negative. Emissions
    /// to environment are positive. (ecoinvent-style; the default.)
    #[default]
    ProducerPositive,

    /// Products consumed are positive. (ILCD-style.)
    ConsumerPositive,
}

impl SignConvention {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ProducerPositive => "producer-positive",
            Self::ConsumerPositive => "consumer-positive",
        }
    }
}
