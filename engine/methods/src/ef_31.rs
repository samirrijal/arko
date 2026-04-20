//! EF 3.1 impact method — V1 scope: EN 15804+A2 core emission indicators.
//!
//! The European Commission's Environmental Footprint 3.1 method,
//! harmonised by the JRC (Joint Research Centre) and maintained via
//! the European Platform on LCA (EPLCA). EF 3.1 defines 16 midpoint
//! impact categories; Arko V1 ships the **7 emission-based core
//! indicators** required by EN 15804+A2:
//!
//! 1. Climate change (`climate-change`) — kg CO2-eq. Uses IPCC AR6
//!    GWP100 factors; same factor table as
//!    [`crate::standard::ipcc_ar6_gwp100`].
//! 2. Ozone depletion (`ozone-depletion`) — kg CFC-11-eq.
//! 3. Photochemical ozone formation (`photochemical-ozone-formation`)
//!    — kg NMVOC-eq.
//! 4. Acidification (`acidification`) — mol H+-eq. Compartment-
//!    dependent CFs (air emissions count; water does not).
//! 5. Eutrophication, freshwater (`eutrophication-freshwater`)
//!    — kg P-eq. Compartment-dependent.
//! 6. Eutrophication, marine (`eutrophication-marine`)
//!    — kg N-eq. Compartment-dependent.
//! 7. Eutrophication, terrestrial (`eutrophication-terrestrial`)
//!    — mol N-eq. Compartment-dependent.
//!
//! ## What V1 does *not* ship (deferred to V2)
//!
//! - **Additional indicators per EN 15804+A2 annex:** particulate
//!   matter, human toxicity (cancer + non-cancer), ecotoxicity
//!   freshwater, ionising radiation, land use, resource use
//!   (fossils + minerals/metals). These are optional-reporting
//!   categories in EN 15804+A2; V1 ships the mandatory-core set.
//! - **Water use.** Regionalised per watershed/country; requires a
//!   matcher axis (`CasRegion` or compartment-with-region) that V1
//!   does not have. Defer until aligned with ReCiPe 2016's
//!   regionalised midpoints — doing it twice is waste.
//!
//! See `DECISIONS.md` entry `D-0015` for the full scope rationale
//! and the shippable-EPD-floor framing.
//!
//! ## Scaffold state
//!
//! At scaffold time (2026-04-21), the 7 categories are present with
//! correct IDs, names, and units but **empty factor lists**. Factor
//! data entry is tracked as a separate landing against the JRC EF
//! 3.1 characterisation-factor spreadsheet (primary source: JRC
//! EPLCA, `EF-3.1_CF.xlsx` in the EF 3.1 main reference package).
//! Each factor will carry a source comment noting file, sheet,
//! version, and publication date per the factor-table entry
//! discipline.
//!
//! ## References
//!
//! - European Commission, Joint Research Centre (2021–2024).
//!   *Environmental Footprint 3.1 reference package.*
//!   EPLCA, <https://eplca.jrc.ec.europa.eu/>
//! - EN 15804:2012+A2:2019 *Sustainability of construction works
//!   — Environmental product declarations — Core rules for the
//!   product category of construction products*.

use crate::method::{CharacterizationFactor, ImpactCategory, ImpactMethod};

/// EF 3.1 V1 — 7 emission-based core indicators of EN 15804+A2.
///
/// `(id, version) = ("ef-3.1", "1")`.
///
/// Category order is stable and matches EN 15804+A2 presentation
/// order. Callers that index into `method.categories` by position
/// can rely on this order; method revisions that change it will
/// bump the `version` key.
#[must_use]
pub fn ef_31() -> ImpactMethod {
    ImpactMethod {
        id: "ef-3.1".into(),
        version: "1".into(),
        name: "EF 3.1".into(),
        categories: vec![
            category(
                "climate-change",
                "Climate change (EF 3.1)",
                "kg CO2-eq",
                climate_change_factors(),
            ),
            category(
                "ozone-depletion",
                "Ozone depletion (EF 3.1)",
                "kg CFC-11-eq",
                ozone_depletion_factors(),
            ),
            category(
                "photochemical-ozone-formation",
                "Photochemical ozone formation (EF 3.1)",
                "kg NMVOC-eq",
                photochemical_ozone_formation_factors(),
            ),
            category(
                "acidification",
                "Acidification (EF 3.1)",
                "mol H+-eq",
                acidification_factors(),
            ),
            category(
                "eutrophication-freshwater",
                "Eutrophication, freshwater (EF 3.1)",
                "kg P-eq",
                eutrophication_freshwater_factors(),
            ),
            category(
                "eutrophication-marine",
                "Eutrophication, marine (EF 3.1)",
                "kg N-eq",
                eutrophication_marine_factors(),
            ),
            category(
                "eutrophication-terrestrial",
                "Eutrophication, terrestrial (EF 3.1)",
                "mol N-eq",
                eutrophication_terrestrial_factors(),
            ),
        ],
    }
}

