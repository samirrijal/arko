//! Event-free, tree-based ILCD parser built on `roxmltree`.
//!
//! ILCD documents declare two namespaces side-by-side:
//!
//! - `http://lca.jrc.it/ILCD/Process` (or `…/Flow`, `…/UnitGroup`) as
//!   the default namespace.
//! - `http://lca.jrc.it/ILCD/Common` under the `common:` prefix, used
//!   for things like `<common:UUID>` and `<common:shortDescription>`.
//!
//! `roxmltree::Node::has_tag_name(&str)` matches local name only, so
//! we don't have to track the namespace plumbing — `"UUID"` matches
//! both `<UUID>` and `<common:UUID>` transparently.

use crate::{
    error::IlcdError,
    model::{Direction, Exchange, ProcessDataset, ProcessInformation, QuantitativeReference},
};
use roxmltree::{Document, Node};

/// Parse an ILCD `<processDataSet>` XML document into a `ProcessDataset`.
pub fn parse_process(xml: &str) -> Result<ProcessDataset, IlcdError> {
    let doc = Document::parse(xml)?;
    let root = doc.root_element();

    if !root.has_tag_name("processDataSet") {
        return Err(IlcdError::UnexpectedRoot(root.tag_name().name().to_owned()));
    }

    let process_information = parse_process_information(&root)?;
    let quantitative_reference = parse_quantitative_reference(&root)?;
    let exchanges = parse_exchanges(&root)?;

    let target = quantitative_reference.reference_to_reference_flow;
    if !exchanges.iter().any(|e| e.data_set_internal_id == target) {
        return Err(IlcdError::MissingReferenceFlow { id: target });
    }

    Ok(ProcessDataset {
        process_information,
        quantitative_reference,
        exchanges,
    })
}

fn parse_process_information(root: &Node<'_, '_>) -> Result<ProcessInformation, IlcdError> {
    let pi = first_child(root, &["processInformation"])
        .ok_or(IlcdError::MissingElement("processInformation"))?;

    let dsi = first_child(&pi, &["dataSetInformation"])
        .ok_or(IlcdError::MissingElement("dataSetInformation"))?;

    let uuid = first_child(&dsi, &["UUID"])
        .map(text)
        .ok_or(IlcdError::MissingElement("UUID"))?;

    let name_node = first_child(&dsi, &["name"]);
    let base_name = name_node
        .as_ref()
        .and_then(|n| first_child(n, &["baseName"]))
        .map(text)
        .unwrap_or_default();
    let treatment_standards_routes = name_node
        .as_ref()
        .and_then(|n| first_child(n, &["treatmentStandardsRoutes"]))
        .map(text)
        .filter(|s| !s.is_empty());

    let location = first_child(&pi, &["geography"])
        .and_then(|g| first_child(&g, &["locationOfOperationSupplyOrProduction"]))
        .and_then(|n| n.attribute("location").map(str::to_owned));

    let reference_year = first_child(&pi, &["time"])
        .and_then(|t| first_child(&t, &["referenceYear"]))
        .map(|n| {
            let raw = text(n);
            raw.trim()
                .parse::<i32>()
                .map_err(|e| IlcdError::InvalidText {
                    elem: "referenceYear",
                    value: raw,
                    reason: e.to_string(),
                })
        })
        .transpose()?;

    Ok(ProcessInformation {
        uuid,
        base_name,
        treatment_standards_routes,
        location,
        reference_year,
    })
}

fn parse_quantitative_reference(root: &Node<'_, '_>) -> Result<QuantitativeReference, IlcdError> {
    let pi = first_child(root, &["processInformation"])
        .ok_or(IlcdError::MissingElement("processInformation"))?;
    let qr = first_child(&pi, &["quantitativeReference"])
        .ok_or(IlcdError::MissingElement("quantitativeReference"))?;

    let target_node = first_child(&qr, &["referenceToReferenceFlow"])
        .ok_or(IlcdError::MissingElement("referenceToReferenceFlow"))?;
    let raw = text(target_node);
    let reference_to_reference_flow =
        raw.trim()
            .parse::<i32>()
            .map_err(|e| IlcdError::InvalidText {
                elem: "referenceToReferenceFlow",
                value: raw,
                reason: e.to_string(),
            })?;

    let r#type = qr.attribute("type").map(str::to_owned);

    Ok(QuantitativeReference {
        reference_to_reference_flow,
        r#type,
    })
}

