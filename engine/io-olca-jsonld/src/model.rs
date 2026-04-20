//! openLCA JSON-LD domain model — a narrow Rust representation of the
//! five object kinds the v0.1 reader touches: `Process`, `Flow`,
//! `FlowProperty`, `UnitGroup`, and the `Exchange` / `FlowPropertyFactor`
//! / `Unit` sub-objects.
//!
//! The naming follows openLCA schema terminology (`processType`,
//! `defaultProvider`, `flowType`, `referenceFlowProperty`,
//! `referenceUnit`) rather than ILCD terminology, so that round-trips
//! with openLCA / Federal LCA Commons tooling stay legible.
//!
//! # v0.1 scope
//!
//! What the reader populates (beef-bundle-shaped — see SUPPORTED.md):
//! - Process identity, `processType`, `defaultAllocationMethod`,
//!   exchanges.
//! - Exchange: `internalId`, `amount`, `input`, `avoidedProduct`,
//!   `quantitativeReference`, flow reference, `defaultProvider` UUID,
//!   unit + flow-property UUIDs.
//! - Flow: `@id`, `name`, `flowType`, optional `cas`, optional `formula`,
//!   `flowProperties[]` with the one marked `referenceFlowProperty`
//!   identified.
//! - FlowProperty: `@id`, `name`, reference unit-group UUID.
//! - UnitGroup: `@id`, `name`, `units[]` with one marked `referenceUnit`,
//!   each carrying a `conversionFactor` (this-unit → reference unit).
//!
//! Anything not listed above — actors, sources, locations, categories,
//! dq_systems, allocation factors, parameters, `avoidedProduct: true`
//! handling beyond propagation — is deliberately out of scope for the
//! v0.1 reader and is documented in SUPPORTED.md.

use serde::{Deserialize, Serialize};

/// One openLCA `Process` document (`{ "@type": "Process", ... }`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OlcaProcess {
    pub id: String,
    pub name: String,
    /// `"UNIT_PROCESS"` / `"LCI_RESULT"`. v0.1 handles UNIT_PROCESS;
    /// LCI_RESULT inputs (aggregated LCI) are accepted by the parser
    /// but flagged by the adapter if encountered — beef bundle is
    /// entirely UNIT_PROCESS.
    pub process_type: OlcaProcessType,
    /// `"PHYSICAL_ALLOCATION"`, `"ECONOMIC_ALLOCATION"`,
    /// `"CAUSAL_ALLOCATION"`, `"USE_DEFAULT_ALLOCATION"`, or absent.
    /// v0.1 carries this through but applies no allocation logic —
    /// beef bundle process exchanges are already allocated at the
    /// source. See SUPPORTED.md.
    pub default_allocation_method: Option<String>,
    pub exchanges: Vec<OlcaExchange>,
}

impl OlcaProcess {
    /// The exchange marked `quantitativeReference: true`. v0.1 requires
    /// exactly one such exchange per process; the parser surfaces this
    /// as an error if zero or multiple are marked.
    #[must_use]
    pub fn reference_exchange(&self) -> Option<&OlcaExchange> {
        self.exchanges.iter().find(|e| e.quantitative_reference)
    }
}

/// openLCA `processType` enum. Extend here as the reader grows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OlcaProcessType {
    UnitProcess,
    LciResult,
}

/// One entry in a Process's `exchanges` array.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OlcaExchange {
    /// openLCA's per-exchange integer — stable within the process,
    /// useful for cross-referencing error messages back to source JSON.
    pub internal_id: i32,
    pub amount: f64,
    /// `true` for inputs, `false` for outputs. The adapter translates
    /// this to `arko_io_ilcd::Direction::Input` / `Output`; sign
    /// handling in matrix construction is the consumer's job.
    pub input: bool,
    /// openLCA convention: a positive amount on an avoided-product
    /// exchange represents the avoided quantity (the *negative*
    /// technosphere entry). v0.1 carries the flag through but does not
    /// sign-flip in the adapter — downstream matrix assembly must
    /// honour the flag. Beef bundle has none.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub avoided_product: bool,
    /// `true` for the one exchange that is the process's functional
    /// unit (the column's reference output).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub quantitative_reference: bool,
    /// UUID of the flow this exchange crosses the boundary on.
    pub flow_uuid: String,
    /// Embedded flow-type hint — comes from the `flow.flowType` field
    /// inlined into the exchange's embedded ref. Preserved so the
    /// adapter can classify A-column vs B-row without re-reading the
    /// flow file when the bundle is shallow (beef bundle inlines every
    /// flow's type on every exchange reference).
    pub flow_type: OlcaFlowType,
    /// UUID of the unit the `amount` is quoted in. May differ from the
    /// flow's reference unit (e.g. exchange quotes `t`, flow reports in
    /// `kg`). The adapter converts via unit-group `conversionFactor`.
    pub unit_uuid: String,
    /// UUID of the flow property the exchange's `unit` belongs to.
    /// v0.1 constraint: must equal the flow's reference flow-property
    /// UUID. Cross-property conversion (mass ↔ energy for a fuel) is
    /// out of scope for v0.1; enforced by the adapter.
    pub flow_property_uuid: String,
    /// UUID of the process that produces this product flow, when the
    /// exchange is a technosphere input wired through openLCA's
    /// "default provider" mechanism. Populates A-matrix off-diagonal
    /// entries. `None` for elementary flows and for technosphere flows
    /// that the publisher left unwired.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_provider_uuid: Option<String>,
}

