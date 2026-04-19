//! Standard-library impact methods.
//!
//! v0.0.1 ships **two** IPCC GWP100 presets so users can:
//!
//! - Report against the current IPCC assessment ([`ipcc_ar6_gwp100`]).
//!   This is the documented default recommendation for new studies —
//!   it's what PCR committees (EN 15804+A2, EPD International 2024
//!   onwards, EF 3.1 as harmonized by the EC JRC) have been migrating
//!   to since 2022. CH4 is split fossil vs non-fossil per AR6 WG1
//!   Chapter 7 Table 7.15 (`29.8` / `27.0`).
//!
//! - Reproduce historical EPDs authored under the prior assessment
//!   ([`ipcc_ar5_gwp100`]). CH4 is a single value (`28`) per AR5 WG1
//!   Chapter 8 Table 8.A.1 (no c-c feedback); origin is ignored.
//!
//! The two methods carry distinct `(id, version)` keys so a study's
//! `MethodRef` unambiguously selects one. A consultancy can ship an
//! AR5 legacy-verification report and an AR6 go-forward report in the
//! same workspace without any cross-contamination at the factor level.
//!
//! **Disclaimer.** These factors are drawn from the public IPCC tables
//! cited in each preset. The file is not audited by IPCC authors.
//! Users publishing results should verify against the latest errata
//! and cite the WG1 reports directly in their EPDs.
//!
//! **References.**
//! - AR6: Forster, P., et al. (2021). *The Earth's Energy Budget,
//!   Climate Feedbacks, and Climate Sensitivity.* In *Climate Change
//!   2021: The Physical Science Basis.* Cambridge University Press,
//!   Chapter 7, Table 7.15.
//! - AR5: Myhre, G., et al. (2013). *Anthropogenic and Natural
//!   Radiative Forcing.* In *Climate Change 2013: The Physical
//!   Science Basis.* Cambridge University Press, Chapter 8,
//!   Table 8.A.1 (GWP100 without climate-carbon feedback column).

use crate::method::{CharacterizationFactor, FactorMatch, ImpactCategory, ImpactMethod};
use arko_core::meta::FlowOrigin;

/// IPCC AR6 GWP 100-year factors (kg CO2-eq per kg emitted).
///
/// **Default recommendation** for new Arko studies. CH4 is split
/// fossil vs non-fossil via [`FactorMatch::CasOrigin`]; flows with
/// `FlowOrigin::Unspecified` will not match either CH4 factor and will
/// surface in `CMatrixBuild::unmatched_flows` — by design, to avoid
/// silently applying the fossil CH4 GWP to an unclassified flow.
///
/// `(id, version) = ("ipcc-ar6-gwp100", "1")`.
pub fn ipcc_ar6_gwp100() -> ImpactMethod {
    ImpactMethod {
        id: "ipcc-ar6-gwp100".into(),
        version: "1".into(),
        name: "IPCC AR6 GWP100".into(),
        categories: vec![ImpactCategory {
            id: "gwp100".into(),
            name: "Climate change (GWP100, AR6)".into(),
            unit: "kg CO2-eq".into(),
            factors: ar6_gwp100_factors(),
        }],
    }
}

/// IPCC AR5 GWP 100-year factors (kg CO2-eq per kg emitted),
/// without climate-carbon feedback.
///
/// Shipped for **legacy-verification parity**: existing EPDs authored
/// under AR5 (the default assessment from 2013 through ~2021) must be
/// reproducible bit-for-bit when re-verified. New studies should
/// prefer [`ipcc_ar6_gwp100`].
///
/// AR5 did not differentiate fossil vs non-fossil CH4 — both resolve
/// to the same `28` — so the CH4 matcher is origin-agnostic
/// ([`FactorMatch::Cas`]) and any `FlowOrigin` matches.
///
/// `(id, version) = ("ipcc-ar5-gwp100", "1")`.
pub fn ipcc_ar5_gwp100() -> ImpactMethod {
    ImpactMethod {
        id: "ipcc-ar5-gwp100".into(),
        version: "1".into(),
        name: "IPCC AR5 GWP100".into(),
        categories: vec![ImpactCategory {
            id: "gwp100".into(),
            name: "Climate change (GWP100, AR5, no c-c feedback)".into(),
            unit: "kg CO2-eq".into(),
            factors: ar5_gwp100_factors(),
        }],
    }
}

