//! Cross-implementation parity smoke on the JRC EF carpet LCI.
//!
//! # Why this test exists
//!
//! Phase 1 Week 5 step (b) closed with [`ef_carpet_calc_smoke`] —
//! Arko's pipeline produces a finite AR6 GWP100 number on the carpet
//! LCI (`9.180244e0 kg CO2-eq / m²` as of 2026-04-19). That smoke
//! proved the pipeline **runs** on real EF content; it deliberately
//! did not assert the number matches anything.
//!
//! This smoke closes that gap. The reference value comes from a
//! fully **independent** Python implementation (see
//! `scratch/parity/carpet_reference.py`):
//!
//! - different XML parser    (Python `lxml`   vs Arko `roxmltree`/`quick-xml`)
//! - different CAS matcher   (Python dict     vs Arko `FactorMatch::CasOrigin`)
//! - different origin rule   (reimplemented   vs `classify_flow_origin`)
//! - different arithmetic    (`sum()` in CPython vs sparse matvec + LU)
//! - **shared** only at the AR6 Table 7.15 factor **values**, which
//!   are tabulated IPCC constants.
//!
//! A pass therefore exercises flow-matching + amount extraction +
//! CF lookup + arithmetic wiring against an engine written
//! independently. It does **not** verify the AR6 factor values
//! themselves — that's the job of the hand-calc seed vectors in
//! [`arko_differential::seed`].
//!
//! # Scope bounds
//!
//! Carpet is a pre-aggregated LCI result: `A = 1×1`, reference flow
//! is the repurposing credit of 1 m². The parity test therefore
//! covers the **characterization** path (B × x × C collapses to
//! `C @ b`) but not LU factorization — that stays covered by the
//! hand-calc `l1_coupled_two_process` seed vector. A future vector on
//! a multi-process ÖKOBAUDAT bundle will close the LU-parity gap.
//!
//! # Tolerance class
//!
//! [`ToleranceClass::CrossImpl`] — `ε_abs = 1e-9`, `ε_rel = 1e-6`.
//! The two implementations share the factor table, so the expected
//! divergence is strictly floating-point rounding in the accumulation
//! order (Python `sum()` vs Arko's sparse matvec). `CrossImpl` is still
//! the right class: the reference number did not come from Arko itself,
//! and tightening to `ReferenceParity` would over-claim.
//!
//! # How to run
//!
//! ```bash
//! EF_REFERENCE_BUNDLE=/path/to/bundle cargo test \
//!   -p arko-io-ilcd-linker --test ef_carpet_parity_smoke \
//!   -- --ignored --nocapture
//! ```

use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::sync::Mutex;

use arko_core::{
    license::LicenseTier,
    matrices::{SparseMatrix, SparseVector},
    meta::{FlowMeta, FlowOrigin, LicenseTierRef, ProcessMeta},
    sign::SignConvention,
    study::{MethodRef, Study},
    units::Unit,
};
use arko_differential::{
    run_single_vector, ConformanceLevel, TestVector, ToleranceClass, VectorVerdict,
};
use arko_io_ilcd::parse_process;
use arko_io_ilcd_linker::{
    build_typed_column, DirectoryBundle, Flow, FlowOrigin as LinkerOrigin, FlowProperty, FlowType,
    LinkError, LinkResolver, UnitGroup,
};
use arko_methods::{build_c_matrix, MethodRegistry};
use arko_solvers_dense::DenseLuSolver;
use sprs::TriMat;

const BUNDLE_ENV_VAR: &str = "EF_REFERENCE_BUNDLE";
const TARGET_PROCESS_UUID: &str = "972cd3cd-25bf-4b70-96e9-eab4bed329f7";

/// Reference impact value from the independent Python reference
/// implementation at `scratch/parity/carpet_reference.py`, run
/// 2026-04-19 against the same EF 3.1 background-processes bundle.
///
/// Sourced by parsing the carpet process ILCD XML with `lxml`,
/// classifying flow origins from the basename parenthetical, and
/// applying AR6 WG1 Ch7 Table 7.15 factors directly. 44 flows matched
/// (identical to Arko's match set).
const PY_REFERENCE_AR6_GWP100: f64 = 9.180_243_685_487_213_e0;

struct CachingResolver<R: LinkResolver> {
    inner: R,
    flows: Mutex<HashMap<String, Flow>>,
    flow_properties: Mutex<HashMap<String, FlowProperty>>,
    unit_groups: Mutex<HashMap<String, UnitGroup>>,
}

