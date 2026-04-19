//! Resolver-only smoke test against the JRC Environmental Footprint
//! **reference package** (the infrastructure-only bundle: flows,
//! flowproperties, unitgroups, lciamethods, sources, contacts, **no
//! processes**).
//!
//! # Why this is a separate smoke from `ef_reference_smoke`
//!
//! Phase 1 Week 5 discovered that the JRC publishes two structurally
//! different EF-related bundles:
//!
//! - The **EF reference package** (this test's input): the canonical
//!   EF 3.x infrastructure — elementary-flow master list, flow-
//!   property master, unit-group backbone, LCIA methods. Build against
//!   this to produce EF-compliant datasets. Ships with zero
//!   processes. Hosted on EPLCA under "Developer - Environmental
//!   Footprint".
//! - The **EF node process exports** (`ef_reference_smoke`'s input):
//!   actual process datasets produced by nodes (EC, ecoinvent, Sphera,
//!   industry bodies) that use the reference-package infrastructure.
//!   Processes included; the bundle's flow/flowproperty/unitgroup
//!   subset covers only the node-specific slice it uses.
//!
//! `ef_reference_smoke` drives one process through the full parse +
//! bridge pipeline. This smoke drives the bridge by itself — the
//! `resolver → flow → flowproperty → unitgroup → reference-unit`
//! chain — over **every** flow in the reference package (the JRC EF
//! 3.1 package ships ~94k elementary flows). At that scale, it's the
//! broadest reader-level generalisation signal available to v0.1
//! without assembling a multi-process bundle.
//!
//! # Claim shape
//!
//! *"For every flow in the JRC's canonical EF 3.x elementary-flow
//! master, `arko-io-ilcd-linker` parses it and walks its reference-
//! unit chain without engine error."* That's the falsifiable claim.
//! Publisher-side gaps (a referenced flow-property or unit-group XML
//! not in the bundle; a flow with no `<quantitativeReference>` and no
//! inline unit-group ref) are tolerated — the engine correctly refuses
//! to invent a unit when the derivation path is missing.
//!
//! # How to run locally
//!
//! 1. Download **EF reference package 3.x** from EPLCA under
//!    *Developer - Environmental Footprint*.
//! 2. Unzip. Expected top-level layout:
//!    `<root>/ILCD/{flows,flowproperties,unitgroups,lciamethods,
//!    sources,contacts}/`, plus `<root>/other/`, `<root>/META-INF/`,
//!    `<root>/schemas/`.
//! 3. Point the test at the `ILCD/` subdirectory:
//!
//!    ```bash
//!    EF_REFERENCE_PACKAGE_BUNDLE=/path/to/EF-v3.1/ILCD \
//!      cargo test -p arko-io-ilcd-linker \
//!      --test ef_reference_package_resolver_smoke \
//!      -- --ignored --nocapture
//!    ```
//!
//! # Performance — test-local cache
//!
//! The engine's `DirectoryBundle` is intentionally cache-less at v0.1
//! (caching policy deferred until profiling evidence names a need —
//! see the `LinkResolver` trait doc). At 94k flows × ~101 flow-
//! properties × ~11 unit-groups that trait contract costs tens of
//! thousands of redundant XML re-parses per resolve. A debug-build
//! first run on 2026-04-19 exceeded one hour with no output and was
//! killed; a release-build second run also blew past ten minutes.
//! Both runs were flying blind — the test printed nothing until the
//! end.
//!
//! The fix in this file is **test-local**: a `CachingResolver`
//! wrapper around `DirectoryBundle`, plus a progress line every 2000
//! flows so the test is no longer opaque. This is the profiling-
//! evidence answer for *this test*, not a statement that caching
//! belongs inside the crate. If/when other workloads show the same
//! pattern, the crate-level decision can be revisited with real
//! numbers; until then, only this test pays for the cache, and the
//! engine stays naive.
//!
//! [crate-level]: https://docs.rs/arko-io-ilcd-linker/latest/arko_io_ilcd_linker/
//!
//! # What is asserted
//!
//! For every `flows/<UUID>.xml` in the bundle:
//! - Filename-derived UUID parses cleanly as an `arko-io-ilcd`
//!   `Flow` via `LinkResolver::resolve_flow`.
//! - `resolve_reference_unit_from_flow` either returns a
//!   `ReferenceUnit` with a non-empty `unit_name`, or fails with a
//!   publisher-gap variant (`LinkError::Io` for missing cross-ref
//!   XMLs, `LinkError::FlowHasNoUnitDerivation` for flows without a
//!   `<quantitativeReference>`).
//!
//! Any other `LinkError` variant, or a non-UTF8 filename, counts as
//! an engine-level failure and fails the test. The reference-package
//! expectation is **zero engine failures** (it's the JRC's own data).
//!
//! # Observed 2026-04-19 characterisation — JRC EF 3.1 reference package
//!
//! First run on 2026-04-19 against the JRC EF reference package 3.1
//! downloaded from EPLCA under *Developer - Environmental Footprint*.
//! Bundle contained **94,062 elementary-flow XML files** under
//! `flows/`, plus the supporting `flowproperties/`, `unitgroups/`,
//! `lciamethods/`, `sources/`, `contacts/` subdirectories
//! (the infrastructure-only bundle — zero processes, as expected).
//!
//! Result: **94,062 / 94,062 flows parsed; 94,062 fully resolved
//! through the chain; 0 publisher gaps; 0 engine failures.**
//! Runtime 821.0 s on the maintainer Windows laptop (debug build;
//! the release build was never needed once the test-local cache was
//! added). Flow-type distribution: Elementary 93,993 / Waste 68 /
//! Product 1. Reference-unit distribution, full set (only 8 distinct
//! units across 94k flows): `kg` 92,802; `kBq` 964; `m²` 150;
//! `m²·a` 76; `MJ` 34; `m³` 29; `kg·a` 6; `m³·a` 1.
//!
//! **Rate profile note (unexplained, non-blocking).** Throughput
//! held at ~1,650 flows/s through the first ~50k flows (UUID-sorted),
//! then degraded monotonically to ~115 flows/s by the end of the run.
//! The most likely explanation is memory pressure from this test's
//! `CachingResolver` caching every parsed `Flow` alongside the
//! (intentionally cached) flow-properties and unit-groups — each
//! flow UUID appears exactly once in the outer loop, so the flow
//! cache has zero hit-rate benefit and grows to 94k parsed
//! structures. A follow-up cleanup could drop the flow cache and
//! keep only flow-property + unit-group caches; not worth a re-run
//! purely for faster numbers, since the correctness claim is
//! already banked.
//!
//! **Claims this run supports:**
//! - `arko-io-ilcd` parses 100% of the JRC EF 3.1 elementary-flow
//!   master without engine error.
//! - `arko-io-ilcd-linker`'s `resolve_reference_unit_from_flow` walks
//!   100% of those flows' reference-unit chains without engine error.
//! - Zero publisher-side gaps in the JRC's own curated master,
//!   consistent with the opening expectation that EF reference is
//!   cleaner than ÖKOBAUDAT (which ran 3.4% gaps on 3,075 processes).
//! - Reader-level generalisation breadth from ÖKOBAUDAT's few-thousand
//!   flows to ~94k is now evidenced in one step.
//!
//! **Claims this run does *not* yet support:**
//! - Process-level ingest on EF (this bundle ships no processes; the
//!   single-process `ef_reference_smoke.rs` result from the EC EF node
//!   remains the only EF process evidence, and it's N=1).
//! - Multi-process ILCD ingest on EF at any scale.
//! - End-to-end calculation correctness on EF data (no calculation
//!   performed; methods layer lands in Phase 1 Week 6).
//! - Coverage beyond EF (ÖKOBAUDAT + EF are the two passing open-EU
//!   databases; a third foreground-free database is still `D-0010`'s
//!   TBD).
//!
//! Bundle is not redistributed. Reproducible by downloading the EF
//! reference package 3.x zip from EPLCA Developer - Environmental
//! Footprint and pointing `EF_REFERENCE_PACKAGE_BUNDLE` at the
//! unpacked `ILCD/` subdirectory.

