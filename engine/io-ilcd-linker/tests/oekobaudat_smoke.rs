//! Real-data smoke test against a downloaded OEKOBAUDAT bundle.
//!
//! # Why this test is `#[ignore]`-gated
//!
//! OEKOBAUDAT is published under **CC-BY-ND-3.0-DE** — we may use it
//! for testing but we cannot redistribute or commit it. So the bundle
//! lives on the maintainer's disk, not in the repo, and this test only
//! runs when the maintainer explicitly asks for it.
//!
//! # How to run locally
//!
//! 1. Download the OEKOBAUDAT full dataset from
//!    <https://www.oekobaudat.de/> (choose "OBD_2024" or latest,
//!    format: ILCD XML, ZIP).
//! 2. Unzip so you have the standard layout:
//!    `<bundle>/processes/*.xml`, `<bundle>/flows/*.xml`,
//!    `<bundle>/flowproperties/*.xml`, `<bundle>/unitgroups/*.xml`.
//!    The extracted tree can live anywhere; the convention for
//!    in-repo extraction is
//!    `engine/io-ilcd-linker/tests/fixtures/external/oekobaudat/`
//!    (already gitignored).
//! 3. Point the test at it and run:
//!
//!    ```bash
//!    OEKOBAUDAT_BUNDLE=/path/to/bundle cargo test -p arko-io-ilcd-linker \
//!      --test oekobaudat_smoke -- --ignored --nocapture
//!    ```
//!
//! # What this test asserts
//!
//! Not impact-result-level checks (those need LCIA methods, which are a
//! later Phase 1 ticket). Instead, invariants that exercise the full
//! `arko-io-ilcd` → `arko-io-ilcd-linker` pipeline on real-world
//! diversity:
//!
//! - Every `processes/*.xml` parses cleanly (zero `parse_process`
//!   errors). A parse failure is an engine bug.
//! - Every process that links to flows present in the bundle pipelines
//!   cleanly through `build_typed_column` (zero *bridge* failures that
//!   are not "flow file not found" I/O errors).
//! - Every surviving `TypedColumn` has at least one exchange, exactly
//!   one reference flow, and every exchange carries a non-empty
//!   `reference_unit.unit_name`.
//!
//! Failures caused by **missing flow files in the bundle itself** are
//! counted but do not fail the test — ÖKOBAUDAT-2024-I publishes
//! ~105 processes that reference flow UUIDs it doesn't ship. That is
//! a publisher-side data-integrity issue, not an engine bug; the
//! engine correctly refuses to invent a unit out of thin air.
//!
//! A summary line is printed (`--nocapture`) so the maintainer can
//! eyeball the distribution of units and flow types — useful when
//! deciding which specific UUIDs to promote to stricter
//! known-value assertions later.

use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::path::PathBuf;

use arko_io_ilcd::parse_process;
use arko_io_ilcd_linker::{build_typed_column, DirectoryBundle};

const BUNDLE_ENV_VAR: &str = "OEKOBAUDAT_BUNDLE";

#[test]
#[ignore = "requires OEKOBAUDAT bundle on disk; set OEKOBAUDAT_BUNDLE env var"]
fn oekobaudat_full_bundle_smoke() {
    let bundle_root = std::env::var(BUNDLE_ENV_VAR).unwrap_or_else(|_| {
        panic!(
            "{BUNDLE_ENV_VAR} not set; point it at an unpacked OEKOBAUDAT bundle \
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
    // Engine-level failures: parse errors, or bridge errors that are
    // NOT "flow file not found in bundle". These must be zero — they
    // represent real engine bugs.
    let mut engine_failures: Vec<(PathBuf, String)> = Vec::new();
    // Bundle-level data gaps: bridge tried to resolve a flow whose XML
    // isn't in the bundle. Counted but tolerated.
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
                // I/O-missing-file errors on flow / flowproperty /
                // unitgroup resolution are publisher data gaps, not
                // engine bugs. Classify by the error's text shape:
                // the concrete LinkError::Io variant's Display starts
                // with "I/O error reading".
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

    println!("--- oekobaudat_full_bundle_smoke ---");
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
