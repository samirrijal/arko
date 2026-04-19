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
//! # Unit-resolution order (§D-0009)
//!
//! ILCD+EPD v1.2 exchanges (ÖKOBAUDAT, EPD Norge, Environdec) may carry
//! an inline `<epd:referenceToUnitGroupDataSet>` that overrides the
//! flow-dataset's declared reference property. When present, it takes
//! priority and we resolve the unit group directly. If both the inline
//! reference AND the flow-chain resolve — and they **disagree on
//! UUID** — we record a `BridgeWarning::UnitGroupDisagreement` on the
//! column and proceed with the inline one. Silent preference would
//! mask real source-data bugs (kg vs m³ of concrete); a warning lets
//! practitioners audit.
//!
//! # v0.1 scope and assumptions
//!
//! - **Vanilla ILCD amount pass-through.** `resultingAmount` is quoted
//!   in the reference unit. No multi-property unit math (mass ↔ energy
//!   for a fuel) at this stage — that lives in `arko-units` later.
//! - **ILCD+EPD modules are carried as-is.** `epd_modules` is preserved
//!   verbatim on `TypedExchange` — per-stage semantics (EN 15804+A2
//!   Module D sign convention, C-module scenarios) are honoured by
//!   downstream calc code, not collapsed here.
//! - **Fail-fast on hard errors.** First unresolvable reference bubbles
//!   the underlying `LinkError`. For whole-bundle scans, wrap the caller.
//! - **One file read per unique flow.** No cross-exchange caching at
//!   v0.1; directory bundle is fast enough on ÖKOBAUDAT scale.

use arko_io_ilcd::{Direction, EpdModuleAmount, ProcessDataset};
use serde::{Deserialize, Serialize};

use crate::error::LinkError;
use crate::flow::{FlowOrigin, FlowType};
use crate::resolver::{resolve_reference_unit_from_flow, LinkResolver, ReferenceUnit};

/// Where the reference unit for a `TypedExchange` came from.
///
/// Preserved so downstream code (and auditors) can tell an inline-EPD
/// override from a chain-walked default. Vanilla ILCD always produces
/// `FlowChain`; ILCD+EPD indicator flows usually produce `EpdInline`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnitResolutionSource {
    /// Walked flow → flow-property → unit-group (vanilla ILCD path).
    FlowChain,
    /// Read directly from `<epd:referenceToUnitGroupDataSet>` on the
    /// exchange's `<c:other>` block (ILCD+EPD v1.2 path).
    EpdInline,
}

/// A non-fatal anomaly the bridge tolerated. Collected on
/// `TypedColumn::warnings` alongside whatever warnings the underlying
/// `ProcessDataset` already carried.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum BridgeWarning {
    /// Exchange's inline `<epd:referenceToUnitGroupDataSet>` UUID
    /// disagrees with the UUID produced by walking the flow → flow-
    /// property → unit-group chain. The bridge proceeds with the
    /// inline one (per §D-0009) but the practitioner should audit
    /// — this typically signals a flow dataset that was curated in
    /// isolation from the EPD publisher's unit conventions.
    UnitGroupDisagreement {
        data_set_internal_id: i32,
        flow_uuid: String,
        inline_unit_group_uuid: String,
        chain_unit_group_uuid: String,
    },
    /// The inline `<epd:referenceToUnitGroupDataSet>` UUID points at a
    /// file not present in the local bundle — routine for ÖKOBAUDAT,
    /// which references ~12 JRC reference-data UUIDs (`93a60a57-…`,
    /// `1ebf3012-…`, etc.) without shipping their XMLs. The bridge
    /// falls back to the `<common:shortDescription>` text the
    /// publisher placed inline as the unit label, records this
    /// warning, and proceeds.
    InlineUnitGroupUnresolved {
        data_set_internal_id: i32,
        flow_uuid: String,
        inline_unit_group_uuid: String,
        fallback_unit_label: String,
    },
}

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
    /// Carbon-cycle origin classifier copied from the resolved
    /// `Flow.origin`. Drives AR6-style fossil/non-fossil
    /// characterization downstream (chiefly CH4: AR6 GWP100 fossil =
    /// 29.8, non-fossil = 27.0). `Unspecified` is the safe default —
    /// AR6 matchers will skip rather than guess.
    #[serde(default, skip_serializing_if = "FlowOrigin::is_unspecified")]
    pub origin: FlowOrigin,
    /// Vanilla-ILCD `resultingAmount`, quoted in
    /// `reference_unit.unit_name`. For ILCD+EPD indicator flows this
    /// is usually 0.0 (no scalar provided); consult `epd_modules`.
    pub amount: f64,
    pub reference_unit: ReferenceUnit,
    pub unit_source: UnitResolutionSource,
    /// ILCD+EPD v1.2 stage-stratified amounts, pass-through from the
    /// source exchange. Empty for vanilla ILCD rows. Each entry keeps
    /// its module code (`A1-A3`, `C2`, `D`, …) and optional scenario
    /// so downstream calc code can apply EN 15804+A2 sign rules
    /// (Module D negatives = environmental benefits).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub epd_modules: Vec<EpdModuleAmount>,
    /// `true` for the exchange the process dataset's
    /// `<quantitativeReference>` points at. At most one per column.
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
    /// Parse-level anomalies the upstream `ProcessDataset` flagged
    /// (e.g. inferred exchange direction). Pass-through copy.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parse_warnings: Vec<arko_io_ilcd::ParseWarning>,
    /// Bridge-level anomalies this function detected (e.g. inline
    /// vs chain unit-group disagreement).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub bridge_warnings: Vec<BridgeWarning>,
}

