//! Minimal `flowDataSet` model and parser.
//!
//! An ILCD Flow links an elementary or product flow (the thing flowing
//! across the system boundary) to the unit system it's measured in.
//! A Flow references one or more `FlowProperty` datasets; exactly one
//! of them is the *reference* property (the one whose unit is used
//! when reporting amounts).

use crate::error::LinkError;
use crate::xml::{first_child, node_text, parse_f64, parse_int};
use roxmltree::Document;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// ILCD `typeOfDataSet` — whether this flow is elementary (crosses the
/// system boundary to nature), a product (crosses between processes),
/// or waste.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FlowType {
    Elementary,
    Product,
    Waste,
    /// Anything we don't recognise (ILCD allows a few rare types like
    /// `"Other flow"`). Preserved so we don't silently drop unusual
    /// data, but callers generally treat it as opaque.
    Other,
}

/// One `<flowProperty>` entry in the flow's flow-property table —
/// the mapping from internal ID to an external FlowProperty dataset.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FlowPropertyRef {
    /// `dataSetInternalID` — the local integer the flow's
    /// `referenceToReferenceFlowProperty` points at.
    pub internal_id: i32,
    /// UUID of the `flowPropertyDataSet` this entry resolves to.
    pub flow_property_uuid: String,
    /// Conversion factor from *this* flow property's unit to the
    /// reference flow property's unit. For the reference entry itself,
    /// this is `1.0`.
    pub mean_value: f64,
}

/// A parsed `flowDataSet`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Flow {
    pub uuid: String,
    pub base_name: String,
    pub flow_type: FlowType,
    /// CAS number, if the dataset is an elementary chemical flow.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cas: Option<String>,
    /// Internal ID of the reference flow property (the one whose unit
    /// is used for `meanAmount` / `resultingAmount`).
    ///
    /// `None` when the flow dataset omits `<quantitativeReference>`
    /// altogether — routine for ILCD+EPD v1.2 indicator flows (PERE,
    /// GWP, …) whose unit is published inline on the process exchange
    /// via `<epd:referenceToUnitGroupDataSet>`. Bridge code must fall
    /// back to the inline ref in that case; flow-chain resolution is
    /// not available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reference_flow_property_id: Option<i32>,
    /// Flow-property table. Always contains at least the reference
    /// entry; some flows declare multiple (e.g. energy ↔ mass for a
    /// fuel).
    pub flow_properties: Vec<FlowPropertyRef>,
}

impl Flow {
    /// Lookup the reference flow-property entry. `None` if:
    /// - the flow declared no `<quantitativeReference>` at all (common
    ///   for ILCD+EPD indicator flows), or
    /// - it declared one that doesn't match any entry in the flow-
    ///   property table.
    ///
    /// Bridge code distinguishes the two via `reference_flow_property_id`
    /// itself.
    #[must_use]
    pub fn reference_flow_property(&self) -> Option<&FlowPropertyRef> {
        let id = self.reference_flow_property_id?;
        self.flow_properties.iter().find(|fp| fp.internal_id == id)
    }
}

