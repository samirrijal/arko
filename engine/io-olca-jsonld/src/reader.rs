//! JSON parsers for the four openLCA object kinds the v0.1 reader
//! handles: `Process`, `Flow`, `FlowProperty`, `UnitGroup`.
//!
//! The reader is no-IO: each `parse_*` function takes a JSON string and
//! returns a typed native struct. Filesystem layout and cross-ref
//! resolution live in `bundle`.
//!
//! # Correctness posture
//!
//! - **Strict on `@type`.** If the top-level JSON's `@type` doesn't
//!   match the expected document kind, fail loudly — this catches
//!   "fed a Flow file to the Process parser" at the earliest possible
//!   point.
//! - **Strict on required IDs.** Missing `@id` / `internalId` / `flow`
//!   references are hard errors.
//! - **Permissive on unknown fields.** openLCA adds fields across
//!   schema versions; unknown keys are silently ignored so minor
//!   upstream changes don't break reads.

use crate::error::OlcaError;
use crate::model::{
    normalize_cas, OlcaExchange, OlcaFlow, OlcaFlowProperty, OlcaFlowPropertyFactor, OlcaFlowType,
    OlcaProcess, OlcaProcessType, OlcaUnit, OlcaUnitGroup,
};
use serde_json::Value;
use std::path::Path;

pub fn parse_process(json: &str, path: &Path) -> Result<OlcaProcess, OlcaError> {
    let v: Value = serde_json::from_str(json).map_err(|source| OlcaError::Json {
        path: path.to_path_buf(),
        source,
    })?;
    expect_type(&v, "Process", path)?;

    let id = string_field(&v, "@id", path)?;
    let name = optional_string_field(&v, "name").unwrap_or_default();
    let process_type = parse_process_type(&v, path)?;
    let default_allocation_method = optional_string_field(&v, "defaultAllocationMethod");

    let exchanges = match v.get("exchanges") {
        Some(Value::Array(arr)) => arr
            .iter()
            .map(|e| parse_exchange(e, &id, path))
            .collect::<Result<Vec<_>, _>>()?,
        Some(other) => {
            return Err(OlcaError::InvalidField {
                path: path.to_path_buf(),
                field: "exchanges",
                value: other.to_string(),
                reason: "expected array".to_owned(),
            });
        }
        None => Vec::new(),
    };

    // v0.1 constraint: exactly one exchange is the quantitative
    // reference. Beef bundle satisfies this. Surface zero or multiple
    // as a structural error.
    let qref_count = exchanges.iter().filter(|e| e.quantitative_reference).count();
    if qref_count != 1 {
        return Err(OlcaError::InvalidField {
            path: path.to_path_buf(),
            field: "exchanges[].quantitativeReference",
            value: qref_count.to_string(),
            reason: "exactly one exchange must be marked quantitativeReference: true".to_owned(),
        });
    }

    Ok(OlcaProcess {
        id,
        name,
        process_type,
        default_allocation_method,
        exchanges,
    })
}

pub fn parse_flow(json: &str, path: &Path) -> Result<OlcaFlow, OlcaError> {
    let v: Value = serde_json::from_str(json).map_err(|source| OlcaError::Json {
        path: path.to_path_buf(),
        source,
    })?;
    expect_type(&v, "Flow", path)?;

    let id = string_field(&v, "@id", path)?;
    let name = optional_string_field(&v, "name").unwrap_or_default();
    let flow_type = parse_flow_type(&v, "flowType", path)?;
    let cas = optional_string_field(&v, "cas")
        .filter(|s| !s.is_empty())
        .map(|s| normalize_cas(&s));
    let formula = optional_string_field(&v, "formula").filter(|s| !s.is_empty());

    let flow_properties = match v.get("flowProperties") {
        Some(Value::Array(arr)) => arr
            .iter()
            .map(|e| parse_flow_property_factor(e, path))
            .collect::<Result<Vec<_>, _>>()?,
        Some(other) => {
            return Err(OlcaError::InvalidField {
                path: path.to_path_buf(),
                field: "flowProperties",
                value: other.to_string(),
                reason: "expected array".to_owned(),
            });
        }
        None => Vec::new(),
    };

    // Enforce exactly-one reference flow property — beef bundle
    // satisfies this. Zero or multiple ref-flow-properties indicates a
    // malformed flow dataset; fail early.
    let ref_count = flow_properties
        .iter()
        .filter(|f| f.reference_flow_property)
        .count();
    if ref_count != 1 {
        return Err(OlcaError::InvalidField {
            path: path.to_path_buf(),
            field: "flowProperties[].referenceFlowProperty",
            value: ref_count.to_string(),
            reason: "exactly one flowProperties entry must be marked referenceFlowProperty: true"
                .to_owned(),
        });
    }

    Ok(OlcaFlow {
        id,
        name,
        flow_type,
        cas,
        formula,
        flow_properties,
    })
}

