//! Per-dimension statistics computed from a collection of Monte Carlo
//! samples.
//!
//! Convention: the `samples` vector is kept (sorted, ascending) alongside
//! the summary stats so callers can build histograms or run additional
//! percentiles without re-running the MC. This costs memory (`8 · N`
//! bytes per output dimension) but it is almost always the right trade
//! for LCA workloads where `N ≤ 10_000` and we need to present full
//! distributions in the UI.

use serde::{Deserialize, Serialize};

/// Summary statistics for one output dimension of a Monte Carlo run.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DimensionStats {
    pub mean: f64,
    pub sd: f64,
    pub min: f64,
    pub max: f64,
    pub p05: f64,
    pub p25: f64,
    pub p50: f64,
    pub p75: f64,
    pub p95: f64,
    /// `sd / sqrt(n)` — the MC standard error. Drives `W_MC_NONCONVERGENT`
    /// per spec §13.2 when it exceeds `config.convergence_threshold`.
    pub standard_error: f64,
    /// Full sample vector, sorted ascending. Useful for histograms.
    pub samples: Vec<f64>,
}

impl DimensionStats {
    /// Build stats from a raw sample vector. Sorts in place.
    pub fn from_samples(mut samples: Vec<f64>) -> Self {
        let n = samples.len();
        if n == 0 {
            return Self::empty();
        }

        let sum: f64 = samples.iter().sum();
        let mean = sum / (n as f64);
        let variance_denom = if n > 1 { (n - 1) as f64 } else { 1.0 };
        let variance = samples.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / variance_denom;
        let sd = variance.sqrt();
        let standard_error = sd / (n as f64).sqrt();

        samples.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let pct = |p: f64| -> f64 {
            if n == 1 {
                return samples[0];
            }
            let rank = p * ((n - 1) as f64);
            let lo = rank.floor() as usize;
            let hi = rank.ceil() as usize;
            if lo == hi {
                samples[lo]
            } else {
                let frac = rank - lo as f64;
                samples[lo] * (1.0 - frac) + samples[hi] * frac
            }
        };

        Self {
            mean,
            sd,
            min: samples[0],
            max: samples[n - 1],
            p05: pct(0.05),
            p25: pct(0.25),
            p50: pct(0.50),
            p75: pct(0.75),
            p95: pct(0.95),
            standard_error,
            samples,
        }
    }

    fn empty() -> Self {
        Self {
            mean: 0.0,
            sd: 0.0,
            min: 0.0,
            max: 0.0,
            p05: 0.0,
            p25: 0.0,
            p50: 0.0,
            p75: 0.0,
            p95: 0.0,
            standard_error: 0.0,
            samples: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn stats_of_constant_sequence() {
        let s = DimensionStats::from_samples(vec![5.0; 100]);
        assert_eq!(s.mean, 5.0);
        assert_eq!(s.sd, 0.0);
        assert_eq!(s.standard_error, 0.0);
        assert_eq!(s.min, 5.0);
        assert_eq!(s.max, 5.0);
        assert_eq!(s.p50, 5.0);
    }

    #[test]
    fn stats_of_0_to_100_inclusive() {
        let s = DimensionStats::from_samples((0..=100).map(|i| i as f64).collect());
        assert_relative_eq!(s.mean, 50.0, epsilon = 1e-12);
        assert_relative_eq!(s.p50, 50.0, epsilon = 1e-12);
        assert_eq!(s.min, 0.0);
        assert_eq!(s.max, 100.0);
        assert_relative_eq!(s.p05, 5.0, epsilon = 1e-12);
        assert_relative_eq!(s.p95, 95.0, epsilon = 1e-12);
    }

    #[test]
    fn empty_returns_zeros() {
        let s = DimensionStats::from_samples(Vec::new());
        assert_eq!(s.mean, 0.0);
        assert!(s.samples.is_empty());
    }

    #[test]
    fn roundtrips_through_json() {
        let s = DimensionStats::from_samples(vec![1.0, 2.0, 3.0, 4.0, 5.0]);
        let j = serde_json::to_string(&s).unwrap();
        let back: DimensionStats = serde_json::from_str(&j).unwrap();
        assert_eq!(s, back);
    }
}
