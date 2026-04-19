//! End-to-end test: take the standard IPCC AR6 GWP100 preset, build a
//! `C` matrix against a flow list mirroring a small real-world LCA,
//! plug it into a full `Study`, solve, and check that the impact
//! score is what Excel-level arithmetic would say.
//!
//! This is the first test in the repo that exercises the full chain
//! `methods -> builder -> pipeline::compute -> impact` — and therefore
//! the first non-trivial down-payment on "Arko can do real LCA
//! calculations on real inputs."

use approx::assert_relative_eq;
use arko_core::{
    compute,
    license::LicenseTier,
    matrices::{SparseMatrix, SparseVector},
    meta::{FlowMeta, FlowOrigin, LicenseTierRef, ProcessMeta},
    sign::SignConvention,
    study::{MethodRef, Study},
    units::Unit,
};
use arko_methods::{build_c_matrix, MethodRegistry};
use arko_solvers_dense::DenseLuSolver;
use sprs::TriMat;

fn flow(id: &str, name: &str, cas: Option<&str>, origin: FlowOrigin) -> FlowMeta {
    FlowMeta {
        id: id.into(),
        name: name.into(),
        unit: Unit::new("kg"),
        compartment: vec!["emission".into(), "air".into()],
        cas: cas.map(String::from),
        origin,
    }
}

#[test]
fn standard_registry_contains_ipcc_gwp100_and_it_resolves() {
    let reg = MethodRegistry::standard();
    // v0.0.1 ships AR6 (default) + AR5 (legacy parity).
    assert_eq!(reg.len(), 2);
    let m = reg
        .lookup(&MethodRef {
            id: "ipcc-ar6-gwp100".into(),
            version: "1".into(),
        })
        .unwrap();
    assert_eq!(m.categories[0].id, "gwp100");
}

#[test]
fn build_c_matrix_against_ipcc_flows() {
    let reg = MethodRegistry::standard();
    let method = reg
        .lookup(&MethodRef {
            id: "ipcc-ar6-gwp100".into(),
            version: "1".into(),
        })
        .unwrap();

    // CO2 is origin-agnostic, CH4 is tagged fossil so the AR6
    // CasOrigin{fossil} matcher fires. H2O has no factor at all.
    let flows = vec![
        flow("f_co2", "Carbon dioxide", Some("124-38-9"), FlowOrigin::Unspecified),
        flow("f_ch4", "Methane", Some("74-82-8"), FlowOrigin::Fossil),
        flow("f_h2o", "Water vapour", Some("7732-18-5"), FlowOrigin::Unspecified),
    ];

    let b = build_c_matrix(method, &flows).unwrap();
    assert_eq!(b.matrix.shape(), (1, 3));
    assert_eq!(b.impacts.len(), 1);
    assert_eq!(b.impacts[0].id, "gwp100");
    assert_eq!(b.unmatched_flows, vec!["f_h2o".to_string()]);

    // Exactly 2 nonzeros: CO2 via Cas, fossil-CH4 via CasOrigin.
    assert_eq!(b.matrix.nnz(), 2);
}

#[test]
fn ar6_rejects_unspecified_origin_ch4_as_unmatched() {
    // An ecoinvent-imported CH4 flow that forgot to carry the fossil/non-fossil
    // classifier must NOT silently pick up the fossil factor. Instead the flow
    // surfaces in `unmatched_flows` so the UI can prompt the user to tag it.
    let reg = MethodRegistry::standard();
    let method = reg
        .lookup(&MethodRef {
            id: "ipcc-ar6-gwp100".into(),
            version: "1".into(),
        })
        .unwrap();

    let flows = vec![
        flow("f_co2", "Carbon dioxide", Some("124-38-9"), FlowOrigin::Unspecified),
        flow("f_ch4_unknown", "Methane", Some("74-82-8"), FlowOrigin::Unspecified),
    ];

    let b = build_c_matrix(method, &flows).unwrap();
    assert_eq!(
        b.unmatched_flows,
        vec!["f_ch4_unknown".to_string()],
        "Unspecified-origin CH4 must surface as unmatched under AR6, not inherit the fossil factor"
    );
    // CO2 still matches, so exactly one nonzero.
    assert_eq!(b.matrix.nnz(), 1);
}

