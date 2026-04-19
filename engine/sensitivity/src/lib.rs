//! Arko incremental recalculation — see `specs/calc/v0.1.md` §10.
//!
//! The user workflow is "edit one parameter, see the result update."
//! Refactoring `A` from scratch every keystroke is wasteful; the
//! Sherman-Morrison-Woodbury identity gives us `s_new` from `s` using
//! only `r` solves against the already-factored `A`, where `r` is the
//! rank of the update (1 for a single entry edit, 1 for column
//! replacement, 2 for add/remove edge).
//!
//! ```text
//!   (A + U·V^T) · s_new = f
//!   =>  s_new = s - A⁻¹·U · (I + V^T·A⁻¹·U)⁻¹ · V^T·s
//! ```
//!
//! The `Solver` trait does not currently expose a cached factorization,
//! so each internal "solve against A" call re-factors. The net speed
//! win against a full refactor + mat-vec is still `r / (r + 1)` — not
//! optimal but correct. A future `FactoredSolver` trait is the v0.2
//! optimization; the public API of this crate does not change.
//!
//! **Staleness (§10.2).** `FactoredSystem::generation` starts at 0 for
//! a freshly-solved system and increments once per successful
//! incremental update. `refactor()` resets the counter to 0; callers
//! should record that reset in provenance.

pub mod factored;
pub mod updates;

pub use factored::{FactoredSystem, SensitivityError};
