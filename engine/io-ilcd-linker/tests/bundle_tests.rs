//! Integration tests against a synthetic minimal ILCD bundle.
//!
//! The bundle under `tests/fixtures/minimal_bundle/` is hand-crafted
//! (not a slice of a real dataset) so the test exercises every link in
//! the chain with predictable values: a fossil-CO2 flow whose reference
//! property is Mass and whose reference unit is kg.

use arko_io_ilcd_linker::{resolve_reference_unit, DirectoryBundle, FlowType, LinkResolver};

const BUNDLE_ROOT: &str = "tests/fixtures/minimal_bundle";
const FLOW_UUID: &str = "00000000-0000-0000-0000-000000000001";
const FLOW_PROPERTY_UUID: &str = "00000000-0000-0000-0000-000000000010";
const UNIT_GROUP_UUID: &str = "00000000-0000-0000-0000-000000000100";

#[test]
fn directory_bundle_resolves_flow() {
    let bundle = DirectoryBundle::new(BUNDLE_ROOT);
    let flow = bundle.resolve_flow(FLOW_UUID).expect("flow resolves");

    assert_eq!(flow.uuid, FLOW_UUID);
    assert_eq!(flow.base_name, "Carbon dioxide, fossil");
    assert_eq!(flow.flow_type, FlowType::Elementary);
    assert_eq!(flow.cas.as_deref(), Some("124-38-9"));
    assert_eq!(flow.reference_flow_property_id, 0);

    let ref_fp = flow
        .reference_flow_property()
        .expect("reference flow property present");
    assert_eq!(ref_fp.flow_property_uuid, FLOW_PROPERTY_UUID);
}

#[test]
fn directory_bundle_resolves_flow_property() {
    let bundle = DirectoryBundle::new(BUNDLE_ROOT);
    let fp = bundle
        .resolve_flow_property(FLOW_PROPERTY_UUID)
        .expect("flow property resolves");

    assert_eq!(fp.uuid, FLOW_PROPERTY_UUID);
    assert_eq!(fp.base_name, "Mass");
    assert_eq!(fp.reference_unit_group_uuid, UNIT_GROUP_UUID);
}

#[test]
fn directory_bundle_resolves_unit_group() {
    let bundle = DirectoryBundle::new(BUNDLE_ROOT);
    let ug = bundle
        .resolve_unit_group(UNIT_GROUP_UUID)
        .expect("unit group resolves");

    assert_eq!(ug.uuid, UNIT_GROUP_UUID);
    assert_eq!(ug.base_name, "Units of mass");
    assert_eq!(ug.reference_unit_id, 0);
    assert_eq!(ug.units.len(), 2);

    let kg = ug.reference_unit().expect("reference unit present");
    assert_eq!(kg.name, "kg");
    assert_eq!(kg.internal_id, 0);
}

#[test]
fn resolve_reference_unit_walks_the_chain() {
    let bundle = DirectoryBundle::new(BUNDLE_ROOT);
    let ru = resolve_reference_unit(&bundle, FLOW_UUID).expect("chain resolves");

    assert_eq!(ru.flow_uuid, FLOW_UUID);
    assert_eq!(ru.flow_name, "Carbon dioxide, fossil");
    assert_eq!(ru.flow_property_uuid, FLOW_PROPERTY_UUID);
    assert_eq!(ru.flow_property_name, "Mass");
    assert_eq!(ru.unit_group_uuid, UNIT_GROUP_UUID);
    assert_eq!(ru.unit_group_name, "Units of mass");
    assert_eq!(ru.unit_name, "kg");
}

#[test]
fn missing_flow_file_is_io_error() {
    let bundle = DirectoryBundle::new(BUNDLE_ROOT);
    let result = bundle.resolve_flow("deadbeef-0000-0000-0000-000000000000");
    assert!(
        matches!(result, Err(arko_io_ilcd_linker::LinkError::Io { .. })),
        "expected Io error, got {result:?}",
    );
}