/// openLCA `flowType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OlcaFlowType {
    ProductFlow,
    ElementaryFlow,
    WasteFlow,
}

/// One openLCA `Flow` document.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OlcaFlow {
    pub id: String,
    pub name: String,
    pub flow_type: OlcaFlowType,
    /// CAS number as published — beef bundle uses zero-padded form
    /// (`"000074-82-8"`). The adapter normalises by trimming leading
    /// zeros before the dash so matching against AR6-style tables
    /// works. Empty-string CAS is normalised to `None` at parse time.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cas: Option<String>,
    /// Chemical formula, when published (e.g. `"CH4"`). v0.1 preserves
    /// it for future dimensional-analysis hooks but the matrix path
    /// does not consume it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub formula: Option<String>,
    pub flow_properties: Vec<OlcaFlowPropertyFactor>,
}

impl OlcaFlow {
    /// The `flowProperties` entry marked `referenceFlowProperty: true`.
    /// The parser guarantees exactly one per flow or it would have
    /// failed earlier.
    #[must_use]
    pub fn reference_flow_property(&self) -> Option<&OlcaFlowPropertyFactor> {
        self.flow_properties
            .iter()
            .find(|f| f.reference_flow_property)
    }
}

/// One entry in a Flow's `flowProperties` array.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OlcaFlowPropertyFactor {
    pub flow_property_uuid: String,
    /// Exactly one entry per flow is marked `true`.
    pub reference_flow_property: bool,
    /// Conversion factor from *this* flow property to the reference
    /// flow property — in the flow property's own unit. For the
    /// reference entry itself, `1.0`. v0.1 adapter never applies this
    /// (cross-property conversion deferred) but the field is preserved.
    pub conversion_factor: f64,
}

/// One openLCA `FlowProperty` document.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OlcaFlowProperty {
    pub id: String,
    pub name: String,
    /// UUID of the unit group that defines the reference unit for this
    /// property.
    pub unit_group_uuid: String,
}

/// One openLCA `UnitGroup` document.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OlcaUnitGroup {
    pub id: String,
    pub name: String,
    pub units: Vec<OlcaUnit>,
}

impl OlcaUnitGroup {
    #[must_use]
    pub fn reference_unit(&self) -> Option<&OlcaUnit> {
        self.units.iter().find(|u| u.reference_unit)
    }

    #[must_use]
    pub fn unit_by_id(&self, uuid: &str) -> Option<&OlcaUnit> {
        self.units.iter().find(|u| u.id == uuid)
    }
}

/// One `Unit` entry inside a `UnitGroup`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OlcaUnit {
    pub id: String,
    pub name: String,
    /// Factor converting *this* unit to the group's reference unit:
    /// `amount_in_ref = amount_in_this_unit * conversion_factor`.
    /// For the reference entry itself, `1.0`.
    pub conversion_factor: f64,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub reference_unit: bool,
}

/// Trim leading zeros from the numeric part of an openLCA-style CAS
/// number — beef bundle publishes `"000074-82-8"`; downstream AR6
/// tables key on `"74-82-8"`. Idempotent on already-trimmed CAS.
///
/// Returns the input unchanged if it does not match the expected
/// `NNNNNN-NN-N` shape.
#[must_use]
pub fn normalize_cas(raw: &str) -> String {
    // openLCA's padded form is exactly the canonical form with leading
    // zeros on the first group. Split on the first `-`; if the left
    // side parses as an integer, emit it without the zeros.
    let Some((head, rest)) = raw.split_once('-') else {
        return raw.to_owned();
    };
    let trimmed_head = head.trim_start_matches('0');
    if trimmed_head.is_empty() {
        // Pathological all-zero head ("000-00-0") — leave caller to
        // notice.
        return raw.to_owned();
    }
    if !trimmed_head.chars().all(|c| c.is_ascii_digit()) {
        return raw.to_owned();
    }
    format!("{trimmed_head}-{rest}")
}

