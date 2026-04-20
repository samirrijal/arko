//! openLCA JSON-LD reader — parses `Process`, `Flow`, `FlowProperty`,
//! and `UnitGroup` documents from a USDA LCA Commons / Federal LCA
//! Commons on-disk bundle into typed Rust structs, with an adapter
//! that produces `arko_io_ilcd_linker::TypedColumn` values the rest of
//! the engine already consumes.
//!
//! # Why a separate reader
//!
//! USDA LCA Commons (and the wider Federal LCA Commons) publish in
//! **openLCA JSON-LD**, not ILCD XML. The on-disk format — per-object
//! JSON files under `processes/`, `flows/`, `flow_properties/`,
//! `unit_groups/` — is structurally similar to ILCD but differs in
//! every concrete shape: `@id` / `@type` / inline embedded refs,
//! UUID-addressed units instead of `dataSetInternalID` integers,
//! `referenceUnit: true` flags instead of ID pointers, origin encoded
//! as a trailing comma-qualifier rather than a parenthetical.
//!
//! The v0.1 reader is **scoped to the beef bundle** — the five-process
//! cow-calf-finisher subgraph from the USDA LCA Commons — and
//! deliberately narrow everywhere else. See `SUPPORTED.md` at the
//! crate root for the full list of what's unsupported and why.
//!
//! # Shape
//!
//! ```text
//! reader.rs   →  parse_process / parse_flow / parse_flow_property / parse_unit_group
//!                (no-IO, consume JSON strings, emit native olca types)
//!
//! bundle.rs   →  OlcaBundle  (on-disk directory walker + lazy loader)
//!
//! adapter.rs  →  olca_to_typed_column  (the ONLY place that touches
//!                arko_io_ilcd_linker's TypedColumn/TypedExchange)
//! ```
//!
//! # License posture
//!
//! USDA LCA Commons data is CC0 1.0 Universal (public-domain
//! dedication, mandatory at submission). This reader ships with no
//! attribution plumbing because none is legally required; see
//! `arko/docs/licenses/usda-lca-commons.md` for the full analysis.

pub mod adapter;
pub mod bundle;
pub mod error;
pub mod model;
pub mod reader;

pub use adapter::olca_to_typed_column;
pub use bundle::OlcaBundle;
pub use error::OlcaError;
pub use model::{
    classify_flow_origin_from_name, normalize_cas, OlcaExchange, OlcaFlow, OlcaFlowProperty,
    OlcaFlowPropertyFactor, OlcaFlowType, OlcaProcess, OlcaProcessType, OlcaUnit, OlcaUnitGroup,
};
pub use reader::{parse_flow, parse_flow_property, parse_process, parse_unit_group};
