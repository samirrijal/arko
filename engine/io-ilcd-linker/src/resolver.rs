//! The `LinkResolver` trait and the flow → reference-unit chain walk.
//!
//! Every bundle backend (directory today, zip or HTTP tomorrow)
//! implements `LinkResolver`. The free function `resolve_reference_unit`
//! is the canonical way to ask "given a flow UUID, what unit is its
//! reference amount reported in?"

use crate::error::LinkError;
use crate::flow::Flow;
use crate::flowproperty::FlowProperty;
use crate::unitgroup::UnitGroup;
use serde::{Deserialize, Serialize};

/// Something that can find Flow / FlowProperty / UnitGroup datasets
/// by UUID. Implementations are free to lazy-load, cache, or go over
/// the network — the trait only demands a UUID-keyed lookup.
pub trait LinkResolver {
    fn resolve_flow(&self, uuid: &str) -> Result<Flow, LinkError>;
    fn resolve_flow_property(&self, uuid: &str) -> Result<FlowProperty, LinkError>;
    fn resolve_unit_group(&self, uuid: &str) -> Result<UnitGroup, LinkError>;
}

/// The walked chain from a flow UUID to its reference unit — returned
/// as one bundle so callers can attribute a unit to an exchange
/// without re-walking the graph.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReferenceUnit {
    pub flow_uuid: String,
    pub flow_name: String,
    pub flow_property_uuid: String,
    pub flow_property_name: String,
    pub unit_group_uuid: String,
    pub unit_group_name: String,
    /// The unit label itself (e.g. `"kg"`, `"MJ"`).
    pub unit_name: String,
}

/// Walk flow → flow-property → unit-group → reference-unit and return
/// the chain. Fails with a typed `LinkError` at the first missing link.
pub fn resolve_reference_unit<R: LinkResolver + ?Sized>(
    resolver: &R,
    flow_uuid: &str,
) -> Result<ReferenceUnit, LinkError> {
    let flow = resolver.resolve_flow(flow_uuid)?;
    resolve_reference_unit_from_flow(resolver, &flow)
}

/// Chain walk starting from an already-resolved `Flow`. Lets bridge
/// code (e.g. `build_typed_column`) that already loaded a flow avoid a
/// duplicate file read.
pub fn resolve_reference_unit_from_flow<R: LinkResolver + ?Sized>(
    resolver: &R,
    flow: &Flow,
) -> Result<ReferenceUnit, LinkError> {
    let ref_fp_ref =
        flow.reference_flow_property()
            .ok_or_else(|| LinkError::MissingInternalId {
                path: std::path::PathBuf::new(),
                elem: "flowProperties",
                referrer: "referenceToReferenceFlowProperty",
                id: flow.reference_flow_property_id,
            })?;

    let flow_property = resolver.resolve_flow_property(&ref_fp_ref.flow_property_uuid)?;
    let unit_group = resolver.resolve_unit_group(&flow_property.reference_unit_group_uuid)?;
    let unit = unit_group
        .reference_unit()
        .ok_or_else(|| LinkError::MissingInternalId {
            path: std::path::PathBuf::new(),
            elem: "units",
            referrer: "referenceToReferenceUnit",
            id: unit_group.reference_unit_id,
        })?;

    Ok(ReferenceUnit {
        flow_uuid: flow.uuid.clone(),
        flow_name: flow.base_name.clone(),
        flow_property_uuid: flow_property.uuid.clone(),
        flow_property_name: flow_property.base_name.clone(),
        unit_group_uuid: unit_group.uuid.clone(),
        unit_group_name: unit_group.base_name.clone(),
        unit_name: unit.name.clone(),
    })
}