pub fn parse_flow_property(json: &str, path: &Path) -> Result<OlcaFlowProperty, OlcaError> {
    let v: Value = serde_json::from_str(json).map_err(|source| OlcaError::Json {
        path: path.to_path_buf(),
        source,
    })?;
    expect_type(&v, "FlowProperty", path)?;

    let id = string_field(&v, "@id", path)?;
    let name = optional_string_field(&v, "name").unwrap_or_default();

    let unit_group = v
        .get("unitGroup")
        .ok_or_else(|| OlcaError::MissingField {
            path: path.to_path_buf(),
            field: "unitGroup",
        })?;
    let unit_group_uuid = string_field(unit_group, "@id", path)?;

    Ok(OlcaFlowProperty {
        id,
        name,
        unit_group_uuid,
    })
}

pub fn parse_unit_group(json: &str, path: &Path) -> Result<OlcaUnitGroup, OlcaError> {
    let v: Value = serde_json::from_str(json).map_err(|source| OlcaError::Json {
        path: path.to_path_buf(),
        source,
    })?;
    expect_type(&v, "UnitGroup", path)?;

    let id = string_field(&v, "@id", path)?;
    let name = optional_string_field(&v, "name").unwrap_or_default();

    let units = match v.get("units") {
        Some(Value::Array(arr)) => arr
            .iter()
            .map(|u| parse_unit(u, path))
            .collect::<Result<Vec<_>, _>>()?,
        Some(other) => {
            return Err(OlcaError::InvalidField {
                path: path.to_path_buf(),
                field: "units",
                value: other.to_string(),
                reason: "expected array".to_owned(),
            });
        }
        None => Vec::new(),
    };

    let ref_count = units.iter().filter(|u| u.reference_unit).count();
    if ref_count != 1 {
        return Err(OlcaError::InvalidField {
            path: path.to_path_buf(),
            field: "units[].referenceUnit",
            value: ref_count.to_string(),
            reason: "exactly one units entry must be marked referenceUnit: true".to_owned(),
        });
    }

    Ok(OlcaUnitGroup { id, name, units })
}

