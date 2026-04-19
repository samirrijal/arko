//! Integration test: the §16 worked example from `specs/calc/v0.1.md`.
//!
//! Now an end-to-end test — constructs the study, runs the pipeline
//! against the dense LU solver, and asserts `s`, `g`, `h` match the
//! expected values within §8.1 reference-parity tolerance.

use approx::assert_relative_eq;
use arko_core::{
    compute,
    license::LicenseTier,
    matrices::{SparseMatrix, SparseVector},
    meta::{FlowMeta, ImpactMeta, LicenseTierRef, ProcessMeta},
    sign::SignConvention,
    study::{MethodRef, Study},
    units::Unit,
    EffectiveRestriction,
};
use arko_solvers_dense::DenseLuSolver;
use sprs::TriMat;

/// Construct the two-process study from spec §16.
fn minimal_two_process_study() -> Study {
    // Brightway convention: A[product][activity]. Process 0 (steel)
    // consumes 0.5 MJ electricity, so that coefficient sits at
    // A[1][0] — row = the product being consumed (electricity,
    // product index 1), column = the activity doing the consuming
    // (steel, activity index 0). The spec §16 appendix writes the
    // matrix with the off-diagonal in the other triangle, but the
    // engine solves A·s = f literally without transposing, and the
    // narrative "s = [1, 0.5]" only holds under this convention.
    let mut a = TriMat::new((2, 2));
    a.add_triplet(0, 0, 1.0);
    a.add_triplet(1, 0, -0.5);
    a.add_triplet(1, 1, 1.0);
    let technosphere: SparseMatrix = a.to_csr();

    let mut b = TriMat::new((1, 2));
    b.add_triplet(0, 0, 2.0);
    b.add_triplet(0, 1, 0.1);
    let biosphere: SparseMatrix = b.to_csr();

    let mut c = TriMat::new((1, 1));
    c.add_triplet(0, 0, 1.0);
    let characterization: SparseMatrix = c.to_csr();

    let functional_unit = SparseVector::new(2, vec![0], vec![1.0]);

    let permissive = LicenseTier::permissive("custom-user");

    let processes = vec![
        ProcessMeta {
            id: "p0".into(),
            name: "steel".into(),
            reference_product: "steel".into(),
            reference_unit: Unit::new("kg"),
            allocation: None,
            license_tier: LicenseTierRef(0),
            geography: Some("GLO".into()),
        },
        ProcessMeta {
            id: "p1".into(),
            name: "electricity".into(),
            reference_product: "electricity".into(),
            reference_unit: Unit::new("MJ"),
            allocation: None,
            license_tier: LicenseTierRef(0),
            geography: Some("GLO".into()),
        },
    ];

    let flows = vec![FlowMeta {
        id: "f0".into(),
        name: "CO2".into(),
        unit: Unit::new("kg"),
        compartment: vec!["emission".into(), "air".into()],
        cas: Some("124-38-9".into()),
        origin: Default::default(),
    }];

    let impacts = vec![ImpactMeta {
        id: "gwp100".into(),
        name: "Global Warming Potential (100a)".into(),
        unit: Unit::new("kg CO2-eq"),
    }];

    Study {
        technosphere,
        biosphere,
        characterization,
        functional_unit,
        processes,
        flows,
        impacts,
        parameters: Vec::new(),
        license_tiers: vec![permissive],
        method: MethodRef {
            id: "minimal-gwp".into(),
            version: "0.1".into(),
        },
        sign_convention: SignConvention::ProducerPositive,
    }
}

#[test]
fn minimal_example_shape_invariants() {
    let s = minimal_two_process_study();

    assert_eq!(s.n_processes(), 2);
    assert_eq!(s.n_flows(), 1);
    assert_eq!(s.n_impacts(), 1);

    assert_eq!(s.technosphere.shape(), (2, 2));
    assert_eq!(s.biosphere.shape(), (1, 2));
    assert_eq!(s.characterization.shape(), (1, 1));
    assert_eq!(s.functional_unit.dim(), 2);

    let tier_idx = s.processes[0].license_tier.0 as usize;
    assert!(tier_idx < s.license_tiers.len());
}

#[test]
fn canonical_hash_is_deterministic() {
    let s1 = minimal_two_process_study();
    let s2 = minimal_two_process_study();
    assert_eq!(
        s1.canonical_hash(),
        s2.canonical_hash(),
        "canonical_hash MUST be deterministic — spec §7.1 determinism contract",
    );
    assert_eq!(
        s1.canonical_hash().len(),
        64,
        "BLAKE3 hex output is 64 chars"
    );
}

#[test]
fn minimal_example_solve_matches_spec() {
    // Expected per spec §16:
    //   s = [1.0, 0.5]
    //   g = [2.05]   (= 2.0·1.0 + 0.1·0.5)
    //   h = [2.05]
    let study = minimal_two_process_study();
    let result = compute(&study, &DenseLuSolver).expect("pipeline should succeed");

    // §8.1 reference parity tolerance: ε_abs = 1e-12.
    assert_relative_eq!(result.scaling[0], 1.0, epsilon = 1e-12);
    assert_relative_eq!(result.scaling[1], 0.5, epsilon = 1e-12);
    assert_relative_eq!(result.inventory[0], 2.05, epsilon = 1e-12);
    assert_relative_eq!(result.impact[0], 2.05, epsilon = 1e-12);

    // §11.2 effective restriction — permissive custom-user tier only.
    assert_eq!(
        result.effective_restriction,
        EffectiveRestriction {
            allow_publish: true,
            allow_share: true,
            allow_export: true,
            sources: vec!["custom-user".into()],
        }
    );

    // Both processes contributed (s=[1.0, 0.5], both > EPS_PRESENCE).
    assert_eq!(result.contributing_processes, vec![0, 1]);
}

#[test]
fn solve_rejects_shape_mismatch() {
    // Functional unit with wrong length.
    let mut study = minimal_two_process_study();
    study.functional_unit = SparseVector::new(3, vec![0], vec![1.0]);
    let err = compute(&study, &DenseLuSolver).unwrap_err();
    assert_eq!(err.code(), "E_SHAPE_MISMATCH");
}