#[test]
fn full_pipeline_with_real_method_and_real_numbers() {
    // Two-process study where:
    //   A = I (trivial technosphere, decouples the test from solver
    //        math so we're really testing C).
    //   B: process 0 emits 2 kg CO2 per functional unit, process 1
    //      emits 0.1 kg CH4 (fossil).
    //   f = [1, 0] → s = [1, 0] (only process 0 runs)
    //   g = B · s = [2, 0] — 2 kg CO2 from process 0, 0 kg CH4
    //
    // Under IPCC AR6 GWP100 (fossil CH4 = 29.8):
    //   h = 2·1.0 + 0·29.8 = 2.0 kg CO2-eq
    //
    // Now swap to f = [0, 1] and we should see only CH4:
    //   s = [0, 1]; g = [0, 0.1]; h = 0.1·29.8 = 2.98 kg CO2-eq

    // Technosphere = identity 2x2.
    let mut a = TriMat::new((2, 2));
    a.add_triplet(0, 0, 1.0);
    a.add_triplet(1, 1, 1.0);
    let technosphere: SparseMatrix = a.to_csr();

    // Biosphere: flow 0 (CO2) = 2 from process 0; flow 1 (CH4 fossil) = 0.1 from process 1.
    let mut b = TriMat::new((2, 2));
    b.add_triplet(0, 0, 2.0);
    b.add_triplet(1, 1, 0.1);
    let biosphere: SparseMatrix = b.to_csr();

    let flows = vec![
        flow("f_co2", "CO2", Some("124-38-9"), FlowOrigin::Unspecified),
        flow("f_ch4", "CH4", Some("74-82-8"), FlowOrigin::Fossil),
    ];

    let reg = MethodRegistry::standard();
    let method = reg
        .lookup(&MethodRef {
            id: "ipcc-ar6-gwp100".into(),
            version: "1".into(),
        })
        .unwrap();
    let built = build_c_matrix(method, &flows).unwrap();

    // Case 1: only process 0 runs.
    let study = Study {
        technosphere: technosphere.clone(),
        biosphere: biosphere.clone(),
        characterization: built.matrix.clone(),
        functional_unit: SparseVector::new(2, vec![0], vec![1.0]),
        processes: vec![
            ProcessMeta {
                id: "p0".into(),
                name: "co2-emitter".into(),
                reference_product: "widget-a".into(),
                reference_unit: Unit::new("kg"),
                allocation: None,
                license_tier: LicenseTierRef(0),
                geography: None,
            },
            ProcessMeta {
                id: "p1".into(),
                name: "ch4-emitter".into(),
                reference_product: "widget-b".into(),
                reference_unit: Unit::new("kg"),
                allocation: None,
                license_tier: LicenseTierRef(0),
                geography: None,
            },
        ],
        flows: flows.clone(),
        impacts: built.impacts.clone(),
        parameters: Vec::new(),
        license_tiers: vec![LicenseTier::permissive("custom-user")],
        method: MethodRef {
            id: "ipcc-ar6-gwp100".into(),
            version: "1".into(),
        },
        sign_convention: SignConvention::ProducerPositive,
    };

    let r1 = compute(&study, &DenseLuSolver).unwrap();
    assert_relative_eq!(r1.scaling[0], 1.0, epsilon = 1e-12);
    assert_relative_eq!(r1.scaling[1], 0.0, epsilon = 1e-12);
    assert_relative_eq!(r1.inventory[0], 2.0, epsilon = 1e-12);
    assert_relative_eq!(r1.inventory[1], 0.0, epsilon = 1e-12);
    // 2 kg CO2 × 1.0 GWP = 2 kg CO2-eq.
    assert_relative_eq!(r1.impact[0], 2.0, epsilon = 1e-12);

    // Case 2: only process 1 runs.
    let mut study2 = study.clone();
    study2.functional_unit = SparseVector::new(2, vec![1], vec![1.0]);
    let r2 = compute(&study2, &DenseLuSolver).unwrap();
    assert_relative_eq!(r2.inventory[0], 0.0, epsilon = 1e-12);
    assert_relative_eq!(r2.inventory[1], 0.1, epsilon = 1e-12);
    // 0.1 kg fossil CH4 × 29.8 GWP (AR6) = 2.98 kg CO2-eq.
    assert_relative_eq!(r2.impact[0], 2.98, epsilon = 1e-12);
}

#[test]
fn methods_json_roundtrip() {
    let reg = MethodRegistry::standard();
    let m = reg
        .lookup(&MethodRef {
            id: "ipcc-ar6-gwp100".into(),
            version: "1".into(),
        })
        .unwrap();
    let j = serde_json::to_string(m).unwrap();
    let back: arko_methods::ImpactMethod = serde_json::from_str(&j).unwrap();
    assert_eq!(m, &back);
}

