//! Distribution families per spec §9.1.
//!
//! Every variant carries **self-contained parameters** so that a
//! distribution value can be serialized, cached, hashed, and sampled
//! from independently. Parameter validation is explicit and strict:
//! we'd rather refuse a bad study at construction time than ship
//! silent NaNs through the pipeline.

use rand::Rng;
use rand_distr::{
    Beta, Distribution as RandDistribution, LogNormal as LogNormalDist, Normal as NormalDist,
    Triangular as TriangularDist, Uniform as UniformDist,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// The five permitted distribution families.
///
/// Serialization uses an internally-tagged form:
///
/// ```json
/// { "kind": "lognormal", "geometric_mean": 10.0, "geometric_sd": 1.5 }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Distribution {
    /// ecoinvent-default. Parameterized by geometric mean (the median
    /// of the distribution) and geometric standard deviation
    /// (dimensionless; equals `exp(σ)` where σ is the underlying
    /// normal's sd). A `geometric_sd` of 1.0 degenerates to a point.
    LogNormal { geometric_mean: f64, geometric_sd: f64 },

    Normal { mean: f64, sd: f64 },

    Triangular { min: f64, mode: f64, max: f64 },

    Uniform { min: f64, max: f64 },

    /// Beta-PERT. `lambda` controls the mode's weight; ecoinvent-style
    /// defaults to `4.0`.
    Pert {
        min: f64,
        mode: f64,
        max: f64,
        #[serde(default = "default_pert_lambda")]
        lambda: f64,
    },
}

const fn default_pert_lambda() -> f64 {
    4.0
}

/// Errors returned by distribution construction and sampling.
#[derive(Debug, Error, Clone, PartialEq, Serialize, Deserialize)]
pub enum UncertaintyError {
    #[error("invalid distribution parameter: {0}")]
    InvalidParameter(String),

    #[error("sampler failure: {0}")]
    SamplerFailed(String),
}

impl Distribution {
    /// A point value — implemented as a lognormal with `geometric_sd = 1.0`
    /// per spec §9.1 closing paragraph. Samples always return `value`.
    pub const fn point(value: f64) -> Self {
        Self::LogNormal {
            geometric_mean: value,
            geometric_sd: 1.0,
        }
    }

    /// Validate parameters without sampling. Idempotent; never allocates.
    pub fn validate(&self) -> Result<(), UncertaintyError> {
        match *self {
            Self::LogNormal {
                geometric_mean,
                geometric_sd,
            } => {
                if !geometric_mean.is_finite() || geometric_mean <= 0.0 {
                    return Err(bad(format!(
                        "LogNormal: geometric_mean must be finite and > 0; got {geometric_mean}"
                    )));
                }
                if !geometric_sd.is_finite() || geometric_sd < 1.0 {
                    return Err(bad(format!(
                        "LogNormal: geometric_sd must be finite and >= 1.0; got {geometric_sd}"
                    )));
                }
            }
            Self::Normal { mean, sd } => {
                if !mean.is_finite() {
                    return Err(bad(format!("Normal: mean must be finite; got {mean}")));
                }
                if !sd.is_finite() || sd <= 0.0 {
                    return Err(bad(format!("Normal: sd must be finite and > 0; got {sd}")));
                }
            }
            Self::Triangular { min, mode, max } => {
                if ![min, mode, max].iter().all(|x| x.is_finite()) {
                    return Err(bad("Triangular: parameters must all be finite".into()));
                }
                if !(min <= mode && mode <= max) {
                    return Err(bad(format!(
                        "Triangular: require min <= mode <= max; got min={min}, mode={mode}, max={max}"
                    )));
                }
                if (min - max).abs() < f64::EPSILON {
                    return Err(bad(format!(
                        "Triangular: min ≈ max ({min}); use a point value instead"
                    )));
                }
            }
            Self::Uniform { min, max } => {
                if !min.is_finite() || !max.is_finite() {
                    return Err(bad("Uniform: parameters must be finite".into()));
                }
                if !(min < max) {
                    return Err(bad(format!(
                        "Uniform: require min < max; got min={min}, max={max}"
                    )));
                }
            }
            Self::Pert {
                min,
                mode,
                max,
                lambda,
            } => {
                if ![min, mode, max, lambda].iter().all(|x| x.is_finite()) {
                    return Err(bad("Pert: parameters must all be finite".into()));
                }
                if !(min <= mode && mode <= max) {
                    return Err(bad(format!(
                        "Pert: require min <= mode <= max; got min={min}, mode={mode}, max={max}"
                    )));
                }
                if (min - max).abs() < f64::EPSILON {
                    return Err(bad(format!(
                        "Pert: min ≈ max ({min}); use a point value instead"
                    )));
                }
                if lambda <= 0.0 {
                    return Err(bad(format!("Pert: lambda must be > 0; got {lambda}")));
                }
            }
        }
        Ok(())
    }

