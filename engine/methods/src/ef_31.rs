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

use crate::method::{CharacterizationFactor, FactorMatch, ImpactCategory, ImpactMethod};

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

/// CAS-matcher factor with attribution note. Local to this module —
/// `standard.rs` has the same helper for AR6/AR5; duplicated here so
/// the two modules can evolve independently.
fn cas_factor(cas: &str, value: f64, note: &str) -> CharacterizationFactor {
    CharacterizationFactor {
        match_on: FactorMatch::Cas { cas: cas.into() },
        value,
        note: Some(note.into()),
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
    // Source: JRC EF 3.1 reference package, ILCD LCIA method dataset
    // `b5c629d6-def3-11e6-bf01-fe55135034f3.xml` ("Ozone depletion"),
    // dataSetVersion 02.00.012, dateOfLastRevision 2016-12-20. Model:
    // WMO (1999) Scientific Assessment of Ozone Depletion — steady-state
    // ODP referenced to CFC-11. Indicator unit: kg CFC-11-eq.
    //
    // Taxonomy pre-check: 124 factors, global scope (no `<location>`
    // tag present anywhere in the dataset — unlike AC), all compartments
    // are air subcompartments (urban/non-urban/stratosphere/unspecified).
    // CFs are uniform across all air subcompartments for each substance.
    // No origin splits (CFCs are manufactured, no biogenic variant).
    // `Cas` is the correct matcher — species identity is the load-bearing
    // selectivity axis, same pattern as AR6 CO2.
    //
    // 25 text-distinct substances in the source table collapse to
    // 23 unique CAS numbers after two aliasing pairs:
    //   - "CFC-10" and "Carbon tetrachloride" (same CAS 56-23-5, CF 0.72)
    //   - "Methyl bromide" and "Halon-1001" (same CAS 74-83-9, CF 0.57)
    // One matcher per CAS catches both text representations.
    //
    // Four species in the source table have low market prevalence and
    // non-trivial CAS lookup (Halon-1201, Halon-1202, Halon-2311,
    // Halon-2401) — omitted from V1 to avoid guessed CAS numbers.
    // Their absence means such flows fall into unmatched_flows, which
    // is the correct surfacing behaviour (silently-wrong CF is worse
    // than visibly-missing CF).
    //
    // Seeds: CFC-11 is the basic seed (reference species by definition,
    // CF = 1.0); Halon-2402 is the edge seed (highest CF at 15.7 —
    // 15× CFC-11, validates matcher on the top-ranked species).
    vec![
        cas_factor("75-69-4", 1.0, "CFC-11 (trichlorofluoromethane) — basic seed, reference species"),
        cas_factor("75-71-8", 0.73, "CFC-12 (dichlorodifluoromethane)"),
        cas_factor("76-13-1", 0.81, "CFC-113 (1,1,2-trichlorotrifluoroethane)"),
        cas_factor("76-14-2", 0.5, "CFC-114 (dichlorotetrafluoroethane)"),
        cas_factor("76-15-3", 0.26, "CFC-115 (chloropentafluoroethane)"),
        cas_factor("56-23-5", 0.72, "Carbon tetrachloride / CFC-10"),
        cas_factor("71-55-6", 0.14, "1,1,1-trichloroethane (methyl chloroform)"),
        cas_factor("74-83-9", 0.57, "Methyl bromide / Halon-1001"),
        cas_factor("74-87-3", 0.015, "Chloromethane (methyl chloride)"),
        cas_factor("353-59-3", 6.9, "Halon-1211 (bromochlorodifluoromethane)"),
        cas_factor("75-63-8", 15.2, "Halon-1301 (bromotrifluoromethane)"),
        cas_factor("124-73-2", 15.7, "Halon-2402 (dibromotetrafluoroethane) — edge seed, highest ODP"),
        cas_factor("75-45-6", 0.034, "HCFC-22 (chlorodifluoromethane)"),
        cas_factor("306-83-2", 0.01, "HCFC-123 (2,2-dichloro-1,1,1-trifluoroethane)"),
        cas_factor("2837-89-0", 0.022, "HCFC-124 (2-chloro-1,1,1,2-tetrafluoroethane)"),
        cas_factor("1717-00-6", 0.102, "HCFC-141b (1,1-dichloro-1-fluoroethane)"),
        cas_factor("75-68-3", 0.057, "HCFC-142b (1-chloro-1,1-difluoroethane)"),
        cas_factor("422-56-0", 0.025, "HCFC-225ca"),
        cas_factor("507-55-1", 0.033, "HCFC-225cb"),
    ]
}

fn photochemical_ozone_formation_factors() -> Vec<CharacterizationFactor> {
    // Source: JRC EF 3.1 reference package, ILCD LCIA method dataset
    // `b5c610fe-def3-11e6-bf01-fe55135034f3.xml`
    // ("Photochemical ozone formation - human health"), model
    // LOTOS-EUROS (Van Zelm et al. 2008), indicator unit kg NMVOC-eq.
    //
    // Taxonomy pre-check: 1401 factors (681 pan-European defaults
    // via `<location/>`, 720 country-specific). V1 ships the
    // pan-European defaults; per-country resolution deferred to
    // `CasRegion` V2 — same pattern as Acidification. All compartments
    // are air subcompartments (urban/non-urban/stratosphere/indoor/
    // unspecified); CFs are uniform across subcompartments per
    // substance.
    //
    // Matcher choice: `CasCompartment` prefixed on `["emission", "air"]`.
    // Rationale mirrors AC — POCP is a tropospheric photochemistry
    // phenomenon, and VOC-to-water/soil flows (real in inventories:
    // solvent releases, alcohol spills) must not receive a POCP CF.
    // Unlike OD (where non-air CFC flows are not a realistic inventory
    // case), oxygenated VOC to water is a legitimate inventory entry.
    //
    // Scope: ~18 high-prevalence species with well-known CAS. The
    // source table has 132 inventory items; the long tail of
    // substituted benzenes, alkanes >C7, and niche oxygenates can
    // be added incrementally. Species without CAS (e.g., the generic
    // "non-methane volatile organic compounds" group flow) are
    // deferred — they need a `FlowId` or `NameAndCompartment`
    // matcher, which is out of scope for a CAS-first V1.
    //
    // Seeds: ethylene is the basic seed (74-85-1 = 1.69, classic
    // POCP reference species); 1,3,5-trimethylbenzene is the edge
    // seed (108-67-8 = 2.33, highest CF in the table — structurally
    // distinct aromatic versus the olefin basic seed).
    let air = || vec!["emission".into(), "air".into()];
    let cc = |cas: &str, value: f64, note: &str| CharacterizationFactor {
        match_on: FactorMatch::CasCompartment {
            cas: cas.into(),
            compartment: air(),
        },
        value,
        note: Some(note.into()),
    };
    vec![
        // NOx species — POCP depends on both VOC and NOx emissions.
        cc("10102-43-9", 1.0, "EF 3.1 POCP — NO to air"),
        cc("10102-44-0", 1.0, "EF 3.1 POCP — NO2 to air"),
        // Alkenes / olefins.
        cc("74-85-1", 1.69, "EF 3.1 POCP — Ethylene to air (basic seed)"),
        cc("115-07-1", 1.9, "EF 3.1 POCP — Propene to air"),
        cc("106-99-0", 1.44, "EF 3.1 POCP — 1,3-Butadiene to air"),
        cc("78-79-5", 1.84, "EF 3.1 POCP — Isoprene to air"),
        // Aromatics.
        cc("71-43-2", 0.368, "EF 3.1 POCP — Benzene to air"),
        cc("108-88-3", 1.08, "EF 3.1 POCP — Toluene to air"),
        cc("108-38-3", 1.87, "EF 3.1 POCP — m-Xylene to air"),
        cc("95-47-6", 1.78, "EF 3.1 POCP — o-Xylene to air"),
        cc("106-42-3", 1.71, "EF 3.1 POCP — p-Xylene to air"),
        cc("108-67-8", 2.33, "EF 3.1 POCP — 1,3,5-Trimethylbenzene to air (edge seed)"),
        // Aldehydes.
        cc("50-00-0", 0.877, "EF 3.1 POCP — Formaldehyde to air"),
        cc("75-07-0", 1.08, "EF 3.1 POCP — Acetaldehyde to air"),
        // Alcohols / oxygenated solvents.
        cc("67-56-1", 0.236, "EF 3.1 POCP — Methanol to air"),
        cc("64-17-5", 0.674, "EF 3.1 POCP — Ethanol to air"),
        cc("67-64-1", 0.159, "EF 3.1 POCP — Acetone to air"),
    ]
}

fn acidification_factors() -> Vec<CharacterizationFactor> {
    // Source: JRC EF 3.1 reference package, ILCD LCIA method dataset
    // `b5c611c6-def3-11e6-bf01-fe55135034f3.xml` ("Acidification"),
    // dataSetVersion 01.04.000, dateOfLastRevision 2022-06-17,
    // Accumulated Exceedance model (Seppälä et al. 2006; Posch et al. 2008).
    //
    // The published dataset is fully regionalised — 935 factors covering
    // 36 European countries × substance × air-subcompartment. V1 ships
    // the pan-European defaults (the entries published with empty
    // `<location/>`, representing the region-neutral European CF). Per-
    // country resolution defers to a future `CasRegion` matcher (D-0015
    // "water-use deferral" discussion — same mechanism applies here).
    //
    // Within the air compartment, CFs are uniform across all air
    // subcompartments (urban, non-urban/high-stack, stratosphere,
    // unspecified) — checked across every `<location/>` entry in the
    // source file. The `CasCompartment` matcher's role for AC is
    // therefore `air vs non-air`: SO2/NH3/NOx emissions to water or soil
    // are legitimately not acidifying and must not receive a CF.
    //
    // Seeds: SO2 is the basic seed (common acidifier, canonical CF
    // reference); NH3 is the edge seed (highest CF 3.02 ≫ SO2 1.31,
    // validates matcher correctness on an unambiguous ranking-by-value
    // case).
    let air = || vec!["emission".into(), "air".into()];
    vec![
        CharacterizationFactor {
            match_on: FactorMatch::CasCompartment {
                cas: "7446-09-5".into(),
                compartment: air(),
            },
            value: 1.31,
            note: Some("EF 3.1 Acidification — SO2 to air (basic seed)".into()),
        },
        CharacterizationFactor {
            match_on: FactorMatch::CasCompartment {
                cas: "7446-11-9".into(),
                compartment: air(),
            },
            value: 1.04821,
            note: Some("EF 3.1 Acidification — SO3 to air".into()),
        },
        CharacterizationFactor {
            match_on: FactorMatch::CasCompartment {
                cas: "7664-41-7".into(),
                compartment: air(),
            },
            value: 3.02,
            note: Some("EF 3.1 Acidification — NH3 to air (edge seed — highest CF)".into()),
        },
        CharacterizationFactor {
            match_on: FactorMatch::CasCompartment {
                cas: "10102-43-9".into(),
                compartment: air(),
            },
            value: 1.13467,
            note: Some("EF 3.1 Acidification — NO to air (distinct from NO2)".into()),
        },
        CharacterizationFactor {
            match_on: FactorMatch::CasCompartment {
                cas: "10102-44-0".into(),
                compartment: air(),
            },
            value: 0.74,
            note: Some("EF 3.1 Acidification — NO2 to air".into()),
        },
    ]
}

fn eutrophication_freshwater_factors() -> Vec<CharacterizationFactor> {
    // Source: JRC EF 3.1 reference package, ILCD LCIA method dataset
    // `b53ec18f-7377-4ad3-86eb-cc3f4f276b2b.xml` ("Eutrophication,
    // freshwater"), Struijs et al. (2008) CARMEN (water-borne) +
    // EUTREND (air-borne) model, indicator unit kg P-eq.
    //
    // Taxonomy pre-check: 29 factors, no `<location>` tag anywhere —
    // the dataset itself ships pre-averaged European CFs (per the
    // method's `generalComment`: "Averaged characterization factors
    // from country dependent characterization factors"). 3 species
    // (phosphorus / phosphate / phosphoric acid) × 2 effective
    // compartments (water, soil). CFs differ by a factor of 20
    // between water and soil emissions: soil→freshwater transfer is
    // ~5%, so the soil CF is 0.05× the water CF.
    //
    // This is the load-bearing `CasCompartment` case — same CAS,
    // different compartment, different CF. Same matcher variant AC
    // used for air-vs-non-air exclusion; here the axis is genuinely
    // selectivity (two CFs per species, one per compartment).
    //
    // Known V1 limitation: the JRC dataset lists fresh-water and
    // unspecified-water entries but no sea-water entries. A
    // "phosphate to sea water" inventory flow would match the
    // `["emission", "water"]` prefix and receive the freshwater CF —
    // physically wrong for freshwater eutrophication (P released
    // offshore doesn't drive freshwater eutrophication). The JRC
    // source itself treats unspecified-water as freshwater-equivalent
    // (same CF 0.33 for phosphate), so the prefix matcher matches
    // the dataset's own convention. Fresh-vs-sea-water discrimination
    // deferred to the same future session that introduces `CasRegion`.
    //
    // Deferred species: "Phosphorus, total" is an aggregate group
    // flow without a clean CAS (it's the sum of elemental P +
    // compounds). Needs `FlowId` or `NameAndCompartment` matcher —
    // out of scope for CAS-first V1.
    //
    // Seeds: elemental phosphorus to water = 1.0 (basic, reference
    // species for kg P-eq); elemental phosphorus to soil = 0.05
    // (edge — same CAS, different compartment, 20× CF gap). This
    // edge seed is the whole reason `CasCompartment` exists.
    let water = || vec!["emission".into(), "water".into()];
    let soil = || vec!["emission".into(), "soil".into()];
    let cc = |cas: &str, compartment: Vec<String>, value: f64, note: &str| {
        CharacterizationFactor {
            match_on: FactorMatch::CasCompartment {
                cas: cas.into(),
                compartment,
            },
            value,
            note: Some(note.into()),
        }
    };
    vec![
        // Elemental phosphorus (CAS 7723-14-0) — reference species.
        cc("7723-14-0", water(), 1.0, "EF 3.1 EU-fw — P to water (basic seed, reference species)"),
        cc("7723-14-0", soil(), 0.05, "EF 3.1 EU-fw — P to soil (edge seed, same CAS 20x lower)"),
        // Phosphate ion (CAS 14265-44-2).
        cc("14265-44-2", water(), 0.33, "EF 3.1 EU-fw — Phosphate to water"),
        cc("14265-44-2", soil(), 0.016, "EF 3.1 EU-fw — Phosphate to soil"),
        // Phosphoric acid (CAS 7664-38-2).
        cc("7664-38-2", water(), 0.32, "EF 3.1 EU-fw — Phosphoric acid to water"),
        cc("7664-38-2", soil(), 0.016, "EF 3.1 EU-fw — Phosphoric acid to soil"),
    ]
}

fn eutrophication_marine_factors() -> Vec<CharacterizationFactor> {
    // Source: JRC EF 3.1 reference package, method dataset
    // `b5c619fa-def3-11e6-bf01-fe55135034f3.xml`
    // "Eutrophication marine", dataSetVersion 02.00.010,
    // dateOfLastRevision 2016-12-20, referenceYear 2017, reference
    // unit kg N-eq, underlying model Struijs et al. (2008)
    // CARMEN/EUTREND (same transfer-matrix family as EU-freshwater,
    // different receiving compartment — the marine zone).
    //
    // Regionalisation: 590 factors total in the dataset; ~540
    // country-specific (`<location>XX</location>`) and ~50
    // pan-European defaults (no `<location>` tag). We take only the
    // pan-European defaults for V1; `CasRegion` is the deferred
    // variant that would let us expose country-specific CFs.
    //
    // Compartment uniformity: same pattern as AC, POCP, EU-freshwater
    // — every air subcompartment (unspecified, lower stratosphere,
    // non-urban, urban, long-term) receives the same CF per species,
    // and every water subcompartment (fresh water, sea water,
    // unspecified, long-term) receives the same CF per species. The
    // `CasCompartment` prefix match on `["emission","air"]` /
    // `["emission","water"]` therefore captures the dataset's own
    // classification without resolution loss.
    //
    // Fresh-vs-sea-water deliberately collapsed in EF 3.1 EU-marine:
    // ammonia-to-fresh-water = 0.824 = ammonia-to-sea-water. The
    // CARMEN transfer matrix treats any water emission as eventually
    // reaching the marine zone. Unlike EU-freshwater (where the same
    // water-prefix collapse is a V1 limit, 0.33 vs a hypothetical
    // lower sea CF), here it is physically correct — no silent-bug
    // surface.
    //
    // Deferred species: "Nitrogen, total (excluding N2)" (reference
    // species at 1.0 kg N-eq per kg N in water) and "Nitrogen oxides"
    // (NOx group flow) are aggregate inventory flows without clean
    // single-substance CAS numbers. Both need `FlowId` or
    // `NameAndCompartment` matchers — out of scope for CAS-first V1.
    // NOx-as-NO2-equivalent at 0.389 kg N-eq/kg is captured by the
    // explicit NO2 factor below, so NOx-labelled inventory flows
    // already route through NO2's CF at data-entry time if the
    // importer normalises "NOx → NO2-equivalent".
    //
    // Seeds: ammonia-to-water = 0.824 (basic, highly bioavailable N
    // species discharged to water); ammonia-to-air = 0.092 (edge,
    // same CAS, ~9x lower because atmospheric NH3 mostly re-deposits
    // on land rather than entering marine systems). The water/air
    // ratio captures `CasCompartment`'s discrimination on a
    // well-studied species.
    let air = || vec!["emission".into(), "air".into()];
    let water = || vec!["emission".into(), "water".into()];
    let cc = |cas: &str, compartment: Vec<String>, value: f64, note: &str| {
        CharacterizationFactor {
            match_on: FactorMatch::CasCompartment {
                cas: cas.into(),
                compartment,
            },
            value,
            note: Some(note.into()),
        }
    };
    vec![
        // Ammonia NH3 (CAS 7664-41-7).
        cc("7664-41-7", water(), 0.824, "EF 3.1 EU-m — NH3 to water (basic seed, highly bioavailable)"),
        cc("7664-41-7", air(), 0.092, "EF 3.1 EU-m — NH3 to air (edge seed, same CAS ~9x lower)"),
        // Ammonium NH4+ (CAS 14798-03-9).
        cc("14798-03-9", water(), 0.778, "EF 3.1 EU-m — NH4+ to water"),
        cc("14798-03-9", air(), 0.087, "EF 3.1 EU-m — NH4+ to air"),
        // Nitrate NO3- (CAS 14797-55-8).
        cc("14797-55-8", water(), 0.226, "EF 3.1 EU-m — NO3- to water"),
        cc("14797-55-8", air(), 0.028, "EF 3.1 EU-m — NO3- to air"),
        // Nitrite NO2- (CAS 14797-65-0) — water-only species in the
        // dataset (no air-emission factors listed).
        cc("14797-65-0", water(), 0.304, "EF 3.1 EU-m — NO2- to water"),
        // Nitrogen dioxide NO2 (CAS 10102-44-0) — air-only.
        cc("10102-44-0", air(), 0.389, "EF 3.1 EU-m — NO2 to air"),
        // Nitrogen monoxide NO (CAS 10102-43-9) — air-only, highest
        // air CF among the single-substance N species.
        cc("10102-43-9", air(), 0.596, "EF 3.1 EU-m — NO to air"),
    ]
}

fn eutrophication_terrestrial_factors() -> Vec<CharacterizationFactor> {
    // Source: JRC EF 3.1 reference package, method dataset
    // `b5c614d2-def3-11e6-bf01-fe55135034f3.xml`
    // "Eutrophication, terrestrial", dataSetVersion 01.02.009,
    // dateOfLastRevision 2016-12-20, reference unit **mol N-eq**
    // (not kg — the Seppälä et al. (2006) accumulated-exceedance
    // model uses molar equivalents). Underlying model: Seppälä et
    // al. (2006), distinct from the CARMEN/EUTREND family used for
    // EU-freshwater and EU-marine.
    //
    // Regionalisation: 575 factors total; 540 country-specific and
    // ~35 pan-European defaults. We take only the pan-European
    // defaults for V1.
    //
    // Compartment scope: EU-terrestrial is an **air-only** category
    // — every factor in the dataset matches air subcompartments
    // (unspecified, non-urban, urban, lower stratosphere, long-term).
    // No water or soil factors. The `CasCompartment` prefix
    // `["emission","air"]` therefore acts as a safety gate: a
    // cataloguer-mistyped "ammonia to water" inventory flow will
    // correctly *not* match an EU-terrestrial factor, surfacing the
    // data-entry error rather than silently feeding N-deposition
    // modelling with a water release.
    //
    // Compartment uniformity: every air subcompartment receives the
    // same CF per species. Two species (ammonium, nitrate) show
    // small rounding variance across subcompartments in the source
    // file (NH4+: 12.72166667 vs 12.7217; NO3-: 3.160645161 vs
    // 3.16065). The longer-precision values appear in the
    // "air, unspecified" primary entries and are adopted here; the
    // shorter-precision values are evident artefacts of JRC
    // spreadsheet truncation rather than physical distinctions.
    //
    // Deferred species: "Nitrogen oxides" (NOx group flow at 4.26
    // mol N-eq/kg) has no clean single-substance CAS — needs
    // `FlowId` or `NameAndCompartment`, out of scope for CAS-first
    // V1. NOx-as-NO2-equivalent at 4.26 is captured by the explicit
    // NO2 factor below.
    //
    // Seeds: NH3-to-air = 13.47 (basic — reference species,
    // highest-CF N emission for terrestrial deposition); NO-to-air
    // = 6.532 (edge — CAS-level discrimination against NO2's 4.26,
    // two chemically similar species with materially different
    // atmospheric fates). Same matcher variant `CasCompartment`
    // binds both: the edge seed exercises CAS-axis selectivity
    // within a fixed compartment.
    let air = || vec!["emission".into(), "air".into()];
    let cc = |cas: &str, value: f64, note: &str| {
        CharacterizationFactor {
            match_on: FactorMatch::CasCompartment {
                cas: cas.into(),
                compartment: air(),
            },
            value,
            note: Some(note.into()),
        }
    };
    vec![
        // Ammonia NH3 (CAS 7664-41-7) — reference species, basic seed.
        cc("7664-41-7", 13.47, "EF 3.1 EU-t — NH3 to air (basic seed, reference species)"),
        // Ammonium NH4+ (CAS 14798-03-9).
        cc("14798-03-9", 12.72166667, "EF 3.1 EU-t — NH4+ to air"),
        // Nitrate NO3- (CAS 14797-55-8).
        cc("14797-55-8", 3.160645161, "EF 3.1 EU-t — NO3- to air"),
        // Nitrite NO2- (CAS 14797-65-0).
        cc("14797-65-0", 4.26, "EF 3.1 EU-t — NO2- to air"),
        // Nitrogen dioxide NO2 (CAS 10102-44-0).
        cc("10102-44-0", 4.26, "EF 3.1 EU-t — NO2 to air"),
        // Nitrogen monoxide NO (CAS 10102-43-9) — edge seed; higher
        // CF than NO2 because a larger molar fraction oxidises &
        // deposits as N before atmospheric export.
        cc("10102-43-9", 6.532, "EF 3.1 EU-t — NO to air (edge seed, CAS-axis vs NO2)"),
    ]
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

    // ---- Acidification seeds ----------------------------------------
    //
    // Per the factor-table entry discipline: each factor-bearing
    // category gets ≥2 seed tests (one basic + one edge). For AC the
    // seeds are SO2 (basic, canonical acidifier) and NH3 (edge, highest
    // CF at 3.02 ≫ SO2 1.31 — the unambiguous ranking catches
    // matcher bugs that same-value-case seeds would miss).

    fn ac_factors() -> Vec<CharacterizationFactor> {
        ef_31()
            .categories
            .into_iter()
            .find(|c| c.id == "acidification")
            .unwrap()
            .factors
    }

    fn ac_factor(cas: &str) -> CharacterizationFactor {
        ac_factors()
            .into_iter()
            .find(|f| matches!(&f.match_on, FactorMatch::CasCompartment { cas: c, .. } if c == cas))
            .unwrap_or_else(|| panic!("no Acidification factor for CAS {cas}"))
    }

    #[test]
    fn ef_31_acidification_so2_seed_value() {
        // Basic seed: SO2 (7446-09-5) = 1.31 mol H+-eq/kg
        // Source: ILCD b5c611c6...xml, `<location/>` entry for
        // sulfur dioxide → air (all subcompartments share the value).
        let f = ac_factor("7446-09-5");
        assert_eq!(f.value, 1.31);
    }

    #[test]
    fn ef_31_acidification_nh3_seed_value() {
        // Edge seed: NH3 (7664-41-7) = 3.02 mol H+-eq/kg — the
        // highest-CF substance in AC, and on a different base species
        // from SO2 (nitrogen vs sulfur). Exercises the matcher's
        // ability to distinguish species while sharing a compartment.
        let f = ac_factor("7664-41-7");
        assert_eq!(f.value, 3.02);
    }

    #[test]
    fn ef_31_acidification_ranking_nh3_greater_than_so2_greater_than_no2() {
        // The published EF 3.1 AC ranking on the three most-cited
        // species. If this ever breaks without a method-version bump,
        // the factor table is wrong.
        assert!(ac_factor("7664-41-7").value > ac_factor("7446-09-5").value);
        assert!(ac_factor("7446-09-5").value > ac_factor("10102-44-0").value);
    }

    #[test]
    fn ef_31_acidification_matchers_target_air_compartment() {
        // AC is an air-emission category. Every factor must be a
        // `CasCompartment` matcher prefixed on `["emission", "air"]`.
        // This is the load-bearing invariant — without it, SO2 to
        // water would receive an acidification CF (it must not).
        for f in ac_factors() {
            match &f.match_on {
                FactorMatch::CasCompartment { compartment, .. } => {
                    assert_eq!(
                        compartment,
                        &vec!["emission".to_string(), "air".to_string()],
                        "AC factor must match `emission/air` prefix"
                    );
                }
                other => panic!("AC factor has non-CasCompartment matcher: {other:?}"),
            }
        }
    }

    // ---- Ozone depletion seeds --------------------------------------
    //
    // Basic: CFC-11 (75-69-4) = 1.0 (reference species by definition).
    // Edge: Halon-2402 (124-73-2) = 15.7 (highest ODP in the table —
    // 15× the reference, catches matcher bugs that mid-range seeds
    // would miss).

    fn od_factors() -> Vec<CharacterizationFactor> {
        ef_31()
            .categories
            .into_iter()
            .find(|c| c.id == "ozone-depletion")
            .unwrap()
            .factors
    }

    fn od_factor(cas: &str) -> CharacterizationFactor {
        od_factors()
            .into_iter()
            .find(|f| matches!(&f.match_on, FactorMatch::Cas { cas: c } if c == cas))
            .unwrap_or_else(|| panic!("no OD factor for CAS {cas}"))
    }

    #[test]
    fn ef_31_od_cfc11_is_reference_species() {
        // CFC-11 (trichlorofluoromethane, CAS 75-69-4) = 1.0 by
        // definition. If this value ever drifts, the unit of the
        // category has changed and the preset has a data-entry bug.
        assert_eq!(od_factor("75-69-4").value, 1.0);
    }

    #[test]
    fn ef_31_od_halon_2402_edge_seed_value() {
        // Halon-2402 (dibromotetrafluoroethane, CAS 124-73-2) = 15.7.
        // Highest-ODP species in the EF 3.1 table.
        assert_eq!(od_factor("124-73-2").value, 15.7);
    }

    #[test]
    fn ef_31_od_halon_ranking_exceeds_cfc() {
        // Halons exceed CFCs by an order of magnitude — physical
        // intuition baked into the Montreal Protocol's phase-out
        // urgency. Halon-2402 > Halon-1301 > Halon-1211 > CFC-11.
        assert!(od_factor("124-73-2").value > od_factor("75-63-8").value);
        assert!(od_factor("75-63-8").value > od_factor("353-59-3").value);
        assert!(od_factor("353-59-3").value > od_factor("75-69-4").value);
    }

    #[test]
    fn ef_31_od_matchers_are_cas_only() {
        // OD has no compartment or origin load-bearing axis. Every
        // factor must be a plain `Cas` matcher. If this ever fails,
        // either the dataset changed (OD exposes a new axis — time
        // for a taxonomy check) or a factor entry has drifted away
        // from the authorship discipline.
        for f in od_factors() {
            assert!(
                matches!(f.match_on, FactorMatch::Cas { .. }),
                "OD factor must be a plain CAS matcher: {:?}",
                f.match_on
            );
        }
    }

    // ---- POCP seeds -------------------------------------------------

    fn pocp_factors() -> Vec<CharacterizationFactor> {
        ef_31()
            .categories
            .into_iter()
            .find(|c| c.id == "photochemical-ozone-formation")
            .unwrap()
            .factors
    }

    fn pocp_factor(cas: &str) -> CharacterizationFactor {
        pocp_factors()
            .into_iter()
            .find(|f| matches!(&f.match_on, FactorMatch::CasCompartment { cas: c, .. } if c == cas))
            .unwrap_or_else(|| panic!("no POCP factor for CAS {cas}"))
    }

    #[test]
    fn ef_31_pocp_ethylene_basic_seed_value() {
        // Ethylene (CAS 74-85-1) = 1.69 — classic POCP reference
        // species in tropospheric-ozone photochemistry.
        assert_eq!(pocp_factor("74-85-1").value, 1.69);
    }

    #[test]
    fn ef_31_pocp_trimethylbenzene_edge_seed_value() {
        // 1,3,5-Trimethylbenzene (CAS 108-67-8) = 2.33 — highest CF
        // in the EF 3.1 POCP table (pan-European defaults).
        assert_eq!(pocp_factor("108-67-8").value, 2.33);
    }

    #[test]
    fn ef_31_pocp_ranking_mesitylene_greater_than_propene_greater_than_methanol() {
        // Structural intuition: substituted aromatic > olefin >
        // oxygenated alcohol. If this ever inverts without a method-
        // version bump, a factor value has drifted.
        assert!(pocp_factor("108-67-8").value > pocp_factor("115-07-1").value);
        assert!(pocp_factor("115-07-1").value > pocp_factor("67-56-1").value);
    }

    #[test]
    fn ef_31_pocp_matchers_target_air_compartment() {
        for f in pocp_factors() {
            match &f.match_on {
                FactorMatch::CasCompartment { compartment, .. } => {
                    assert_eq!(
                        compartment,
                        &vec!["emission".to_string(), "air".to_string()],
                        "POCP factor must match `emission/air` prefix"
                    );
                }
                other => panic!("POCP factor has non-CasCompartment matcher: {other:?}"),
            }
        }
    }

    fn eu_fw_factor(cas: &str, compartment_tail: &str) -> CharacterizationFactor {
        eutrophication_freshwater_factors()
            .into_iter()
            .find(|f| match &f.match_on {
                FactorMatch::CasCompartment { cas: c, compartment } => {
                    c == cas && compartment.get(1).map(String::as_str) == Some(compartment_tail)
                }
                _ => false,
            })
            .unwrap_or_else(|| panic!("no EU-fw factor for CAS {cas} / {compartment_tail}"))
    }

    #[test]
    fn ef_31_eu_fw_phosphorus_to_water_basic_seed_value() {
        // Elemental P (CAS 7723-14-0) to water = 1.0 — reference
        // species for kg P-eq. Every other EU-fw CF is a ratio against
        // this. If this drifts, the whole category's units are wrong.
        assert_eq!(eu_fw_factor("7723-14-0", "water").value, 1.0);
    }

    #[test]
    fn ef_31_eu_fw_phosphorus_to_soil_edge_seed_value() {
        // Same CAS, different compartment — P-to-soil = 0.05 (20x
        // lower than P-to-water). This is the canonical
        // `CasCompartment` case: one CAS, distinct CFs per
        // compartment. A plain `Cas` matcher could not represent this
        // without collision, which is why the variant exists.
        assert_eq!(eu_fw_factor("7723-14-0", "soil").value, 0.05);
    }

    #[test]
    fn ef_31_eu_fw_phosphorus_water_ranking_exceeds_soil() {
        // Structural invariant: for every EU-fw species with both
        // compartment variants, the water CF exceeds the soil CF
        // (P released directly to water is more bioavailable to
        // freshwater eutrophication than P deposited on soil, which
        // only partially leaches to water). If any soil >= water
        // inversion appears, the CARMEN/EUTREND transfer factor has
        // been mis-transcribed.
        assert!(eu_fw_factor("7723-14-0", "water").value > eu_fw_factor("7723-14-0", "soil").value);
        assert!(eu_fw_factor("14265-44-2", "water").value > eu_fw_factor("14265-44-2", "soil").value);
        assert!(eu_fw_factor("7664-38-2", "water").value > eu_fw_factor("7664-38-2", "soil").value);
    }

    #[test]
    fn ef_31_eu_fw_matchers_target_water_or_soil_compartment() {
        for f in eutrophication_freshwater_factors() {
            match &f.match_on {
                FactorMatch::CasCompartment { compartment, .. } => {
                    let tail = compartment.get(1).map(String::as_str);
                    assert!(
                        compartment.first().map(String::as_str) == Some("emission")
                            && (tail == Some("water") || tail == Some("soil")),
                        "EU-fw factor must match `emission/water` or `emission/soil` prefix, got {compartment:?}"
                    );
                }
                other => panic!("EU-fw factor has non-CasCompartment matcher: {other:?}"),
            }
        }
    }

    fn eu_m_factor(cas: &str, compartment_tail: &str) -> CharacterizationFactor {
        eutrophication_marine_factors()
            .into_iter()
            .find(|f| match &f.match_on {
                FactorMatch::CasCompartment { cas: c, compartment } => {
                    c == cas && compartment.get(1).map(String::as_str) == Some(compartment_tail)
                }
                _ => false,
            })
            .unwrap_or_else(|| panic!("no EU-m factor for CAS {cas} / {compartment_tail}"))
    }

    #[test]
    fn ef_31_eu_m_ammonia_to_water_basic_seed_value() {
        // NH3 (CAS 7664-41-7) to water = 0.824 — dominant bioavailable
        // N emission to marine systems among the single-substance
        // species that have CAS numbers.
        assert_eq!(eu_m_factor("7664-41-7", "water").value, 0.824);
    }

    #[test]
    fn ef_31_eu_m_ammonia_to_air_edge_seed_value() {
        // Same CAS, air compartment — 0.092, ~9x lower than the
        // water factor. This water/air split is the canonical
        // `CasCompartment` case for EU-marine and mirrors the
        // EU-freshwater P water/soil seed pair.
        assert_eq!(eu_m_factor("7664-41-7", "air").value, 0.092);
    }

    #[test]
    fn ef_31_eu_m_water_exceeds_air_for_each_two_compartment_species() {
        // For every species where both compartments are entered
        // (NH3, NH4+, NO3-), the water CF exceeds the air CF. If any
        // inversion appears, the CARMEN marine transfer factor has
        // been mis-transcribed.
        assert!(eu_m_factor("7664-41-7", "water").value > eu_m_factor("7664-41-7", "air").value);
        assert!(eu_m_factor("14798-03-9", "water").value > eu_m_factor("14798-03-9", "air").value);
        assert!(eu_m_factor("14797-55-8", "water").value > eu_m_factor("14797-55-8", "air").value);
    }

    #[test]
    fn ef_31_eu_m_matchers_target_air_or_water_compartment() {
        for f in eutrophication_marine_factors() {
            match &f.match_on {
                FactorMatch::CasCompartment { compartment, .. } => {
                    let tail = compartment.get(1).map(String::as_str);
                    assert!(
                        compartment.first().map(String::as_str) == Some("emission")
                            && (tail == Some("air") || tail == Some("water")),
                        "EU-m factor must match `emission/air` or `emission/water` prefix, got {compartment:?}"
                    );
                }
                other => panic!("EU-m factor has non-CasCompartment matcher: {other:?}"),
            }
        }
    }

    fn eu_t_factor(cas: &str) -> CharacterizationFactor {
        eutrophication_terrestrial_factors()
            .into_iter()
            .find(|f| matches!(&f.match_on, FactorMatch::CasCompartment { cas: c, .. } if c == cas))
            .unwrap_or_else(|| panic!("no EU-t factor for CAS {cas}"))
    }

    #[test]
    fn ef_31_eu_t_ammonia_basic_seed_value() {
        // NH3 (CAS 7664-41-7) = 13.47 mol N-eq/kg — reference species
        // for terrestrial N deposition in EF 3.1. If this drifts,
        // the whole category's scale is wrong (every other CF is a
        // ratio against this within Seppälä's AE framework).
        assert_eq!(eu_t_factor("7664-41-7").value, 13.47);
    }

    #[test]
    fn ef_31_eu_t_no_vs_no2_edge_seed_value() {
        // CAS-axis edge: NO (10102-43-9) = 6.532 > NO2
        // (10102-44-0) = 4.26, despite both being "NOx" family. The
        // difference reflects NO's larger molar N-deposition
        // fraction in the Seppälä model. A plain `Cas` matcher
        // cannot fold these into one; the distinct entries are
        // load-bearing.
        assert_eq!(eu_t_factor("10102-43-9").value, 6.532);
        assert_eq!(eu_t_factor("10102-44-0").value, 4.26);
        assert!(eu_t_factor("10102-43-9").value > eu_t_factor("10102-44-0").value);
    }

    #[test]
    fn ef_31_eu_t_matchers_target_air_compartment_only() {
        // EU-terrestrial is air-only in EF 3.1. Any non-air
        // compartment matcher would be a category-scope bug.
        for f in eutrophication_terrestrial_factors() {
            match &f.match_on {
                FactorMatch::CasCompartment { compartment, .. } => {
                    assert_eq!(
                        compartment,
                        &vec!["emission".to_string(), "air".to_string()],
                        "EU-t factor must match `emission/air` prefix"
                    );
                }
                other => panic!("EU-t factor has non-CasCompartment matcher: {other:?}"),
            }
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
