//! Event-free, tree-based ecospold2 parser built on `roxmltree`.
//!
//! We lean on `roxmltree::Node::has_tag_name` which matches on local
//! name only — so the default namespace declared on `<ecoSpold>`
//! (`http://www.EcoInvent.org/EcoSpold02`) is correctly ignored without
//! our having to track it.

use crate::{
    error::Ecospold2Error,
    model::{
        Activity, ActivityDataset, Direction, ElementaryExchange, Geography, IntermediateExchange,
    },
};
use roxmltree::{Document, Node};

/// Parse an ecospold2 XML document into an `ActivityDataset`.
///
/// Accepts both `<activityDataset>` and `<childActivityDataset>` as the
/// inner envelope — ecoinvent v3 uses the `child` variant for datasets
/// that inherit from a master template.
pub fn parse_dataset(xml: &str) -> Result<ActivityDataset, Ecospold2Error> {
    let doc = Document::parse(xml)?;
    let root = doc.root_element();

    if !root.has_tag_name("ecoSpold") {
        return Err(Ecospold2Error::UnexpectedRoot(
            root.tag_name().name().to_owned(),
        ));
    }

    let dataset = first_child(&root, &["activityDataset", "childActivityDataset"])
        .ok_or(Ecospold2Error::MissingElement("activityDataset"))?;

    let description = first_child(&dataset, &["activityDescription"])
        .ok_or(Ecospold2Error::MissingElement("activityDescription"))?;

    let activity_node = first_child(&description, &["activity"])
        .ok_or(Ecospold2Error::MissingElement("activity"))?;
    let activity = parse_activity(&activity_node)?;

    let geography = first_child(&description, &["geography"])
        .map(|n| parse_geography(&n))
        .transpose()?;

    let intermediate_exchanges;
    let elementary_exchanges;
    if let Some(flow_data) = first_child(&dataset, &["flowData"]) {
        intermediate_exchanges = flow_data
            .children()
            .filter(|n| n.is_element() && n.has_tag_name("intermediateExchange"))
            .map(|n| parse_intermediate_exchange(&n))
            .collect::<Result<Vec<_>, _>>()?;
        elementary_exchanges = flow_data
            .children()
            .filter(|n| n.is_element() && n.has_tag_name("elementaryExchange"))
            .map(|n| parse_elementary_exchange(&n))
            .collect::<Result<Vec<_>, _>>()?;
    } else {
        intermediate_exchanges = Vec::new();
        elementary_exchanges = Vec::new();
    }

    Ok(ActivityDataset {
        activity,
        geography,
        intermediate_exchanges,
        elementary_exchanges,
    })
}

fn parse_activity(node: &Node<'_, '_>) -> Result<Activity, Ecospold2Error> {
    let id = attr_required(node, "activity", "id")?;
    let activity_name_id = attr_required(node, "activity", "activityNameId")?;
    let activity_type = attr_parse::<u8>(node, "activity", "activityType")?;
    let special_activity_type = attr_parse_opt::<u8>(node, "activity", "specialActivityType")?;

    let name = first_child(node, &["activityName"])
        .map(text)
        .unwrap_or_default();

    Ok(Activity {
        id,
        activity_name_id,
        name,
        activity_type,
        special_activity_type,
    })
}

fn parse_geography(node: &Node<'_, '_>) -> Result<Geography, Ecospold2Error> {
    // ecoinvent v3 writes `<shortname>`; some older dumps use `<shortName>`.
    let short_name = first_child(node, &["shortname", "shortName"])
        .map(text)
        .ok_or(Ecospold2Error::MissingElement("shortname"))?;
    Ok(Geography { short_name })
}

fn parse_intermediate_exchange(
    node: &Node<'_, '_>,
) -> Result<IntermediateExchange, Ecospold2Error> {
    let id = attr_required(node, "intermediateExchange", "id")?;
    let intermediate_exchange_id =
        attr_required(node, "intermediateExchange", "intermediateExchangeId")?;
    let activity_link_id = attr_opt(node, "activityLinkId").filter(|s| !s.is_empty());
    let amount = attr_parse_finite(node, "intermediateExchange", "amount")?;
    let unit_name = attr_required(node, "intermediateExchange", "unitName")?;

    let name = first_child(node, &["name"]).map(text).unwrap_or_default();
    let direction = parse_direction(node, &id)?;

    Ok(IntermediateExchange {
        id,
        intermediate_exchange_id,
        activity_link_id,
        name,
        amount,
        unit_name,
        direction,
    })
}