fn category(
    id: &str,
    name: &str,
    unit: &str,
    factors: Vec<CharacterizationFactor>,
) -> ImpactCategory {
    ImpactCategory {
        id: id.into(),
        name: name.into(),
        unit: unit.into(),
        factors,
    }
}

// ---- Factor tables ---------------------------------------------------
//
// Each function below returns the characterisation factors for one
// EF 3.1 category. Factor data is entered in a separate landing
// against the JRC EF 3.1 CF spreadsheet (see module doc). At
// scaffold time, every list is empty — the categories exist with
// correct shape so the `ImpactMethod` is registrable-with-empty-CFs
// and the registry's shape tests pass. An empty factor list means
// every flow will appear in `CMatrixBuild::unmatched_flows` for
// that category: by design, surfacing the scaffold state to callers
// rather than silently returning zero impact.

fn climate_change_factors() -> Vec<CharacterizationFactor> {
    // TODO: EF 3.1 specifies IPCC AR6 GWP100 for Climate change.
    // Factor entry will either (a) re-use `crate::standard::ar6_gwp100_factors`
    // via a shared helper, or (b) duplicate the AR6 table locally so
    // the EF 3.1 preset is self-contained. Decide at factor-entry time.
    vec![]
}

fn ozone_depletion_factors() -> Vec<CharacterizationFactor> {
    vec![]
}

fn photochemical_ozone_formation_factors() -> Vec<CharacterizationFactor> {
    vec![]
}

fn acidification_factors() -> Vec<CharacterizationFactor> {
    vec![]
}

fn eutrophication_freshwater_factors() -> Vec<CharacterizationFactor> {
    vec![]
}

fn eutrophication_marine_factors() -> Vec<CharacterizationFactor> {
    vec![]
}

fn eutrophication_terrestrial_factors() -> Vec<CharacterizationFactor> {
    vec![]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ef_31_id_and_version_are_canonical() {
        let m = ef_31();
        assert_eq!(m.id, "ef-3.1");
        assert_eq!(m.version, "1");
        assert_eq!(m.name, "EF 3.1");
    }

    #[test]
    fn ef_31_has_seven_categories() {
        let m = ef_31();
        assert_eq!(
            m.categories.len(),
            7,
            "EF 3.1 V1 ships the 7 emission-based core indicators of EN 15804+A2"
        );
    }

    #[test]
    fn ef_31_category_ids_match_en15804_core_emission_set() {
        let m = ef_31();
        let ids: Vec<&str> = m.categories.iter().map(|c| c.id.as_str()).collect();
        assert_eq!(
            ids,
            vec![
                "climate-change",
                "ozone-depletion",
                "photochemical-ozone-formation",
                "acidification",
                "eutrophication-freshwater",
                "eutrophication-marine",
                "eutrophication-terrestrial",
            ],
            "category order is stable and matches EN 15804+A2 presentation order"
        );
    }

    #[test]
    fn ef_31_units_match_published_spec() {
        let m = ef_31();
        let units: Vec<(&str, &str)> = m
            .categories
            .iter()
            .map(|c| (c.id.as_str(), c.unit.as_str()))
            .collect();
        assert_eq!(
            units,
            vec![
                ("climate-change", "kg CO2-eq"),
                ("ozone-depletion", "kg CFC-11-eq"),
                ("photochemical-ozone-formation", "kg NMVOC-eq"),
                ("acidification", "mol H+-eq"),
                ("eutrophication-freshwater", "kg P-eq"),
                ("eutrophication-marine", "kg N-eq"),
                ("eutrophication-terrestrial", "mol N-eq"),
            ]
        );
    }

    #[test]
    fn ef_31_scaffold_has_empty_factor_lists() {
        // Scaffold contract: this test confirms the preset is in its
        // pre-data-entry state. It will be **deleted** in the factor-
        // entry landing once the first factors are added. Its
        // presence on `main` is a signal that factor entry has not
        // yet started.
        let m = ef_31();
        for c in &m.categories {
            assert!(
                c.factors.is_empty(),
                "scaffold state: category `{}` must have no factors yet",
                c.id
            );
        }
    }

    #[test]
    fn ef_31_json_round_trip() {
        // Serde shape is stable so a persisted study referencing the
        // scaffold survives later factor-entry amendments.
        let m = ef_31();
        let s = serde_json::to_string(&m).unwrap();
        let back: ImpactMethod = serde_json::from_str(&s).unwrap();
        assert_eq!(m, back);
    }
}