    /// Draw one sample. Validates on every call — validation is O(1)
    /// and catching malformed distributions on the hot path is cheaper
    /// than producing NaNs that corrupt a downstream MC mean.
    pub fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Result<f64, UncertaintyError> {
        self.validate()?;
        let v = match *self {
            Self::LogNormal {
                geometric_mean,
                geometric_sd,
            } => {
                let sigma = geometric_sd.ln();
                if sigma.abs() < 1e-12 {
                    // Degenerate point value.
                    return Ok(geometric_mean);
                }
                let mu = geometric_mean.ln();
                LogNormalDist::new(mu, sigma)
                    .map_err(|e| bad(format!("LogNormal construction: {e}")))?
                    .sample(rng)
            }
            Self::Normal { mean, sd } => NormalDist::new(mean, sd)
                .map_err(|e| bad(format!("Normal construction: {e}")))?
                .sample(rng),
            Self::Triangular { min, mode, max } => {
                // rand_distr::Triangular::new signature is (min, max, mode).
                TriangularDist::new(min, max, mode)
                    .map_err(|e| bad(format!("Triangular construction: {e}")))?
                    .sample(rng)
            }
            Self::Uniform { min, max } => UniformDist::new(min, max).sample(rng),
            Self::Pert {
                min,
                mode,
                max,
                lambda,
            } => {
                let range = max - min;
                let alpha = 1.0 + lambda * (mode - min) / range;
                let beta = 1.0 + lambda * (max - mode) / range;
                let x = Beta::new(alpha, beta)
                    .map_err(|e| bad(format!("Beta (PERT) construction: {e}")))?
                    .sample(rng);
                min + x * range
            }
        };
        if !v.is_finite() {
            return Err(UncertaintyError::SamplerFailed(format!(
                "sampler produced non-finite value {v}"
            )));
        }
        Ok(v)
    }
}

fn bad(msg: String) -> UncertaintyError {
    UncertaintyError::InvalidParameter(msg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand_mt::Mt64;

    fn rng() -> Mt64 {
        Mt64::new(42)
    }

    #[test]
    fn point_value_is_deterministic() {
        let d = Distribution::point(5.0);
        let mut r = rng();
        for _ in 0..100 {
            assert_eq!(d.sample(&mut r).unwrap(), 5.0);
        }
    }

    #[test]
    fn validate_catches_bad_lognormal() {
        assert!(matches!(
            Distribution::LogNormal {
                geometric_mean: -1.0,
                geometric_sd: 1.5,
            }
            .validate(),
            Err(UncertaintyError::InvalidParameter(_))
        ));
        assert!(matches!(
            Distribution::LogNormal {
                geometric_mean: 1.0,
                geometric_sd: 0.5, // < 1.0
            }
            .validate(),
            Err(UncertaintyError::InvalidParameter(_))
        ));
    }

    #[test]
    fn validate_catches_bad_normal() {
        assert!(Distribution::Normal { mean: 0.0, sd: 0.0 }.validate().is_err());
        assert!(Distribution::Normal { mean: 0.0, sd: -1.0 }.validate().is_err());
    }

    #[test]
    fn validate_catches_out_of_order_triangular() {
        assert!(Distribution::Triangular {
            min: 10.0,
            mode: 5.0,
            max: 0.0,
        }
        .validate()
        .is_err());
    }

    #[test]
    fn validate_catches_bad_uniform() {
        assert!(Distribution::Uniform { min: 10.0, max: 10.0 }.validate().is_err());
        assert!(Distribution::Uniform { min: 10.0, max: 5.0 }.validate().is_err());
    }

    #[test]
    fn normal_samples_stay_close_to_mean() {
        let d = Distribution::Normal { mean: 100.0, sd: 1.0 };
        let mut r = rng();
        let sum: f64 = (0..10_000).map(|_| d.sample(&mut r).unwrap()).sum();
        let mean = sum / 10_000.0;
        // With sd=1 and n=10000, std error ≈ 0.01; five-sigma envelope
        assert!((mean - 100.0).abs() < 0.05, "got {mean}");
    }

    #[test]
    fn uniform_samples_respect_bounds() {
        let d = Distribution::Uniform { min: 3.0, max: 7.0 };
        let mut r = rng();
        for _ in 0..1_000 {
            let v = d.sample(&mut r).unwrap();
            assert!(v >= 3.0 && v < 7.0, "out of bounds: {v}");
        }
    }

    #[test]
    fn triangular_samples_respect_bounds() {
        let d = Distribution::Triangular {
            min: 0.0,
            mode: 5.0,
            max: 10.0,
        };
        let mut r = rng();
        for _ in 0..1_000 {
            let v = d.sample(&mut r).unwrap();
            assert!(v >= 0.0 && v <= 10.0, "out of bounds: {v}");
        }
    }

    #[test]
    fn pert_samples_respect_bounds() {
        let d = Distribution::Pert {
            min: 0.0,
            mode: 3.0,
            max: 10.0,
            lambda: 4.0,
        };
        let mut r = rng();
        for _ in 0..1_000 {
            let v = d.sample(&mut r).unwrap();
            assert!(v >= 0.0 && v <= 10.0, "out of bounds: {v}");
        }
    }

    #[test]
    fn json_roundtrip_every_variant() {
        let cases = [
            Distribution::LogNormal {
                geometric_mean: 10.0,
                geometric_sd: 1.5,
            },
            Distribution::Normal {
                mean: 0.0,
                sd: 1.0,
            },
            Distribution::Triangular {
                min: 0.0,
                mode: 5.0,
                max: 10.0,
            },
            Distribution::Uniform { min: -1.0, max: 1.0 },
            Distribution::Pert {
                min: 0.0,
                mode: 3.0,
                max: 10.0,
                lambda: 4.0,
            },
        ];
        for d in cases {
            let s = serde_json::to_string(&d).unwrap();
            let back: Distribution = serde_json::from_str(&s).unwrap();
            assert_eq!(d, back);
        }
    }
}
