//! End-to-end calc-correctness smoke on real JRC EF data.
//!
//! # Why this test exists
//!
//! Phase 1 Week 5 step (b) closed with the 94k-flow resolver-only smoke
//! on the JRC EF 3.1 reference package: the linker resolves flow →
//! flow-property → unit-group chains cleanly at scale. That proved the
//! *reader-to-bridge* leg.
//!
//! What it deliberately did **not** prove is the *bridge-to-engine* leg:
//! does the linker's `TypedColumn` output structurally feed
//! `arko-methods::build_c_matrix` and `arko-core::pipeline::compute`,
//! and does the pipeline produce a finite impact number on real EF
//! content?
//!
//! This test answers exactly that, and only that. It is **not** a
//! methodology validation:
//!
//! - It does not assert the output matches a published reference value
//!   (no published number exists for this exact EF dataset).
//! - It does not exercise multi-process LU decomposition (the chosen
//!   process is an LCI result — the upstream graph is collapsed into
//!   one column; LU is already covered in [`minimal_example`]).
//! - It does not yet handle CH4 fossil/non-fossil splitting under AR6:
//!   `arko-io-ilcd-linker::Flow` carries no `FlowOrigin` field
//!   (compartment/name parsing in the reader is its own follow-up
//!   piece of work). Under AR6 the CH4 column will surface as
//!   `unmatched_flows`; under AR5's origin-agnostic CAS matcher it
//!   will match. Both behaviours are correct given current inputs;
//!   the smoke prints the count so the gap is visible.
//!
//! What it **does** prove on success:
//!
//! - `build_typed_column` output bridges into `FlowMeta` cleanly at
//!   ~20k flow scale.
//! - `build_c_matrix` constructs a `(k × m)` C matrix over real EF flow
//!   IDs without panicking, on both AR6 and AR5.
//! - The full `compute()` pipeline returns a finite impact number on
//!   the resulting `Study`.
//! - The AR6-vs-AR5 unmatched-flow delta makes the FlowOrigin gap
//!   concrete (a number, not a hypothetical).
//!
//! # The chosen process
//!
//! UUID `972cd3cd-25bf-4b70-96e9-eab4bed329f7` —
//! *"Repurposing of sports surfacing carpet — Avoided production of
//! carpet manufacturing for landscaping applications"*. Single-process
//! LCI result, EU+EFTA+UK 2023, reference flow 1 m² synthetic turf
//! system.
//!
//! Why this one: it's the single EF process currently on disk
//! (downloaded from the EC EF node single-process export, see
//! [`ef_reference_smoke`]). Not the advisor-ideal smoke target — that
//! would be a small-exchange-count CO2-dominated unit process with a
//! known published GWP100 reference value. We have ~20k exchanges,
//! avoided-burden semantics (so the result is an environmental
//! *credit* per m² that may be net-positive or net-negative depending
//! on what dominates), and no published reference.
//!
//! These are real constraints we acknowledge in the result
//! interpretation, not dealbreakers for a wiring smoke.
//!
//! # Sign convention
//!
//! Exchange amounts are passed through to the biosphere matrix
//! verbatim — no Input/Output sign flip applied. ILCD direction
//! handling at the bridge layer is its own follow-up; for the wiring
//! smoke the goal is "does the pipeline produce a finite number,"
//! not "is the number's sign-convention-correct interpretation
//! defensible across all flow types."
//!
//! # How to run
//!
//! ```bash
//! EF_REFERENCE_BUNDLE=/path/to/bundle cargo test \
//!   -p arko-io-ilcd-linker --test ef_carpet_calc_smoke \
//!   -- --ignored --nocapture
//! ```

use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Instant;