#[test]
fn ar5_and_ar6_agree_where_they_should_and_disagree_where_they_should() {
    // A study with the SAME underlying CH4 emissions split across a fossil row
    // and a non-fossil row, run through both methods. This test guards against
    // a whole class of bugs the single-origin vectors can't catch:
    //
    //   - If AR6 ever silently applied the fossil factor to a non-fossil flow
    //     (or vice versa), the impact would collapse to a single-origin value
    //     (2.98 or 2.70 kg CO2-eq) rather than the correct split-weighted 2.868.
    //   - If AR5's origin-agnostic Cas matcher ever started respecting origin
    //     tags (regression toward AR6 semantics), its two-column match would
    //     drop to one column and the AR5 result would fall below 2.8.
    //   - If the two methods ever produced equal numbers on this split input,
    //     something has gone wrong in method dispatch.
    //
    // Setup: 0.06 kg fossil CH4 + 0.04 kg non-fossil CH4 from the same process.
    //   AR6:  h = 0.06·29.8 + 0.04·27.0 = 1.788 + 1.08 = 2.868 kg CO2-eq
    //   AR5:  h = 0.10·28                              = 2.800 kg CO2-eq

    let mut a = TriMat::new((1, 1));
    a.add_triplet(0, 0, 1.0);
    let technosphere: SparseMatrix = a.to_csr();

    let mut b = TriMat::new((2, 1));
    b.add_triplet(0, 0, 0.06); // fossil CH4
    b.add_triplet(1, 0, 0.04); // non-fossil CH4
    let biosphere: SparseMatrix = b.to_csr();

    let flows = vec![
        flow(
            "f_ch4_fossil",
            "CH4 fossil",
            Some("74-82-8"),
            FlowOrigin::Fossil,
        ),
        flow(
            "f_ch4_nonfossil",
            "CH4 non-fossil",
            Some("74-82-8"),
            FlowOrigin::NonFossil,
        ),
    ];

    let reg = MethodRegistry::standard();

    let base_study = |c_matrix: SparseMatrix, impacts: Vec<arko_core::meta::ImpactMeta>, method: MethodRef| Study {
        technosphere: technosphere.clone(),
        biosphere: biosphere.clone(),
        characterization: c_matrix,
        functional_unit: SparseVector::new(1, vec![0], vec![1.0]),
        processes: vec![ProcessMeta {
            id: "p0".into(),
            name: "process".into(),
            reference_product: "widget".into(),
            reference_unit: Unit::new("kg"),
            allocation: None,
            license_tier: LicenseTierRef(0),
            geography: None,
        }],
        flows: flows.clone(),
        impacts,
        parameters: Vec::new(),
        license_tiers: vec![LicenseTier::permissive("custom-user")],
        method,
        sign_convention: SignConvention::ProducerPositive,
    };

    // AR6: CasOrigin matchers hit each column by origin. Both flows match.
    let ar6_method_ref = MethodRef {
        id: "ipcc-ar6-gwp100".into(),
        version: "1".into(),
    };
    let ar6 = reg.lookup(&ar6_method_ref).unwrap();
    let ar6_built = build_c_matrix(ar6, &flows).unwrap();
    assert!(
        ar6_built.unmatched_flows.is_empty(),
        "AR6 must match both fossil and non-fossil CH4 columns: {:?}",
        ar6_built.unmatched_flows
    );
    let ar6_study = base_study(
        ar6_built.matrix.clone(),
        ar6_built.impacts.clone(),
        ar6_method_ref,
    );
    let ar6_r = compute(&ar6_study, &DenseLuSolver).unwrap();
    // AR6: 0.06·29.8 + 0.04·27.0 = 2.868.
    assert_relative_eq!(ar6_r.impact[0], 2.868, epsilon = 1e-12);
    // Regression guards: AR6 must NOT collapse to a single-origin answer.
    assert!(
        (ar6_r.impact[0] - 2.98).abs() > 1e-6,
        "AR6 collapsed to fossil-only factor (2.98) — origin split is broken"
    );
    assert!(
        (ar6_r.impact[0] - 2.70).abs() > 1e-6,
        "AR6 collapsed to non-fossil-only factor (2.70) — origin split is broken"
    );

    // AR5: plain Cas matcher is origin-agnostic; hits both columns with 28.0.
    let ar5_method_ref = MethodRef {
        id: "ipcc-ar5-gwp100".into(),
        version: "1".into(),
    };
    let ar5 = reg.lookup(&ar5_method_ref).unwrap();
    let ar5_built = build_c_matrix(ar5, &flows).unwrap();
    assert!(
        ar5_built.unmatched_flows.is_empty(),
        "AR5 must match both CH4 columns via origin-agnostic CAS: {:?}",
        ar5_built.unmatched_flows
    );
    let ar5_study = base_study(
        ar5_built.matrix.clone(),
        ar5_built.impacts.clone(),
        ar5_method_ref,
    );
    let ar5_r = compute(&ar5_study, &DenseLuSolver).unwrap();
    // AR5: 0.10 kg CH4 × 28 = 2.8 (origin-agnostic).
    assert_relative_eq!(ar5_r.impact[0], 2.8, epsilon = 1e-12);

    // Cross-method: the two assessments must produce different numbers on this
    // input. Equal results would mean the CH4 split was collapsed somewhere.
    assert!(
        (ar5_r.impact[0] - ar6_r.impact[0]).abs() > 1e-6,
        "AR5 ({}) and AR6 ({}) must disagree on split CH4 emissions",
        ar5_r.impact[0],
        ar6_r.impact[0]
    );
}
