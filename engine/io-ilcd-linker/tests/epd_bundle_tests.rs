//! Integration tests against a synthetic minimal ILCD+EPD v1.2 bundle.
//!
//! The bundle under `tests/fixtures/epd_minimal_bundle/` is hand-crafted
//! (not a slice of ÖKOBAUDAT, for licensing reasons — ÖKOBAUDAT is
//! CC-BY-ND-3.0-DE) so the tests exercise the v1.2 extension paths with
//! predictable values:
//!
//! - Reference-flow exchange omits `<exchangeDirection>` → exercises
//!   `ParseWarning::InferredDirection`.
//! - Indicator flow (PERE) carries only `<epd:amount>` module entries
//!   including a negative Module D → exercises EN 15804+A2 sign pass-
//!   through.
//! - Indicator flow declares Mass in its own flow chain, but the process
//!   exchange inlines `<epd:referenceToUnitGroupDataSet>` → Energy →
//!   exercises inline-priority AND `BridgeWarning::UnitGroupDisagreement`.

use arko_io_ilcd::{parse_process, Direction, ParseWarning};
use arko_io_ilcd_linker::{
    build_typed_column, BridgeWarning, DirectoryBundle, FlowType, UnitResolutionSource,
};

const BUNDLE_ROOT: &str = "tests/fixtures/epd_minimal_bundle";
const CONCRETE_FLOW_UUID: &str = "aaaaaaaa-0000-0000-0000-000000000001";
const PERE_FLOW_UUID: &str = "aaaaaaaa-0000-0000-0000-000000000002";
const MASS_UNIT_GROUP_UUID: &str = "aaaaaaaa-0000-0000-0000-000000000100";
const ENERGY_UNIT_GROUP_UUID: &str = "aaaaaaaa-0000-0000-0000-000000000101";
const PROCESS_UUID: &str = "aaaaaaaa-0000-0000-0000-000000000500";

fn load_epd_process() -> arko_io_ilcd::ProcessDataset {
    let path = format!("{BUNDLE_ROOT}/processes/{PROCESS_UUID}.xml");
    let xml = std::fs::read_to_string(&path).expect("epd process fixture readable");
    parse_process(&xml).expect("epd process fixture parses")
}

#[test]
fn parse_infers_direction_on_reference_exchange() {
    let dataset = load_epd_process();

    // Exactly one inferred-direction warning, on the reference flow.
    let inferred: Vec<&ParseWarning> = dataset
        .warnings
        .iter()
        .filter(|w| {
            matches!(
                w,
                ParseWarning::InferredDirection {
                    is_reference_flow: true,
                    ..
                }
            )
        })
        .collect();
    assert_eq!(
        inferred.len(),
        1,
        "expected exactly one reference-flow inferred-direction warning, got {:?}",
        dataset.warnings
    );

    let ref_exchange = dataset
        .exchanges
        .iter()
        .find(|ex| ex.data_set_internal_id == 0)
        .expect("reference exchange present");
    assert!(ref_exchange.exchange_direction_inferred);
    assert_eq!(ref_exchange.direction, Direction::Output);

    let pere_exchange = dataset
        .exchanges
        .iter()
        .find(|ex| ex.data_set_internal_id == 1)
        .expect("PERE exchange present");
    assert!(!pere_exchange.exchange_direction_inferred);
    assert_eq!(pere_exchange.direction, Direction::Input);
}

#[test]
fn parse_captures_epd_modules_with_negative_module_d() {
    let dataset = load_epd_process();

    let pere = dataset
        .exchanges
        .iter()
        .find(|ex| ex.data_set_internal_id == 1)
        .expect("PERE exchange present");

    assert!(
        !pere.epd_modules.is_empty(),
        "PERE exchange must carry epd_modules"
    );

    let a1_a3 = pere
        .epd_modules
        .iter()
        .find(|m| m.module == "A1-A3")
        .expect("A1-A3 module present");
    assert!((a1_a3.amount - 7.5).abs() < f64::EPSILON);
    assert!(a1_a3.scenario.is_none());

    let module_d = pere
        .epd_modules
        .iter()
        .find(|m| m.module == "D")
        .expect("Module D present");
    assert_eq!(module_d.scenario.as_deref(), Some("Recycled"));
    assert!(
        module_d.amount < 0.0,
        "Module D must be negative per EN 15804+A2 (benefits beyond system boundary), got {}",
        module_d.amount
    );
    assert!((module_d.amount - -0.48).abs() < f64::EPSILON);
}

