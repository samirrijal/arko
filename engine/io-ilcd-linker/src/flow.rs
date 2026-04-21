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

/// Carbon-cycle origin classifier for elementary flows whose
/// characterization factor depends on fossil vs non-fossil provenance
/// (chiefly CH4 under IPCC AR6 GWP100, where fossil = 29.8 and
/// non-fossil = 27.0).
///
/// This mirrors `arko_core::meta::FlowOrigin` rather than depending on
/// it — the linker is a reader/bridge layer and shouldn't pull in the
/// engine-side meta types. Callers that produce `FlowMeta` from a
/// linker `Flow` translate at the boundary (the smoke test
/// `ef_carpet_calc_smoke.rs` is the current example).
///
/// # Source of the classification
///
/// JRC EF / ÖKOBAUDAT / ILCD-network publishers encode the origin in
/// the flow `<baseName>` as a trailing parenthetical:
///
/// - `methane (fossil)` → `Fossil`
/// - `methane (biogenic)` → `Biogenic`
/// - `methane (land use change)` → `LandUseChange`
///
/// The parser ([`classify_flow_origin`]) recognises these and a small
/// set of defensive synonyms; anything unrecognised falls through to
/// `Unspecified` rather than guessing.
///
/// The 2026-04-21 split of the historical `NonFossil` variant into
/// `Biogenic` and `LandUseChange` mirrors the change in
/// `arko_core::meta::FlowOrigin`. Before the split, this parser
/// classified `methane (land use change)` as `NonFossil` and the
/// engine routed it through AR6's biogenic CH4 CF (27.0), silently
/// wrong: AR6 GWP100 treats LULUC CH4 as fossil-equivalent (29.8).
/// EF 3.1 makes the same distinction explicit. The two-variant
/// resolution preserves correctness for both methods.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FlowOrigin {
    /// Carbon released from a geological reservoir (oil, gas, coal,
    /// peat, limestone). AR6 GWP100 CH4 = 29.8.
    Fossil,
    /// Carbon released from the contemporary biosphere: biogenic
    /// emissions, short-cycle carbon, combustion of recent-
    /// photosynthesis carbon. AR6 GWP100 CH4 = 27.0. **Excludes**
    /// land-use-change emissions, which carry their own variant
    /// because their CFs differ across methods (AR6/EF 3.1 group
    /// LULUC CH4 with fossil at 29.8, not with biogenic at 27.0).
    Biogenic,
    /// Carbon released as part of land-use-change accounting:
    /// deforestation, peatland conversion, soil-carbon stock change.
    /// Distinct from `Biogenic` because EF 3.1 and AR6 GWP100 group
    /// LULUC CH4 with fossil CH4 at 29.8, not with biogenic at 27.0.
    /// Method authors must provide an explicit `LandUseChange` CF if
    /// they want LULUC flows characterised; the engine does not
    /// silently fold LULUC into `Fossil` or `Biogenic`.
    LandUseChange,
    /// Either the flow is not origin-sensitive (CO2, N2O, …) or the
    /// publisher did not classify it. Per spec §matching, AR6's
    /// `CasOrigin` matchers do **not** match `Unspecified` — missing
    /// information surfaces as `unmatched_flows` rather than being
    /// silently characterized.
    #[default]
    Unspecified,
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
    /// Fossil / non-fossil / unspecified, derived from the basename
    /// parenthetical. See [`FlowOrigin`] for the classification
    /// vocabulary and rationale.
    #[serde(default, skip_serializing_if = "FlowOrigin::is_unspecified")]
    pub origin: FlowOrigin,
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

impl FlowOrigin {
    /// `true` iff this origin carries no information — used by the
    /// `skip_serializing_if` guard so default-valued fields stay out
    /// of serialized JSON.
    #[must_use]
    pub fn is_unspecified(&self) -> bool {
        matches!(self, Self::Unspecified)
    }
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

    let origin = classify_flow_origin(&base_name);

    let flow_properties = parse_flow_properties(&root, path)?;