impl<R: LinkResolver> CachingResolver<R> {
    fn new(inner: R) -> Self {
        Self {
            inner,
            flows: Mutex::new(HashMap::new()),
            flow_properties: Mutex::new(HashMap::new()),
            unit_groups: Mutex::new(HashMap::new()),
        }
    }
}

impl<R: LinkResolver> LinkResolver for CachingResolver<R> {
    fn resolve_flow(&self, uuid: &str) -> Result<Flow, LinkError> {
        if let Some(f) = self.flows.lock().unwrap().get(uuid) {
            return Ok(f.clone());
        }
        let f = self.inner.resolve_flow(uuid)?;
        self.flows
            .lock()
            .unwrap()
            .insert(uuid.to_string(), f.clone());
        Ok(f)
    }

    fn resolve_flow_property(&self, uuid: &str) -> Result<FlowProperty, LinkError> {
        if let Some(fp) = self.flow_properties.lock().unwrap().get(uuid) {
            return Ok(fp.clone());
        }
        let fp = self.inner.resolve_flow_property(uuid)?;
        self.flow_properties
            .lock()
            .unwrap()
            .insert(uuid.to_string(), fp.clone());
        Ok(fp)
    }

    fn resolve_unit_group(&self, uuid: &str) -> Result<UnitGroup, LinkError> {
        if let Some(ug) = self.unit_groups.lock().unwrap().get(uuid) {
            return Ok(ug.clone());
        }
        let ug = self.inner.resolve_unit_group(uuid)?;
        self.unit_groups
            .lock()
            .unwrap()
            .insert(uuid.to_string(), ug.clone());
        Ok(ug)
    }
}