use std::collections::{BTreeMap, HashMap};
use std::ffi::OsStr;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Instant;

use arko_io_ilcd_linker::{
    resolve_reference_unit_from_flow, DirectoryBundle, Flow, FlowProperty, LinkError,
    LinkResolver, UnitGroup,
};

const BUNDLE_ENV_VAR: &str = "EF_REFERENCE_PACKAGE_BUNDLE";

/// Test-local caching adapter.
///
/// The engine's `DirectoryBundle` is intentionally cache-less at v0.1
/// (caching policy deferred until profiling evidence names a need —
/// see `LinkResolver` doc). On a 94k-flow bundle with ~101 flow-
/// properties and ~11 unit-groups that trait contract costs us tens
/// of thousands of redundant XML re-parses per resolve. This wrapper
/// is the profiling-evidence answer for *this test*, not a statement
/// that caching belongs in the crate.
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
        if let Some(hit) = self.flows.lock().unwrap().get(uuid) {
            return Ok(hit.clone());
        }
        let fresh = self.inner.resolve_flow(uuid)?;
        self.flows
            .lock()
            .unwrap()
            .insert(uuid.to_string(), fresh.clone());
        Ok(fresh)
    }
    fn resolve_flow_property(&self, uuid: &str) -> Result<FlowProperty, LinkError> {
        if let Some(hit) = self.flow_properties.lock().unwrap().get(uuid) {
            return Ok(hit.clone());
        }
        let fresh = self.inner.resolve_flow_property(uuid)?;
        self.flow_properties
            .lock()
            .unwrap()
            .insert(uuid.to_string(), fresh.clone());
        Ok(fresh)
    }
    fn resolve_unit_group(&self, uuid: &str) -> Result<UnitGroup, LinkError> {
        if let Some(hit) = self.unit_groups.lock().unwrap().get(uuid) {
            return Ok(hit.clone());
        }
        let fresh = self.inner.resolve_unit_group(uuid)?;
        self.unit_groups
            .lock()
            .unwrap()
            .insert(uuid.to_string(), fresh.clone());
        Ok(fresh)
    }
}

