//! Bridge from `arko-io-ilcd::ProcessDataset` to a typed column that
//! carries each exchange together with its flow's reference unit.
//!
//! The ILCD reader (`arko-io-ilcd`) produces a `ProcessDataset` whose
//! exchanges reference flows by UUID only; amounts are quoted in "the
//! flow's reference unit" without saying which unit that is. The
//! matrix-assembly code downstream needs to know the unit (to keep `A`
//! dimensionally honest) and the flow type (elementary → `B` row,
//! product → `A` column). `build_typed_column` does that resolution in
//! one pass, returning a `TypedColumn` ready for column-builder code.
//!
//! # v0.1 scope and assumptions
//!
//! - **Amount pass-through.** ILCD semantics: `resultingAmount` is in
//!   the flow's reference flow property's reference unit. We take that
//!   at face value and label the amount with the resolved reference
//!   unit. No multi-property unit math (mass ↔ energy for a fuel) at
//!   this stage — that lives in `arko-units` and hooks in later.
//! - **Fail-fast.** The first exchange whose flow (or flow-property,
//!   or unit-group) can't be resolved surfaces the underlying
//!   `LinkError`. No partial columns. For a whole-bundle scan with
//!   per-process error collection, wrap the caller.
//! - **One file read per unique flow.** We don't cache across
//!   exchanges that share a flow UUID — the current directory bundle
//!   is fast enough on ÖKOBAUDAT-scale data. Repeat-flow de-duping
//!   lives with a future caching `LinkResolver`.

use arko_io_ilcd::{Direction, ProcessDataset};
use serde::{Deserialize, Serialize};

use crate::error::LinkError;
use crate::flow::FlowType;
use crate::resolver::{resolve_reference_unit_from_flow, LinkResolver, ReferenceUnit};

/// One exchange from a `ProcessDataset`, enriched with its flow's
/// resolved reference unit and flow type.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypedExchange {
    /// `dataSetInternalID` — preserved so callers can cross-reference
    /// the source XML (e.g. for error messages or UI drill-down).
    pub data_set_internal_id: i32,
    pub direction: Direction,
    pub flow_uuid: String,
    /// Copied from the resolved `Flow.base_name` for convenience.
    /// Prefer this over the exchange's `flow_short_description`, which
    /// can be stale or abbreviated.
    pub flow_name: String,
    /// Elementary vs product vs waste — determines whether this
    /// exchange feeds a row of `B` or a column of `A`.
    pub flow_type: FlowType,
    /// `resultingAmount` from the exchange, quoted in
    /// `reference_unit.unit_name`.
    pub amount: f64,
    pub reference_unit: ReferenceUnit,
    /// `true` for the exchange that the process dataset's
    /// `<quantitativeReference>` points at. At most one exchange per
    /// column has this set.
    pub is_reference_flow: bool,
}

/// A process dataset with every exchange resolved to its reference
/// unit — ready to feed column-builder code.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypedColumn {
    pub process_uuid: String,
    pub process_name: String,
    /// `dataSetInternalID` of the exchange marked as the declared
    /// reference flow. Matches exactly one `TypedExchange`'s
    /// `data_set_internal_id`.
    pub reference_exchange_internal_id: i32,
    pub exchanges: Vec<TypedExchange>,
}

/// Walk every exchange in `dataset`, resolve its flow and reference
/// unit through `resolver`, and return the typed column. Fails on the
/// first unresolvable reference.
pub fn build_typed_column<R: LinkResolver + ?Sized>(
    dataset: &ProcessDataset,
    resolver: &R,
) -> Result<TypedColumn, LinkError> {
    let reference_internal_id = dataset.quantitative_reference.reference_to_reference_flow;

    let mut exchanges = Vec::with_capacity(dataset.exchanges.len());
    for ex in &dataset.exchanges {
        let flow = resolver.resolve_flow(&ex.flow_uuid)?;
        let reference_unit = resolve_reference_unit_from_flow(resolver, &flow)?;
        exchanges.push(TypedExchange {
            data_set_internal_id: ex.data_set_internal_id,
            direction: ex.direction,
            flow_uuid: ex.flow_uuid.clone(),
            flow_name: flow.base_name,
            flow_type: flow.flow_type,
            amount: ex.resulting_amount,
            reference_unit,
            is_reference_flow: ex.data_set_internal_id == reference_internal_id,
        });
    }

    Ok(TypedColumn {
        process_uuid: dataset.process_information.uuid.clone(),
        process_name: dataset.process_information.base_name.clone(),
        reference_exchange_internal_id: reference_internal_id,
        exchanges,
    })
}
