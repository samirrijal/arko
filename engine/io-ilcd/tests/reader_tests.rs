//! Integration tests for the ILCD reader.

use arko_io_ilcd::{parse_process, Direction, IlcdError};

const CEMENT_XML: &str = include_str!("fixtures/cement_es.xml");

#[test]
fn parses_cement_fixture() {
    let ds = parse_process(CEMENT_XML).expect("cement_es.xml should parse");

    assert_eq!(
        ds.process_information.uuid,
        "11111111-2222-3333-4444-555555555555"
    );
    assert_eq!(
        ds.process_information.base_name,
        "Portland cement, CEM I"
    );
    assert_eq!(
        ds.process_information.treatment_standards_routes.as_deref(),
        Some("average Spanish plant, dry process")
    );
    assert_eq!(ds.process_information.location.as_deref(), Some("ES"));
    assert_eq!(ds.process_information.reference_year, Some(2023));

    assert_eq!(ds.quantitative_reference.reference_to_reference_flow, 1);
    assert_eq!(
        ds.quantitative_reference.r#type.as_deref(),
        Some("Reference flow(s)")
    );

    assert_eq!(ds.exchanges.len(), 3);
}

#[test]
fn resolves_reference_exchange() {
    let ds = parse_process(CEMENT_XML).unwrap();
    let r = ds.reference_exchange().expect("reference exchange present");
    assert_eq!(r.data_set_internal_id, 1);
    assert_eq!(r.flow_uuid, "aaaaaaaa-0000-0000-0000-000000000001");
    assert_eq!(
        r.flow_short_description.as_deref(),
        Some("Portland cement, CEM I")
    );
    assert_eq!(r.direction, Direction::Output);
    assert_eq!(r.mean_amount, 1.0);
    assert_eq!(r.resulting_amount, 1.0);
}

#[test]
fn parses_input_exchange_with_parameter_binding() {
    let ds = parse_process(CEMENT_XML).unwrap();
    let elec = ds
        .exchanges
        .iter()
        .find(|e| e.data_set_internal_id == 2)
        .unwrap();
    assert_eq!(elec.direction, Direction::Input);
    assert_eq!(elec.mean_amount, 0.110);
    assert_eq!(
        elec.reference_to_variable.as_deref(),
        Some("elec_intensity")
    );
    assert_eq!(elec.data_derivation_type_status.as_deref(), Some("Calculated"));
    assert_eq!(
        elec.flow_uri.as_deref(),
        Some("../flows/aaaaaaaa-0000-0000-0000-000000000002.xml")
    );
}

#[test]
fn parses_elementary_emission_exchange() {
    let ds = parse_process(CEMENT_XML).unwrap();
    let co2 = ds
        .exchanges
        .iter()
        .find(|e| e.data_set_internal_id == 3)
        .unwrap();
    assert_eq!(co2.direction, Direction::Output);
    assert_eq!(co2.mean_amount, 0.85);
    assert_eq!(
        co2.flow_short_description.as_deref(),
        Some("Carbon dioxide, fossil")
    );
}

#[test]
fn roundtrips_through_json() {
    let ds = parse_process(CEMENT_XML).unwrap();
    let json = serde_json::to_string(&ds).unwrap();
    let back: arko_io_ilcd::ProcessDataset = serde_json::from_str(&json).unwrap();
    assert_eq!(ds, back);
}

#[test]
fn rejects_non_process_root() {
    let xml = r#"<?xml version="1.0"?><notProcessDataSet/>"#;
    let err = parse_process(xml).unwrap_err();
    assert!(matches!(err, IlcdError::UnexpectedRoot(_)));
}

#[test]
fn rejects_missing_uuid() {
    let xml = r#"<?xml version="1.0"?>
    <processDataSet xmlns="http://lca.jrc.it/ILCD/Process"
                    xmlns:common="http://lca.jrc.it/ILCD/Common">
      <processInformation>
        <dataSetInformation>
          <name><baseName xml:lang="en">x</baseName></name>
        </dataSetInformation>
        <quantitativeReference type="Reference flow(s)">
          <referenceToReferenceFlow>1</referenceToReferenceFlow>
        </quantitativeReference>
      </processInformation>
      <exchanges>
        <exchange dataSetInternalID="1">
          <referenceToFlowDataSet refObjectId="00000000-0000-0000-0000-000000000001"/>
          <exchangeDirection>Output</exchangeDirection>
          <meanAmount>1.0</meanAmount>
        </exchange>
      </exchanges>
    </processDataSet>"#;
    let err = parse_process(xml).unwrap_err();
    assert!(matches!(err, IlcdError::MissingElement("UUID")));
}

