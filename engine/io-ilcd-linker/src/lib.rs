//! ILCD cross-document linker — resolves refs across a multi-file
//! bundle. ILCD publishes each domain object (flow, flow property, unit
//! group, source, contact, LCIA method) as its own XML document; a
//! process dataset references them by `refObjectId` (UUID). This crate
//! walks those references.
//!
//! # Why a separate crate
//!
//! `arko-io-ilcd` is deliberately no-IO: it parses a single XML string
//! into a typed `ProcessDataset`. Bundle handling — filesystem layout,
//! lazy file reads, cross-ref resolution — is a different axis of
//! complexity and belongs here.
//!
//! # Scope (v0.1)
//!
//! What this crate parses (minimal subset — just enough to chain a
//! flow to its SI reference unit):
//! - **Flow** (`flowDataSet`): UUID, base name, `typeOfDataSet`,
//!   CAS (optional), reference flow property (by internal ID), and
//!   the flow-property table mapping internal IDs to flow-property
//!   UUIDs + mean-value conversion factors.
//! - **FlowProperty** (`flowPropertyDataSet`): UUID, base name,
//!   reference unit group UUID.
//! - **UnitGroup** (`unitGroupDataSet`): UUID, base name, reference
//!   unit by internal ID, and the unit table (internal ID → unit
//!   name + mean-value factor).
//!
//! What this crate does **not** parse (deferred):
//! - Source and Contact datasets (pure provenance; not on the unit-
//!   resolution path).
//! - LCIA Method datasets (belongs to `arko-methods`, not here).
//! - Compliance and modelling metadata.
//! - ZIP-packaged bundles — v0.1 is directory-backed only. The
//!   `LinkResolver` trait is the natural extension point; a future
//!   `ZipBundle` implementation will plug in without disturbing
//!   callers.
//!
//! # Correctness posture
//!
//! Strict on missing cross-references (returns a typed `LinkError`);
//! permissive on unknown XML elements (silently ignored). One-shot
//! resolution — each call re-reads and re-parses the backing file.
//! Caching is deferred to the first profiling-driven need, on the
//! principle that it's easier to add a cache than to reason about
//! staleness.

pub mod bundle;
pub mod column;
pub mod error;
pub mod flow;
pub mod flowproperty;
pub mod resolver;
pub mod unitgroup;
mod xml;

pub use bundle::DirectoryBundle;
pub use column::{
    build_typed_column, BridgeWarning, TypedColumn, TypedExchange, UnitResolutionSource,
};
pub use error::LinkError;
pub use flow::{Flow, FlowPropertyRef, FlowType};
pub use flowproperty::FlowProperty;
pub use resolver::{
    resolve_reference_unit, resolve_reference_unit_from_flow, LinkResolver, ReferenceUnit,
};
pub use unitgroup::{Unit, UnitGroup};
