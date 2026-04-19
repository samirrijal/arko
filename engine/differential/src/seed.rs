//! Hand-crafted seed test vectors.
//!
//! The spec's ≥10,000-vector corpus (§14.2) is populated over time from
//! reference implementations (Brightway 2.5, OpenLCA, SimaPro). At
//! v0.0.1 the on-disk corpus does not yet exist; these Rust-native
//! seed vectors are what ships with the crate so the framework itself
//! has something to execute.
//!
//! Each seed vector is:
//! - **Fully self-contained** — no external fixtures, no method
//!   registry lookups at test time.
//! - **Numerically trivial to audit** — identity technospheres and
//!   hand-calculated impacts where possible.
//! - **Tagged to the right `ConformanceLevel`** — L1 covers the basic
//!   three-equation pipeline, L2 adds allocation-bearing processes,
//!   L3 covers determinism under repeated solves.
//!
//! As the on-disk corpus fills in, these seed vectors stay put — they
//! are the "kernel" coverage that runs even in a build with no corpus
//! directory available.

use crate::vector::{ConformanceLevel, TestVector, ToleranceClass};
use arko_core::{
    license::LicenseTier,
    matrices::{SparseMatrix, SparseVector},
    meta::{FlowMeta, FlowOrigin, ImpactMeta, LicenseTierRef, ProcessMeta},
    sign::SignConvention,
    study::{MethodRef, Study},
    units::Unit,
};
use sprs::TriMat;

/// Build every shipped seed vector. Order is stable: L1 vectors first,
/// then L2, then L3. Within a level, declaration order = return order.
pub fn seed_vectors() -> Vec<TestVector> {
    vec![
        l1_identity_single_impact(),
        l1_two_process_independent(),
        l1_coupled_two_process(),
        l1_ch4_non_fossil_origin_split(),
        l2_process_with_allocation_metadata(),
        l3_repeated_solve_determinism(),
    ]
}

// --- L1: basic pipeline correctness ----------------------------------

/// L1 — identity A, single process, single CO2 flow, single GWP
/// category. `h = s · B_00 · C_00 = 1 · 2 · 1 = 2`.
fn l1_identity_single_impact() -> TestVector {
    // A = [1]
    let mut a = TriMat::new((1, 1));
    a.add_triplet(0, 0, 1.0);
    let technosphere: SparseMatrix = a.to_csr();

    // B = [2]  (2 kg CO2 per unit activity)
    let mut b = TriMat::new((1, 1));
    b.add_triplet(0, 0, 2.0);
    let biosphere: SparseMatrix = b.to_csr();

    // C = [1]  (CO2 characterization factor 1 kg CO2-eq/kg)
    let mut c = TriMat::new((1, 1));
    c.add_triplet(0, 0, 1.0);
    let characterization: SparseMatrix = c.to_csr();

    // f = [1]
    let functional_unit = SparseVector::new(1, vec![0], vec![1.0]);

    let study = Study {
        technosphere,
        biosphere,
        characterization,
        functional_unit,
        processes: vec![ProcessMeta {
            id: "p0".into(),
            name: "unit-activity".into(),
            reference_product: "widget".into(),
            reference_unit: Unit::new("kg"),
            allocation: None,
            license_tier: LicenseTierRef(0),
            geography: None,
        }],
        flows: vec![FlowMeta {
            id: "co2".into(),
            name: "Carbon dioxide".into(),
            unit: Unit::new("kg"),
            compartment: vec!["emission".into(), "air".into()],
            cas: Some("124-38-9".into()),
            origin: FlowOrigin::Unspecified,
        }],
        impacts: vec![ImpactMeta {
            id: "gwp100".into(),
            name: "Global warming potential (100yr)".into(),
            unit: Unit::new("kg CO2-eq"),
        }],
        parameters: Vec::new(),
        license_tiers: vec![LicenseTier::permissive("custom-user")],
        method: MethodRef {
            id: "ipcc-ar6-gwp100".into(),
            version: "1".into(),
        },
        sign_convention: SignConvention::ProducerPositive,
    };

    TestVector {
        id: "l1_identity_single_impact".into(),
        level: ConformanceLevel::L1Basic,
        description: "Identity technosphere, single process emitting 2 kg CO2, CF=1 → h=[2]".into(),
        study,
        expected_h: vec![2.0],
        tolerance_class: ToleranceClass::ReferenceParity,
        notes: Some("Minimal end-to-end smoke test for the three-equation pipeline.".into()),
    }
}

