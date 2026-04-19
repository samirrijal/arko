//! ILCD domain model — a *narrow* Rust representation of one ILCD
//! process dataset, wide enough to seed a column of `A` (and rows of
//! `B`) but not so wide it tracks every optional JRC field.
//!
//! The naming follows the ILCD format ("dataSetInternalID",
//! "meanAmount", "exchangeDirection") rather than ecoinvent terminology
//! so that round-trips with EU-side tooling stay legible.

use serde::{Deserialize, Serialize};

/// One parsed ILCD process dataset — the result of reading one
/// `<processDataSet>` document.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProcessDataset {
    pub process_information: ProcessInformation,
    pub quantitative_reference: QuantitativeReference,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exchanges: Vec<Exchange>,
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
