//! ILCD XML reader — parses EU JRC ILCD `<processDataSet>` documents
//! into the types defined in `model`.
//!
//! ILCD is the format underlying:
//! - **PEF** (Product Environmental Footprint) — the EU Commission's
//!   harmonized methodology, mandatory for several Green Deal product
//!   categories.
//! - **EPDs** issued under EN 15804+A2 by EPD-Norge, EPD Italy,
//!   GlobalEPD (AENOR), DAPcons, and other ECO Platform members.
//! - The **JRC LCDN** (Life Cycle Data Network) reference data.
//!
//! For Spanish industry — KarbonGarbi's first market — this is the
//! single most consequential format outside ecoinvent.
//!
//! # Scope (v0.1)
//!
//! What this crate parses:
//! - The outer `<processDataSet>` envelope (any namespace).
//! - `<processInformation>`: dataset UUID, base name, treatment-
//!   standards-routes qualifier, geography location code, reference
//!   year.
//! - `<quantitativeReference>`: which `dataSetInternalID` is the
//!   declared reference flow, with the `type` qualifier preserved.
//! - `<exchanges><exchange>` records: internal ID, linked flow UUID
//!   and label, direction, mean and resulting amounts, optional
//!   parameter binding, optional data-derivation status.
//!
//! What this crate does **not** do (deferred):
//! - Flow, FlowProperty, UnitGroup, Source, Contact, LCIA Method
//!   datasets — separate readers, future crates.
//! - Modelling and validation metadata, administrative metadata,
//!   compliance declarations.
//! - Parameters and variables (the `<mathematicalRelations>` block) —
//!   parsed-name-only at v0.1; expression evaluation belongs in
//!   `arko-parameters`.
//! - LCIA results bundled inside the process dataset (`<LCIAResults>`)
//!   — present in some published EPDs, but they are *outputs*, not
//!   inputs, and we recompute from B and C anyway.
//! - Cross-document linking against flow / unit-group catalogues —
//!   handled by a future `arko-io-ilcd-linker`.
//!
//! # Correctness posture
//!
//! Permissive on unknown elements (silently ignored), strict on
//! structural violations (missing required fields, unparseable
//! numbers, dangling reference-flow pointers). One file → one process
//! → one column of `A`.

pub mod error;
pub mod model;
pub mod reader;

pub use error::IlcdError;
pub use model::{
    Direction, EpdModuleAmount, Exchange, ParseWarning, ProcessDataset, ProcessInformation,
    QuantitativeReference,
};
pub use reader::parse_process;
