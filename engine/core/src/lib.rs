//! Arko calculation engine — core types.
//!
//! This crate implements the data model described in
//! `specs/calc/v0.1.md` sections §4 (data model), §5 (parameters),
//! §6.3 (solver contract), §11 (license tiers), §12 (provenance), and
//! §13 (errors / warnings), plus the three-equation pipeline of §4.3.
//!
//! What lives here:
//! - Pure data types (zero logic beyond trivial constructors).
//! - The `Solver` trait — contract between core and any backend.
//! - The `pipeline::compute` entry point that stitches solver + `B·s`
//!   + `C·g` + license-join into one result.
//!
//! What does **not** live here:
//! - Any concrete solver (→ `arko-solvers-*`).
//! - Full §6.1 validation order (→ `arko-validation`).
//! - Parameter expression evaluation and AD (→ `arko-parameters`).
//! - License-rule firing / publish-time enforcement (→ `arko-license`).
//! - Anything touching disk, network, or a UI.

pub mod error;
pub mod license;
pub mod matrices;
pub mod meta;
pub mod parameters;
pub mod pipeline;
pub mod provenance;
pub mod sign;
pub mod solver;
pub mod study;
pub mod units;
pub mod warning;

pub use error::EngineError;
pub use license::{DerivativeAction, DerivativeRule, DerivativeTrigger, LicenseTier};
pub use matrices::{DenseVector, SparseMatrix, SparseVector};
pub use meta::{
    Allocation, AllocationMode, FlowMeta, FlowOrigin, ImpactMeta, LicenseTierRef, ProcessMeta,
};
pub use parameters::{Expression, Parameter, ParameterId};
pub use pipeline::{compute, Computed, EPS_PRESENCE};
pub use provenance::{EffectiveRestriction, Provenance};
pub use sign::SignConvention;
pub use solver::Solver;
pub use study::{MethodRef, Study};
pub use units::Unit;
pub use warning::{Warning, WarningCode};

/// Specification version this crate implements.
///
/// When the calc spec ships a breaking change, this constant moves in
/// lockstep. See `specs/calc/v0.1.md` §15.
pub const SPEC_VERSION: &str = "0.1";