/// AR6 Table 7.15 subset. Values verified against the published table
/// 2026-04-19:
///
/// - CO2: `1.0` (reference by definition).
/// - CH4 **fossil**: `29.8` (AR6 WG1 Ch7 T7.15). Matched via
///   `CasOrigin`.
/// - CH4 **non-fossil**: `27.0` (AR6 WG1 Ch7 T7.15). Matched via
///   `CasOrigin`.
/// - N2O: `273` (AR6 WG1 Ch7 T7.15; no fossil/non-fossil split).
/// - SF6: `25_200`. NF3: `17_400`.
/// - HFC-134a: `1_530`. HFC-23: `14_600`. HFC-32: `771`.
/// - CF4 (PFC-14): `7_380`. C2F6 (PFC-116): `12_400`.
fn ar6_gwp100_factors() -> Vec<CharacterizationFactor> {
    // CH4 order matters for the tests that read factors[1] / factors[2];
    // keep fossil before non-fossil.
    vec![
        cas_factor("124-38-9", 1.0, "Carbon dioxide (CO2), AR6 reference"),
        cas_origin_factor(
            "74-82-8",
            FlowOrigin::Fossil,
            29.8,
            "Methane (CH4), fossil — AR6 WG1 Ch7 T7.15",
        ),
        cas_origin_factor(
            "74-82-8",
            FlowOrigin::NonFossil,
            27.0,
            "Methane (CH4), non-fossil — AR6 WG1 Ch7 T7.15",
        ),
        cas_factor(
            "10024-97-2",
            273.0,
            "Nitrous oxide (N2O) — AR6 WG1 Ch7 T7.15",
        ),
        cas_factor("2551-62-4", 25_200.0, "Sulfur hexafluoride (SF6)"),
        cas_factor("7783-54-2", 17_400.0, "Nitrogen trifluoride (NF3)"),
        cas_factor(
            "811-97-2",
            1_530.0,
            "HFC-134a (1,1,1,2-tetrafluoroethane)",
        ),
        cas_factor("75-46-7", 14_600.0, "HFC-23 (trifluoromethane)"),
        cas_factor("75-10-5", 771.0, "HFC-32 (difluoromethane)"),
        cas_factor("75-73-0", 7_380.0, "PFC-14 (tetrafluoromethane, CF4)"),
        cas_factor("76-16-4", 12_400.0, "PFC-116 (hexafluoroethane, C2F6)"),
    ]
}

/// AR5 Table 8.A.1 subset (GWP100 without climate-carbon feedback).
/// Values verified against the published table 2026-04-19:
///
/// - CO2: `1.0`.
/// - CH4: `28` (single value; AR5 did not split fossil vs non-fossil).
/// - N2O: `265`.
/// - SF6: `23_500`. NF3: `16_100`.
/// - HFC-134a: `1_300`. HFC-23: `12_400`. HFC-32: `677`.
/// - CF4 (PFC-14): `6_630`. C2F6 (PFC-116): `11_100`.
fn ar5_gwp100_factors() -> Vec<CharacterizationFactor> {
    let entries: &[(&str, f64, &str)] = &[
        ("124-38-9", 1.0, "Carbon dioxide (CO2), AR5 reference"),
        (
            "74-82-8",
            28.0,
            "Methane (CH4) — AR5 WG1 Ch8 T8.A.1 (no c-c fb)",
        ),
        (
            "10024-97-2",
            265.0,
            "Nitrous oxide (N2O) — AR5 WG1 Ch8 T8.A.1",
        ),
        ("2551-62-4", 23_500.0, "Sulfur hexafluoride (SF6)"),
        ("7783-54-2", 16_100.0, "Nitrogen trifluoride (NF3)"),
        ("811-97-2", 1_300.0, "HFC-134a"),
        ("75-46-7", 12_400.0, "HFC-23"),
        ("75-10-5", 677.0, "HFC-32"),
        ("75-73-0", 6_630.0, "PFC-14 (CF4)"),
        ("76-16-4", 11_100.0, "PFC-116 (C2F6)"),
    ];

    entries
        .iter()
        .map(|(cas, value, note)| cas_factor(cas, *value, note))
        .collect()
}

fn cas_factor(cas: &str, value: f64, note: &str) -> CharacterizationFactor {
    CharacterizationFactor {
        match_on: FactorMatch::Cas { cas: cas.into() },
        value,
        note: Some(note.into()),
    }
}

