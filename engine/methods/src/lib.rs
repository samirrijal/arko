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
//! The factor table itself is deliberately small in v0.0.1: we ship
//! **two** IPCC climate-change presets — [`standard::ipcc_ar6_gwp100`]
//! (default recommendation for new studies) and
//! [`standard::ipcc_ar5_gwp100`] (legacy-verification parity for EPDs
//! authored before the AR6 migration). More methods (ReCiPe, CML,
//! TRACI, EF 3.1) are earmarked for v0.2 once the registry has been
//! battle-tested against real ecoinvent data.
//!
//! # Matching semantics
//!
//! A [`CharacterizationFactor`] carries a [`FactorMatch`] rule that
//! names one flow. Matchers are ordered by specificity:
//! 1. `CasOrigin` — CAS **plus** exact `FlowOrigin`. Used where the
//!    characterization value depends on fossil vs non-fossil
//!    provenance (e.g., AR6 CH4: `29.8` fossil vs `27.0` non-fossil).
//!    Unspecified-origin flows do **not** match — missing information
//!    is surfaced rather than silently papered over.
//! 2. `Cas` — plain CAS registry match, origin-agnostic. The default
//!    for species whose GWP does not split (CO2, N2O, SF6, …).
//! 3. `FlowId` — stable flow id fallback when CAS is absent (rare).
//! 4. `NameAndCompartment` — last-resort fuzzy match for legacy
//!    datasets where CAS numbers were lost in translation.

pub mod builder;
pub mod method;
pub mod registry;
pub mod standard;

pub use builder::{build_c_matrix, CMatrixBuild, CMatrixError};
pub use method::{
    CharacterizationFactor, FactorMatch, ImpactCategory, ImpactMethod,
};
pub use registry::{MethodNotFound, MethodRegistry};