use arko_core::{
    compute,
    license::LicenseTier,
    matrices::{SparseMatrix, SparseVector},
    meta::{FlowMeta, FlowOrigin, LicenseTierRef, ProcessMeta},
    sign::SignConvention,
    study::{MethodRef, Study},
    units::Unit,
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

/// Test-local caching wrapper so the second pass over flows (to harvest
/// CAS for `FlowMeta`) doesn't re-walk the filesystem. The engine
/// itself is deliberately cache-less at v0.1; this is a test concern,
/// not an engine concern.
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
fn ef_carpet_calc_smoke() {
    let bundle_root = std::env::var(BUNDLE_ENV_VAR).unwrap_or_else(|_| {
        panic!(
            "{BUNDLE_ENV_VAR} not set; point it at an unpacked EF reference package \
             (see test module docs for instructions)"
        )
    });
    let bundle_root = PathBuf::from(bundle_root);
    let processes_dir = bundle_root.join("processes");
    assert!(
        processes_dir.is_dir(),
        "expected processes subdirectory at {}",
        processes_dir.display()
    );

    // Find the carpet process XML by UUID-prefix filename match.
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

    println!("--- ef_carpet_calc_smoke ---");
    println!("process file: {}", process_path.display());

    let resolver = CachingResolver::new(DirectoryBundle::new(&bundle_root));

    let xml = std::fs::read_to_string(&process_path).expect("read process xml");
    let dataset = parse_process(&xml).expect("parse_process");

    let t0 = Instant::now();
    let column = build_typed_column(&dataset, &resolver).expect("build_typed_column");
    let typed_column_secs = t0.elapsed().as_secs_f64();

    println!("process uuid:   {}", column.process_uuid);
    println!("process name:   {}", column.process_name);
    println!(
        "exchanges:      {} total ({:.1}s to build typed column with caching resolver)",
        column.exchanges.len(),
        typed_column_secs
    );

    // Reference flow: drives technosphere A and demand vector f.
    let ref_exchange = column
        .exchanges
        .iter()
        .find(|e| e.is_reference_flow)
        .expect("reference flow present");
    println!(
        "reference flow: {} ({}) — amount {} {}",
        ref_exchange.flow_name,
        ref_exchange.flow_uuid,
        ref_exchange.amount,
        ref_exchange.reference_unit.unit_name,
    );

    // Elementary, non-reference exchanges become biosphere rows.
    let elementary: Vec<_> = column
        .exchanges
        .iter()
        .filter(|e| matches!(e.flow_type, FlowType::Elementary) && !e.is_reference_flow)
        .collect();

    let mut other_counts: HashMap<&'static str, usize> = HashMap::new();
    for ex in &column.exchanges {
        if ex.is_reference_flow {
            continue;
        }
        let bucket = match ex.flow_type {
            FlowType::Elementary => "elementary (-> B)",
            FlowType::Product => "product non-ref (skipped)",
            FlowType::Waste => "waste (skipped)",
            FlowType::Other => "other (skipped)",
        };
        *other_counts.entry(bucket).or_default() += 1;
    }
    println!("flow-type breakdown of non-reference exchanges:");
    let mut counts_vec: Vec<_> = other_counts.into_iter().collect();
    counts_vec.sort();
    for (bucket, n) in &counts_vec {
        println!("  {bucket:32} {n}");
    }

    // Build FlowMeta — one per elementary exchange. CAS comes from
    // re-resolving via the cache (free after first pass). Origin is
    // already on the typed exchange (parsed from the basename
    // parenthetical by the linker).
    let t1 = Instant::now();
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
    let cas_with = flows.iter().filter(|f| f.cas.is_some()).count();
    let origin_fossil = flows
        .iter()
        .filter(|f| matches!(f.origin, FlowOrigin::Fossil))
        .count();
    let origin_non_fossil = flows
        .iter()
        .filter(|f| matches!(f.origin, FlowOrigin::NonFossil))
        .count();
    println!(
        "FlowMeta built:  {} elementary flows ({} with CAS, {} without) in {:.2}s",
        flows.len(),
        cas_with,
        flows.len() - cas_with,
        t1.elapsed().as_secs_f64(),
    );
    println!(
        "  origin tagged: {origin_fossil} fossil, {origin_non_fossil} non-fossil, {} unspecified",
        flows.len() - origin_fossil - origin_non_fossil,
    );

    let m = flows.len();
    assert!(m > 0, "no elementary flows — nothing to characterize");

    // Biosphere: m × 1, one column = the elementary exchange amounts.
    // Sign convention: amounts passed through verbatim (no Input/Output
    // flip). See module docs.
    let mut b = TriMat::new((m, 1));
    for (row, ex) in elementary.iter().enumerate() {
        if ex.amount != 0.0 {
            b.add_triplet(row, 0, ex.amount);
        }
    }
    let biosphere: SparseMatrix = b.to_csr();

    // Technosphere: 1 × 1 identity. LCI result is pre-aggregated.
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

    for method_id in ["ipcc-ar6-gwp100", "ipcc-ar5-gwp100"] {
        println!();
        println!("=== {method_id} ===");
        let method_ref = MethodRef {
            id: method_id.into(),
            version: "1".into(),
        };
        let method = reg.lookup(&method_ref).expect("method present in registry");

        let t2 = Instant::now();
        let built = build_c_matrix(method, &flows).expect("build_c_matrix");
        let c_secs = t2.elapsed().as_secs_f64();

        let nnz = built.matrix.nnz();
        let unmatched = built.unmatched_flows.len();
        let matched = m - unmatched;
        println!(
            "C matrix:        {}×{} with {} nonzeros ({:.2}s)",
            built.matrix.shape().0,
            built.matrix.shape().1,
            nnz,
            c_secs,
        );
        println!(
            "match rate:      {matched}/{m} flows matched ({:.2}%), {unmatched} unmatched",
            100.0 * matched as f64 / m as f64,
        );

        let study = Study {
            technosphere: technosphere.clone(),
            biosphere: biosphere.clone(),
            characterization: built.matrix.clone(),
            functional_unit: functional_unit.clone(),
            processes: vec![process_meta.clone()],
            flows: flows.clone(),
            impacts: built.impacts.clone(),
            parameters: Vec::new(),
            license_tiers: vec![LicenseTier::permissive("ef-reference-bundle")],
            method: method_ref,
            sign_convention: SignConvention::ProducerPositive,
        };

        let t3 = Instant::now();
        let result = compute(&study, &DenseLuSolver).expect("compute");
        let compute_secs = t3.elapsed().as_secs_f64();

        let impact = result.impact[0];
        let impact_unit = &built.impacts[0].unit;
        println!(
            "impact[0]:       {impact:.6e} {impact_unit} (computed in {compute_secs:.3}s)",
        );

        // Wiring assertions — the only things this smoke can defensibly assert.
        assert!(
            impact.is_finite(),
            "{method_id} produced non-finite impact: {impact}"
        );
        assert_eq!(
            result.scaling.len(),
            1,
            "scaling vector should be length 1 (single-process LCI result)"
        );
        assert!(
            (result.scaling[0] - 1.0).abs() < 1e-12,
            "scaling[0] should be 1.0 for f=[1.0] against identity A; got {}",
            result.scaling[0]
        );
        assert_eq!(
            result.inventory.len(),
            m,
            "inventory length should equal flow count"
        );

        // Top-5 unmatched flows, by name, for visibility.
        if !built.unmatched_flows.is_empty() {
            println!("first 5 unmatched flow ids (informational):");
            for id in built.unmatched_flows.iter().take(5) {
                let name = flows
                    .iter()
                    .find(|f| f.id == *id)
                    .map(|f| f.name.as_str())
                    .unwrap_or("<unknown>");
                println!("  {id}  {name}");
            }
        }
    }

    println!();
    println!("--- end ef_carpet_calc_smoke ---");
}

/// Cross-crate mapping: linker `FlowOrigin` → engine `FlowOrigin`.
/// Lives in this test (not in the linker) so the linker doesn't take
/// a dependency on `arko-core`. Production code that bridges
/// `TypedColumn` to `Study` will own its own version of this.
fn linker_origin_to_core(o: LinkerOrigin) -> FlowOrigin {
    match o {
        LinkerOrigin::Fossil => FlowOrigin::Fossil,
        LinkerOrigin::NonFossil => FlowOrigin::NonFossil,
        LinkerOrigin::Unspecified => FlowOrigin::Unspecified,
    }
}