    Ok(Flow {
        uuid,
        base_name,
        flow_type,
        origin,
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

/// Derive [`FlowOrigin`] from a flow's `<baseName>`.
///
/// The basename's trailing parenthetical carries the publisher's
/// origin classification, e.g. `methane (fossil)` /
/// `methane (biogenic)` / `methane (land use change)`. Recognised
/// tags map to `Fossil` / `Biogenic` / `LandUseChange`; anything else
/// (no parenthetical, unknown vocabulary) falls through to
/// `Unspecified` so that origin-specific matchers surface the gap
/// rather than silently guess.
///
/// Comparison is case-insensitive on the inner tag. The recognised
/// vocabulary covers JRC EF / ÖKOBAUDAT / ILCD-network publishers
/// observed in the wild as of v0.0.1; new variants land here as we
/// encounter them.
///
/// `"non-fossil"` is treated as `Biogenic` for backward compatibility
/// with publishers that haven't adopted the LULUC distinction —
/// `non-fossil` historically meant "biogenic", since LULUC is rarely
/// tagged that way at the publisher layer. `"short cycle"` and
/// `"from soil or biomass stocks"` are biogenic synonyms in the
/// publisher vocabulary.
fn classify_flow_origin(base_name: &str) -> FlowOrigin {
    let trimmed = base_name.trim();
    let Some(open) = trimmed.rfind('(') else {
        return FlowOrigin::Unspecified;
    };
    if !trimmed.ends_with(')') {
        return FlowOrigin::Unspecified;
    }
    // Inner = the contents between the last `(` and the trailing `)`.
    let inner = trimmed[open + 1..trimmed.len() - 1]
        .trim()
        .to_ascii_lowercase();
    match inner.as_str() {
        "fossil" => FlowOrigin::Fossil,
        "biogenic"
        | "non-fossil"
        | "short cycle"
        | "from soil or biomass stocks" => FlowOrigin::Biogenic,
        "land use change" => FlowOrigin::LandUseChange,
        _ => FlowOrigin::Unspecified,
    }
}

#[cfg(test)]
mod origin_tests {
    use super::{classify_flow_origin, FlowOrigin};

    #[test]
    fn fossil_parenthetical_is_fossil() {
        assert_eq!(
            classify_flow_origin("methane (fossil)"),
            FlowOrigin::Fossil
        );
        assert_eq!(
            classify_flow_origin("Carbon dioxide (fossil)"),
            FlowOrigin::Fossil
        );
    }

    #[test]
    fn biogenic_and_synonyms_classify_as_biogenic() {
        // The pre-2026-04-21 enum collapsed `biogenic`,
        // `non-fossil`, `short cycle`, and `from soil or biomass
        // stocks` into one variant alongside `land use change`. The
        // biogenic synonyms remain together; LULUC was split out
        // (see `land_use_change_classifies_as_land_use_change`).
        for name in [
            "methane (biogenic)",
            "carbon dioxide (non-fossil)",
            "methane (short cycle)",
            "carbon dioxide (from soil or biomass stocks)",
        ] {
            assert_eq!(
                classify_flow_origin(name),
                FlowOrigin::Biogenic,
                "expected Biogenic for {name}"
            );
        }
    }

    #[test]
    fn land_use_change_classifies_as_land_use_change() {
        // Pre-2026-04-21 this returned `NonFossil` and the engine
        // routed LULUC methane through AR6's biogenic CH4 CF (27.0)
        // — silently wrong. AR6 GWP100 and EF 3.1 both treat LULUC
        // CH4 as fossil-equivalent (29.8). The dedicated variant
        // forces method presets to make the LULUC semantic explicit.
        assert_eq!(
            classify_flow_origin("methane (land use change)"),
            FlowOrigin::LandUseChange
        );
        assert_eq!(
            classify_flow_origin("Carbon dioxide (land use change)"),
            FlowOrigin::LandUseChange
        );
    }

    #[test]
    fn case_is_insensitive_inside_parens() {
        assert_eq!(
            classify_flow_origin("methane (FOSSIL)"),
            FlowOrigin::Fossil
        );
        assert_eq!(
            classify_flow_origin("methane (Biogenic)"),
            FlowOrigin::Biogenic
        );
        assert_eq!(
            classify_flow_origin("methane (Land Use Change)"),
            FlowOrigin::LandUseChange
        );
    }

    #[test]
    fn no_parenthetical_is_unspecified() {
        assert_eq!(classify_flow_origin("Carbon dioxide"), FlowOrigin::Unspecified);
        assert_eq!(classify_flow_origin("water"), FlowOrigin::Unspecified);
    }

    #[test]
    fn unrecognised_parenthetical_is_unspecified() {
        // We don't guess. AR6 surfacing the flow as unmatched is
        // preferable to silently characterizing it under the wrong
        // factor.
        assert_eq!(
            classify_flow_origin("methane (high-altitude)"),
            FlowOrigin::Unspecified
        );
        assert_eq!(
            classify_flow_origin("methane (anaerobic digestion)"),
            FlowOrigin::Unspecified
        );
    }

    #[test]
    fn only_trailing_parenthetical_is_inspected() {
        // A non-trailing parenthetical (e.g. an inline qualifier) is
        // ignored; we only look at the last `(...)` group, and only
        // when it sits at the end of the basename.
        assert_eq!(
            classify_flow_origin("methane (anthropogenic) emission"),
            FlowOrigin::Unspecified,
            "non-trailing parenthetical should not classify"
        );
        // Last parenthetical wins when basename ends with one.
        assert_eq!(
            classify_flow_origin("methane (urban) (fossil)"),
            FlowOrigin::Fossil
        );
    }

    #[test]
    fn whitespace_trimmed_inside_parens() {
        assert_eq!(
            classify_flow_origin("methane (  fossil  )"),
            FlowOrigin::Fossil
        );
    }
}
