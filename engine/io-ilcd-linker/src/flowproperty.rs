//! Minimal `flowPropertyDataSet` model and parser.
//!
//! A FlowProperty names a quantity-kind (Mass, Volume, Energy, Number
//! of items, …) and points at the UnitGroup that defines its reference
//! unit. It's the middle link in the flow → unit resolution chain:
//!
//! ```text
//! Flow --reference-flow-property-id--> FlowProperty --reference-unit-group--> UnitGroup --reference-unit-id--> Unit
//! ```

use crate::error::LinkError;
use crate::xml::{first_child, node_text};
use roxmltree::Document;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// A parsed `flowPropertyDataSet` — only the fields needed to chain
/// through to the unit group.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FlowProperty {
    pub uuid: String,
    pub base_name: String,
    /// UUID of the `unitGroupDataSet` that defines the reference unit
    /// for this property.
    pub reference_unit_group_uuid: String,
}

/// Parse a `<flowPropertyDataSet>` XML document.
pub fn parse_flow_property(xml: &str, path: &Path) -> Result<FlowProperty, LinkError> {
    let doc = Document::parse(xml).map_err(|source| LinkError::Xml {
        path: path.to_path_buf(),
        source,
    })?;
    let root = doc.root_element();
    if !root.has_tag_name("flowPropertyDataSet") {
        return Err(LinkError::UnexpectedRoot {
            path: path.to_path_buf(),
            expected: "flowPropertyDataSet",
            got: root.tag_name().name().to_owned(),
        });
    }

    let fi = first_child(&root, "flowPropertiesInformation").ok_or_else(|| {
        LinkError::MissingElement {
            path: path.to_path_buf(),
            elem: "flowPropertiesInformation",
        }
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

    let qr =
        first_child(&fi, "quantitativeReference").ok_or_else(|| LinkError::MissingElement {
            path: path.to_path_buf(),
            elem: "quantitativeReference",
        })?;
    let ref_node = first_child(&qr, "referenceToReferenceUnitGroup").ok_or_else(|| {
        LinkError::MissingElement {
            path: path.to_path_buf(),
            elem: "referenceToReferenceUnitGroup",
        }
    })?;
    let reference_unit_group_uuid = ref_node
        .attribute("refObjectId")
        .ok_or_else(|| LinkError::MissingAttribute {
            path: path.to_path_buf(),
            elem: "referenceToReferenceUnitGroup",
            attr: "refObjectId",
        })?
        .to_owned();

    Ok(FlowProperty {
        uuid,
        base_name,
        reference_unit_group_uuid,
    })
}
