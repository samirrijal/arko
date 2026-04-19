//! Conversion tests — every number in the common LCA "look it up in a
//! textbook" set, to make sure the analyzer's scale arithmetic is
//! correct end-to-end.

use approx::assert_relative_eq;
use arko_units::{convert_str, ParsedUnit};

#[test]
fn mass_chain() {
    assert_relative_eq!(convert_str(1.0, "t", "kg").unwrap(), 1000.0, epsilon = 1e-9);
    assert_relative_eq!(convert_str(1.0, "kg", "g").unwrap(), 1000.0, epsilon = 1e-9);
    assert_relative_eq!(convert_str(1.0, "t", "g").unwrap(), 1e6, epsilon = 1e-3);
    assert_relative_eq!(convert_str(1.0, "mg", "g").unwrap(), 1e-3, epsilon = 1e-12);
}

#[test]
fn length_chain() {
    assert_relative_eq!(convert_str(1.0, "km", "m").unwrap(), 1000.0, epsilon = 1e-9);
    assert_relative_eq!(convert_str(1.0, "cm", "m").unwrap(), 1e-2, epsilon = 1e-15);
    assert_relative_eq!(convert_str(1.0, "mm", "m").unwrap(), 1e-3, epsilon = 1e-15);
    assert_relative_eq!(convert_str(1.0, "um", "mm").unwrap(), 1e-3, epsilon = 1e-15);
}

#[test]
fn volume_chain() {
    assert_relative_eq!(convert_str(1.0, "L", "m3").unwrap(), 1e-3, epsilon = 1e-15);
    assert_relative_eq!(convert_str(1.0, "mL", "L").unwrap(), 1e-3, epsilon = 1e-15);
    assert_relative_eq!(convert_str(1.0, "m3", "L").unwrap(), 1000.0, epsilon = 1e-9);
}

#[test]
fn energy_chain() {
    // 1 kWh = 3600000 J = 3.6 MJ.
    assert_relative_eq!(convert_str(1.0, "kWh", "J").unwrap(), 3.6e6, epsilon = 1e-3);
    assert_relative_eq!(convert_str(1.0, "MJ", "J").unwrap(), 1e6, epsilon = 1e-3);
    assert_relative_eq!(
        convert_str(1.0, "GJ", "MJ").unwrap(),
        1000.0,
        epsilon = 1e-6
    );
    // 1 TJ = 277778 kWh.
    let v = convert_str(1.0, "TJ", "kWh").unwrap();
    assert_relative_eq!(v, 1e12 / 3.6e6, epsilon = 1e-3);
}

#[test]
fn time_chain() {
    assert_relative_eq!(convert_str(1.0, "min", "s").unwrap(), 60.0, epsilon = 1e-12);
    assert_relative_eq!(convert_str(1.0, "h", "s").unwrap(), 3600.0, epsilon = 1e-9);
    assert_relative_eq!(convert_str(1.0, "h", "min").unwrap(), 60.0, epsilon = 1e-12);
    assert_relative_eq!(convert_str(1.0, "d", "h").unwrap(), 24.0, epsilon = 1e-12);
    // 1 yr = 365.25 days.
    assert_relative_eq!(convert_str(1.0, "yr", "d").unwrap(), 365.25, epsilon = 1e-9);
}

#[test]
fn compound_transport_unit_tkm_to_kg_m() {
    // 1 t·km = 1000 kg·m * 1000? No: 1 t·km = 1 t * 1 km = 1000 kg * 1000 m = 1e6 kg·m.
    let v = convert_str(1.0, "t.km", "kg.m").unwrap();
    assert_relative_eq!(v, 1e6, epsilon = 1e-3);
}

#[test]
fn tagged_units_are_self_compatible() {
    // kg CO2-eq → kg CO2-eq: identity conversion factor = 1.
    let v = convert_str(5.5, "kg CO2-eq", "kg CO2-eq").unwrap();
    assert_relative_eq!(v, 5.5, epsilon = 1e-15);
}

#[test]
fn tagged_units_reject_cross_tag_conversion() {
    // kg CO2-eq → kg SO2-eq is a characterization problem, not a
    // unit problem. Must surface as a tag error.
    let err = convert_str(1.0, "kg CO2-eq", "kg SO2-eq").unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("tag"),
        "tag-mismatch error should mention tags, got `{msg}`"
    );
}

#[test]
fn cross_dimension_conversion_is_error() {
    let err = convert_str(1.0, "kg", "MJ").unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("dimension"),
        "dim-mismatch error should mention dimensions, got `{msg}`"
    );
}

#[test]
fn unicode_middle_dot_and_period_are_equivalent() {
    let a = ParsedUnit::parse("kg·m").unwrap();
    let b = ParsedUnit::parse("kg.m").unwrap();
    assert_eq!(a.dimension, b.dimension);
    assert_relative_eq!(a.scale_to_si, b.scale_to_si, epsilon = 1e-15);
}

#[test]
fn round_trip_conversion_is_identity() {
    // Take 1 MJ → kWh → MJ and expect back to 1 within 1e-12.
    let kwh = convert_str(1.0, "MJ", "kWh").unwrap();
    let back = convert_str(kwh, "kWh", "MJ").unwrap();
    assert_relative_eq!(back, 1.0, epsilon = 1e-15);
}
