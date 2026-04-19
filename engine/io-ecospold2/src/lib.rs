//! ecospold2 XML reader — parses ecoinvent activity-dataset documents
//! into the types defined in `model`.
//!
//! # Scope (v0.1)
//!
//! What this crate parses:
//! - The outer `<ecoSpold>` / `<activityDataset>` envelope and the
//!   `<childActivityDataset>` variant (ecoinvent v3 "child" datasets).
//! - `<activity>` metadata: id, activityNameId, activityType, name.
//! - `<geography>` short name (e.g., `"GLO"`, `"RER"`, `"ES"`).
//! - `<intermediateExchange>` records: id, amount, unit, activityLinkId,
//!   input/output group, and whether it is the reference product.
//! - `<elementaryExchange>` records: id, amount, unit, compartment /
//!   subcompartment, CAS number, input/output group.
//!
//! What this crate does **not** do (deferred):
//! - Uncertainty distributions (spec §9 will drive this).
//! - Variable / parameter elements (handled by `arko-parameters` once
//!   we link parameter namespaces across datasets).
//! - Production volume blocks.
//! - `modellingAndValidation` metadata.
//! - `administrativeInformation` metadata.
//! - Master-data cross-references (handled by a future `arko-io-ecoinvent-linker`).
//! - Matrix assembly. Parsing one file gives you one column; assembling
//!   `A` / `B` / `C` across thousands of datasets is the linker's job.
//!
//! # Correctness posture
//!
//! This reader is **permissive on unknown elements** (silently ignored)
//! and **strict on structural violations** (missing required fields,
//! unparseable amounts, etc.). The intent is that it round-trips every
//! well-formed ecoinvent document without hand-authored exceptions.

pub mod error;
pub mod model;
pub mod reader;

pub use error::Ecospold2Error;
pub use model::{
    Activity, ActivityDataset, Direction, ElementaryExchange, Geography, IntermediateExchange,
};
pub use reader::parse_dataset;