/// L1 — two independent processes, only process 1 has demand (f_1=1).
/// A=I, so s=[0,1]. B picks up a CH4 emission of 0.1 kg from process 1.
/// The CH4 flow is explicitly marked `FlowOrigin::Fossil`, so the
/// AR6 GWP100 factor is `29.8` → h = [0.1 · 29.8] = [2.98].
///
/// Flow-origin note: under AR6 CH4 is split fossil (29.8) vs
/// non-fossil (27.0). Seed corpus vectors name their origin
/// explicitly so the expected_h values are unambiguous and so a
/// future swap to non-fossil CH4 would change the vector id as well
/// as the number.
fn l1_two_process_independent() -> TestVector {
    let mut a = TriMat::new((2, 2));
    a.add_triplet(0, 0, 1.0);
    a.add_triplet(1, 1, 1.0);
    let technosphere: SparseMatrix = a.to_csr();

    let mut b = TriMat::new((2, 2));
    b.add_triplet(0, 0, 2.0); // CO2 from process 0 (unused in this vector).
    b.add_triplet(1, 1, 0.1); // CH4 (fossil) from process 1.
    let biosphere: SparseMatrix = b.to_csr();

    // C: single row, GWP100 — CO2=1, fossil CH4=29.8 per AR6 WG1 Ch7 T7.15.
    let mut c = TriMat::new((1, 2));
    c.add_triplet(0, 0, 1.0);
    c.add_triplet(0, 1, 29.8);
    let characterization: SparseMatrix = c.to_csr();

    let functional_unit = SparseVector::new(2, vec![1], vec![1.0]);

    let processes = vec![
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
    ];

    let flows = vec![
        FlowMeta {
            id: "co2".into(),
            name: "Carbon dioxide".into(),
            unit: Unit::new("kg"),
            compartment: vec!["emission".into(), "air".into()],
            cas: Some("124-38-9".into()),
            origin: FlowOrigin::Unspecified,
        },
        FlowMeta {
            id: "ch4_fossil".into(),
            name: "Methane".into(),
            unit: Unit::new("kg"),
            compartment: vec!["emission".into(), "air".into()],
            cas: Some("74-82-8".into()),
            origin: FlowOrigin::Fossil,
        },
    ];

    let study = Study {
        technosphere,
        biosphere,
        characterization,
        functional_unit,
        processes,
        flows,
        impacts: vec![ImpactMeta {
            id: "gwp100".into(),
            name: "Global warming potential (100yr)".into(),
            unit: Unit::new("kg CO2-eq"),
        }],
        parameters: Vec::new(),
        license_tiers: vec![LicenseTier::permissive("custom-user")],
        method: MethodRef {
            id: "ipcc-ar6-gwp100".into(),
            version: "1".into(),
        },
        sign_convention: SignConvention::ProducerPositive,
    };

    TestVector {
        id: "l1_two_process_independent".into(),
        level: ConformanceLevel::L1Basic,
        description:
            "Two independent processes, demand only on p1 (fossil CH4) → h = 0.1 · 29.8 = 2.98"
                .into(),
        study,
        expected_h: vec![2.98],
        tolerance_class: ToleranceClass::ReferenceParity,
        notes: Some(
            "CH4 is tagged FlowOrigin::Fossil; AR6 WG1 Ch7 T7.15 fossil CH4 GWP100 = 29.8.".into(),
        ),
    }
}

