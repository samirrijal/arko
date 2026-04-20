//! Cross-implementation parity smoke on the USDA LCA Commons beef
//! cattle finishing bundle — the multi-process LU-parity closure.
//!
//! # Why this test exists
//!
//! [`ef_carpet_parity_smoke`][crate::ef_carpet_parity_smoke] closed the
//! single-column characterization path: carpet is a pre-aggregated LCI
//! (A = 1×1), so the Arko-vs-Python comparison exercised flow
//! matching, CAS lookup, and the AR6 factor application — but not LU
//! factorization. Its own doc flagged the gap:
//!
//! > "A future vector on a multi-process ÖKOBAUDAT bundle will close
//! >  the LU-parity gap."
//!
//! The USDA beef bundle turned out to be the better closure target:
//! five unit processes, non-diagonal technosphere (calf → finishing,
//! pasture → calf, vitamin → finishing + calf, feed → finishing),
//! and a genuine 5×5 solve. Solving it requires LU factorization —
//! the carpet test's `C @ b` shortcut does not apply here.
//!
//! # Independence posture
//!
//! Pipeline independence (same discipline as carpet, extended to LU):
//!
//! - **JSON parser:** Python stdlib `json` vs Arko `serde_json`.
//! - **Bundle walker:** Python filesystem glob + dict lookups vs Arko
//!   `OlcaBundle`.
//! - **Unit convert:** reimplemented from scratch in Python vs Arko
//!   `adapter.rs`.
//! - **CAS normalise / origin rule:** reimplemented in Python (trim
//!   leading zeros; comma-tail classifier) vs Arko's
//!   `normalize_cas` + `classify_flow_origin_from_name`.
//! - **Linear solve:** Python `numpy.linalg.solve` (LAPACK `dgesv`,
//!   partial-pivot LU) vs Arko `DenseLuSolver` (nalgebra partial-pivot
//!   LU). Same algorithm class, different implementations.
//! - **Shared only:** the AR6 WG1 Table 7.15 factor values (tabulated
//!   IPCC constants — same posture as `carpet_reference.py`).
//!
//! A pass therefore exercises A-matrix wiring (diagonals from reference
//! outputs, off-diagonals routed through `defaultProvider`) against an
//! independent linear-system solver, plus flow-matching across multiple
//! species. Factor-value correctness is **not** proven by this test —
//! that is the job of the hand-calc seed vectors in
//! [`arko_differential::seed`] (wiring-vs-factor regime per
//! `feedback_arko_correctness_regime`).
//!
//! # Scope bounds
//!
//! - AR6 GWP100 characterization only. The beef bundle surfaces three
//!   AR6-characterized elementary species: CO2 (CAS 124-38-9), CH4
//!   non-fossil (CAS 74-82-8, comma-tail "biogenic"), and N2O (CAS
//!   10024-97-2). Ammonia and the other elementary outputs carry no
//!   AR6 GWP factor and contribute zero.
//! - `ProducerPositive` sign convention. The adapter emits magnitudes;
//!   the test flips input signs at A-assembly time.
//! - v0.1 openLCA JSON-LD only (see
//!   `engine/io-olca-jsonld/SUPPORTED.md`).
//!
//! # Tolerance class
//!
//! [`ToleranceClass::CrossImpl`] — `ε_abs = 1e-9`, `ε_rel = 1e-6`.
//! Both implementations share the AR6 factor value; the expected
//! divergence is floating-point rounding in the LU pivot order.
//! Tightening to `ReferenceParity` would over-claim: the Python number
//! did not come from Arko itself.
//!
//! # How to run
//!
//! ```bash
//! USDA_BEEF_BUNDLE=/path/to/USDA_Beef cargo test -p arko-io-olca-jsonld \
//!   --test beef_multi_process_parity_smoke -- --ignored --nocapture
//! ```

use std::collections::BTreeMap;
use std::path::PathBuf;

use arko_core::{
    license::LicenseTier,
    matrices::{SparseMatrix, SparseVector},
    meta::{FlowMeta, FlowOrigin as CoreFlowOrigin, LicenseTierRef, ProcessMeta},
    sign::SignConvention,
    study::{MethodRef, Study},
    units::Unit,
};
use arko_differential::{
    run_single_vector, ConformanceLevel, TestVector, ToleranceClass, VectorVerdict,
};
use arko_io_ilcd_linker::{FlowOrigin as LinkerOrigin, FlowType};
use arko_io_olca_jsonld::{olca_to_typed_column, OlcaBundle};
use arko_methods::{build_c_matrix, MethodRegistry};
use arko_solvers_dense::DenseLuSolver;
use sprs::TriMat;

const BUNDLE_ENV_VAR: &str = "USDA_BEEF_BUNDLE";
const FINISHING_UUID: &str = "1b97b691-7c00-4150-9e97-df2020bfd203";

