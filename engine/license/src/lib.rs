//! Arko license-tier evaluator — see `specs/calc/v0.1.md` §11 and
//! `specs/license/README.md`.
//!
//! Responsibilities split from `arko-core`:
//!
//! - **Core** owns the storage types (`LicenseTier`, `DerivativeRule`,
//!   `EffectiveRestriction`) and the §11.2 boolean-AND join executed as
//!   part of `pipeline::compute`.
//! - **This crate** (`arko-license`) owns the publish-time layer: it
//!   walks a computed result's contributing processes, fires every
//!   applicable `DerivativeRule`, checks tier expiries, and returns an
//!   [`Authorization`] decision for a given [`Intent`].
//!
//! The guiding principle from `specs/license/README.md` is preserved:
//! **the solve is always legal.** This crate is only consulted when the
//! caller wants to do something with the result — publish a report,
//! share inventory to another workspace, or export raw data.

pub mod authorize;
pub mod fire;
pub mod presets;

pub use authorize::{authorize, Authorization, Intent, Outcome};
pub use fire::{fire_rules, FiredRule};