#[test]
fn rejects_dangling_reference_flow() {
    let xml = r#"<?xml version="1.0"?>
    <processDataSet xmlns="http://lca.jrc.it/ILCD/Process"
                    xmlns:common="http://lca.jrc.it/ILCD/Common">
      <processInformation>
        <dataSetInformation>
          <common:UUID>22222222-0000-0000-0000-000000000001</common:UUID>
          <name><baseName xml:lang="en">x</baseName></name>
        </dataSetInformation>
        <quantitativeReference type="Reference flow(s)">
          <referenceToReferenceFlow>42</referenceToReferenceFlow>
        </quantitativeReference>
      </processInformation>
      <exchanges>
        <exchange dataSetInternalID="1">
          <referenceToFlowDataSet refObjectId="00000000-0000-0000-0000-000000000001"/>
          <exchangeDirection>Output</exchangeDirection>
          <meanAmount>1.0</meanAmount>
        </exchange>
      </exchanges>
    </processDataSet>"#;
    let err = parse_process(xml).unwrap_err();
    assert!(matches!(err, IlcdError::MissingReferenceFlow { id: 42 }));
}

#[test]
fn rejects_unknown_exchange_direction() {
    let xml = r#"<?xml version="1.0"?>
    <processDataSet xmlns="http://lca.jrc.it/ILCD/Process"
                    xmlns:common="http://lca.jrc.it/ILCD/Common">
      <processInformation>
        <dataSetInformation>
          <common:UUID>33333333-0000-0000-0000-000000000001</common:UUID>
          <name><baseName xml:lang="en">x</baseName></name>
        </dataSetInformation>
        <quantitativeReference type="Reference flow(s)">
          <referenceToReferenceFlow>1</referenceToReferenceFlow>
        </quantitativeReference>
      </processInformation>
      <exchanges>
        <exchange dataSetInternalID="1">
          <referenceToFlowDataSet refObjectId="00000000-0000-0000-0000-000000000001"/>
          <exchangeDirection>Sideways</exchangeDirection>
          <meanAmount>1.0</meanAmount>
        </exchange>
      </exchanges>
    </processDataSet>"#;
    let err = parse_process(xml).unwrap_err();
    assert!(matches!(
        err,
        IlcdError::InvalidText { elem: "exchangeDirection", .. }
    ));
}

#[test]
fn rejects_nonfinite_amount() {
    let xml = r#"<?xml version="1.0"?>
    <processDataSet xmlns="http://lca.jrc.it/ILCD/Process"
                    xmlns:common="http://lca.jrc.it/ILCD/Common">
      <processInformation>
        <dataSetInformation>
          <common:UUID>44444444-0000-0000-0000-000000000001</common:UUID>
          <name><baseName xml:lang="en">x</baseName></name>
        </dataSetInformation>
        <quantitativeReference type="Reference flow(s)">
          <referenceToReferenceFlow>1</referenceToReferenceFlow>
        </quantitativeReference>
      </processInformation>
      <exchanges>
        <exchange dataSetInternalID="1">
          <referenceToFlowDataSet refObjectId="00000000-0000-0000-0000-000000000001"/>
          <exchangeDirection>Output</exchangeDirection>
          <meanAmount>NaN</meanAmount>
        </exchange>
      </exchanges>
    </processDataSet>"#;
    let err = parse_process(xml).unwrap_err();
    assert!(matches!(
        err,
        IlcdError::NumericNonfinite { field: "meanAmount", .. }
    ));
}

#[test]
fn falls_back_to_mean_when_resulting_absent() {
    // Many published EPDs emit only <meanAmount>; the reader copies
    // it forward to resulting_amount so downstream code has a single
    // canonical "amount per reference flow" field.
    let xml = r#"<?xml version="1.0"?>
    <processDataSet xmlns="http://lca.jrc.it/ILCD/Process"
                    xmlns:common="http://lca.jrc.it/ILCD/Common">
      <processInformation>
        <dataSetInformation>
          <common:UUID>55555555-0000-0000-0000-000000000001</common:UUID>
          <name><baseName xml:lang="en">x</baseName></name>
        </dataSetInformation>
        <quantitativeReference type="Reference flow(s)">
          <referenceToReferenceFlow>1</referenceToReferenceFlow>
        </quantitativeReference>
      </processInformation>
      <exchanges>
        <exchange dataSetInternalID="1">
          <referenceToFlowDataSet refObjectId="00000000-0000-0000-0000-000000000001"/>
          <exchangeDirection>Output</exchangeDirection>
          <meanAmount>2.5</meanAmount>
        </exchange>
      </exchanges>
    </processDataSet>"#;
    let ds = parse_process(xml).unwrap();
    let e = &ds.exchanges[0];
    assert_eq!(e.mean_amount, 2.5);
    assert_eq!(e.resulting_amount, 2.5);
}
