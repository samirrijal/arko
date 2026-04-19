//! Small XML-reading helpers shared by the Flow / FlowProperty /
//! UnitGroup parsers. Keeping them here — rather than in each parser —
//! means behavioural changes (e.g. how we handle whitespace-only
//! `<meanValue>` elements) propagate to all three at once.

use crate::error::LinkError;
use roxmltree::Node;
use std::path::Path;

pub(crate) fn first_child<'a, 'input>(
    node: &Node<'a, 'input>,
    name: &str,
) -> Option<Node<'a, 'input>> {
    node.children()
        .find(|n| n.is_element() && n.has_tag_name(name))
}

pub(crate) fn node_text(n: Node<'_, '_>) -> String {
    n.text().map(str::to_owned).unwrap_or_default()
}

pub(crate) fn parse_int(raw: &str, elem: &'static str, path: &Path) -> Result<i32, LinkError> {
    raw.trim()
        .parse::<i32>()
        .map_err(|e| LinkError::InvalidText {
            path: path.to_path_buf(),
            elem,
            value: raw.to_owned(),
            reason: e.to_string(),
        })
}

pub(crate) fn parse_f64(raw: &str, elem: &'static str, path: &Path) -> Result<f64, LinkError> {
    let v = raw
        .trim()
        .parse::<f64>()
        .map_err(|e| LinkError::InvalidText {
            path: path.to_path_buf(),
            elem,
            value: raw.to_owned(),
            reason: e.to_string(),
        })?;
    if !v.is_finite() {
        return Err(LinkError::InvalidText {
            path: path.to_path_buf(),
            elem,
            value: raw.to_owned(),
            reason: "value is not finite".into(),
        });
    }
    Ok(v)
}
