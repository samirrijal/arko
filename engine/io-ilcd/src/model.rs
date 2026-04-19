//! ILCD domain model — a *narrow* Rust representation of one ILCD
//! process dataset, wide enough to seed a column of `A` (and rows of
//! `B`) but not so wide it tracks every optional JRC field.
//!
//! The naming follows the ILCD format ("dataSetInternalID",
//! "meanAmount", "exchangeDirection") rather than ecoinvent terminology
//! so that round-trips with EU-side tooling stay legible.
//!
//! # ILCD vs ILCD+EPD v1.2
//!
//! ÖKOBAUDAT, EPD Norge and Environdec construction-industry EPDs use
//! **ILCD+EPD v1.2** — a DIN EN 15804 superset of vanilla ILCD. We
//! parse both through the same type: EPD-specific fields
//! (`epd_modules`, `epd_unit_group_uuid`, `exchange_direction_inferred`
//! warning) default to empty/None/false when the input is vanilla
//! ILCD, so callers that don't care about EPDs see no new API surface.
//! See `DECISIONS.md` §D-0009.

use serde::{Deserialize, Serialize};

/// One parsed ILCD process dataset — the result of reading one
/// `<processDataSet>` document.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProcessDataset {
    pub process_information: ProcessInformation,
    pub quantitative_reference: QuantitativeReference,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exchanges: Vec<Exchange>,
    /// Non-fatal anomalies the parser decided to tolerate. Empty for
    /// strict vanilla ILCD; may be non-empty for ILCD+EPD v1.2 inputs
    /// where the EPD extension relaxes some vanilla-ILCD requirements
    /// (e.g. omitting `<exchangeDirection>` on the reference flow).
    /// Caller decides routing — `log`, telemetry, CI gate, whatever.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<ParseWarning>,
}

impl ProcessDataset {
    /// Look up the reference exchange by `dataSetInternalID`. Returns
    /// `None` if the quantitative-reference points at an internal ID
    /// not present in `exchanges` — callers should treat that as
    /// `IlcdError::MissingReferenceFlow` (the reader does this on parse).
    pub fn reference_exchange(&self) -> Option<&Exchange> {
        let target = self.quantitative_reference.reference_to_reference_flow;
        self.exchanges
            .iter()
            .find(|e| e.data_set_internal_id == target)
    }
}

/// `<processInformation>` — identity, naming, geography, time.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProcessInformation {
    /// `<dataSetInformation><common:UUID>` — the dataset's stable
    /// identifier across releases.
    pub uuid: String,
    /// `<dataSetInformation><name><baseName xml:lang="en">`. ILCD allows
    /// multilingual names; we take the first `<baseName>` we encounter
    /// (almost always English in practice).
    pub base_name: String,
    /// Optional `<dataSetInformation><name><treatmentStandardsRoutes>`
    /// qualifier — useful for distinguishing competing variants of the
    /// same nominal product.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub treatment_standards_routes: Option<String>,
    /// `<geography><locationOfOperationSupplyOrProduction location="…"/>`
    /// — ISO 3166 country code or ILCD region code (e.g. `"ES"`,
    /// `"RER"`, `"GLO"`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    /// `<time><common:referenceYear>` — the year the data refers to
    /// (not the dataset publication year).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reference_year: Option<i32>,
}

/// `<quantitativeReference>` — points to which exchange is the
/// declared product. ILCD uses an internal integer ID, not a UUID,
/// because the reference is local to this dataset.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuantitativeReference {
    /// `dataSetInternalID` of the exchange that is the reference flow.
    pub reference_to_reference_flow: i32,
    /// Type attribute, e.g. `"Reference flow(s)"`,
    /// `"Functional unit"`, `"Other reference"`. Preserved verbatim.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
}

