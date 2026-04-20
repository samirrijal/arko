//! Real-data smoke test against the USDA LCA Commons beef cattle
//! finishing bundle.
//!
//! # Why this test is `#[ignore]`-gated
//!
//! The USDA LCA Commons data is CC0 1.0 Universal and *could* be
//! committed; we keep the bundle out of the repo anyway to match the
//! ÖKOBAUDAT smoke pattern and to avoid bloat on a repo that tracks
//! code, not datasets. The bundle lives on the maintainer's disk and
//! this test only runs when the maintainer explicitly asks for it.
//!
//! # How to run locally
//!
//! ```bash
//! USDA_BEEF_BUNDLE=/path/to/USDA_Beef cargo test -p arko-io-olca-jsonld \
//!   --test beef_bundle_smoke -- --ignored --nocapture
//! ```
//!
//! # What this asserts
//!
//! Structural invariants only — the parity test (a separate file)
//! carries the numeric assertions.
//!
//! - All 5 processes parse.
//! - The beef-finishing process (`1b97b691…`) has exactly one
//!   quantitative-reference exchange and three in-bundle
//!   `defaultProvider` edges pointing at `9f9e378b…`, `ac2816ed…`, and
//!   `efa8b1d9…`.
//! - `olca_to_typed_column` succeeds on every process.
//! - Methane-biogenic is classified `FlowOrigin::NonFossil`.
//! - The zero-padded CAS `000074-82-8` is trimmed to `74-82-8`.
//! - No exchange surfaces a dangling `defaultProvider`.

use std::collections::BTreeSet;
use std::path::PathBuf;

use arko_io_ilcd_linker::{FlowOrigin, FlowType};
use arko_io_olca_jsonld::{olca_to_typed_column, OlcaBundle};

const BUNDLE_ENV_VAR: &str = "USDA_BEEF_BUNDLE";

const FINISHING: &str = "1b97b691-7c00-4150-9e97-df2020bfd203";
const VITAMIN: &str = "9f9e378b-7faa-4d4c-a419-3374b3632021";
const CALF: &str = "ac2816ed-803d-4436-92b6-2ea9cd5ce67a";
const FEED: &str = "efa8b1d9-dd4a-4d25-b595-2dad6932e428";
const PASTURE: &str = "2185d89c-d65f-4116-99ad-2ee41ba80689";

const METHANE_BIOGENIC_FLOW: &str = "57bdb443-d4a6-423d-8024-959b8261d02e";

#[test]
#[ignore = "requires USDA_BEEF_BUNDLE env var pointing at an unpacked beef bundle"]
fn beef_bundle_end_to_end_structural_smoke() {
    let bundle_root = std::env::var(BUNDLE_ENV_VAR).unwrap_or_else(|_| {
        panic!(
            "{BUNDLE_ENV_VAR} not set; point it at an unpacked USDA beef bundle \
             (see test module docs for instructions)"
        )
    });
    let bundle_root = PathBuf::from(bundle_root);

    let bundle = OlcaBundle::open(&bundle_root).expect("open bundle");
    let uuids: BTreeSet<String> = bundle.process_uuids().into_iter().collect();
    let expected: BTreeSet<String> = [FINISHING, VITAMIN, CALF, FEED, PASTURE]
        .iter()
        .map(|s| s.to_string())
        .collect();
    assert_eq!(
        uuids, expected,
        "beef bundle should have exactly five processes"
    );

    // 1. Beef-finishing DAG edges match scouting.
    let finishing = bundle.load_process(FINISHING).expect("load finishing");
    let finishing_col =
        olca_to_typed_column(&finishing, &bundle).expect("adapt finishing");
    let providers: BTreeSet<String> = finishing
        .exchanges
        .iter()
        .filter_map(|e| e.default_provider_uuid.clone())
        .collect();
    let expected_providers: BTreeSet<String> =
        [VITAMIN, CALF, FEED].iter().map(|s| s.to_string()).collect();
    assert_eq!(
        providers, expected_providers,
        "finishing should have 3 in-bundle default providers (vitamin, calf, feed)"
    );
    assert!(
        finishing_col
            .exchanges
            .iter()
            .filter(|e| e.is_reference_flow)
            .count()
            == 1
    );

    // 2. Calf DAG edges: vitamin, pasture.
    let calf = bundle.load_process(CALF).expect("load calf");
    let calf_providers: BTreeSet<String> = calf
        .exchanges
        .iter()
        .filter_map(|e| e.default_provider_uuid.clone())
        .collect();
    let expected_calf: BTreeSet<String> =
        [VITAMIN, PASTURE].iter().map(|s| s.to_string()).collect();
    assert_eq!(
        calf_providers, expected_calf,
        "calf should have 2 in-bundle default providers (vitamin, pasture)"
    );

    // 3. Leaves have no in-bundle providers.
    for leaf in [VITAMIN, FEED, PASTURE] {
        let p = bundle.load_process(leaf).expect("load leaf");
        let in_bundle = p
            .exchanges
            .iter()
            .filter_map(|e| e.default_provider_uuid.clone())
            .filter(|u| bundle.has_process(u))
            .count();
        assert_eq!(in_bundle, 0, "leaf {leaf} should have no in-bundle providers");
    }

    // 4. Every process adapts without error (no dangling providers,
    //    no cross-property violations).
    for uuid in bundle.process_uuids() {
        let p = bundle.load_process(&uuid).expect("load process");
        let col = olca_to_typed_column(&p, &bundle)
            .unwrap_or_else(|e| panic!("adapt {uuid} failed: {e}"));
        assert!(!col.exchanges.is_empty(), "{uuid} produced empty column");
    }

    // 5. Methane-biogenic flow carries zero-padded CAS → trimmed, and
    //    origin classifies as NonFossil.
    let meth = bundle.load_flow(METHANE_BIOGENIC_FLOW).expect("load meth");
    assert_eq!(
        meth.cas.as_deref(),
        Some("74-82-8"),
        "CAS should be trimmed from zero-padded form"
    );
    let meth_exchange = finishing_col
        .exchanges
        .iter()
        .find(|e| e.flow_uuid == METHANE_BIOGENIC_FLOW)
        .expect("methane-biogenic exchange present on finishing column");
    assert_eq!(meth_exchange.origin, FlowOrigin::NonFossil);
    assert_eq!(meth_exchange.flow_type, FlowType::Elementary);
}
