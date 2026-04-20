//! openLCA JSON-LD reader error type.
//!
//! Errors carry enough context to identify which file, which object,
//! which field failed — diagnostics stay useful across bundles with
//! hundreds of datasets.

use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum OlcaError {
    #[error("I/O error reading `{path}`: {message}")]
    Io { path: PathBuf, message: String },

    #[error("JSON parse error in `{path}`: {source}")]
    Json {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },

    #[error("unexpected `@type` `{got}` in `{path}`; expected `{expected}`")]
    UnexpectedType {
        path: PathBuf,
        expected: &'static str,
        got: String,
    },

    #[error("missing required field `{field}` in `{path}`")]
    MissingField {
        path: PathBuf,
        field: &'static str,
    },

    #[error("field `{field}` has invalid value `{value}` in `{path}`: {reason}")]
    InvalidField {
        path: PathBuf,
        field: &'static str,
        value: String,
        reason: String,
    },

    /// An exchange's `defaultProvider` UUID points at a process that is
    /// not present in the bundle. For a partial bundle this may be
    /// legitimate (the upstream subsystem was not shipped), but silently
    /// dropping the edge would produce a column whose upstream demand
    /// vanishes without notice. Surface, don't drop.
    #[error(
        "exchange in `{referrer_process_uuid}` declares defaultProvider `{missing_provider_uuid}` \
         which is not present in the bundle"
    )]
    DanglingDefaultProvider {
        referrer_process_uuid: String,
        missing_provider_uuid: String,
    },

    /// An exchange names a flow that is not present in the bundle.
    /// Same posture as `DanglingDefaultProvider` — surface, don't drop.
    #[error(
        "exchange in `{referrer_process_uuid}` references flow `{missing_flow_uuid}` \
         which is not present in the bundle"
    )]
    DanglingFlowReference {
        referrer_process_uuid: String,
        missing_flow_uuid: String,
    },

    /// An exchange, flow-property, or flow references a unit-group
    /// unit by UUID that is not present in the unit-group's `units`
    /// table. Indicates publisher-side data corruption (or a
    /// schema-version mismatch) — fail fast so the problem is visible.
    #[error(
        "unit `{unit_uuid}` not found in unit-group `{unit_group_uuid}` (unit-group path `{unit_group_path}`)"
    )]
    MissingUnitInGroup {
        unit_uuid: String,
        unit_group_uuid: String,
        unit_group_path: PathBuf,
    },

    /// An exchange's `flowProperty.@id` does not appear in the
    /// referenced flow's `flowProperties` table. v0.1 constraint:
    /// exchanges must quote amounts in one of the flow's declared
    /// properties. Beef bundle satisfies this; we fail loudly on
    /// violation rather than silently assume 1.0 conversion.
    #[error(
        "exchange in `{referrer_process_uuid}` declares flow-property `{exchange_flow_property_uuid}` \
         but flow `{flow_uuid}` does not list it (available: {available:?})"
    )]
    FlowPropertyNotDeclaredOnFlow {
        referrer_process_uuid: String,
        flow_uuid: String,
        exchange_flow_property_uuid: String,
        available: Vec<String>,
    },
}