fn parse_exchange(
    v: &Value,
    process_uuid: &str,
    path: &Path,
) -> Result<OlcaExchange, OlcaError> {
    let internal_id = v
        .get("internalId")
        .and_then(Value::as_i64)
        .ok_or_else(|| OlcaError::MissingField {
            path: path.to_path_buf(),
            field: "exchanges[].internalId",
        })? as i32;

    let amount = v
        .get("amount")
        .and_then(Value::as_f64)
        .ok_or_else(|| OlcaError::MissingField {
            path: path.to_path_buf(),
            field: "exchanges[].amount",
        })?;
    if !amount.is_finite() {
        return Err(OlcaError::InvalidField {
            path: path.to_path_buf(),
            field: "exchanges[].amount",
            value: amount.to_string(),
            reason: "must be finite f64".to_owned(),
        });
    }

    let input = v.get("input").and_then(Value::as_bool).ok_or_else(|| {
        OlcaError::MissingField {
            path: path.to_path_buf(),
            field: "exchanges[].input",
        }
    })?;
    let avoided_product = v
        .get("avoidedProduct")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let quantitative_reference = v
        .get("quantitativeReference")
        .and_then(Value::as_bool)
        .unwrap_or(false);

    let flow = v.get("flow").ok_or_else(|| OlcaError::MissingField {
        path: path.to_path_buf(),
        field: "exchanges[].flow",
    })?;
    let flow_uuid = string_field(flow, "@id", path)?;
    let flow_type = parse_flow_type(flow, "flowType", path).map_err(|_| OlcaError::MissingField {
        path: path.to_path_buf(),
        field: "exchanges[].flow.flowType",
    })?;

    let unit = v.get("unit").ok_or_else(|| OlcaError::MissingField {
        path: path.to_path_buf(),
        field: "exchanges[].unit",
    })?;
    let unit_uuid = string_field(unit, "@id", path)?;

    let flow_property =
        v.get("flowProperty")
            .ok_or_else(|| OlcaError::MissingField {
                path: path.to_path_buf(),
                field: "exchanges[].flowProperty",
            })?;
    let flow_property_uuid = string_field(flow_property, "@id", path)?;

    let default_provider_uuid = v
        .get("defaultProvider")
        .and_then(|dp| dp.get("@id"))
        .and_then(Value::as_str)
        .map(ToOwned::to_owned);

    let _ = process_uuid; // Reserved for error-context enrichment; consumed by the adapter layer.

    Ok(OlcaExchange {
        internal_id,
        amount,
        input,
        avoided_product,
        quantitative_reference,
        flow_uuid,
        flow_type,
        unit_uuid,
        flow_property_uuid,
        default_provider_uuid,
    })
}

fn parse_flow_property_factor(
    v: &Value,
    path: &Path,
) -> Result<OlcaFlowPropertyFactor, OlcaError> {
    let fp = v
        .get("flowProperty")
        .ok_or_else(|| OlcaError::MissingField {
            path: path.to_path_buf(),
            field: "flowProperties[].flowProperty",
        })?;
    let flow_property_uuid = string_field(fp, "@id", path)?;
    let reference_flow_property = v
        .get("referenceFlowProperty")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let conversion_factor = v
        .get("conversionFactor")
        .and_then(Value::as_f64)
        .unwrap_or(1.0);

    Ok(OlcaFlowPropertyFactor {
        flow_property_uuid,
        reference_flow_property,
        conversion_factor,
    })
}

fn parse_unit(v: &Value, path: &Path) -> Result<OlcaUnit, OlcaError> {
    let id = string_field(v, "@id", path)?;
    let name = optional_string_field(v, "name").unwrap_or_default();
    let conversion_factor = v
        .get("conversionFactor")
        .and_then(Value::as_f64)
        .unwrap_or(1.0);
    let reference_unit = v
        .get("referenceUnit")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    Ok(OlcaUnit {
        id,
        name,
        conversion_factor,
        reference_unit,
    })
}

fn parse_process_type(v: &Value, path: &Path) -> Result<OlcaProcessType, OlcaError> {
    let s = v
        .get("processType")
        .and_then(Value::as_str)
        .ok_or_else(|| OlcaError::MissingField {
            path: path.to_path_buf(),
            field: "processType",
        })?;
    match s {
        "UNIT_PROCESS" => Ok(OlcaProcessType::UnitProcess),
        "LCI_RESULT" => Ok(OlcaProcessType::LciResult),
        other => Err(OlcaError::InvalidField {
            path: path.to_path_buf(),
            field: "processType",
            value: other.to_owned(),
            reason: "expected UNIT_PROCESS or LCI_RESULT".to_owned(),
        }),
    }
}