/// Walk every exchange in `dataset`, resolve its flow and reference
/// unit through `resolver`, and return the typed column. Fails on the
/// first unresolvable reference; surfaces inline-vs-chain unit-group
/// disagreements as warnings.
pub fn build_typed_column<R: LinkResolver + ?Sized>(
    dataset: &ProcessDataset,
    resolver: &R,
) -> Result<TypedColumn, LinkError> {
    let reference_internal_id = dataset.quantitative_reference.reference_to_reference_flow;

    let mut exchanges = Vec::with_capacity(dataset.exchanges.len());
    let mut bridge_warnings: Vec<BridgeWarning> = Vec::new();

    for ex in &dataset.exchanges {
        let flow = resolver.resolve_flow(&ex.flow_uuid)?;

        // Unit resolution: inline EPD ref takes priority per §D-0009.
        // When both paths resolve, disagreement is logged as a warning
        // rather than silently resolved.
        let (reference_unit, unit_source) = if let Some(inline_uuid) = &ex.epd_unit_group_uuid {
            match resolver.resolve_unit_group(inline_uuid) {
                Ok(ug) => {
                    let unit = ug
                        .reference_unit()
                        .ok_or_else(|| LinkError::MissingInternalId {
                            path: std::path::PathBuf::new(),
                            elem: "units",
                            referrer: "epd:referenceToUnitGroupDataSet",
                            id: ug.reference_unit_id,
                        })?;
                    let ref_unit = ReferenceUnit {
                        flow_uuid: flow.uuid.clone(),
                        flow_name: flow.base_name.clone(),
                        flow_property_uuid: String::new(),
                        flow_property_name: String::new(),
                        unit_group_uuid: ug.uuid.clone(),
                        unit_group_name: ug.base_name.clone(),
                        unit_name: unit.name.clone(),
                    };

                    // Audit pass: if the flow-chain also resolves AND
                    // points at a different unit-group UUID, warn. We
                    // don't fail — preference is fixed (§D-0009) —
                    // but the practitioner needs to know.
                    if let Ok(chain_ru) = resolve_reference_unit_from_flow(resolver, &flow) {
                        if chain_ru.unit_group_uuid != ug.uuid {
                            bridge_warnings.push(BridgeWarning::UnitGroupDisagreement {
                                data_set_internal_id: ex.data_set_internal_id,
                                flow_uuid: ex.flow_uuid.clone(),
                                inline_unit_group_uuid: ug.uuid.clone(),
                                chain_unit_group_uuid: chain_ru.unit_group_uuid,
                            });
                        }
                    }

                    (ref_unit, UnitResolutionSource::EpdInline)
                }
                // Inline UUID not resolvable (common with ÖKOBAUDAT:
                // references JRC reference-data UUIDs it doesn't ship).
                // Fall back to the inline `shortDescription` the
                // publisher provided — it IS the authoritative unit
                // label per ILCD+EPD v1.2 convention. Warn so the
                // practitioner knows the UUID-level link is broken.
                Err(LinkError::Io { .. }) => {
                    let label = ex.epd_unit_group_short_description.clone().ok_or_else(|| {
                        LinkError::MissingElement {
                            path: std::path::PathBuf::new(),
                            elem: "epd:referenceToUnitGroupDataSet/shortDescription",
                        }
                    })?;
                    bridge_warnings.push(BridgeWarning::InlineUnitGroupUnresolved {
                        data_set_internal_id: ex.data_set_internal_id,
                        flow_uuid: ex.flow_uuid.clone(),
                        inline_unit_group_uuid: inline_uuid.clone(),
                        fallback_unit_label: label.clone(),
                    });
                    let ref_unit = ReferenceUnit {
                        flow_uuid: flow.uuid.clone(),
                        flow_name: flow.base_name.clone(),
                        flow_property_uuid: String::new(),
                        flow_property_name: String::new(),
                        unit_group_uuid: inline_uuid.clone(),
                        unit_group_name: String::new(),
                        unit_name: label,
                    };
                    (ref_unit, UnitResolutionSource::EpdInline)
                }
                Err(other) => return Err(other),
            }
        } else {
            let ref_unit = resolve_reference_unit_from_flow(resolver, &flow)?;
            (ref_unit, UnitResolutionSource::FlowChain)
        };

        exchanges.push(TypedExchange {
            data_set_internal_id: ex.data_set_internal_id,
            direction: ex.direction,
            flow_uuid: ex.flow_uuid.clone(),
            flow_name: flow.base_name,
            flow_type: flow.flow_type,
            origin: flow.origin,
            amount: ex.resulting_amount,
            reference_unit,
            unit_source,
            epd_modules: ex.epd_modules.clone(),
            is_reference_flow: ex.data_set_internal_id == reference_internal_id,
        });
    }

    Ok(TypedColumn {
        process_uuid: dataset.process_information.uuid.clone(),
        process_name: dataset.process_information.base_name.clone(),
        reference_exchange_internal_id: reference_internal_id,
        exchanges,
        parse_warnings: dataset.warnings.clone(),
        bridge_warnings,
    })
}