/// L1 — a non-trivial technosphere that requires an actual linear solve.
///
/// Process 0 produces a widget; process 1 produces a sub-assembly that
/// process 0 consumes. `A = [[1, -0.5], [0, 1]]`, `f = [1, 0]`.
///
/// Solution: `s = [1, 0]` because A is upper-triangular and f's bottom
/// entry is 0 — but the off-diagonal forces the solver to actually back-
/// substitute rather than trivially read off the diagonal, which
/// catches implementations that short-circuit "looks like identity."
///
/// Biosphere: process 0 emits 3 kg CO2, process 1 emits 0.2 kg CH4
/// (fossil — tagged `FlowOrigin::Fossil`).
///
/// Inventory for s=[1,0]: g = [3, 0]; h = [3 · 1 + 0 · 29.8] = [3.0].
///
/// To make the back-substitution observable, we instead choose
/// `f = [2, 4]` so `s` must satisfy:
///   s_1 = 4
///   s_0 - 0.5·s_1 = 2   →   s_0 = 4
/// Giving s = [4, 4], g = [12, 0.8], h = 12·1 + 0.8·29.8 = 12 + 23.84 = 35.84.
fn l1_coupled_two_process() -> TestVector {
    let mut a = TriMat::new((2, 2));
    a.add_triplet(0, 0, 1.0);
    a.add_triplet(0, 1, -0.5);
    a.add_triplet(1, 1, 1.0);
    let technosphere: SparseMatrix = a.to_csr();

    let mut b = TriMat::new((2, 2));
    b.add_triplet(0, 0, 3.0);
    b.add_triplet(1, 1, 0.2);
    let biosphere: SparseMatrix = b.to_csr();

    let mut c = TriMat::new((1, 2));
    c.add_triplet(0, 0, 1.0);
    c.add_triplet(0, 1, 29.8);
    let characterization: SparseMatrix = c.to_csr();

    let functional_unit = SparseVector::new(2, vec![0, 1], vec![2.0, 4.0]);

    let processes = vec![
        ProcessMeta {
            id: "p0".into(),
            name: "assembler".into(),
            reference_product: "widget".into(),
            reference_unit: Unit::new("kg"),
            allocation: None,
            license_tier: LicenseTierRef(0),
            geography: None,
        },
        ProcessMeta {
            id: "p1".into(),
            name: "sub-assembly-maker".into(),
            reference_product: "sub-assembly".into(),
            reference_unit: Unit::new("kg"),
            allocation: None,
            license_tier: LicenseTierRef(0),
            geography: None,
        },
    ];

    let flows = vec![
        FlowMeta {
            id: "co2".into(),
            name: "Carbon dioxide".into(),
            unit: Unit::new("kg"),
            compartment: vec!["emission".into(), "air".into()],
            cas: Some("124-38-9".into()),
            origin: FlowOrigin::Unspecified,
        },
        FlowMeta {
            id: "ch4_fossil".into(),
            name: "Methane".into(),
            unit: Unit::new("kg"),
            compartment: vec!["emission".into(), "air".into()],
            cas: Some("74-82-8".into()),
            origin: FlowOrigin::Fossil,
        },
    ];

    let study = Study {
        technosphere,
        biosphere,
        characterization,
        functional_unit,
        processes,
        flows,
        impacts: vec![ImpactMeta {
            id: "gwp100".into(),
            name: "Global warming potential (100yr)".into(),
            unit: Unit::new("kg CO2-eq"),
        }],
        parameters: Vec::new(),
        license_tiers: vec![LicenseTier::permissive("custom-user")],
        method: MethodRef {
            id: "ipcc-ar6-gwp100".into(),
            version: "1".into(),
        },
        sign_convention: SignConvention::ProducerPositive,
    };

    TestVector {
        id: "l1_coupled_two_process".into(),
        level: ConformanceLevel::L1Basic,
        description:
            "Upper-triangular A with off-diagonal coupling (fossil CH4) → forces real back-substitution"
                .into(),
        study,
        expected_h: vec![35.84],
        tolerance_class: ToleranceClass::ReferenceParity,
        notes: Some(
            "f = [2, 4] → s = [4, 4]; g = [12, 0.8]; h = 12·1 + 0.8·29.8 = 35.84."
                .into(),
        ),
    }
}