#[test]
#[ignore = "requires JRC EF reference package on disk; set EF_REFERENCE_PACKAGE_BUNDLE env var"]
fn ef_reference_package_resolver_bundle_smoke() {
    let bundle_root = std::env::var(BUNDLE_ENV_VAR).unwrap_or_else(|_| {
        panic!(
            "{BUNDLE_ENV_VAR} not set; point it at the unpacked ILCD/ \
             subdirectory of a JRC EF reference package (see module docs)"
        )
    });
    let bundle_root = PathBuf::from(bundle_root);
    let flows_dir = bundle_root.join("flows");
    assert!(
        flows_dir.is_dir(),
        "expected flows subdirectory at {}",
        flows_dir.display()
    );

    let resolver = CachingResolver::new(DirectoryBundle::new(&bundle_root));

    let mut flow_xmls: Vec<PathBuf> = std::fs::read_dir(&flows_dir)
        .expect("read flows dir")
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension() == Some(OsStr::new("xml")))
        .collect();
    flow_xmls.sort();
    assert!(
        !flow_xmls.is_empty(),
        "no *.xml files found under {}",
        flows_dir.display()
    );

    let t_start = Instant::now();

    let mut flow_parse_ok: usize = 0;
    let mut unit_resolve_ok: usize = 0;
    let mut flow_type_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut unit_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut gap_io: usize = 0;
    let mut gap_no_derivation: usize = 0;
    let mut engine_failures: Vec<(PathBuf, String)> = Vec::new();

    let total = flow_xmls.len();
    println!("flows to resolve: {total}");
    const PROGRESS_EVERY: usize = 2_000;

    for (i, path) in flow_xmls.iter().enumerate() {
        if i > 0 && i % PROGRESS_EVERY == 0 {
            let elapsed = t_start.elapsed().as_secs_f64();
            let rate = (i as f64) / elapsed;
            let eta = ((total - i) as f64) / rate.max(0.001);
            println!(
                "  progress: {i}/{total} ({:.1}%) — {:.1}s elapsed, {:.0} flows/s, ETA ~{:.0}s",
                (i as f64) * 100.0 / (total as f64),
                elapsed,
                rate,
                eta,
            );
        }
        // UUID = filename stem, per ILCD convention.
        let Some(uuid) = path.file_stem().and_then(|s| s.to_str()) else {
            engine_failures.push((path.clone(), "non-UTF8 filename".into()));
            continue;
        };

        let flow = match resolver.resolve_flow(uuid) {
            Ok(f) => f,
            Err(e) => {
                engine_failures.push((path.clone(), format!("resolve_flow: {e}")));
                continue;
            }
        };
        flow_parse_ok += 1;
        *flow_type_counts
            .entry(format!("{:?}", flow.flow_type))
            .or_default() += 1;

        // Use the `_from_flow` variant to avoid a second resolve_flow
        // filesystem hit per flow — halves the I/O cost at 94k scale.
        match resolve_reference_unit_from_flow(&resolver, &flow) {
            Ok(ref_unit) => {
                assert!(
                    !ref_unit.unit_name.is_empty(),
                    "{uuid}: resolved ReferenceUnit with empty unit_name"
                );
                *unit_counts.entry(ref_unit.unit_name.clone()).or_default() += 1;
                unit_resolve_ok += 1;
            }
            Err(LinkError::Io { .. }) => {
                gap_io += 1;
            }
            Err(LinkError::FlowHasNoUnitDerivation { .. }) => {
                gap_no_derivation += 1;
            }
            Err(other) => {
                engine_failures.push((
                    path.clone(),
                    format!("resolve_reference_unit_from_flow: {other}"),
                ));
            }
        }
    }

    let elapsed = t_start.elapsed();

    println!("--- ef_reference_package_resolver_bundle_smoke ---");
    println!(
        "flows: {flow_parse_ok}/{} parsed; {unit_resolve_ok} fully resolved; \
         {} publisher gaps ({gap_io} I/O + {gap_no_derivation} no-derivation); \
         {} engine failures",
        flow_xmls.len(),
        gap_io + gap_no_derivation,
        engine_failures.len(),
    );
    println!(
        "elapsed: {:.1} s ({:.3} ms/flow)",
        elapsed.as_secs_f64(),
        (elapsed.as_secs_f64() * 1000.0) / (flow_xmls.len() as f64),
    );
    println!("flow type distribution:");
    for (ft, count) in &flow_type_counts {
        println!("  {ft:12} {count}");
    }
    println!("top-20 reference units:");
    let mut unit_vec: Vec<_> = unit_counts.iter().collect();
    unit_vec.sort_by(|a, b| b.1.cmp(a.1).then_with(|| a.0.cmp(b.0)));
    for (unit, count) in unit_vec.iter().take(20) {
        println!("  {unit:24} {count}");
    }
    if !engine_failures.is_empty() {
        println!("first 10 ENGINE failures (these must be zero):");
        for (path, reason) in engine_failures.iter().take(10) {
            println!("  {}: {reason}", path.display());
        }
    }

    assert!(
        engine_failures.is_empty(),
        "{} flow(s) failed with engine-level errors — see printed list above",
        engine_failures.len()
    );
}