fn parse_flow_type(v: &Value, field: &'static str, path: &Path) -> Result<OlcaFlowType, OlcaError> {
    let s = v
        .get(field)
        .and_then(Value::as_str)
        .ok_or_else(|| OlcaError::MissingField {
            path: path.to_path_buf(),
            field,
        })?;
    match s {
        "PRODUCT_FLOW" => Ok(OlcaFlowType::ProductFlow),
        "ELEMENTARY_FLOW" => Ok(OlcaFlowType::ElementaryFlow),
        "WASTE_FLOW" => Ok(OlcaFlowType::WasteFlow),
        other => Err(OlcaError::InvalidField {
            path: path.to_path_buf(),
            field,
            value: other.to_owned(),
            reason: "expected PRODUCT_FLOW, ELEMENTARY_FLOW, or WASTE_FLOW".to_owned(),
        }),
    }
}

fn expect_type(v: &Value, expected: &'static str, path: &Path) -> Result<(), OlcaError> {
    let got = v
        .get("@type")
        .and_then(Value::as_str)
        .ok_or_else(|| OlcaError::MissingField {
            path: path.to_path_buf(),
            field: "@type",
        })?;
    if got != expected {
        return Err(OlcaError::UnexpectedType {
            path: path.to_path_buf(),
            expected,
            got: got.to_owned(),
        });
    }
    Ok(())
}

fn string_field(v: &Value, field: &'static str, path: &Path) -> Result<String, OlcaError> {
    v.get(field)
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
        .ok_or_else(|| OlcaError::MissingField {
            path: path.to_path_buf(),
            field,
        })
}