#[test]
#[ignore = "requires EF reference package on disk; set EF_REFERENCE_BUNDLE env var"]
fn ef_carpet_parity_smoke() {
    let bundle_root = std::env::var(BUNDLE_ENV_VAR).unwrap_or_else(|_| {
        panic!(
            "{BUNDLE_ENV_VAR} not set; point it at an unpacked EF reference package \
             (see ef_carpet_calc_smoke for instructions)"
        )
    });
    let bundle_root = PathBuf::from(bundle_root);
    let processes_dir = bundle_root.join("processes");
    assert!(
        processes_dir.is_dir(),
        "expected processes subdirectory at {}",
        processes_dir.display()
    );

    let process_path = std::fs::read_dir(&processes_dir)
        .expect("read processes dir")
        .filter_map(Result::ok)
        .map(|e| e.path())
        .find(|p| {
            p.extension() == Some(OsStr::new("xml"))
                && p.file_stem()
                    .and_then(|s| s.to_str())
                    .is_some_and(|s| s.starts_with(TARGET_PROCESS_UUID))
        })
        .unwrap_or_else(|| {
            panic!(
                "expected a process file beginning with `{TARGET_PROCESS_UUID}` under {}",
                processes_dir.display()
            )
        });

    println!("--- ef_carpet_parity_smoke ---");
    println!("process:        {}", process_path.display());
    println!("py reference:   {PY_REFERENCE_AR6_GWP100} kg CO2-eq / m²");

    let resolver = CachingResolver::new(DirectoryBundle::new(&bundle_root));
    let xml = std::fs::read_to_string(&process_path).expect("read process xml");
    let dataset = parse_process(&xml).expect("parse_process");

    let column = build_typed_column(&dataset, &resolver).expect("build_typed_column");
    let ref_exchange = column
        .exchanges
        .iter()
        .find(|e| e.is_reference_flow)
        .expect("reference flow present");
    let elementary: Vec<_> = column
        .exchanges
        .iter()
        .filter(|e| matches!(e.flow_type, FlowType::Elementary) && !e.is_reference_flow)
        .collect();

    let flows: Vec<FlowMeta> = elementary
        .iter()
        .map(|ex| {
            let flow = resolver
                .resolve_flow(&ex.flow_uuid)
                .expect("re-resolve flow for CAS");
            FlowMeta {
                id: ex.flow_uuid.clone(),
                name: ex.flow_name.clone(),
                unit: Unit::new(&ex.reference_unit.unit_name),
                compartment: Vec::new(),
                cas: flow.cas.clone(),
                origin: linker_origin_to_core(ex.origin),
            }
        })
        .collect();

    let m = flows.len();
    let mut b = TriMat::new((m, 1));
    for (row, ex) in elementary.iter().enumerate() {
        if ex.amount != 0.0 {
            b.add_triplet(row, 0, ex.amount);
        }
    }
    let biosphere: SparseMatrix = b.to_csr();

    let mut a = TriMat::new((1, 1));
    a.add_triplet(0, 0, 1.0);
    let technosphere: SparseMatrix = a.to_csr();

    let functional_unit = SparseVector::new(1, vec![0], vec![1.0]);

    let process_meta = ProcessMeta {
        id: column.process_uuid.clone(),
        name: column.process_name.clone(),
        reference_product: ref_exchange.flow_name.clone(),
        reference_unit: Unit::new(&ref_exchange.reference_unit.unit_name),
        allocation: None,
        license_tier: LicenseTierRef(0),
        geography: Some("EU+EFTA+UK".into()),
    };

    let reg = MethodRegistry::standard();
    let method_ref = MethodRef {
        id: "ipcc-ar6-gwp100".into(),
        version: "1".into(),
    };
    let method = reg.lookup(&method_ref).expect("AR6 method in registry");
    let built = build_c_matrix(method, &flows).expect("build_c_matrix");

    let study = Study {
        technosphere,
        biosphere,
        characterization: built.matrix,
        functional_unit,
        processes: vec![process_meta],
        flows,
        impacts: built.impacts,
        parameters: Vec::new(),
        license_tiers: vec![LicenseTier::permissive("ef-reference-bundle")],
        method: method_ref,
        sign_convention: SignConvention::ProducerPositive,
    };

    let vector = TestVector {
        id: "ef-carpet-ar6-gwp100-py-parity".into(),
        level: ConformanceLevel::L1Basic,
        description:
            "Arko vs independent Python reference on JRC EF carpet LCI under AR6 GWP100"
                .into(),
        study,
        expected_h: vec![PY_REFERENCE_AR6_GWP100],
        tolerance_class: ToleranceClass::CrossImpl,
        notes: Some(
            "Reference from scratch/parity/carpet_reference.py (lxml parser, \
             independent CAS matcher, hand-encoded AR6 Table 7.15 factors). \
             Carpet is a pre-aggregated LCI (A=1×1): this exercises flow-matching \
             and characterization, not LU factorization."
                .into(),
        ),
    };

    let verdict = run_single_vector(&vector, &DenseLuSolver);

    println!();
    match &verdict {
        VectorVerdict::Pass {
            duration_us,
            max_abs_deviation,
            max_rel_deviation,
        } => {
            println!("VERDICT:        PASS");
            println!("  max |dev|:    {max_abs_deviation:.3e}");
            println!("  max rel dev:  {max_rel_deviation:.3e}");
            println!("  tolerance:    eps_abs={:e}, eps_rel={:e}",
                ToleranceClass::CrossImpl.eps_abs(),
                ToleranceClass::CrossImpl.eps_rel());
            println!("  compute:      {duration_us} µs");
        }
        VectorVerdict::Fail {
            index,
            got,
            want,
            observed_deviation,
            tolerance_applied,
            duration_us,
        } => {
            println!("VERDICT:        FAIL");
            println!("  index:        {index}");
            println!("  got:          {got}");
            println!("  want:         {want}");
            println!("  |dev|:        {observed_deviation:.3e}");
            println!("  tolerance:    {tolerance_applied:.3e}");
            println!("  compute:      {duration_us} µs");
        }
        VectorVerdict::EngineError { code, message } => {
            println!("VERDICT:        ENGINE_ERROR {code}: {message}");
        }
        VectorVerdict::ShapeMismatch {
            expected_impacts,
            study_impacts,
        } => {
            println!(
                "VERDICT:        SHAPE_MISMATCH (expected {expected_impacts}, got {study_impacts})"
            );
        }
    }
    println!("--- end ef_carpet_parity_smoke ---");

    assert!(
        matches!(verdict, VectorVerdict::Pass { .. }),
        "parity vector must pass; got {verdict:?}"
    );
}

fn linker_origin_to_core(o: LinkerOrigin) -> FlowOrigin {
    match o {
        LinkerOrigin::Fossil => FlowOrigin::Fossil,
        LinkerOrigin::Biogenic => FlowOrigin::Biogenic,
        LinkerOrigin::LandUseChange => FlowOrigin::LandUseChange,
        LinkerOrigin::Unspecified => FlowOrigin::Unspecified,
    }
}