/// One `<exchange>` record — either a technosphere flow (links to
/// another `processDataSet`) or an elementary flow (links to a
/// biosphere `flowDataSet`). ILCD does not separate them at the
/// schema level; the distinction is in the linked flow's `<typeOfDataSet>`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Exchange {
    /// `dataSetInternalID` — local integer identity, used by
    /// `<referenceToReferenceFlow>` and by user-facing tooling.
    pub data_set_internal_id: i32,
    /// `refObjectId` of the `<referenceToFlowDataSet>` — UUID of the
    /// `flowDataSet` this exchange points at. The flow itself lives in
    /// a separate XML; the linker resolves it.
    pub flow_uuid: String,
    /// `<common:shortDescription>` from the `referenceToFlowDataSet`
    /// child — human-readable label, often the only thing visible to
    /// reviewers.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flow_short_description: Option<String>,
    /// `uri` attribute of `<referenceToFlowDataSet>`, when present.
    /// Resolves to a canonical location of the flow XML.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flow_uri: Option<String>,
    pub direction: Direction,
    /// `<meanAmount>` — the unscaled, raw amount the dataset author
    /// entered.
    pub mean_amount: f64,
    /// `<resultingAmount>` — `meanAmount` after applying any
    /// parameter / variable scaling. Many ILCD datasets emit only one
    /// of the two; the reader populates whichever is present and falls
    /// back to `meanAmount` if `resultingAmount` is absent.
    pub resulting_amount: f64,
    /// `<referenceToVariable>` — name of a parameter that drives this
    /// exchange's amount, when the dataset is parameterized.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reference_to_variable: Option<String>,
    /// `<dataDerivationTypeStatus>` — provenance hint
    /// (e.g. `"Measured"`, `"Calculated"`, `"Estimated"`,
    /// `"Unknown derivation"`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data_derivation_type_status: Option<String>,
    /// ILCD+EPD v1.2 stage-stratified amounts
    /// (`<epd:amount epd:module="A1-A3">`, etc.) found inside the
    /// exchange's `<c:other>` block. Each entry preserves the module
    /// code (`"A1-A3"`, `"C2"`, `"D"`, …) and optional scenario
    /// (`"Recycled"`, `"Landfilled"`) so downstream calc code can
    /// apply EN 15804+A2 stage rules (e.g. Module D benefits with
    /// negative signs). Empty for vanilla ILCD datasets.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub epd_modules: Vec<EpdModuleAmount>,
    /// ILCD+EPD v1.2 inline `<epd:referenceToUnitGroupDataSet>` — a
    /// UUID that short-circuits the Flow → FlowProperty → UnitGroup
    /// chain for this specific exchange. When present, the bridge
    /// **must** prefer this over the chain walker (§D-0009). Rare on
    /// vanilla ILCD exchanges; ubiquitous on EN 15804 indicator flows.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub epd_unit_group_uuid: Option<String>,
    /// `<common:shortDescription>` text from the inline
    /// `<epd:referenceToUnitGroupDataSet>`, when present. ILCD+EPD
    /// publishers lean on this as the authoritative human-readable
    /// unit label — it's what drives the "kg CO₂-Äq." / "MJ" /
    /// "kg N-Äq." strings in downstream UIs. The bridge uses it as a
    /// fallback label when the inline unit-group UUID points at an
    /// external JRC reference-data file that isn't in the local bundle
    /// (ubiquitous: ÖKOBAUDAT references ~12 JRC unit groups by UUID
    /// but ships only the package-local subset).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub epd_unit_group_short_description: Option<String>,
    /// `true` if the XML did not contain `<exchangeDirection>` and the
    /// parser filled it in as `Output` by convention (ILCD+EPD v1.2
    /// allows this on the reference flow). Always `false` for strict
    /// vanilla ILCD inputs. Surfaces as a `ParseWarning::InferredDirection`
    /// on the dataset so callers can tell "output by declaration" from
    /// "output by convention."
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub exchange_direction_inferred: bool,
}

/// One `<epd:amount epd:module="…" [epd:scenario="…"]>` entry.
///
/// The module code is free-text per EN 15804+A2 (so new stages added
/// in later amendments deserialize transparently). Common values:
/// `"A1-A3"`, `"A4"`, `"A5"`, `"B1"`..`"B7"`, `"C1"`..`"C4"`, `"D"`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EpdModuleAmount {
    pub module: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scenario: Option<String>,
    /// Signed amount. **Module D uses EN 15804+A2 convention: negative
    /// values are environmental benefits** (avoided burdens from
    /// recycling / energy recovery). Parsers do not normalise sign —
    /// the sign the dataset author wrote is the sign we store.
    pub amount: f64,
}

/// A non-fatal parse anomaly. The dataset was accepted but something
/// about it deviated from strict vanilla ILCD; the caller sees the
/// `Vec<ParseWarning>` on `ProcessDataset` and decides what to do with
/// it (log, escalate, gate CI, surface in UI, etc.).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ParseWarning {
    /// An exchange lacked `<exchangeDirection>`; the parser filled in
    /// `Output` per the ILCD+EPD v1.2 reference-flow convention. If
    /// this fires on a non-reference exchange, it indicates a genuinely
    /// malformed dataset — the conventional default may be wrong.
    InferredDirection {
        data_set_internal_id: i32,
        is_reference_flow: bool,
    },
}

/// ILCD `<exchangeDirection>` — strictly `Input` or `Output`. Unlike
/// ecospold2, there is no numeric "group" tagging.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    Input,
    Output,
}

impl Direction {
    pub const fn is_input(self) -> bool {
        matches!(self, Self::Input)
    }

    pub const fn is_output(self) -> bool {
        matches!(self, Self::Output)
    }
}