fn parse_exchanges(root: &Node<'_, '_>) -> Result<Vec<Exchange>, IlcdError> {
    let Some(exchanges_node) = first_child(root, &["exchanges"]) else {
        return Ok(Vec::new());
    };
    exchanges_node
        .children()
        .filter(|n| n.is_element() && n.has_tag_name("exchange"))
        .map(|n| parse_exchange(&n))
        .collect()
}

fn parse_exchange(node: &Node<'_, '_>) -> Result<Exchange, IlcdError> {
    let data_set_internal_id = attr_parse::<i32>(node, "exchange", "dataSetInternalID")?;

    let flow_ref = first_child(node, &["referenceToFlowDataSet"])
        .ok_or(IlcdError::MissingElement("referenceToFlowDataSet"))?;
    let flow_uuid = attr_required(&flow_ref, "referenceToFlowDataSet", "refObjectId")?;
    let flow_uri = flow_ref.attribute("uri").map(str::to_owned);
    let flow_short_description = first_child(&flow_ref, &["shortDescription"])
        .map(text)
        .filter(|s| !s.is_empty());

    let direction_node = first_child(node, &["exchangeDirection"])
        .ok_or(IlcdError::MissingElement("exchangeDirection"))?;
    let direction = parse_direction(text(direction_node))?;

    let mean_amount = first_child(node, &["meanAmount"])
        .map(|n| parse_finite(n, "meanAmount"))
        .transpose()?;
    let resulting_amount = first_child(node, &["resultingAmount"])
        .map(|n| parse_finite(n, "resultingAmount"))
        .transpose()?;

    let (mean_amount, resulting_amount) = match (mean_amount, resulting_amount) {
        (Some(m), Some(r)) => (m, r),
        (Some(m), None) => (m, m),
        (None, Some(r)) => (r, r),
        (None, None) => return Err(IlcdError::MissingElement("meanAmount")),
    };

    let reference_to_variable = first_child(node, &["referenceToVariable"])
        .map(text)
        .filter(|s| !s.is_empty());

    let data_derivation_type_status = first_child(node, &["dataDerivationTypeStatus"])
        .map(text)
        .filter(|s| !s.is_empty());

    Ok(Exchange {
        data_set_internal_id,
        flow_uuid,
        flow_short_description,
        flow_uri,
        direction,
        mean_amount,
        resulting_amount,
        reference_to_variable,
        data_derivation_type_status,
    })
}

fn parse_direction(raw: String) -> Result<Direction, IlcdError> {
    match raw.trim() {
        "Input" | "input" | "INPUT" => Ok(Direction::Input),
        "Output" | "output" | "OUTPUT" => Ok(Direction::Output),
        _ => Err(IlcdError::InvalidText {
            elem: "exchangeDirection",
            value: raw,
            reason: "expected `Input` or `Output`".to_owned(),
        }),
    }
}

fn parse_finite(node: Node<'_, '_>, elem: &'static str) -> Result<f64, IlcdError> {
    let raw = text(node);
    let v = raw
        .trim()
        .parse::<f64>()
        .map_err(|e| IlcdError::InvalidText {
            elem,
            value: raw.clone(),
            reason: e.to_string(),
        })?;
    if !v.is_finite() {
        return Err(IlcdError::NumericNonfinite {
            field: elem,
            value: raw,
        });
    }
    Ok(v)
}

// ---- helpers ----------------------------------------------------------

fn first_child<'a, 'input>(node: &Node<'a, 'input>, names: &[&str]) -> Option<Node<'a, 'input>> {
    node.children()
        .find(|n| n.is_element() && names.iter().any(|name| n.has_tag_name(*name)))
}

fn text(node: Node<'_, '_>) -> String {
    node.text().map(str::to_owned).unwrap_or_default()
}

fn attr_required(
    node: &Node<'_, '_>,
    elem: &'static str,
    attr: &'static str,
) -> Result<String, IlcdError> {
    node.attribute(attr)
        .map(str::to_owned)
        .ok_or(IlcdError::MissingAttribute { elem, attr })
}

fn attr_parse<T: std::str::FromStr>(
    node: &Node<'_, '_>,
    elem: &'static str,
    attr: &'static str,
) -> Result<T, IlcdError>
where
    T::Err: std::fmt::Display,
{
    let raw = attr_required(node, elem, attr)?;
    raw.parse::<T>().map_err(|e| IlcdError::InvalidAttribute {
        elem,
        attr,
        value: raw,
        reason: e.to_string(),
    })
}