/// Reference h from the independent Python implementation at
/// `scratch/parity/beef_reference.py`, run 2026-04-20 against the
/// USDA LCA Commons beef cattle finishing bundle.
///
/// Pipeline:
/// - `json` (stdlib) parse of all 5 process files + referenced
///   flows / flow_properties / unit_groups
/// - 5×5 A matrix: diagonals from reference outputs in ref units,
///   off-diagonals from `defaultProvider` edges pointing in-bundle
/// - B rows keyed by `(normalize_cas(cas), comma_tail_origin(name))`
///   — elementary flows aggregated across UUID-distinct-but-
///   species-identical variants (e.g., CO2 to air vs water)
/// - AR6 Table 7.15 applied as CAS+origin lookup (origin-specific
///   first, then CAS-only)
/// - `numpy.linalg.solve(A, f)` with `f = e_finishing` → scaling
///   vector, then `h = sum(cf_k * (row_k @ s))` across all matched
///   species
///
/// Species the bundle surfaces that AR6 characterises: CO2 (CAS
/// 124-38-9), CH4 non-fossil (CAS 74-82-8, comma-tail "biogenic"),
/// N2O (CAS 10024-97-2). Ammonia and others in the bundle have no
/// AR6 GWP factor and contribute zero.
///
/// Shared with Arko only at the AR6 factor values — a small
/// tabulated IPCC constant set (same posture as
/// `carpet_reference.py`).
const PY_REFERENCE_BEEF_AR6_GWP100: f64 = 1.123_484_925_812_979_9_e1;