fn cas_origin_factor(
    cas: &str,
    origin: FlowOrigin,
    value: f64,
    note: &str,
) -> CharacterizationFactor {
    CharacterizationFactor {
        match_on: FactorMatch::CasOrigin {
            cas: cas.into(),
            origin,
        },
        value,
        note: Some(note.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- AR6 --------------------------------------------------------

    #[test]
    fn ar6_includes_core_ghgs() {
        let m = ipcc_ar6_gwp100();
        let cases_seen: Vec<String> = m.categories[0]
            .factors
            .iter()
            .filter_map(|f| match &f.match_on {
                FactorMatch::Cas { cas } | FactorMatch::CasOrigin { cas, .. } => Some(cas.clone()),
                _ => None,
            })
            .collect();
        // CO2, CH4, N2O are non-negotiable.
        assert!(cases_seen.contains(&"124-38-9".to_string()));
        assert!(cases_seen.contains(&"74-82-8".to_string()));
        assert!(cases_seen.contains(&"10024-97-2".to_string()));
    }

    #[test]
    fn ar6_co2_reference_factor_is_exactly_one() {
        let m = ipcc_ar6_gwp100();
        let co2 = m.categories[0]
            .factors
            .iter()
            .find(|f| matches!(&f.match_on, FactorMatch::Cas { cas } if cas == "124-38-9"))
            .expect("CO2 must be present");
        assert_eq!(co2.value, 1.0, "CO2 GWP100 is the reference, = 1");
    }

    #[test]
    fn ar6_ch4_is_split_fossil_and_non_fossil() {
        let m = ipcc_ar6_gwp100();
        let fossil = m.categories[0]
            .factors
            .iter()
            .find(|f| {
                matches!(
                    &f.match_on,
                    FactorMatch::CasOrigin { cas, origin: FlowOrigin::Fossil } if cas == "74-82-8"
                )
            })
            .expect("fossil CH4 factor missing");
        let nonfossil = m.categories[0]
            .factors
            .iter()
            .find(|f| matches!(
                &f.match_on,
                FactorMatch::CasOrigin { cas, origin: FlowOrigin::NonFossil } if cas == "74-82-8"
            ))
            .expect("non-fossil CH4 factor missing");
        assert_eq!(fossil.value, 29.8, "AR6 WG1 Ch7 T7.15 fossil CH4 = 29.8");
        assert_eq!(
            nonfossil.value, 27.0,
            "AR6 WG1 Ch7 T7.15 non-fossil CH4 = 27.0"
        );
    }

    #[test]
    fn ar6_n2o_matches_ch7_table() {
        let m = ipcc_ar6_gwp100();
        let n2o = m.categories[0]
            .factors
            .iter()
            .find(|f| matches!(&f.match_on, FactorMatch::Cas { cas } if cas == "10024-97-2"))
            .expect("N2O must be present");
        assert_eq!(n2o.value, 273.0, "AR6 WG1 Ch7 T7.15 N2O = 273");
    }

    #[test]
    fn ar6_every_factor_is_finite_and_nonnegative() {
        let m = ipcc_ar6_gwp100();
        for f in &m.categories[0].factors {
            assert!(f.value.is_finite(), "{:?}", f.match_on);
            assert!(f.value >= 0.0, "{:?}", f.match_on);
        }
    }

    #[test]
    fn ar6_every_factor_has_attribution_note() {
        let m = ipcc_ar6_gwp100();
        for f in &m.categories[0].factors {
            assert!(f.note.is_some(), "note missing for {:?}", f.match_on);
        }
    }

    // ---- AR5 --------------------------------------------------------

    #[test]
    fn ar5_ch4_is_single_valued_and_origin_agnostic() {
        let m = ipcc_ar5_gwp100();
        let ch4 = m.categories[0]
            .factors
            .iter()
            .find(|f| matches!(&f.match_on, FactorMatch::Cas { cas } if cas == "74-82-8"))
            .expect("AR5 CH4 must be present as a plain Cas match");
        assert_eq!(ch4.value, 28.0, "AR5 WG1 Ch8 T8.A.1 CH4 = 28");
        // And the fossil-only variant must NOT be present in AR5.
        let any_origin_specific = m.categories[0].factors.iter().any(|f| {
            matches!(
                &f.match_on,
                FactorMatch::CasOrigin { cas, .. } if cas == "74-82-8"
            )
        });
        assert!(
            !any_origin_specific,
            "AR5 did not split CH4 by origin; shipped factor must not use CasOrigin"
        );
    }

    #[test]
    fn ar5_co2_reference_factor_is_exactly_one() {
        let m = ipcc_ar5_gwp100();
        let co2 = m.categories[0]
            .factors
            .iter()
            .find(|f| matches!(&f.match_on, FactorMatch::Cas { cas } if cas == "124-38-9"))
            .expect("CO2 must be present");
        assert_eq!(co2.value, 1.0);
    }

    #[test]
    fn ar5_n2o_matches_ch8_table() {
        let m = ipcc_ar5_gwp100();
        let n2o = m.categories[0]
            .factors
            .iter()
            .find(|f| matches!(&f.match_on, FactorMatch::Cas { cas } if cas == "10024-97-2"))
            .expect("N2O must be present");
        assert_eq!(n2o.value, 265.0, "AR5 WG1 Ch8 T8.A.1 N2O = 265");
    }

    #[test]
    fn ar5_and_ar6_ids_differ() {
        assert_ne!(ipcc_ar5_gwp100().id, ipcc_ar6_gwp100().id);
    }
}