/// Parse a `<flowDataSet>` XML document. The `path` argument is used
/// only for error messages — it does not need to point at a real file.
pub fn parse_flow(xml: &str, path: &Path) -> Result<Flow, LinkError> {
    let doc = Document::parse(xml).map_err(|source| LinkError::Xml {
        path: path.to_path_buf(),
        source,
    })?;
    let root = doc.root_element();
    if !root.has_tag_name("flowDataSet") {
        return Err(LinkError::UnexpectedRoot {
            path: path.to_path_buf(),
            expected: "flowDataSet",
            got: root.tag_name().name().to_owned(),
        });
    }

    let fi = first_child(&root, "flowInformation").ok_or_else(|| LinkError::MissingElement {
        path: path.to_path_buf(),
        elem: "flowInformation",
    })?;
    let dsi = first_child(&fi, "dataSetInformation").ok_or_else(|| LinkError::MissingElement {
        path: path.to_path_buf(),
        elem: "dataSetInformation",
    })?;
    let uuid =
        first_child(&dsi, "UUID")
            .map(node_text)
            .ok_or_else(|| LinkError::MissingElement {
                path: path.to_path_buf(),
                elem: "UUID",
            })?;
    let base_name = first_child(&dsi, "name")
        .and_then(|n| first_child(&n, "baseName"))
        .map(node_text)
        .unwrap_or_default();
    let cas = first_child(&dsi, "CASNumber")
        .map(node_text)
        .filter(|s| !s.is_empty());

    // `<quantitativeReference>` is required by the vanilla ILCD schema
    // but ILCD+EPD v1.2 indicator flows (PERE, PERM, GWP, and the rest
    // of the EN 15804+A2 catalogue) routinely omit it — their "unit"
    // is injected at process-exchange time via
    // `<epd:referenceToUnitGroupDataSet>`. Treat its absence as
    // "no flow-chain unit available"; the bridge falls back to the
    // inline ref.
    let reference_flow_property_id = match first_child(&fi, "quantitativeReference") {
        Some(qr) => {
            let ref_text = first_child(&qr, "referenceToReferenceFlowProperty")
                .map(node_text)
                .ok_or_else(|| LinkError::MissingElement {
                    path: path.to_path_buf(),
                    elem: "referenceToReferenceFlowProperty",
                })?;
            Some(parse_int(
                &ref_text,
                "referenceToReferenceFlowProperty",
                path,
            )?)
        }
        None => None,
    };

    let flow_type = first_child(&root, "modellingAndValidation")
        .and_then(|mv| first_child(&mv, "LCIMethod"))
        .and_then(|lm| first_child(&lm, "typeOfDataSet"))
        .map(node_text)
        .as_deref()
        .map_or(FlowType::Other, classify_flow_type);

    let flow_properties = parse_flow_properties(&root, path)?;

    Ok(Flow {
        uuid,
        base_name,
        flow_type,
        cas,
        reference_flow_property_id,
        flow_properties,
    })
}

fn parse_flow_properties(
    root: &roxmltree::Node<'_, '_>,
    path: &Path,
) -> Result<Vec<FlowPropertyRef>, LinkError> {
    let Some(fps) = first_child(root, "flowProperties") else {
        return Ok(Vec::new());
    };
    let mut out = Vec::new();
    for fp in fps.children().filter(|n| n.has_tag_name("flowProperty")) {
        let internal_id_str =
            fp.attribute("dataSetInternalID")
                .ok_or_else(|| LinkError::MissingAttribute {
                    path: path.to_path_buf(),
                    elem: "flowProperty",
                    attr: "dataSetInternalID",
                })?;
        let internal_id = parse_int(internal_id_str, "flowProperty", path)?;

        let ref_node = first_child(&fp, "referenceToFlowPropertyDataSet").ok_or_else(|| {
            LinkError::MissingElement {
                path: path.to_path_buf(),
                elem: "referenceToFlowPropertyDataSet",
            }
        })?;
        let flow_property_uuid = ref_node
            .attribute("refObjectId")
            .ok_or_else(|| LinkError::MissingAttribute {
                path: path.to_path_buf(),
                elem: "referenceToFlowPropertyDataSet",
                attr: "refObjectId",
            })?
            .to_owned();

        let mean_value = first_child(&fp, "meanValue")
            .map(node_text)
            .map_or(Ok(1.0), |s| parse_f64(&s, "meanValue", path))?;

        out.push(FlowPropertyRef {
            internal_id,
            flow_property_uuid,
            mean_value,
        });
    }
    Ok(out)
}

fn classify_flow_type(s: &str) -> FlowType {
    match s.trim() {
        "Elementary flow" => FlowType::Elementary,
        "Product flow" => FlowType::Product,
        "Waste flow" => FlowType::Waste,
        _ => FlowType::Other,
    }
}