#[test]
#[ignore = "requires USDA_BEEF_BUNDLE env var pointing at an unpacked beef bundle"]
fn beef_multi_process_parity_smoke() {
    let bundle_root = std::env::var(BUNDLE_ENV_VAR).unwrap_or_else(|_| {
        panic!(
            "{BUNDLE_ENV_VAR} not set; point it at an unpacked USDA beef bundle \
             (see engine/io-olca-jsonld/tests/beef_bundle_smoke.rs for layout)"
        )
    });
    let bundle_root = PathBuf::from(bundle_root);

    let bundle = OlcaBundle::open(&bundle_root).expect("open bundle");
    let uuids: Vec<String> = bundle.process_uuids();
    let n = uuids.len();
    assert_eq!(n, 5, "beef bundle should have exactly 5 processes");
    let col: BTreeMap<String, usize> = uuids
        .iter()
        .enumerate()
        .map(|(i, u)| (u.clone(), i))
        .collect();

    // Load all processes + typed columns once; both passes consume them.
    let columns: Vec<_> = uuids
        .iter()
        .map(|u| {
            let p = bundle.load_process(u).expect("load process");
            let c = olca_to_typed_column(&p, &bundle).expect("adapt");
            (p, c)
        })
        .collect();

    // Pass 1: diagonals, per-column reference metadata, elementary-row index.
    let mut ref_flow_uuid = vec![String::new(); n];
    let mut ref_unit_name = vec![String::new(); n];
    let mut process_name = vec![String::new(); n];
    let mut a = TriMat::new((n, n));
    let mut flow_row: BTreeMap<String, usize> = BTreeMap::new();
    let mut flow_metas: Vec<FlowMeta> = Vec::new();

    for (j, (_p, c)) in columns.iter().enumerate() {
        let ref_ex = c
            .exchanges
            .iter()
            .find(|e| e.is_reference_flow)
            .expect("reference exchange present");
        ref_flow_uuid[j] = ref_ex.flow_uuid.clone();
        ref_unit_name[j] = ref_ex.reference_unit.unit_name.clone();
        process_name[j] = c.process_name.clone();
        let sign = if ref_ex.direction.is_input() { -1.0 } else { 1.0 };
        a.add_triplet(j, j, sign * ref_ex.amount);

        for ex in &c.exchanges {
            if ex.is_reference_flow || !matches!(ex.flow_type, FlowType::Elementary) {
                continue;
            }
            if !flow_row.contains_key(&ex.flow_uuid) {
                let olca_flow = bundle.load_flow(&ex.flow_uuid).expect("load flow for CAS");
                flow_row.insert(ex.flow_uuid.clone(), flow_metas.len());
                flow_metas.push(FlowMeta {
                    id: ex.flow_uuid.clone(),
                    name: ex.flow_name.clone(),
                    unit: Unit::new(&ex.reference_unit.unit_name),
                    compartment: Vec::new(),
                    cas: olca_flow.cas.clone(),
                    origin: linker_origin_to_core(ex.origin),
                });
            }
        }
    }

    // Pass 2: off-diagonals (in-bundle technosphere edges) + B entries.
    let m = flow_metas.len();
    let mut b = TriMat::new((m, n));
    for (j, (p, c)) in columns.iter().enumerate() {
        for typed in &c.exchanges {
            if typed.is_reference_flow {
                continue;
            }
            // Join typed exchange -> raw OlcaExchange by internal_id to
            // retrieve `default_provider_uuid` (TypedExchange does not
            // carry it; the adapter validates, the consumer routes).
            let olca = p
                .exchanges
                .iter()
                .find(|e| e.internal_id == typed.data_set_internal_id)
                .expect("typed-to-olca internal_id join");

            match typed.flow_type {
                FlowType::Product | FlowType::Waste => {
                    if let Some(ref dp) = olca.default_provider_uuid {
                        if let Some(&prov_col) = col.get(dp) {
                            assert_eq!(
                                typed.flow_uuid, ref_flow_uuid[prov_col],
                                "exchange flow must match provider's ref product"
                            );
                            let sign = if typed.direction.is_input() { -1.0 } else { 1.0 };
                            a.add_triplet(prov_col, j, sign * typed.amount);
                        }
                    }
                    // No in-bundle provider -> cut-off; ignored at A.
                }
                FlowType::Elementary => {
                    let row = *flow_row
                        .get(&typed.flow_uuid)
                        .expect("row assigned in pass 1");
                    let sign = if typed.direction.is_input() { -1.0 } else { 1.0 };
                    if typed.amount != 0.0 {
                        b.add_triplet(row, j, sign * typed.amount);
                    }
                }
                FlowType::Other => {
                    panic!(
                        "beef bundle produced FlowType::Other on {} exchange {}; \
                         v0.1 adapter classifies all openLCA flow kinds",
                        typed.flow_uuid, typed.data_set_internal_id
                    );
                }
            }
        }
    }

    let technosphere: SparseMatrix = a.to_csr();
    let biosphere: SparseMatrix = b.to_csr();
    let fu_col = *col.get(FINISHING_UUID).expect("finishing column present");
    let functional_unit = SparseVector::new(n, vec![fu_col], vec![1.0]);

    let process_metas: Vec<ProcessMeta> = (0..n)
        .map(|j| ProcessMeta {
            id: uuids[j].clone(),
            name: process_name[j].clone(),
            reference_product: ref_flow_uuid[j].clone(),
            reference_unit: Unit::new(&ref_unit_name[j]),
            allocation: None,
            license_tier: LicenseTierRef(0),
            geography: Some("US".into()),
        })
        .collect();

    let reg = MethodRegistry::standard();
    let method_ref = MethodRef {
        id: "ipcc-ar6-gwp100".into(),
        version: "1".into(),
    };
    let method = reg.lookup(&method_ref).expect("AR6 method in registry");
    let built = build_c_matrix(method, &flow_metas).expect("build_c_matrix");

    let study = Study {
        technosphere,
        biosphere,
        characterization: built.matrix,
        functional_unit,
        processes: process_metas,
        flows: flow_metas,
        impacts: built.impacts,
        parameters: Vec::new(),
        license_tiers: vec![LicenseTier::permissive("usda-lca-commons-cc0")],
        method: method_ref,
        sign_convention: SignConvention::ProducerPositive,
    };

    let vector = TestVector {
        id: "usda-beef-ar6-gwp100-py-parity".into(),
        level: ConformanceLevel::L1Basic,
        description:
            "Arko vs independent Python reference on USDA beef cattle \
             finishing bundle under AR6 GWP100 (5-process LU solve)"
                .into(),
        study,
        expected_h: vec![PY_REFERENCE_BEEF_AR6_GWP100],
        tolerance_class: ToleranceClass::CrossImpl,
        notes: Some(
            "Reference from scratch/parity/beef_reference.py (stdlib json \
             parser, numpy.linalg.solve via LAPACK dgesv). 5×5 A matrix \
             with calf→finishing, pasture→calf, vitamin→finishing+calf, \
             feed→finishing technosphere edges. Closes the multi-process \
             LU-parity gap that ef_carpet_parity_smoke deliberately did \
             not cover (carpet is A=1×1)."
                .into(),
        ),
    };

    let verdict = run_single_vector(&vector, &DenseLuSolver);

    println!("--- beef_multi_process_parity_smoke ---");
    println!("bundle:         {}", bundle_root.display());
    println!("processes:      {n}");
    println!("B row count:    {m}");
    println!("py reference:   {PY_REFERENCE_BEEF_AR6_GWP100} kg CO2-eq / kg beef LW");
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
            println!(
                "  tolerance:    eps_abs={:e}, eps_rel={:e}",
                ToleranceClass::CrossImpl.eps_abs(),
                ToleranceClass::CrossImpl.eps_rel()
            );
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
    println!("--- end beef_multi_process_parity_smoke ---");

    assert!(
        matches!(verdict, VectorVerdict::Pass { .. }),
        "parity vector must pass; got {verdict:?}"
    );
}

fn linker_origin_to_core(o: LinkerOrigin) -> CoreFlowOrigin {
    match o {
        LinkerOrigin::Fossil => CoreFlowOrigin::Fossil,
        LinkerOrigin::NonFossil => CoreFlowOrigin::NonFossil,
        LinkerOrigin::Unspecified => CoreFlowOrigin::Unspecified,
    }
}
