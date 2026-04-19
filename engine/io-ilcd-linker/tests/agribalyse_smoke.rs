//! Real-data smoke test against a downloaded Agribalyse 3.1 bundle.
//!
//! # Why this is a Week-5 generalisation test
//!
//! Week 4 proved the `arko-io-ilcd` → `arko-io-ilcd-linker` pipeline
//! reads ÖKOBAUDAT 2024-I. The Phase-1 exit criteria want "three free
//! databases importable", but the real question is softer: **is our
//! ILCD reader general, or is it secretly tuned to ÖKOBAUDAT idioms?**
//!
//! Agribalyse (INRAE / ADEME, French agriculture) is the cleanest
//! second test because it's also ILCD but structurally lighter than
//! ÖKOBAUDAT — plain ILCD, not ILCD+EPD v1.2; no stage stratification;
//! no inline unit-group overrides. A reader that passes ÖKOBAUDAT but
//! trips on Agribalyse has hidden coupling. A reader that passes both
//! is actually general.
//!
//! # How to run locally
//!
//! 1. Download **Agribalyse 3.1.1 ILCD bundle** from
//!    <https://doc.agribalyse.fr/documentation-en/agribalyse-data/data-access>
//!    (INRAE-hosted, free, email-gated). Accept CC-BY-4.0 attribution
//!    terms.
//! 2. Unzip into the standard layout (`<root>/processes/*.xml`,
//!    `flows/`, `flowproperties/`, `unitgroups/`). Convention:
//!    `engine/io-ilcd-linker/tests/fixtures/external/agribalyse/`
//!    (already gitignored — we do not redistribute).
//! 3. Point the test at it:
//!
//!    ```bash
//!    AGRIBALYSE_BUNDLE=/path/to/bundle cargo test -p arko-io-ilcd-linker \
//!      --test agribalyse_smoke -- --ignored --nocapture
//!    ```
//!
//! # What this asserts
//!
//! Same invariants as `oekobaudat_smoke`: every process parses, every
//! linkable process pipelines through `build_typed_column` with zero
//! engine-level errors. Bundle data gaps (missing flow / flowproperty
//! / unitgroup XMLs) are tolerated — Agribalyse, unlike ÖKOBAUDAT,
//! typically ships all cross-references it declares, so a clean run
//! should report near-zero data gaps. If gaps are non-trivial,
//! something about the reader is likely wrong, not the bundle.
//!
//! The summary print-out is the generalisation signal: flow-type
//! distribution and reference-unit distribution should look
//! reasonable for agricultural data (MJ, kg, ha, m³, and kg-of-water
//! dominant; no `Gebäudetechnik`-specific oddities).

use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::path::PathBuf;

use arko_io_ilcd::parse_process;
use arko_io_ilcd_linker::{build_typed_column, DirectoryBundle};

const BUNDLE_ENV_VAR: &str = "AGRIBALYSE_BUNDLE";

#[test]
#[ignore = "requires Agribalyse bundle on disk; set AGRIBALYSE_BUNDLE env var"]
fn agribalyse_full_bundle_smoke() {
    let bundle_root = std::env::var(BUNDLE_ENV_VAR).unwrap_or_else(|_| {
        panic!(
            "{BUNDLE_ENV_VAR} not set; point it at an unpacked Agribalyse bundle \
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

    println!("--- agribalyse_full_bundle_smoke ---");
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
