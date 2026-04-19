//! Minimal `unitGroupDataSet` model and parser.
//!
//! A UnitGroup is the last link in the unit-resolution chain. It names
//! a set of interconvertible units (e.g. "Units of mass" containing
//! `kg`, `g`, `t`) and declares which one is the *reference* — the
//! one that downstream flow amounts are reported in.

use crate::error::LinkError;
use crate::xml::{first_child, node_text, parse_f64, parse_int};
use roxmltree::Document;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// One `<unit>` entry inside a UnitGroup.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Unit {
    /// `dataSetInternalID` — the integer the group's
    /// `referenceToReferenceUnit` points at.
    pub internal_id: i32,
    /// Unit label as written in the XML (`"kg"`, `"MJ"`, `"m3"`, …).
    /// Consumers that want a dimensional analysis should feed this
    /// string into `arko-units::parse_unit_expr`.
    pub name: String,
    /// Conversion factor from this unit to the reference unit:
    /// `amount_in_ref = amount_in_this_unit * mean_value`. For the
    /// reference entry itself, `1.0`.
    pub mean_value: f64,
}

/// A parsed `unitGroupDataSet`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnitGroup {
    pub uuid: String,
    pub base_name: String,
    /// Internal ID of the reference unit.
    pub reference_unit_id: i32,
    pub units: Vec<Unit>,
}

impl UnitGroup {
    /// Lookup the reference unit entry. `None` if the declared
    /// `reference_unit_id` is not present in `units` — callers should
    /// treat that as `LinkError::MissingInternalId`.
    #[must_use]
    pub fn reference_unit(&self) -> Option<&Unit> {
        self.units
            .iter()
            .find(|u| u.internal_id == self.reference_unit_id)
    }
}

/// Parse a `<unitGroupDataSet>` XML document.
pub fn parse_unit_group(xml: &str, path: &Path) -> Result<UnitGroup, LinkError> {
    let doc = Document::parse(xml).map_err(|source| LinkError::Xml {
        path: path.to_path_buf(),
        source,
    })?;
    let root = doc.root_element();
    if !root.has_tag_name("unitGroupDataSet") {
        return Err(LinkError::UnexpectedRoot {
            path: path.to_path_buf(),
            expected: "unitGroupDataSet",
            got: root.tag_name().name().to_owned(),
        });
    }

    let ui =
        first_child(&root, "unitGroupInformation").ok_or_else(|| LinkError::MissingElement {
            path: path.to_path_buf(),
            elem: "unitGroupInformation",
        })?;
    let dsi = first_child(&ui, "dataSetInformation").ok_or_else(|| LinkError::MissingElement {
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
        first_child(&ui, "quantitativeReference").ok_or_else(|| LinkError::MissingElement {
            path: path.to_path_buf(),
            elem: "quantitativeReference",
        })?;
    let ref_text = first_child(&qr, "referenceToReferenceUnit")
        .map(node_text)
        .ok_or_else(|| LinkError::MissingElement {
            path: path.to_path_buf(),
            elem: "referenceToReferenceUnit",
        })?;
    let reference_unit_id = parse_int(&ref_text, "referenceToReferenceUnit", path)?;

    let units = parse_units(&root, path)?;

    Ok(UnitGroup {
        uuid,
        base_name,
        reference_unit_id,
        units,
    })
}

fn parse_units(root: &roxmltree::Node<'_, '_>, path: &Path) -> Result<Vec<Unit>, LinkError> {
    let Some(us) = first_child(root, "units") else {
        return Ok(Vec::new());
    };
    let mut out = Vec::new();
    for u in us.children().filter(|n| n.has_tag_name("unit")) {
        let id_str =
            u.attribute("dataSetInternalID")
                .ok_or_else(|| LinkError::MissingAttribute {
                    path: path.to_path_buf(),
                    elem: "unit",
                    attr: "dataSetInternalID",
                })?;
        let internal_id = parse_int(id_str, "unit", path)?;

        let name = first_child(&u, "name").map(node_text).unwrap_or_default();

        let mean_value = first_child(&u, "meanValue")
            .map(node_text)
            .map_or(Ok(1.0), |s| parse_f64(&s, "meanValue", path))?;

        out.push(Unit {
            internal_id,
            name,
            mean_value,
        });
    }
    Ok(out)
}
