//! Real-data smoke test against a downloaded EU JRC **Environmental
//! Footprint reference package** (EF 3.x).
//!
//! # Why this is a Week-5 generalisation test
//!
//! The EF reference packages are the JRC's blessed background data
//! for PEF (Product Environmental Footprint) and EN 15804+A2
//! downstream data. Every European compliance story eventually names
//! them. If our ILCD reader passes ÖKOBAUDAT and Agribalyse but trips
//! on EF, we have a problem — EF is closer to the canonical ILCD
//! spec than either of the other two.
//!
//! Unlike ÖKOBAUDAT (ILCD+EPD v1.2 superset) and Agribalyse (plain
//! ILCD, agriculture focus), EF ships LCIA method datasets alongside
//! the process / flow / unit-group files. At v0.1 we deliberately
//! ignore `<LCIAMethodDataSet>` (it's the `arko-methods` crate's
//! concern, not the linker's). This test asserts the **process
//! pipeline** works on EF; method ingestion lands in Phase 1 Week 6.
//!
//! # How to run locally
//!
//! 1. Download the current **EF reference package** from
//!    <https://eplca.jrc.ec.europa.eu/EnvironmentalFootprint.html>
//!    (EU JRC, free, registration-gated). Pick the ILCD-format zip,
//!    e.g. `EF_reference_package_3.1` or newer.
//! 2. Unzip into the standard ILCD layout (`<root>/processes/*.xml`,
//!    `flows/`, `flowproperties/`, `unitgroups/`). Convention:
//!    `engine/io-ilcd-linker/tests/fixtures/external/ef_reference/`
//!    (already gitignored — we do not redistribute).
//! 3. Point the test at it:
//!
//!    ```bash
//!    EF_REFERENCE_BUNDLE=/path/to/bundle cargo test -p arko-io-ilcd-linker \
//!      --test ef_reference_smoke -- --ignored --nocapture
//!    ```
//!
//! # What this asserts
//!
//! Same invariants as the ÖKOBAUDAT and Agribalyse smokes: every
//! process parses cleanly, every linkable process pipelines through
//! `build_typed_column` with zero engine-level errors, and every
//! exchange carries a non-empty reference unit. Bundle gaps
//! (missing XML cross-refs) are tolerated.
//!
//! Expectation: EF reference package is the cleanest of the three
//! (it's the spec-author's own bundle). A non-trivial gap count
//! here, or any engine failure, almost certainly indicates a real
//! reader bug rather than a publisher issue.

use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::path::PathBuf;

use arko_io_ilcd::parse_process;
use arko_io_ilcd_linker::{build_typed_column, DirectoryBundle};

const BUNDLE_ENV_VAR: &str = "EF_REFERENCE_BUNDLE";

#[test]
#[ignore = "requires EF reference package on disk; set EF_REFERENCE_BUNDLE env var"]
fn ef_reference_full_bundle_smoke() {
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

    let resolver = DirectoryBundle::new(&bundle_root);

    let mut process_xmls: Vec<PathBuf> = std::fs::read_dir(&processes_dir)
        .expect("read processes dir")
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension() == Some(OsStr::new("xml")))
        .collect();
    process_xmls.sort();
    assert!(
        !process_xmls.is_empty(),
        "no *.xml files found under {}",
        processes_dir.display()
    );

    let mut processes_ok: usize = 0;
    let mut exchanges_total: usize = 0;
    let mut unit_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut flow_type_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut engine_failures: Vec<(PathBuf, String)> = Vec::new();
    let mut data_gap_failures: Vec<(PathBuf, String)> = Vec::new();

    for path in &process_xmls {
        let xml = match std::fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) => {
                engine_failures.push((path.clone(), format!("read: {e}")));
                continue;
            }
        };
        let dataset = match parse_process(&xml) {
            Ok(d) => d,
            Err(e) => {
                engine_failures.push((path.clone(), format!("parse_process: {e}")));
                continue;
            }
        };
        let column = match build_typed_column(&dataset, &resolver) {
            Ok(c) => c,
            Err(e) => {
                let msg = format!("build_typed_column: {e}");
                if matches!(
                    e,
                    arko_io_ilcd_linker::LinkError::Io { .. }
                        | arko_io_ilcd_linker::LinkError::FlowHasNoUnitDerivation { .. }
                ) {
                    data_gap_failures.push((path.clone(), msg));
                } else {
                    engine_failures.push((path.clone(), msg));
                }
                continue;
            }
        };

        assert!(
            !column.exchanges.is_empty(),
            "empty exchange list in {} ({})",
            column.process_uuid,
            path.display()
        );

        let ref_count = column
            .exchanges
            .iter()
            .filter(|e| e.is_reference_flow)
            .count();
        assert_eq!(
            ref_count, 1,
            "{}: expected exactly 1 reference exchange, got {ref_count}",
            column.process_uuid
        );

        for ex in &column.exchanges {
            assert!(
                !ex.reference_unit.unit_name.is_empty(),
                "{}: empty unit_name for exchange internal_id {}",
                column.process_uuid,
                ex.data_set_internal_id
            );
            *unit_counts
                .entry(ex.reference_unit.unit_name.clone())
                .or_default() += 1;
            *flow_type_counts
                .entry(format!("{:?}", ex.flow_type))
                .or_default() += 1;
            exchanges_total += 1;
        }
        processes_ok += 1;
    }

    println!("--- ef_reference_full_bundle_smoke ---");
    println!(
        "processes: {processes_ok}/{} ok, {} engine failures, {} bundle data gaps",
        process_xmls.len(),
        engine_failures.len(),
        data_gap_failures.len(),
    );
    println!("exchanges resolved: {exchanges_total}");
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
    if !data_gap_failures.is_empty() {
        println!("first 5 bundle data gaps (tolerated; publisher-side):");
        for (path, reason) in data_gap_failures.iter().take(5) {
            println!("  {}: {reason}", path.display());
        }
    }

    assert!(
        engine_failures.is_empty(),
        "{} process(es) failed with engine-level errors — see printed list above",
        engine_failures.len()
    );
}
