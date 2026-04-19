//! Unit model — see `specs/calc/v0.1.md` §4.5.
//!
//! Units are represented as opaque UCUM-compatible strings. This crate
//! does **not** perform dimensional analysis yet; that lands in a
//! follow-up crate (`arko-units`) with a full UCUM parser. Until then,
//! equality on `Unit` is exact string equality, which means callers
//! **MUST** normalize before comparing (e.g., `"kg"` vs `" kg "`).

use serde::{Deserialize, Serialize};

/// A UCUM / ISO-80000 compatible unit string.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Unit(pub String);

impl Unit {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Sentinel for the dimensionless unit. UCUM uses `"1"`.
    pub fn dimensionless() -> Self {
        Self::new("1")
    }
}

impl From<&str> for Unit {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<String> for Unit {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl std::fmt::Display for Unit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
