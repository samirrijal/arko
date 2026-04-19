//! Arko uncertainty propagation.
//!
//! Implements `specs/calc/v0.1.md` §9: the five permitted distribution
//! families, a Monte Carlo runner that holds the determinism contract
//! of §7 / §9.2, and per-dimension statistics including the standard
//! error that drives `W_MC_NONCONVERGENT` (§13.2).
//!
//! # What's here (v0.1)
//!
//! - [`Distribution`] — the five variants: LogNormal, Normal, Triangular,
//!   Uniform, PERT. Each carries `validate` + `sample` methods and
//!   round-trips cleanly through serde.
//! - [`run_monte_carlo`] — a closure-driven runner. Caller supplies a
//!   per-iteration sampling function; the runner calls it `N` times
//!   against a seeded Mersenne-Twister stream and collects per-output
//!   statistics.
//! - [`DimensionStats`] — mean, sd, min/max, p05 … p95 percentiles,
//!   standard error.
//!
//! # What's **not** here (v0.1)
//!
//! - **Sobol' low-discrepancy sampling** is the spec's default; this
//!   crate currently ships only the "MAY select" Mersenne-Twister path.
//!   Sobol' requires a U(0,1) → distribution pipeline via inverse-CDF
//!   which is a wider refactor; parked for v0.2.
//! - **Correlated draws.** v0.1 treats every uncertain entry as
//!   independent (per spec §9.2 closing paragraph). Pedigree-based
//!   correlation is a v0.2 upgrade.
//! - **Adjoint sensitivity analysis** (§9.3) — belongs with AD work in
//!   `arko-parameters`, not here.

pub mod distribution;
pub mod monte_carlo;
pub mod stats;

pub use distribution::{Distribution, UncertaintyError};
pub use monte_carlo::{run_monte_carlo, MonteCarloConfig, MonteCarloResult};
pub use stats::DimensionStats;

/// Default Monte Carlo iteration count per spec §9.2.
pub const DEFAULT_ITERATIONS: usize = 1000;

/// Default standard-error threshold that triggers `W_MC_NONCONVERGENT`
/// per spec §13.2.
pub const DEFAULT_CONVERGENCE_THRESHOLD: f64 = 5e-2;
