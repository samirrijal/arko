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
    model::{
        Direction, EpdModuleAmount, Exchange, ParseWarning, ProcessDataset, ProcessInformation,
        QuantitativeReference,
    },
};
use roxmltree::{Document, Node};

/// Parse an ILCD `<processDataSet>` XML document into a `ProcessDataset`.
///
/// Accepts both strict vanilla ILCD and ILCD+EPD v1.2 (ÖKOBAUDAT, EPD
/// Norge, Environdec construction category). EPD extensions surface on
/// `Exchange::epd_modules` / `epd_unit_group_uuid`; non-fatal deviations
/// (e.g. reference-flow exchange omitting `<exchangeDirection>`) are
/// collected on `ProcessDataset::warnings` rather than silently accepted.
pub fn parse_process(xml: &str) -> Result<ProcessDataset, IlcdError> {
    let doc = Document::parse(xml)?;
    let root = doc.root_element();

    if !root.has_tag_name("processDataSet") {
        return Err(IlcdError::UnexpectedRoot(root.tag_name().name().to_owned()));
    }

    let process_information = parse_process_information(&root)?;
    let quantitative_reference = parse_quantitative_reference(&root)?;
    let reference_internal_id = quantitative_reference.reference_to_reference_flow;
    let (exchanges, warnings) = parse_exchanges(&root, reference_internal_id)?;

    if !exchanges
        .iter()
        .any(|e| e.data_set_internal_id == reference_internal_id)
    {
        return Err(IlcdError::MissingReferenceFlow {
            id: reference_internal_id,
        });
    }

    Ok(ProcessDataset {
        process_information,
        quantitative_reference,
        exchanges,
        warnings,
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

fn parse_exchanges(
    root: &Node<'_, '_>,
    reference_internal_id: i32,
) -> Result<(Vec<Exchange>, Vec<ParseWarning>), IlcdError> {
    let Some(exchanges_node) = first_child(root, &["exchanges"]) else {
        return Ok((Vec::new(), Vec::new()));
    };
    let mut exchanges = Vec::new();
    let mut warnings = Vec::new();
    for n in exchanges_node
        .children()
        .filter(|n| n.is_element() && n.has_tag_name("exchange"))
    {
        exchanges.push(parse_exchange(&n, reference_internal_id, &mut warnings)?);
    }
    Ok((exchanges, warnings))
}

fn parse_exchange(
    node: &Node<'_, '_>,
    reference_internal_id: i32,
    warnings: &mut Vec<ParseWarning>,
) -> Result<Exchange, IlcdError> {
    let data_set_internal_id = attr_parse::<i32>(node, "exchange", "dataSetInternalID")?;

    let flow_ref = first_child(node, &["referenceToFlowDataSet"])
        .ok_or(IlcdError::MissingElement("referenceToFlowDataSet"))?;
    let flow_uuid = attr_required(&flow_ref, "referenceToFlowDataSet", "refObjectId")?;
    let flow_uri = flow_ref.attribute("uri").map(str::to_owned);
    let flow_short_description = first_child(&flow_ref, &["shortDescription"])
        .map(text)
        .filter(|s| !s.is_empty());

    // Vanilla ILCD requires <exchangeDirection>. ILCD+EPD v1.2
    // routinely omits it on the reference-flow exchange (direction is
    // implicitly Output by EN 15804 convention). We default and WARN
    // rather than silently filling in — the dataset's intent may have
    // been to mark an Input-direction reference flow on a waste-
    // treatment process, where the conventional default is wrong.
    let (direction, exchange_direction_inferred) =
        match first_child(node, &["exchangeDirection"]).map(text) {
            Some(raw) if !raw.trim().is_empty() => (parse_direction(raw)?, false),
            _ => {
                let is_ref = data_set_internal_id == reference_internal_id;
                warnings.push(ParseWarning::InferredDirection {
                    data_set_internal_id,
                    is_reference_flow: is_ref,
                });
                (Direction::Output, true)
            }
        };

    let mean_amount = first_child(node, &["meanAmount"])
        .map(|n| parse_finite(n, "meanAmount"))
        .transpose()?;
    let resulting_amount = first_child(node, &["resultingAmount"])
        .map(|n| parse_finite(n, "resultingAmount"))
        .transpose()?;

    let reference_to_variable = first_child(node, &["referenceToVariable"])
        .map(text)
        .filter(|s| !s.is_empty());

    let data_derivation_type_status = first_child(node, &["dataDerivationTypeStatus"])
        .map(text)
        .filter(|s| !s.is_empty());

    let EpdExtensions {
        modules: epd_modules,
        unit_group_uuid: epd_unit_group_uuid,
        unit_group_short_description: epd_unit_group_short_description,
        saw_any_amount_element: has_epd_amount_element,
    } = parse_epd_extensions(node)?;

    // Vanilla ILCD requires meanAmount and/or resultingAmount.
    // ILCD+EPD v1.2 indicator exchanges (PERE, PERM, GWP…, and the
    // "general reminder flows" in ÖKOBAUDAT) carry only <epd:amount>
    // module entries — we record the vanilla scalar as 0.0 and callers
    // consult `epd_modules` for the real signal.
    //
    // Edge case: an EPD exchange whose every module is INA (empty
    // `<epd:amount>` text). Those entries are dropped in
    // parse_epd_extensions, so `epd_modules` can be empty even though
    // the exchange *intended* to declare modules. Use the "did we see
    // any <epd:amount> element at all" signal (including empty ones)
    // to accept this shape — it is a declared-but-not-assessed row,
    // not a malformed one.
    let (mean_amount, resulting_amount) = match (mean_amount, resulting_amount) {
        (Some(m), Some(r)) => (m, r),
        (Some(m), None) => (m, m),
        (None, Some(r)) => (r, r),
        (None, None) if !epd_modules.is_empty() || has_epd_amount_element => (0.0, 0.0),
        (None, None) => return Err(IlcdError::MissingElement("meanAmount")),
    };

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
        epd_modules,
        epd_unit_group_uuid,
        epd_unit_group_short_description,
        exchange_direction_inferred,
    })
}

/// Parsed ILCD+EPD v1.2 extension payload from an exchange's
/// `<c:other>` block. `saw_any_amount_element` is `true` even when every
/// `<epd:amount>` element had empty (INA) text — it tells the caller
/// "this exchange declared itself as EPD-indicator-shaped, even
/// though no numeric values came through". `unit_group_short_description`
/// is the human-readable unit label (`"kg"`, `"MJ"`, `"kg CO₂-Äq."`)
/// from inside `<epd:referenceToUnitGroupDataSet>`. All fields are
/// empty / None / false for vanilla ILCD exchanges.
struct EpdExtensions {
    modules: Vec<EpdModuleAmount>,
    unit_group_uuid: Option<String>,
    unit_group_short_description: Option<String>,
    saw_any_amount_element: bool,
}

/// Pull EPD v1.2 extensions out of the exchange's `<c:other>` block.
fn parse_epd_extensions(node: &Node<'_, '_>) -> Result<EpdExtensions, IlcdError> {
    let Some(other) = first_child(node, &["other"]) else {
        return Ok(EpdExtensions {
            modules: Vec::new(),
            unit_group_uuid: None,
            unit_group_short_description: None,
            saw_any_amount_element: false,
        });
    };

    let mut modules = Vec::new();
    let mut unit_group_uuid = None;
    let mut unit_group_short_description: Option<String> = None;
    let mut saw_any_amount_element = false;
    for child in other.children().filter(Node::is_element) {
        let local = child.tag_name().name();
        if local == "amount" {
            saw_any_amount_element = true;
            // ILCD+EPD v1.2 publishers disagree on whether `module` /
            // `scenario` are prefixed (`epd:module`) or unprefixed; the
            // iai.kit.edu/EPD/2013 spec uses the latter, but real feeds
            // (including some ÖKOBAUDAT vintages) emit the former.
            // Match by local-name regardless of namespace.
            let module = attr_by_local_name(&child, "module")
                .ok_or(IlcdError::MissingAttribute {
                    elem: "epd:amount",
                    attr: "module",
                })?
                .to_owned();
            let scenario = attr_by_local_name(&child, "scenario").map(str::to_owned);
            // EN 15804+A2 / ILCD+EPD v1.2: an `<epd:amount>` with empty
            // or whitespace-only text means "indicator not assessed"
            // (INA) for that module/scenario. That is semantically
            // distinct from zero, and publishers emit it prolifically
            // for biogenic carbon, land use change, PM, etc. Drop the
            // row silently — downstream code can treat an absent module
            // entry as INA — rather than invent a zero.
            let raw = text(child);
            if raw.trim().is_empty() {
                continue;
            }
            let amount = raw
                .trim()
                .parse::<f64>()
                .map_err(|e| IlcdError::InvalidText {
                    elem: "epd:amount",
                    value: raw.clone(),
                    reason: e.to_string(),
                })
                .and_then(|v| {
                    if v.is_finite() {
                        Ok(v)
                    } else {
                        Err(IlcdError::InvalidText {
                            elem: "epd:amount",
                            value: raw,
                            reason: "non-finite amount".to_owned(),
                        })
                    }
                })?;
            modules.push(EpdModuleAmount {
                module,
                scenario,
                amount,
            });
        } else if local == "referenceToUnitGroupDataSet" && unit_group_uuid.is_none() {
            // First inline unit-group ref wins if there are (illegally)
            // multiple — downstream bridge will still warn on any
            // disagreement with the flow → flow-property chain.
            unit_group_uuid = child.attribute("refObjectId").map(str::to_owned);
            // Capture the inline `<common:shortDescription>` text —
            // bridge code uses it as a fallback label when the UUID
            // points at an external JRC reference dataset that isn't
            // in the local bundle.
            unit_group_short_description = first_child(&child, &["shortDescription"])
                .map(text)
                .filter(|s| !s.trim().is_empty());
        }
    }

    Ok(EpdExtensions {
        modules,
        unit_group_uuid,
        unit_group_short_description,
        saw_any_amount_element,
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

/// Look up an attribute by local name, ignoring namespace.
///
/// roxmltree's `node.attribute("foo")` only matches attributes with an
/// empty namespace; namespace-prefixed attributes (`epd:module`) are
/// invisible to it. For EPD extension parsing we want either form to
/// resolve, so we scan the attribute list by local name.
fn attr_by_local_name<'a>(node: &Node<'a, '_>, local: &str) -> Option<&'a str> {
    node.attributes()
        .find(|a| a.name() == local)
        .map(|a| a.value())
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