#[test]
fn parse_records_inline_unit_group_on_pere() {
    let dataset = load_epd_process();
    let pere = dataset
        .exchanges
        .iter()
        .find(|ex| ex.data_set_internal_id == 1)
        .expect("PERE exchange present");

    assert_eq!(
        pere.epd_unit_group_uuid.as_deref(),
        Some(ENERGY_UNIT_GROUP_UUID),
        "PERE exchange must carry inline unit-group ref"
    );

    let concrete = dataset
        .exchanges
        .iter()
        .find(|ex| ex.data_set_internal_id == 0)
        .expect("concrete exchange present");
    assert!(concrete.epd_unit_group_uuid.is_none());
    assert!(concrete.epd_modules.is_empty());
}

#[test]
fn bridge_resolves_pere_via_inline_energy_unit_group() {
    let bundle = DirectoryBundle::new(BUNDLE_ROOT);
    let dataset = load_epd_process();
    let column = build_typed_column(&dataset, &bundle).expect("column builds");

    assert_eq!(column.process_uuid, PROCESS_UUID);
    assert_eq!(column.exchanges.len(), 2);

    let pere = column
        .exchanges
        .iter()
        .find(|ex| ex.data_set_internal_id == 1)
        .expect("PERE in column");
    assert_eq!(pere.flow_uuid, PERE_FLOW_UUID);
    assert_eq!(pere.flow_type, FlowType::Elementary);
    assert_eq!(pere.unit_source, UnitResolutionSource::EpdInline);
    assert_eq!(pere.reference_unit.unit_group_uuid, ENERGY_UNIT_GROUP_UUID);
    assert_eq!(pere.reference_unit.unit_name, "MJ");
    assert_eq!(pere.epd_modules.len(), 8);
}

#[test]
fn bridge_resolves_concrete_via_flow_chain_kg() {
    let bundle = DirectoryBundle::new(BUNDLE_ROOT);
    let dataset = load_epd_process();
    let column = build_typed_column(&dataset, &bundle).expect("column builds");

    let concrete = column
        .exchanges
        .iter()
        .find(|ex| ex.data_set_internal_id == 0)
        .expect("concrete in column");
    assert_eq!(concrete.flow_uuid, CONCRETE_FLOW_UUID);
    assert_eq!(concrete.flow_type, FlowType::Product);
    assert_eq!(concrete.unit_source, UnitResolutionSource::FlowChain);
    assert_eq!(
        concrete.reference_unit.unit_group_uuid,
        MASS_UNIT_GROUP_UUID
    );
    assert_eq!(concrete.reference_unit.unit_name, "kg");
    assert!(concrete.is_reference_flow);
    assert!(concrete.epd_modules.is_empty());
}

#[test]
fn bridge_emits_disagreement_warning_for_pere() {
    let bundle = DirectoryBundle::new(BUNDLE_ROOT);
    let dataset = load_epd_process();
    let column = build_typed_column(&dataset, &bundle).expect("column builds");

    let disagreements: Vec<&BridgeWarning> = column
        .bridge_warnings
        .iter()
        .filter(|w| matches!(w, BridgeWarning::UnitGroupDisagreement { .. }))
        .collect();
    assert_eq!(
        disagreements.len(),
        1,
        "expected exactly one UnitGroupDisagreement, got {:?}",
        column.bridge_warnings
    );

    let BridgeWarning::UnitGroupDisagreement {
        data_set_internal_id,
        flow_uuid,
        inline_unit_group_uuid,
        chain_unit_group_uuid,
    } = disagreements[0]
    else {
        unreachable!("filtered above");
    };
    assert_eq!(*data_set_internal_id, 1);
    assert_eq!(flow_uuid, PERE_FLOW_UUID);
    assert_eq!(inline_unit_group_uuid, ENERGY_UNIT_GROUP_UUID);
    assert_eq!(chain_unit_group_uuid, MASS_UNIT_GROUP_UUID);
}

#[test]
fn bridge_forwards_parse_warnings_to_column() {
    let bundle = DirectoryBundle::new(BUNDLE_ROOT);
    let dataset = load_epd_process();
    let column = build_typed_column(&dataset, &bundle).expect("column builds");

    // Parse warnings copied verbatim.
    assert_eq!(column.parse_warnings, dataset.warnings);
    assert!(
        column.parse_warnings.iter().any(|w| matches!(
            w,
            ParseWarning::InferredDirection {
                is_reference_flow: true,
                ..
            }
        )),
        "column should carry the reference-flow inferred-direction warning"
    );
}