/// L1 — AR6 CH4 origin split. Same shape as `l1_two_process_independent`
/// but the CH4 flow carries `FlowOrigin::NonFossil`. Under AR6 WG1
/// Ch7 T7.15 the non-fossil CH4 GWP is `27.0`, so h = [0.1 · 27.0] = [2.70].
///
/// Point of the vector: exercise the `FactorMatch::CasOrigin`
/// discrimination end-to-end. An engine that confuses fossil and
/// non-fossil CH4 will silently return `2.98` (the fossil answer)
/// on this study and fail the parity check.
fn l1_ch4_non_fossil_origin_split() -> TestVector {
    let mut a = TriMat::new((2, 2));
    a.add_triplet(0, 0, 1.0);
    a.add_triplet(1, 1, 1.0);
    let technosphere: SparseMatrix = a.to_csr();

    let mut b = TriMat::new((2, 2));
    b.add_triplet(0, 0, 2.0); // CO2 from p0 (unused in this vector).
    b.add_triplet(1, 1, 0.1); // non-fossil CH4 from p1.
    let biosphere: SparseMatrix = b.to_csr();

    // C: CO2=1, non-fossil CH4=27.0 per AR6 WG1 Ch7 T7.15.
    let mut c = TriMat::new((1, 2));
    c.add_triplet(0, 0, 1.0);
    c.add_triplet(0, 1, 27.0);
    let characterization: SparseMatrix = c.to_csr();

    let functional_unit = SparseVector::new(2, vec![1], vec![1.0]);

    let processes = vec![
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
            name: "biogenic-ch4-emitter".into(),
            reference_product: "widget-b".into(),
            reference_unit: Unit::new("kg"),
            allocation: None,
            license_tier: LicenseTierRef(0),
            geography: None,
        },
    ];

    let flows = vec![
        FlowMeta {
            id: "co2".into(),
            name: "Carbon dioxide".into(),
            unit: Unit::new("kg"),
            compartment: vec!["emission".into(), "air".into()],
            cas: Some("124-38-9".into()),
            origin: FlowOrigin::Unspecified,
        },
        FlowMeta {
            id: "ch4_non_fossil".into(),
            name: "Methane".into(),
            unit: Unit::new("kg"),
            compartment: vec!["emission".into(), "air".into()],
            cas: Some("74-82-8".into()),
            origin: FlowOrigin::NonFossil,
        },
    ];

    let study = Study {
        technosphere,
        biosphere,
        characterization,
        functional_unit,
        processes,
        flows,
        impacts: vec![ImpactMeta {
            id: "gwp100".into(),
            name: "Global warming potential (100yr)".into(),
            unit: Unit::new("kg CO2-eq"),
        }],
        parameters: Vec::new(),
        license_tiers: vec![LicenseTier::permissive("custom-user")],
        method: MethodRef {
            id: "ipcc-ar6-gwp100".into(),
            version: "1".into(),
        },
        sign_convention: SignConvention::ProducerPositive,
    };

    TestVector {
        id: "l1_ch4_non_fossil_origin_split".into(),
        level: ConformanceLevel::L1Basic,
        description:
            "Non-fossil CH4 under AR6 → h = 0.1 · 27.0 = 2.70; paired with l1_two_process_independent (fossil, 2.98) to prove the origin split is wired."
                .into(),
        study,
        expected_h: vec![2.70],
        tolerance_class: ToleranceClass::ReferenceParity,
        notes: Some(
            "Regression guard for AR6 fossil/non-fossil CH4 discrimination via FactorMatch::CasOrigin."
                .into(),
        ),
    }
}

// --- L2: allocation / metadata surfaces ------------------------------

