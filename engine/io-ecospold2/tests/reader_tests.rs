//! Integration tests for the ecospold2 reader.

use arko_io_ecospold2::{parse_dataset, Direction};

const STEEL_XML: &str = include_str!("fixtures/steel.xml");

#[test]
fn parses_steel_fixture() {
    let ds = parse_dataset(STEEL_XML).expect("steel.xml should parse");

    assert_eq!(ds.activity.id, "a0000000-0000-0000-0000-000000000001");
    assert_eq!(
        ds.activity.activity_name_id,
        "b0000000-0000-0000-0000-000000000001"
    );
    assert_eq!(ds.activity.name, "steel production, unalloyed");
    assert_eq!(ds.activity.activity_type, 1);
    assert!(ds.activity.special_activity_type.is_none());

    let geo = ds.geography.expect("geography present");
    assert_eq!(geo.short_name, "GLO");

    assert_eq!(ds.intermediate_exchanges.len(), 2);
    assert_eq!(ds.elementary_exchanges.len(), 1);
}

#[test]
fn identifies_reference_product() {
    let ds = parse_dataset(STEEL_XML).unwrap();

    let ref_products: Vec<_> = ds
        .intermediate_exchanges
        .iter()
        .filter(|e| e.is_reference_product())
        .collect();
    assert_eq!(ref_products.len(), 1);

    let rp = ref_products[0];
    assert_eq!(rp.name, "steel, unalloyed");
    assert_eq!(rp.amount, 1.0);
    assert_eq!(rp.unit_name, "kg");
    assert!(rp.direction.is_output());
    assert_eq!(rp.direction.group(), 0);
}

#[test]
fn identifies_technosphere_input() {
    let ds = parse_dataset(STEEL_XML).unwrap();

    let inputs: Vec<_> = ds
        .intermediate_exchanges
        .iter()
        .filter(|e| e.direction.is_input())
        .collect();
    assert_eq!(inputs.len(), 1);

    let inp = inputs[0];
    assert_eq!(inp.name, "electricity, medium voltage");
    assert_eq!(inp.amount, 0.5);
    assert_eq!(inp.unit_name, "MJ");
    assert_eq!(
        inp.activity_link_id.as_deref(),
        Some("a0000000-0000-0000-0000-000000000002"),
    );
    assert_eq!(inp.direction, Direction::Input { group: 5 });
}

#[test]
fn parses_elementary_exchange_compartment() {
    let ds = parse_dataset(STEEL_XML).unwrap();
    let co2 = &ds.elementary_exchanges[0];

    assert_eq!(co2.name, "Carbon dioxide, fossil");
    assert_eq!(co2.amount, 2.0);
    assert_eq!(co2.unit_name, "kg");
    assert_eq!(co2.cas_number.as_deref(), Some("124-38-9"));
    assert_eq!(co2.compartment, "air");
    assert_eq!(
        co2.subcompartment.as_deref(),
        Some("urban air close to ground"),
    );
    assert_eq!(co2.direction, Direction::Output { group: 4 });
}

#[test]
fn roundtrips_through_json() {
    // Anything we can parse should also serialize/deserialize via serde,
    // so downstream tools can cache parsed ecospold2 as JSON blobs.
    let ds = parse_dataset(STEEL_XML).unwrap();
    let json = serde_json::to_string(&ds).unwrap();
    let back: arko_io_ecospold2::ActivityDataset = serde_json::from_str(&json).unwrap();
    assert_eq!(ds, back);
}

#[test]
fn rejects_non_ecospold_root() {
    let xml = r#"<?xml version="1.0"?><notEcoSpold/>"#;
    let err = parse_dataset(xml).unwrap_err();
    assert!(matches!(
        err,
        arko_io_ecospold2::Ecospold2Error::UnexpectedRoot(_),
    ));
}

#[test]
fn rejects_missing_activity_dataset() {
    let xml = r#"<?xml version="1.0"?>
    <ecoSpold xmlns="http://www.EcoInvent.org/EcoSpold02">
      <somethingElse/>
    </ecoSpold>"#;
    let err = parse_dataset(xml).unwrap_err();
    assert!(matches!(
        err,
        arko_io_ecospold2::Ecospold2Error::MissingElement("activityDataset"),
    ));
}

#[test]
fn rejects_exchange_with_both_directions() {
    let xml = r#"<?xml version="1.0"?>
    <ecoSpold xmlns="http://www.EcoInvent.org/EcoSpold02">
      <activityDataset>
        <activityDescription>
          <activity id="a" activityNameId="b" activityType="1">
            <activityName xml:lang="en">x</activityName>
          </activity>
        </activityDescription>
        <flowData>
          <intermediateExchange id="bad"
                                intermediateExchangeId="p"
                                amount="1.0"
                                unitName="kg">
            <name xml:lang="en">ambiguous</name>
            <inputGroup>5</inputGroup>
            <outputGroup>0</outputGroup>
          </intermediateExchange>
        </flowData>
      </activityDataset>
    </ecoSpold>"#;
    let err = parse_dataset(xml).unwrap_err();
    assert!(matches!(
        err,
        arko_io_ecospold2::Ecospold2Error::DirectionAmbiguous { .. },
    ));
}

#[test]
fn accepts_child_activity_dataset_envelope() {
    let xml = r#"<?xml version="1.0"?>
    <ecoSpold xmlns="http://www.EcoInvent.org/EcoSpold02">
      <childActivityDataset>
        <activityDescription>
          <activity id="a" activityNameId="b" activityType="1">
            <activityName xml:lang="en">child</activityName>
          </activity>
        </activityDescription>
        <flowData/>
      </childActivityDataset>
    </ecoSpold>"#;
    let ds = parse_dataset(xml).expect("childActivityDataset envelope must parse");
    assert_eq!(ds.activity.name, "child");
}
