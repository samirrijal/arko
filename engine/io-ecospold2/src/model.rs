//! ecospold2 domain model — a *narrow* Rust representation of one
//! activity dataset. Wide enough to build a matrix column from; not so
//! wide it tracks every optional ecoinvent field.

use serde::{Deserialize, Serialize};

/// One parsed activity dataset — the result of reading one ecospold2 XML.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActivityDataset {
    pub activity: Activity,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub geography: Option<Geography>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub intermediate_exchanges: Vec<IntermediateExchange>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub elementary_exchanges: Vec<ElementaryExchange>,
}

/// `<activity>` — identity + naming for the activity dataset.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Activity {
    /// UUID of this activity instance.
    pub id: String,
    /// UUID of the activity name (master data reference).
    pub activity_name_id: String,
    /// Human-readable English name (`<activityName xml:lang="en">`).
    pub name: String,
    /// Activity type code — ecoinvent uses 1 = unit process, 2 = system,
    /// etc. See ecoinvent master data for the full enumeration.
    pub activity_type: u8,
    /// Optional `specialActivityType` attribute (used for allocation /
    /// aggregation flags in some ecoinvent releases).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub special_activity_type: Option<u8>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Geography {
    /// Short name, e.g. `"GLO"`, `"RER"`, `"ES"`.
    pub short_name: String,
}

/// An intermediate (technosphere) exchange — a column entry in A.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IntermediateExchange {
    /// UUID of this exchange instance.
    pub id: String,
    /// UUID of the intermediate-flow master record.
    pub intermediate_exchange_id: String,
    /// UUID of the linked source activity, if this is an input.
    /// Populated when this input is resolved against the master dataset.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub activity_link_id: Option<String>,
    /// Human-readable English name.
    pub name: String,
    /// Signed amount per unit of the reference product.
    pub amount: f64,
    /// Unit string (ecoinvent uses `"kg"`, `"MJ"`, `"m3"`, `"kWh"`, ...).
    pub unit_name: String,
    pub direction: Direction,
}

impl IntermediateExchange {
    /// True iff this exchange is the activity's reference product
    /// (output group 0 per ecoinvent conventions).
    pub fn is_reference_product(&self) -> bool {
        matches!(self.direction, Direction::Output { group: 0 })
    }

    /// True iff this exchange is a co-product (output group 2).
    pub fn is_by_product(&self) -> bool {
        matches!(self.direction, Direction::Output { group: 2 })
    }
}

/// An elementary (biosphere) exchange — a column entry in B.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ElementaryExchange {
    /// UUID of this exchange instance.
    pub id: String,
    /// UUID of the elementary-flow master record.
    pub elementary_exchange_id: String,
    /// Human-readable English name.
    pub name: String,
    pub amount: f64,
    pub unit_name: String,
    /// Top-level compartment, e.g. `"air"`, `"water"`, `"soil"`,
    /// `"natural resource"`.
    pub compartment: String,
    /// Subcompartment, e.g. `"urban air close to ground"`,
    /// `"surface water"`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subcompartment: Option<String>,
    /// Chemical Abstracts Service number, when assigned.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cas_number: Option<String>,
    pub direction: Direction,
}

/// Whether an exchange is an input or an output, plus the raw group
/// number (0-5) that ecoinvent attaches for finer categorization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Direction {
    /// `<inputGroup>` element. Group meanings for intermediates: 1=materials,
    /// 2=energy, 3=services, 5=from technosphere. For elementaries: 4=natural
    /// resource, 5=land transformation (in v2).
    Input { group: u8 },
    /// `<outputGroup>` element. Group meanings for intermediates:
    /// 0=reference product, 2=by-product. For elementaries: 4=emissions
    /// (to whatever compartment is declared).
    Output { group: u8 },
}

impl Direction {
    pub const fn is_input(self) -> bool {
        matches!(self, Self::Input { .. })
    }

    pub const fn is_output(self) -> bool {
        matches!(self, Self::Output { .. })
    }

    pub const fn group(self) -> u8 {
        match self {
            Self::Input { group } | Self::Output { group } => group,
        }
    }
}