/// L2 — a process that carries a `user` allocation declaration. The
/// allocation is informational at this layer (full allocation rewriting
/// is a v0.2+ concern in `arko-validation`), but the vector *does*
/// exercise that the engine accepts and passes through allocation
/// metadata without mangling numerics.
fn l2_process_with_allocation_metadata() -> TestVector {
    // Same shape as l1_identity_single_impact, but p0 carries allocation.
    let mut a = TriMat::new((1, 1));
    a.add_triplet(0, 0, 1.0);
    let technosphere: SparseMatrix = a.to_csr();

    let mut b = TriMat::new((1, 1));
    b.add_triplet(0, 0, 2.0);
    let biosphere: SparseMatrix = b.to_csr();

    let mut c = TriMat::new((1, 1));
    c.add_triplet(0, 0, 1.0);
    let characterization: SparseMatrix = c.to_csr();

    let functional_unit = SparseVector::new(1, vec![0], vec![1.0]);

    let study = Study {
        technosphere,
        biosphere,
        characterization,
        functional_unit,
        processes: vec![ProcessMeta {
            id: "p0".into(),
            name: "multi-output-activity".into(),
            reference_product: "widget".into(),
            reference_unit: Unit::new("kg"),
            allocation: Some(arko_core::meta::Allocation {
                mode: arko_core::meta::AllocationMode::User,
                user_factors: vec![0.7, 0.3],
            }),
            license_tier: LicenseTierRef(0),
            geography: None,
        }],
        flows: vec![FlowMeta {
            id: "co2".into(),
            name: "Carbon dioxide".into(),
            unit: Unit::new("kg"),
            compartment: vec!["emission".into(), "air".into()],
            cas: Some("124-38-9".into()),
            origin: FlowOrigin::Unspecified,
        }],
        impacts: vec![ImpactMeta {
            id: "gwp100".into(),
            name: "Global warming potential (100yr)".into(),
            unit: Unit::new("kg CO2-eq"),
        }],
        parameters: Vec::new(),
        license_tiers: vec![LicenseTier::permissive("custom-user")],
        method: MethodRef {
            id: "ipcc-ar6-gwp100".into(),
            version: "1".into(),
        },
        sign_convention: SignConvention::ProducerPositive,
    };

    TestVector {
        id: "l2_process_with_allocation_metadata".into(),
        level: ConformanceLevel::L2Full,
        description:
            "Process with user-allocation metadata attached; numeric result unchanged at v0.1"
                .into(),
        study,
        expected_h: vec![2.0],
        tolerance_class: ToleranceClass::ReferenceParity,
        notes: Some(
            "Full allocation rewriting lands in arko-validation v0.2; this vector guards that \
             allocation metadata round-trips through the pipeline today."
                .into(),
        ),
    }
}

// --- L3: determinism & extended properties ---------------------------

/// L3 — same study as L1_two_process_independent, but tagged L3 to
/// ensure the runner exercises a deterministic-repeated-solve path at
/// the highest level. §7.1 determinism is also exercised by the
/// idempotent-recompute property test; this vector simply ensures the
/// report's `highest_level_passed` can *reach* L3 on a passing engine.
fn l3_repeated_solve_determinism() -> TestVector {
    let mut v = l1_two_process_independent();
    v.id = "l3_repeated_solve_determinism".into();
    v.level = ConformanceLevel::L3Elite;
    v.description =
        "Identical to l1_two_process_independent but promoted to L3 so highest-level-passed \
         can reach L3 on an engine that is deterministic and correct."
            .into();
    v.notes = Some(
        "Determinism itself is checked separately by check_idempotent_recompute in \
         tests/property_tests.rs — this vector just ensures an L3 entry exists in the \
         seed corpus."
            .into(),
    );
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seed_vectors_cover_every_level() {
        let vs = seed_vectors();
        assert!(vs.iter().any(|v| v.level == ConformanceLevel::L1Basic));
        assert!(vs.iter().any(|v| v.level == ConformanceLevel::L2Full));
        assert!(vs.iter().any(|v| v.level == ConformanceLevel::L3Elite));
    }

    #[test]
    fn seed_vector_ids_are_unique() {
        let vs = seed_vectors();
        let mut ids: Vec<&str> = vs.iter().map(|v| v.id.as_str()).collect();
        ids.sort_unstable();
        let before = ids.len();
        ids.dedup();
        assert_eq!(before, ids.len(), "seed vector ids must be unique");
    }

    #[test]
    fn seed_vector_expected_h_lengths_match_study_impacts() {
        for v in seed_vectors() {
            assert_eq!(
                v.expected_h.len(),
                v.study.n_impacts(),
                "vector {} expected_h length disagrees with study.n_impacts()",
                v.id,
            );
        }
    }
}