fn parse_elementary_exchange(node: &Node<'_, '_>) -> Result<ElementaryExchange, Ecospold2Error> {
    let id = attr_required(node, "elementaryExchange", "id")?;
    let elementary_exchange_id = attr_required(node, "elementaryExchange", "elementaryExchangeId")?;
    let amount = attr_parse_finite(node, "elementaryExchange", "amount")?;
    let unit_name = attr_required(node, "elementaryExchange", "unitName")?;
    let cas_number = attr_opt(node, "casNumber");

    let name = first_child(node, &["name"]).map(text).unwrap_or_default();

    // ecoinvent nests `<compartment><compartment>X</compartment>
    // <subcompartment>Y</subcompartment></compartment>`.
    let mut compartment = String::new();
    let mut subcompartment = None;
    if let Some(wrap) = first_child(node, &["compartment"]) {
        if let Some(c) = first_child(&wrap, &["compartment"]) {
            compartment = text(c);
        }
        if let Some(sc) = first_child(&wrap, &["subcompartment"]) {
            subcompartment = Some(text(sc));
        }
    }

    let direction = parse_direction(node, &id)?;

    Ok(ElementaryExchange {
        id,
        elementary_exchange_id,
        name,
        amount,
        unit_name,
        compartment,
        subcompartment,
        cas_number,
        direction,
    })
}

fn parse_direction(node: &Node<'_, '_>, id: &str) -> Result<Direction, Ecospold2Error> {
    let input_node = first_child(node, &["inputGroup"]);
    let output_node = first_child(node, &["outputGroup"]);

    match (input_node, output_node) {
        (Some(_), Some(_)) => Err(Ecospold2Error::DirectionAmbiguous { id: id.to_owned() }),
        (Some(n), None) => {
            let g = parse_group(&n, "inputGroup")?;
            Ok(Direction::Input { group: g })
        }
        (None, Some(n)) => {
            let g = parse_group(&n, "outputGroup")?;
            Ok(Direction::Output { group: g })
        }
        (None, None) => Err(Ecospold2Error::DirectionMissing { id: id.to_owned() }),
    }
}

fn parse_group(node: &Node<'_, '_>, elem: &'static str) -> Result<u8, Ecospold2Error> {
    let raw = text(*node);
    raw.trim()
        .parse::<u8>()
        .map_err(|e| Ecospold2Error::InvalidAttribute {
            elem,
            attr: "(text)",
            value: raw.clone(),
            reason: e.to_string(),
        })
}

// ---- helpers ----------------------------------------------------------

fn first_child<'a, 'input>(node: &Node<'a, 'input>, names: &[&str]) -> Option<Node<'a, 'input>> {
    node.children()
        .find(|n| n.is_element() && names.iter().any(|name| n.has_tag_name(*name)))
}

fn text(node: Node<'_, '_>) -> String {
    // ecoinvent often has `<name xml:lang="en">...</name>` — we ignore the
    // xml:lang tagging and take the concatenated text content.
    node.text().map(str::to_owned).unwrap_or_default()
}

fn attr_opt(node: &Node<'_, '_>, name: &str) -> Option<String> {
    node.attribute(name).map(str::to_owned)
}

fn attr_required(
    node: &Node<'_, '_>,
    elem: &'static str,
    attr: &'static str,
) -> Result<String, Ecospold2Error> {
    node.attribute(attr)
        .map(str::to_owned)
        .ok_or(Ecospold2Error::MissingAttribute { elem, attr })
}

fn attr_parse<T: std::str::FromStr>(
    node: &Node<'_, '_>,
    elem: &'static str,
    attr: &'static str,
) -> Result<T, Ecospold2Error>
where
    T::Err: std::fmt::Display,
{
    let raw = attr_required(node, elem, attr)?;
    raw.parse::<T>()
        .map_err(|e| Ecospold2Error::InvalidAttribute {
            elem,
            attr,
            value: raw,
            reason: e.to_string(),
        })
}

fn attr_parse_opt<T: std::str::FromStr>(
    node: &Node<'_, '_>,
    elem: &'static str,
    attr: &'static str,
) -> Result<Option<T>, Ecospold2Error>
where
    T::Err: std::fmt::Display,
{
    let Some(raw) = node.attribute(attr) else {
        return Ok(None);
    };
    raw.parse::<T>()
        .map(Some)
        .map_err(|e| Ecospold2Error::InvalidAttribute {
            elem,
            attr,
            value: raw.to_owned(),
            reason: e.to_string(),
        })
}

fn attr_parse_finite(
    node: &Node<'_, '_>,
    elem: &'static str,
    attr: &'static str,
) -> Result<f64, Ecospold2Error> {
    let v: f64 = attr_parse(node, elem, attr)?;
    if !v.is_finite() {
        return Err(Ecospold2Error::NumericNonfinite {
            field: attr,
            value: v.to_string(),
        });
    }
    Ok(v)
}