fn optional_string_field(v: &Value, field: &str) -> Option<String> {
    v.get(field).and_then(Value::as_str).map(ToOwned::to_owned)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn path() -> PathBuf {
        PathBuf::from("<test>")
    }

    #[test]
    fn parses_minimal_process() {
        let json = r#"{
            "@context": "http://greendelta.github.io/olca-schema/context.jsonld",
            "@type": "Process",
            "@id": "aaaaaaaa-1111-2222-3333-444444444444",
            "name": "test process",
            "processType": "UNIT_PROCESS",
            "exchanges": [
                {
                    "@type": "Exchange",
                    "internalId": 1,
                    "amount": 1.0,
                    "input": false,
                    "quantitativeReference": true,
                    "flow": {"@type": "Flow", "@id": "flow-out", "flowType": "PRODUCT_FLOW"},
                    "unit": {"@type": "Unit", "@id": "unit-kg", "name": "kg"},
                    "flowProperty": {"@type": "FlowProperty", "@id": "fp-mass"}
                }
            ]
        }"#;
        let p = parse_process(json, &path()).expect("parse ok");
        assert_eq!(p.name, "test process");
        assert_eq!(p.process_type, OlcaProcessType::UnitProcess);
        assert_eq!(p.exchanges.len(), 1);
        assert!(p.exchanges[0].quantitative_reference);
    }

    #[test]
    fn process_with_default_provider_captures_uuid() {
        let json = r#"{
            "@type": "Process",
            "@id": "proc-a",
            "processType": "UNIT_PROCESS",
            "exchanges": [
                {
                    "internalId": 1, "amount": 1.0, "input": false, "quantitativeReference": true,
                    "flow": {"@id": "flow-out", "flowType": "PRODUCT_FLOW"},
                    "unit": {"@id": "u-kg"}, "flowProperty": {"@id": "fp-mass"}
                },
                {
                    "internalId": 2, "amount": 3.5, "input": true,
                    "flow": {"@id": "flow-in", "flowType": "PRODUCT_FLOW"},
                    "unit": {"@id": "u-kg"}, "flowProperty": {"@id": "fp-mass"},
                    "defaultProvider": {"@id": "proc-b"}
                }
            ]
        }"#;
        let p = parse_process(json, &path()).expect("parse ok");
        assert_eq!(p.exchanges[1].default_provider_uuid.as_deref(), Some("proc-b"));
    }

    #[test]
    fn process_missing_ref_exchange_errors() {
        let json = r#"{
            "@type": "Process", "@id": "p", "processType": "UNIT_PROCESS",
            "exchanges": [
                {"internalId": 1, "amount": 1.0, "input": true,
                 "flow": {"@id": "f", "flowType": "PRODUCT_FLOW"},
                 "unit": {"@id": "u"}, "flowProperty": {"@id": "fp"}}
            ]
        }"#;
        let e = parse_process(json, &path()).unwrap_err();
        let msg = format!("{e}");
        assert!(
            msg.contains("exactly one exchange"),
            "expected quantitativeReference count error, got {msg}"
        );
    }

    #[test]
    fn process_wrong_type_errors() {
        let json = r#"{"@type": "Flow", "@id": "x"}"#;
        let e = parse_process(json, &path()).unwrap_err();
        match e {
            OlcaError::UnexpectedType { expected, got, .. } => {
                assert_eq!(expected, "Process");
                assert_eq!(got, "Flow");
            }
            other => panic!("expected UnexpectedType, got {other:?}"),
        }
    }

    #[test]
    fn parses_elementary_flow_with_cas_and_formula() {
        let json = r#"{
            "@type": "Flow",
            "@id": "flow-methane-bio",
            "name": "Methane, biogenic",
            "flowType": "ELEMENTARY_FLOW",
            "cas": "000074-82-8",
            "formula": "CH4",
            "flowProperties": [
                {"@type": "FlowPropertyFactor", "referenceFlowProperty": true,
                 "flowProperty": {"@id": "fp-mass"}, "conversionFactor": 1.0}
            ]
        }"#;
        let f = parse_flow(json, &path()).expect("parse ok");
        assert_eq!(f.cas.as_deref(), Some("74-82-8"));
        assert_eq!(f.formula.as_deref(), Some("CH4"));
        assert_eq!(f.flow_type, OlcaFlowType::ElementaryFlow);
        assert_eq!(f.flow_properties.len(), 1);
        assert!(f.flow_properties[0].reference_flow_property);
    }

    #[test]
    fn parses_product_flow_without_cas() {
        let json = r#"{
            "@type": "Flow", "@id": "flow-x", "name": "product x",
            "flowType": "PRODUCT_FLOW",
            "flowProperties": [
                {"referenceFlowProperty": true, "flowProperty": {"@id": "fp-mass"},
                 "conversionFactor": 1.0}
            ]
        }"#;
        let f = parse_flow(json, &path()).expect("parse ok");
        assert!(f.cas.is_none());
        assert_eq!(f.flow_type, OlcaFlowType::ProductFlow);
    }

    #[test]
    fn parses_flow_property() {
        let json = r#"{
            "@type": "FlowProperty", "@id": "fp-mass", "name": "Mass",
            "unitGroup": {"@type": "UnitGroup", "@id": "ug-mass"}
        }"#;
        let fp = parse_flow_property(json, &path()).expect("parse ok");
        assert_eq!(fp.id, "fp-mass");
        assert_eq!(fp.unit_group_uuid, "ug-mass");
    }

    #[test]
    fn parses_unit_group_with_reference_unit() {
        let json = r#"{
            "@type": "UnitGroup", "@id": "ug-mass", "name": "Units of mass",
            "units": [
                {"@id": "u-kg", "name": "kg", "referenceUnit": true, "conversionFactor": 1.0},
                {"@id": "u-t", "name": "t", "conversionFactor": 1000.0},
                {"@id": "u-g", "name": "g", "conversionFactor": 0.001}
            ]
        }"#;
        let ug = parse_unit_group(json, &path()).expect("parse ok");
        assert_eq!(ug.units.len(), 3);
        let r = ug.reference_unit().unwrap();
        assert_eq!(r.name, "kg");
        assert_eq!(r.conversion_factor, 1.0);
        assert_eq!(ug.unit_by_id("u-t").unwrap().conversion_factor, 1000.0);
    }

    #[test]
    fn unit_group_without_exactly_one_ref_errors() {
        let json = r#"{
            "@type": "UnitGroup", "@id": "ug", "name": "nope",
            "units": [
                {"@id": "a", "name": "a", "conversionFactor": 1.0},
                {"@id": "b", "name": "b", "conversionFactor": 2.0}
            ]
        }"#;
        assert!(parse_unit_group(json, &path()).is_err());
    }
}
