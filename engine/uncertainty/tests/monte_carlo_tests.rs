//! Integration tests for the Monte Carlo runner.

use approx::assert_relative_eq;
use arko_uncertainty::{run_monte_carlo, Distribution, MonteCarloConfig, UncertaintyError};

#[test]
fn point_value_converges_trivially() {
    // A single "always 5.0" dimension — zero variance, should converge.
    let cfg = MonteCarloConfig {
        iterations: 100,
        seed: 1,
        ..MonteCarloConfig::default()
    };
    let point = Distribution::point(5.0);

    let r = run_monte_carlo(&cfg, |rng| Ok(vec![point.sample(rng)?])).unwrap();
    assert_eq!(r.iterations, 100);
    assert_eq!(r.per_dimension.len(), 1);
    let d = &r.per_dimension[0];
    assert_eq!(d.mean, 5.0);
    assert_eq!(d.sd, 0.0);
    assert_eq!(d.standard_error, 0.0);
    assert!(r.converged);
}

#[test]
fn normal_has_expected_mean_and_sd() {
    let cfg = MonteCarloConfig {
        iterations: 10_000,
        seed: 42,
        ..MonteCarloConfig::default()
    };
    let d = Distribution::Normal { mean: 100.0, sd: 5.0 };
    let r = run_monte_carlo(&cfg, |rng| Ok(vec![d.sample(rng)?])).unwrap();

    let s = &r.per_dimension[0];
    // With n=10k and sd=5, standard error ≈ 0.05. Three-sigma envelope
    // on the mean is 100 ± 0.15.
    assert!((s.mean - 100.0).abs() < 0.2, "mean was {}", s.mean);
    assert!((s.sd - 5.0).abs() < 0.2, "sd was {}", s.sd);
}

#[test]
fn multi_dimensional_runs_are_consistent() {
    // Two dimensions: a constant 10.0 and a uniform [0, 1).
    let cfg = MonteCarloConfig {
        iterations: 1_000,
        seed: 7,
        ..MonteCarloConfig::default()
    };
    let a = Distribution::point(10.0);
    let b = Distribution::Uniform { min: 0.0, max: 1.0 };

    let r = run_monte_carlo(&cfg, |rng| Ok(vec![a.sample(rng)?, b.sample(rng)?])).unwrap();

    assert_eq!(r.per_dimension.len(), 2);
    assert_eq!(r.per_dimension[0].mean, 10.0);
    // Uniform(0,1) mean ≈ 0.5 with n=1000, standard error ≈ 0.009.
    assert!(
        (r.per_dimension[1].mean - 0.5).abs() < 0.05,
        "mean was {}",
        r.per_dimension[1].mean,
    );
}

#[test]
fn same_seed_reproduces_samples_bit_exactly() {
    let cfg = MonteCarloConfig {
        iterations: 500,
        seed: 999,
        ..MonteCarloConfig::default()
    };
    let d = Distribution::LogNormal {
        geometric_mean: 10.0,
        geometric_sd: 1.5,
    };

    let r1 = run_monte_carlo(&cfg, |rng| Ok(vec![d.sample(rng)?])).unwrap();
    let r2 = run_monte_carlo(&cfg, |rng| Ok(vec![d.sample(rng)?])).unwrap();

    // Full equality — spec §7.1 determinism contract propagated through
    // §9.2 seed contract: bit-identical samples required.
    assert_eq!(r1, r2);
}

#[test]
fn different_seeds_diverge() {
    let d = Distribution::Normal { mean: 0.0, sd: 1.0 };
    let r1 = run_monte_carlo(
        &MonteCarloConfig {
            iterations: 100,
            seed: 1,
            ..MonteCarloConfig::default()
        },
        |rng| Ok(vec![d.sample(rng)?]),
    )
    .unwrap();
    let r2 = run_monte_carlo(
        &MonteCarloConfig {
            iterations: 100,
            seed: 2,
            ..MonteCarloConfig::default()
        },
        |rng| Ok(vec![d.sample(rng)?]),
    )
    .unwrap();

    assert_ne!(r1.per_dimension[0].samples, r2.per_dimension[0].samples);
}

#[test]
fn bad_distribution_surfaces_error() {
    let cfg = MonteCarloConfig {
        iterations: 10,
        seed: 0,
        ..MonteCarloConfig::default()
    };
    let bad = Distribution::Normal {
        mean: 0.0,
        sd: -1.0, // invalid
    };
    let err = run_monte_carlo(&cfg, |rng| Ok(vec![bad.sample(rng)?])).unwrap_err();
    assert!(matches!(err, UncertaintyError::InvalidParameter(_)));
}

#[test]
fn convergence_flag_reflects_threshold() {
    // A wide Uniform with a tight convergence threshold should fail to
    // converge at low iteration counts.
    let cfg = MonteCarloConfig {
        iterations: 50, // deliberately too few for a wide distribution
        seed: 0,
        convergence_threshold: 0.01, // tight
    };
    let d = Distribution::Uniform {
        min: 0.0,
        max: 1_000.0,
    };
    let r = run_monte_carlo(&cfg, |rng| Ok(vec![d.sample(rng)?])).unwrap();
    assert!(!r.converged, "standard_error was {}", r.per_dimension[0].standard_error);

    // And the opposite direction: point value always converges at any threshold.
    let cfg2 = MonteCarloConfig {
        iterations: 50,
        seed: 0,
        convergence_threshold: 0.0, // zero!
    };
    let point = Distribution::point(5.0);
    let r2 = run_monte_carlo(&cfg2, |rng| Ok(vec![point.sample(rng)?])).unwrap();
    assert!(r2.converged);
}

#[test]
fn dimension_mismatch_across_iterations_is_an_error() {
    let cfg = MonteCarloConfig {
        iterations: 5,
        seed: 0,
        ..MonteCarloConfig::default()
    };
    let mut iter_count = 0;
    let err = run_monte_carlo(&cfg, |_rng| {
        iter_count += 1;
        if iter_count == 1 {
            Ok(vec![1.0, 2.0])
        } else {
            Ok(vec![1.0]) // wrong dimensionality
        }
    })
    .unwrap_err();
    assert!(matches!(err, UncertaintyError::SamplerFailed(_)));
}
