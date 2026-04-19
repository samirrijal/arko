//! Process, flow, and impact metadata — see `specs/calc/v0.1.md` §4.1.

use crate::units::Unit;
use serde::{Deserialize, Serialize};

/// Metadata for a technosphere process (one column of A).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProcessMeta {
    /// Stable identifier (e.g., ecoinvent activity id, user-assigned UUID).
    pub id: String,

    /// Human-readable name.
    pub name: String,

    /// The reference product this column represents. Must exist as a row
    /// of A (a technosphere product is both consumed and produced across
    /// the matrix).
    pub reference_product: String,

    /// Unit of the reference product.
    pub reference_unit: Unit,

    /// Allocation mode, per §6.4.
    #[serde(default)]
    pub allocation: Option<Allocation>,

    /// Index into `Study::license_tiers`.
    pub license_tier: LicenseTierRef,

    /// Free-form geography tag, e.g., `"ES"`, `"GLO"`, `"RER"`. Full
    /// regionalized impact is deferred to v0.3; this is informational.
    #[serde(default)]
    pub geography: Option<String>,
}

/// Opaque reference into `Study::license_tiers` — we store an index
/// rather than the full `LicenseTier` inline to keep columns compact and
/// to allow tier deduplication across a study.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct LicenseTierRef(pub u32);

/// Allocation declaration for a multi-functional process. See §6.4.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Allocation {
    pub mode: AllocationMode,

    /// Required only for `AllocationMode::User`: per-co-product factors.
    /// Must sum to 1.0 ± 1e-9 (enforced at construction).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub user_factors: Vec<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AllocationMode {
    Mass,
    Economic,
    Energy,
    User,
    Substitution,
}

/// Origin classifier for an elementary flow.
///
/// Several IPCC AR6 characterization factors depend on whether a species
/// was emitted from fossil-carbon feedstock or from a biogenic / natural
/// source — methane being the canonical example (AR6 WG1 Table 7.15
/// gives `29.8` for fossil CH4 vs `27.0` for non-fossil CH4).
/// Earlier assessment reports (AR5 and older) did not differentiate,
/// which is why `Unspecified` is the default — existing fixtures and
/// JSON blobs continue to round-trip unchanged.
///
/// Match policy (see `arko_methods::FactorMatch::CasOrigin`):
/// an origin-specific matcher requires an **exact** origin match; a
/// flow with `Unspecified` origin will not silently inherit a
/// fossil-only factor. That surfaces as a
/// `CMatrixBuild::unmatched_flows` entry rather than a wrong number.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FlowOrigin {
    /// Origin was not recorded. Default for backward-compatible JSON.
    #[default]
    Unspecified,
    /// Emitted from fossil-carbon feedstock (coal, oil, natural gas).
    Fossil,
    /// Emitted from biogenic or other non-fossil sources (landfill,
    /// livestock, wetland, combustion of recent-photosynthesis carbon).
    NonFossil,
}

impl FlowOrigin {
    /// `true` iff this origin carries no information — used by the
    /// `skip_serializing_if` guard so default-valued fields stay out of
    /// serialized JSON.
    #[must_use]
    pub fn is_unspecified(&self) -> bool {
        matches!(self, Self::Unspecified)
    }
}

/// Metadata for an elementary flow (one row of B).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FlowMeta {
    pub id: String,
    pub name: String,
    pub unit: Unit,

    /// Compartment path, e.g., `["emission", "air", "urban"]`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub compartment: Vec<String>,

    /// CAS number if known (e.g., `"124-38-9"` for CO2).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cas: Option<String>,

    /// Origin classifier for species whose characterization factor
    /// depends on fossil vs non-fossil provenance (e.g., methane under
    /// IPCC AR6). Defaults to [`FlowOrigin::Unspecified`] so existing
    /// fixtures are unaffected.
    #[serde(default, skip_serializing_if = "FlowOrigin::is_unspecified")]
    pub origin: FlowOrigin,
}

/// Metadata for an impact category (one row of C).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImpactMeta {
    /// Stable id, e.g., `"gwp100"`, `"acidification"`.
    pub id: String,
    pub name: String,

    /// Reference unit of the impact score, e.g., `"kg CO2-eq"`.
    pub unit: Unit,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn minimal_flow() -> FlowMeta {
        FlowMeta {
            id: "ch4".into(),
            name: "Methane".into(),
            unit: Unit::new("kg"),
            compartment: vec!["emission".into(), "air".into()],
            cas: Some("74-82-8".into()),
            origin: FlowOrigin::Fossil,
        }
    }

    #[test]
    fn flow_origin_default_is_unspecified() {
        assert_eq!(FlowOrigin::default(), FlowOrigin::Unspecified);
    }

    #[test]
    fn unspecified_origin_is_skipped_in_serialized_json() {
        let mut f = minimal_flow();
        f.origin = FlowOrigin::Unspecified;
        let s = serde_json::to_string(&f).unwrap();
        assert!(
            !s.contains("origin"),
            "Unspecified origin must be skipped for backward-compatible JSON; got: {s}"
        );
    }

    #[test]
    fn fossil_origin_round_trips_through_json() {
        let f = minimal_flow();
        let s = serde_json::to_string(&f).unwrap();
        assert!(s.contains("\"origin\":\"fossil\""));
        let back: FlowMeta = serde_json::from_str(&s).unwrap();
        assert_eq!(back, f);
    }

    #[test]
    fn missing_origin_field_deserializes_to_unspecified() {
        // Simulates a pre-v0.0.1 JSON blob that predates the `origin` field.
        let legacy = r#"{
            "id": "co2",
            "name": "Carbon dioxide",
            "unit": "kg",
            "compartment": ["emission", "air"],
            "cas": "124-38-9"
        }"#;
        let flow: FlowMeta = serde_json::from_str(legacy).unwrap();
        assert_eq!(flow.origin, FlowOrigin::Unspecified);
    }
}
