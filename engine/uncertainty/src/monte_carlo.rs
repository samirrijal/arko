//! Monte Carlo runner per spec §9.2.
//!
//! Given a per-iteration sampling closure, run it `N` times against a
//! seeded Mersenne-Twister stream and accumulate per-dimension
//! statistics. Convergence is signalled by comparing every dimension's
//! standard error to a configurable threshold (default `5e-2`, per
//! spec §13.2 for the `W_MC_NONCONVERGENT` warning).

use crate::{
    distribution::UncertaintyError,
    stats::DimensionStats,
    DEFAULT_CONVERGENCE_THRESHOLD, DEFAULT_ITERATIONS,
};
use rand_mt::Mt64;
use serde::{Deserialize, Serialize};

/// Runtime configuration for a Monte Carlo run.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MonteCarloConfig {
    /// Number of iterations. Default: 1,000 per spec §9.2.
    pub iterations: usize,
    /// Seed for the Mersenne-Twister stream. Same seed ⇒ bit-identical
    /// results under the determinism contract of §7 + §9.2.
    pub seed: u64,
    /// Max permitted standard error per dimension. If any dimension
    /// exceeds this at end-of-run, `converged` is false and callers
    /// should emit `W_MC_NONCONVERGENT`.
    pub convergence_threshold: f64,
}

impl Default for MonteCarloConfig {
    fn default() -> Self {
        Self {
            iterations: DEFAULT_ITERATIONS,
            seed: 0,
            convergence_threshold: DEFAULT_CONVERGENCE_THRESHOLD,
        }
    }
}

/// Result of a Monte Carlo run.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MonteCarloResult {
    pub per_dimension: Vec<DimensionStats>,
    pub iterations: usize,
    /// `true` iff every dimension's standard error is `≤ threshold`.
    pub converged: bool,
}

/// Run Monte Carlo. The closure receives a mutable PRNG reference and
/// must return one `Vec<f64>` per invocation containing **one value
/// per output dimension** — the dimensionality is inferred from the
/// first iteration and enforced on subsequent ones.
///
/// Determinism: the PRNG is constructed once (`Mt64::new(seed)`) and
/// threaded through every iteration. Caller code that samples
/// distributions in a stable order will therefore be bit-reproducible.
pub fn run_monte_carlo<F>(
    config: &MonteCarloConfig,
    mut sample_fn: F,
) -> Result<MonteCarloResult, UncertaintyError>
where
    F: FnMut(&mut Mt64) -> Result<Vec<f64>, UncertaintyError>,
{
    if config.iterations == 0 {
        return Ok(MonteCarloResult {
            per_dimension: Vec::new(),
            iterations: 0,
            converged: true,
        });
    }

    let mut rng = Mt64::new(config.seed);
    let mut per_dim: Vec<Vec<f64>> = Vec::new();

    for iter_idx in 0..config.iterations {
        let samples = sample_fn(&mut rng)?;
        if per_dim.is_empty() {
            per_dim = (0..samples.len())
                .map(|_| Vec::with_capacity(config.iterations))
                .collect();
        } else if samples.len() != per_dim.len() {
            return Err(UncertaintyError::SamplerFailed(format!(
                "iteration {iter_idx} produced {} samples; expected {}",
                samples.len(),
                per_dim.len()
            )));
        }
        for (d, v) in samples.into_iter().enumerate() {
            per_dim[d].push(v);
        }
    }

    let per_dimension: Vec<DimensionStats> =
        per_dim.into_iter().map(DimensionStats::from_samples).collect();

    let converged = per_dimension
        .iter()
        .all(|s| s.standard_error <= config.convergence_threshold);

    Ok(MonteCarloResult {
        per_dimension,
        iterations: config.iterations,
        converged,
    })
}
