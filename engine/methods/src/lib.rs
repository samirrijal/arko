//! Arko impact-method registry.
//!
//! This crate fills the gap between a parsed inventory (whose flows
//! come from `arko-io-ecospold2` and similar import crates) and the
//! characterization matrix `C` that `pipeline::compute` needs. An
//! [`ImpactMethod`] is a named, versioned table of characterization
//! factors; [`MethodRegistry`] indexes them by `MethodRef`; and
//! [`build_c_matrix`] takes `(method, flows)` and produces the sparse
//! `C` matrix + `ImpactMeta` list that a `Study` requires.
//!
//! The factor table is intentionally scoped. The standard registry
//! ships:
//!
//! - [`standard::ipcc_ar6_gwp100`] — default recommendation for new
//!   studies (single-category GWP100 per AR6 WG1 Ch7 Table 7.15).
//! - [`standard::ipcc_ar5_gwp100`] — legacy-verification parity for
//!   EPDs authored before the AR6 migration. **With** climate-carbon
//!   feedback (CH4 = 30, N2O = 273, SF6 = 25_200).
//! - [`ef_31::ef_31`] — EF 3.1 V1, the 7 emission-based core
//!   indicators of EN 15804+A2 (shippable-EPD floor). Scope rationale:
//!   `DECISIONS.md` entry `D-0015`.
//! - [`cml_ia::cml_ia`] — CML-IA baseline v4.8, EN 15804+A2-aligned
//!   subset (7 categories: GWP100 *without* feedback, ODP, POCP, AP,
//!   EP, ADP-elements, ADP-fossil). Legacy-EPD verification preset
//!   plus side-by-side with EF 3.1. Scope rationale: `DECISIONS.md`
//!   entry `D-0017`. License analysis: `docs/licenses/cml-ia-leiden.md`.
//!
//! ReCiPe 2016 Midpoint is the remaining planned Phase-1-exit addition.
//!
//! # Matching semantics
//!
//! A [`CharacterizationFactor`] carries a [`FactorMatch`] rule that
//! selects one flow. The five variants — `Cas`, `CasOrigin`,
//! `CasCompartment`, `FlowId`, `NameAndCompartment` — are distinct
//! matching strategies, **not a priority chain**. Variant declaration
//! order is documentation, not matching priority (`DECISIONS.md`
//! entry `D-0015` records this explicitly to head off the
//! "most-specific-wins" intuition).
//!
//! The builder enforces "at most one factor per (category, flow)" as
//! a hard invariant: overlapping matchers within a category produce
//! [`CMatrixError::DuplicateMatch`] rather than silently picking one
//! or summing values. Authorship discipline is to pick the variant
//! that expresses the factor's selectivity axis:
//!
//! - `Cas` — plain CAS match, origin- and compartment-agnostic. For
//!   species whose CF is uniform across origin and compartment (CO2,
//!   CFCs, SF6, …).
//! - `CasOrigin` — CAS + exact `FlowOrigin`. For CFs that depend on
//!   fossil vs non-fossil provenance (AR6 CH4: `29.8` fossil vs
//!   `27.0` non-fossil). `Unspecified` origin does not match — by
//!   design, surfacing missing information rather than papering
//!   over it.
//! - `CasCompartment` — CAS + compartment prefix. For CFs that
//!   depend on emission compartment (EF 3.1 Acidification: SO2 to
//!   air matches, SO2 to water does not).
//! - `FlowId` — stable flow id fallback when CAS is absent.
//! - `NameAndCompartment` — last-resort fuzzy match for legacy
//!   datasets where CAS was lost in translation.

pub mod builder;
pub mod cml_ia;
pub mod ef_31;
pub mod method;
pub mod recipe_2016;
pub mod registry;
pub mod standard;

pub use builder::{build_c_matrix, CMatrixBuild, CMatrixError};
pub use method::{CharacterizationFactor, FactorMatch, ImpactCategory, ImpactMethod};
pub use registry::{MethodNotFound, MethodRegistry};
