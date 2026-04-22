//! End-to-end smoke test for the LCAx V1 writer: build a minimal
//! two-process Study (the same shape as `core/tests/minimal_example.rs`),
//! run the pipeline, emit an LCAx `Project`, and serialise+round-trip it
//! through `serde_json` to prove the output is a well-formed LCAx
//! document rather than just a Rust value that happens to compile.
//!
//! The smoke test asserts structural invariants only; the "does the math
//! mean anything" side is `core/tests/minimal_example.rs`'s job. Here we
//! verify:
//!
//! - the writer succeeds on a pipeline-produced `Computed`;
//! - the emitted JSON round-trips back into `lcax_models::project::Project`
//!   without loss (the JSON is schema-shaped);
//! - the synthetic Project wraps exactly one Assembly → one Product →
//!   one EPD, which is the V1 shape documented in `lib.rs`;
//! - the single impact value lands at `LifeCycleModule::A1A3` under the
//!   expected `ImpactCategoryKey::GWP` (stage-assignment decision from
//!   `DECISIONS.md` D-0018);
//! - the method identifier (`minimal-gwp@0.1`) is preserved in
//!   `EPD.comment`, so downstream consumers can recover the methodology
//!   even when `standard` is `UNKNOWN` (refinement #2 in D-0018).

use arko_core::{
    compute,
    license::LicenseTier,
    matrices::{SparseMatrix, SparseVector},
    meta::{FlowMeta, ImpactMeta, LicenseTierRef, ProcessMeta},
    sign::SignConvention,
    study::{MethodRef, Study},
    units::Unit as ArkoUnit,
};
use arko_io_lcax::{write_lcax_project, EpdDocumentMetadata};
use arko_solvers_dense::DenseLuSolver;
use lcax_models::epd::Standard;
use lcax_models::life_cycle_base::{ImpactCategoryKey, LifeCycleModule};
use lcax_models::product::ImpactData;
use lcax_models::project::Project;
use sprs::TriMat;

fn minimal_two_process_study() -> Study {
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
            reference_unit: ArkoUnit::new("kg"),
            allocation: None,
            license_tier: LicenseTierRef(0),
            geography: Some("GLO".into()),
        },
        ProcessMeta {
            id: "p1".into(),
            name: "electricity".into(),
            reference_product: "electricity".into(),
            reference_unit: ArkoUnit::new("MJ"),
            allocation: None,
            license_tier: LicenseTierRef(0),
            geography: Some("GLO".into()),
        },
    ];

    let flows = vec![FlowMeta {
        id: "f0".into(),
        name: "CO2".into(),
        unit: ArkoUnit::new("kg"),
        compartment: vec!["emission".into(), "air".into()],
        cas: Some("124-38-9".into()),
        origin: Default::default(),
    }];

    let impacts = vec![ImpactMeta {
        id: "gwp100".into(),
        name: "Global Warming Potential (100a)".into(),
        unit: ArkoUnit::new("kg CO2-eq"),
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
fn writes_project_with_single_assembly_product_epd() {
    let study = minimal_two_process_study();
    let computed = compute(&study, &DenseLuSolver).expect("pipeline succeeds");
    let metadata = EpdDocumentMetadata::with_product_name("Steel, minimal fixture");

    let project = write_lcax_project(&study, &computed, &metadata).expect("writer succeeds");

    assert_eq!(project.assemblies.len(), 1);
    let assembly = match &project.assemblies[0] {
        lcax_models::assembly::AssemblyReference::Assembly(a) => a,
        _ => panic!("expected inline Assembly"),
    };
    assert_eq!(assembly.products.len(), 1);
    let product = match &assembly.products[0] {
        lcax_models::product::ProductReference::Product(p) => p,
        _ => panic!("expected inline Product"),
    };
    assert_eq!(product.impact_data.len(), 1);
    let epd = match &product.impact_data[0] {
        ImpactData::EPD(lcax_models::epd::EPDReference::EPD(epd)) => epd,
        _ => panic!("expected inline EPD"),
    };

    // GWP value landed under A1A3 (the V1 stage default).
    let gwp = epd
        .impacts
        .get(&ImpactCategoryKey::GWP)
        .expect("GWP category present");
    let a1a3 = gwp
        .get(&LifeCycleModule::A1A3)
        .expect("A1A3 stage present")
        .expect("A1A3 value is Some");
    // From spec §16: h = [2.05].
    assert!(
        (a1a3 - 2.05).abs() < 1e-12,
        "A1A3 GWP should be 2.05 (spec §16), got {a1a3}"
    );

    // Unknown-standard mapping: `minimal-gwp` is not EN 15804. `Standard`
    // from lcax_models doesn't `derive(Debug)`, so we match rather than
    // assert_eq — the semantics are the same.
    assert!(matches!(epd.standard, Standard::UNKNOWN));

    // Method id+version preserved in EPD.comment (refinement #2 in D-0018).
    let comment = epd.comment.as_deref().unwrap_or("");
    assert!(
        comment.contains("minimal-gwp@0.1"),
        "EPD.comment must preserve method identity, got {comment:?}"
    );
}

#[test]
fn emitted_json_round_trips_as_lcax_project() {
    let study = minimal_two_process_study();
    let computed = compute(&study, &DenseLuSolver).expect("pipeline succeeds");
    let metadata = EpdDocumentMetadata::with_product_name("RoundTrip Fixture");

    let project = write_lcax_project(&study, &computed, &metadata).expect("writer succeeds");
    let json = serde_json::to_string(&project).expect("serialises");

    // Round-trip through the schema-shaped type proves the JSON is
    // LCAx-conformant (not just Rust-serde-conformant): any field shape
    // the deserialiser rejects will surface here.
    let reparsed: Project = serde_json::from_str(&json).expect("round-trips");
    assert_eq!(reparsed.name, "RoundTrip Fixture");
    assert_eq!(reparsed.assemblies.len(), 1);
    // ImpactCategoryKey/LifeCycleModule don't derive Debug — match on
    // the vec explicitly.
    assert_eq!(reparsed.impact_categories.len(), 1);
    assert!(matches!(
        reparsed.impact_categories[0],
        ImpactCategoryKey::GWP
    ));
    assert_eq!(reparsed.life_cycle_modules.len(), 1);
    assert!(matches!(
        reparsed.life_cycle_modules[0],
        LifeCycleModule::A1A3
    ));
}