/// Derive `FlowOrigin` from an openLCA flow's `name`.
///
/// openLCA / USDA LCA Commons encode origin as a trailing
/// comma-separated qualifier:
///
/// - `"Methane, fossil"` → `Fossil`
/// - `"Methane, biogenic"` → `NonFossil`
/// - `"Methane, from soil or biomass stocks"` → `NonFossil`
///
/// Anything unrecognised (no trailing comma-qualifier, or an unknown
/// tag) → `Unspecified` so AR6 matchers surface the gap rather than
/// silently guessing.
///
/// TODO(consolidation): `arko_io_ilcd_linker::classify_flow_origin`
/// implements the same idea for the parenthetical form
/// (`"methane (biogenic)"`). When both readers are mature enough to
/// share a flow-origin classifier, lift the union of synonyms into a
/// common place. Until a third consumer appears, duplication is
/// preferable to a premature shared-types crate.
#[must_use]
pub fn classify_flow_origin_from_name(name: &str) -> arko_io_ilcd_linker::FlowOrigin {
    use arko_io_ilcd_linker::FlowOrigin;
    let trimmed = name.trim();
    // The origin qualifier is the last comma-separated segment in
    // openLCA's elementary-flow naming convention. We only classify
    // when the tail matches the known vocabulary.
    let Some(comma) = trimmed.rfind(',') else {
        return FlowOrigin::Unspecified;
    };
    let tail = trimmed[comma + 1..].trim().to_ascii_lowercase();
    match tail.as_str() {
        "fossil" => FlowOrigin::Fossil,
        "biogenic"
        | "non-fossil"
        | "land use change"
        | "short cycle"
        | "from soil or biomass stocks" => FlowOrigin::NonFossil,
        _ => FlowOrigin::Unspecified,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arko_io_ilcd_linker::FlowOrigin;

    #[test]
    fn cas_zero_padded_is_trimmed() {
        assert_eq!(normalize_cas("000074-82-8"), "74-82-8");
        assert_eq!(normalize_cas("000124-38-9"), "124-38-9");
    }

    #[test]
    fn cas_already_normalized_is_idempotent() {
        assert_eq!(normalize_cas("74-82-8"), "74-82-8");
    }

    #[test]
    fn cas_single_leading_zero_trimmed() {
        assert_eq!(normalize_cas("07782-44-7"), "7782-44-7");
    }

    #[test]
    fn cas_non_numeric_head_unchanged() {
        // Defensive: if some publisher ever ships an alphanumeric
        // "CAS", leave it alone rather than corrupting it.
        assert_eq!(normalize_cas("abc-82-8"), "abc-82-8");
    }

    #[test]
    fn cas_no_dash_unchanged() {
        assert_eq!(normalize_cas("000074828"), "000074828");
    }

    #[test]
    fn origin_comma_biogenic_is_non_fossil() {
        assert_eq!(
            classify_flow_origin_from_name("Methane, biogenic"),
            FlowOrigin::NonFossil
        );
    }

    #[test]
    fn origin_comma_fossil_is_fossil() {
        assert_eq!(
            classify_flow_origin_from_name("Methane, fossil"),
            FlowOrigin::Fossil
        );
        assert_eq!(
            classify_flow_origin_from_name("Carbon dioxide, fossil"),
            FlowOrigin::Fossil
        );
    }

    #[test]
    fn origin_synonyms_are_non_fossil() {
        for name in [
            "Carbon dioxide, biogenic",
            "Methane, land use change",
            "Carbon dioxide, from soil or biomass stocks",
            "Carbon dioxide, non-fossil",
        ] {
            assert_eq!(
                classify_flow_origin_from_name(name),
                FlowOrigin::NonFossil,
                "expected NonFossil for {name}"
            );
        }
    }

    #[test]
    fn origin_no_comma_is_unspecified() {
        assert_eq!(
            classify_flow_origin_from_name("Ammonia"),
            FlowOrigin::Unspecified
        );
        assert_eq!(
            classify_flow_origin_from_name("Water, well, in ground"),
            // The tail is "in ground", not an origin qualifier, so we
            // correctly do not classify.
            FlowOrigin::Unspecified
        );
    }

    #[test]
    fn origin_case_insensitive_tail() {
        assert_eq!(
            classify_flow_origin_from_name("Methane, Biogenic"),
            FlowOrigin::NonFossil
        );
        assert_eq!(
            classify_flow_origin_from_name("Methane, FOSSIL"),
            FlowOrigin::Fossil
        );
    }

    #[test]
    fn origin_unrecognised_tail_is_unspecified() {
        assert_eq!(
            classify_flow_origin_from_name("Methane, anaerobic digestion"),
            FlowOrigin::Unspecified
        );
    }
}
