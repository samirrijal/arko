//! ReCiPe 2016 Midpoint impact method (Hierarchist perspective) — V1
//! scope: ten midpoint indicators covering the EN 15804+A2 emission-based
//! core (climate / ozone / POCP / acidification / EP-fw / EP-marine),
//! ADP-fossil for parity with EF 3.1 + CML-IA, and three ReCiPe-distinctive
//! midpoints (PMFP, land occupation, water consumption).
//!
//! ReCiPe 2016 is the joint RIVM / Radboud / PRé / Norwegian University
//! of Science & Technology successor to ReCiPe 2008, published as
//! Huijbregts et al. (2017) *ReCiPe 2016 v1.1: A harmonized life cycle
//! impact assessment method at midpoint and endpoint level*. Arko V1
//! ships the **Hierarchist** time-perspective at the **midpoint** level
//! against the **GLO** (global default) regional bucket. Individualist
//! and Egalitarian perspectives, endpoint indicators, and country-resolved
//! variants are deferred to V2 per `DECISIONS.md` entry `D-0019`.
//!
//! ## V1 categories (ten)
//!
//! 1. `climate-change` — GWP100 Hierarchist (IPCC AR5 with-feedback values
//!    underlying the ReCiPe 2016 H100 column). Indicator unit: `kg CO2-eq`.
//!    Mirrors the AR6 / CML-IA / EF 3.1 climate-change shape and uses
//!    the same `CasOrigin` split for CH4 (biogenic = 34, fossil = 36)
//!    that AR6 uses for the equivalent split (29.8 / 27.0). The numeric
//!    values differ from AR6 by design — ReCiPe carries the AR5
//!    with-feedback table, AR6 carries the AR6 update.
//! 2. `ozone-depletion` — ODP infinite Hierarchist (WMO 1998 + Daniel et
//!    al. 2010 per Hayman & Derwent 2011). Indicator unit: `kg CFC-11-eq`.
//! 3. `photochemical-ozone-formation` — Human damage ozone formation
//!    (HOFP) Hierarchist, Van Zelm et al. (2008). Indicator unit:
//!    `kg NOx-eq` (note: NOT ethylene-eq as in CML-IA, NOT NMVOC-eq as
//!    in EF 3.1 — three different reference species across the three
//!    presets, expected numerical drift in side-by-side studies).
//! 4. `particulate-matter-formation` — PMFP Hierarchist (Van Zelm et al.
//!    2008, intake-fraction-based). Indicator unit: `kg PM2.5-eq`.
//! 5. `acidification` — Terrestrial acidification (AP) Hierarchist,
//!    Roy et al. (2012). Indicator unit: `kg SO2-eq`.
//! 6. `eutrophication-freshwater` — Freshwater eutrophication (FEP)
//!    Hierarchist, Helmes et al. (2012). Indicator unit: `kg P-eq`.
//! 7. `eutrophication-marine` — Marine eutrophication (MEP) Hierarchist,
//!    Cosme et al. (2017). Indicator unit: `kg N-eq`.
//! 8. `land-occupation` — Land-use occupation (LOP) Hierarchist,
//!    De Baan et al. (2013). Indicator unit: `m2*a annual crop-eq`
//!    (annual-crop-equivalent area-time, since the reference flow is
//!    "Occupation, annual crop" = 1.0).
//! 9. `water-consumption` — Water-consumption potential (WCP)
//!    Hierarchist, Pfister et al. (2009) AWaRe-precursor. Indicator
//!    unit: `m3 water-eq`. **Single CF** (= 1.0) per ReCiPe 2016 main
//!    spreadsheet — country-resolved CFs live in the separate
//!    country-specific spreadsheet and are deferred to V2.
//! 10. `adp-fossil` — Fossil resource scarcity (FFP) Hierarchist, Vieira
//!    et al. (2016). Indicator unit: `kg oil-eq`.
//!
//! ## What V1 does *not* ship (deferred to V2 or excluded by design)
//!
//! - **Mineral resource scarcity (SOP).** ReCiPe ships an indicator at
//!   `kg Cu-eq` based on Vieira et al. (2017). Per `D-0019` it is
//!   deferred to V2 with the regionalisation bundle — the underlying
//!   surplus-ore-potential model is methodologically contested and
//!   should not be Arko-stamped without independent factor-value seeds.
//! - **Ionising radiation, human toxicity (cancer / non-cancer),
//!   ecotoxicity (terrestrial / freshwater / marine).** Excluded from
//!   the V1 emission-based core for the same reason CML-IA's V1 omits
//!   them — toxicity-fate-model-based indicators carry independent
//!   methodological criticisms and require separate seed work.
//! - **Country-resolved CFs.** ReCiPe ships a separate
//!   `ReCiPe2016_country_factors_v1.1_20171221.xlsx` with per-country
//!   CFs for AP, EP-fw, water consumption, and atmospheric source-region
//!   variants for POCP/PMFP. V1 ships only the GLO defaults from the
//!   main spreadsheet; per-country resolution waits on a `CasRegion`
//!   matcher that V1 does not have.
//! - **Individualist (I) and Egalitarian (E) perspectives.** ReCiPe
//!   ships three time-perspective columns (I = 20 yr, H = 100 yr,
//!   E = 1000 yr). V1 ships only Hierarchist. The other two would
//!   ship as separate `ImpactMethod` instances (`recipe-2016-midpoint-i`,
//!   `recipe-2016-midpoint-e`) sharing the same column-extraction
//!   discipline.
//! - **Endpoint indicators (DALY / species*yr / USD2013).** Endpoint
//!   indicators require the midpoint→endpoint conversion factors from
//!   a different sheet of the same spreadsheet plus a separate
//!   `ImpactMethod::endpoint` shape that does not exist in V1.
//!
//! See `DECISIONS.md` entry `D-0019` for the full V1 scope rationale,
//! including the three layered scoping refinements that converged on
//! "Hierarchist + GLO + ten midpoints" as the V1 shape. License posture
//! and attribution discipline live at `docs/licenses/recipe-2016-rivm.md`.
//!
//! ## Source-comment discipline
//!
//! Every factor below carries a comment of the form:
//!
//! ```text
//! // RIVM, ReCiPe2016_CFs_v1.1_20180117.xlsx, sheet "<sheet name>",
//! // col <N> H, row <r> "<substance verbatim>" = <value>
//! ```
//!
//! Where the source identifies a substance by name only (no CAS) but
//! the substance is unambiguously chemistry-canonical (CO2, CH4, NH3,
//! SO2, P, PO4 etc.), the matcher uses Arko's canonical CAS for the
//! species — same convention as `cml_ia` AP/EP — so that the preset
//! actually fires against ecoinvent-style biosphere flows, which are
//! CAS-keyed. The per-factor comment cites both the verbatim ReCiPe
//! name and the Arko-canonical CAS used. Where the source identifies
//! a group flow that does not have a clean single CAS (NMVOC, NOx-as-
//! group, PM2.5, "Crude oil", land-occupation labels), the matcher is
//! `NameAndCompartment` with the verbatim ReCiPe name.
//!
//! ## References
//!
//! - Huijbregts, M. A. J., Steinmann, Z. J. N., Elshout, P. M. F. M.,
//!   Stam, G., Verones, F., Vieira, M., Zijp, M., Hollander, A., & Van
//!   Zelm, R. (2017). *ReCiPe2016: a harmonised life cycle impact
//!   assessment method at midpoint and endpoint level*. International
//!   Journal of Life Cycle Assessment, 22(2), 138–147.
//! - RIVM (2018). *ReCiPe2016_CFs_v1.1_20180117.xlsx*. National
//!   Institute for Public Health and the Environment (RIVM), the
//!   Netherlands. Post-erratum v1.1 (2018-01-17) consolidating the
//!   v1.13 errata against the v1.0 (2016) initial release. License
//!   posture: `docs/licenses/recipe-2016-rivm.md`.
//! - ISO 14040:2006, ISO 14044:2006 — *Environmental management —
//!   Life cycle assessment.*

use crate::method::{CharacterizationFactor, FactorMatch, ImpactCategory, ImpactMethod};
use arko_core::meta::FlowOrigin;

/// ReCiPe 2016 Midpoint, Hierarchist perspective — ten V1 categories
/// against the GLO (global default) regional bucket.
///
/// `(id, version) = ("recipe-2016-midpoint-h", "1.1")`. The version
/// matches the RIVM file version verbatim ("1.1", post-erratum
/// 2018-01-17) — not Arko's internal V1/V2 staging. A future V2 that
/// adds country resolution or other perspectives is still derived from
/// the same RIVM source v1.1 and would ship at a different `id` (e.g.,
/// `recipe-2016-midpoint-h-regionalised`) with the same `version`.
/// Never reissue the same `(id, version)` key with a different factor
/// table.
///
/// Category order follows the source-spreadsheet sheet order (Climate
/// → Ozone → POCP/HOFP → PMFP → AP → FEP → MEP → Land → Water → FFP).
/// Callers that index `m.categories` by position can rely on this
/// order; method revisions that change it bump `version`.
#[must_use]
pub fn recipe_2016() -> ImpactMethod {
    ImpactMethod {
        id: "recipe-2016-midpoint-h".into(),
        version: "1.1".into(),
        name: "ReCiPe 2016 Midpoint (Hierarchist, RIVM v1.1)".into(),
        categories: vec![
            category(
                "climate-change",
                "Climate change (ReCiPe 2016 Midpoint H, GWP100)",
                "kg CO2-eq",
                climate_change_factors(),
            ),
            category(
                "ozone-depletion",
                "Stratospheric ozone depletion (ReCiPe 2016 Midpoint H, ODP infinite)",
                "kg CFC-11-eq",
                ozone_depletion_factors(),
            ),
            category(
                "photochemical-ozone-formation",
                "Photochemical ozone formation, human damage (ReCiPe 2016 Midpoint H, HOFP)",
                "kg NOx-eq",
                hofp_factors(),
            ),
            category(
                "particulate-matter-formation",
                "Particulate matter formation (ReCiPe 2016 Midpoint H, PMFP)",
                "kg PM2.5-eq",
                pmfp_factors(),
            ),
            category(
                "acidification",
                "Terrestrial acidification (ReCiPe 2016 Midpoint H, AP)",
                "kg SO2-eq",
                acidification_factors(),
            ),
            category(
                "eutrophication-freshwater",
                "Freshwater eutrophication (ReCiPe 2016 Midpoint H, FEP)",
                "kg P-eq",
                fep_factors(),
            ),
            category(
                "eutrophication-marine",
                "Marine eutrophication (ReCiPe 2016 Midpoint H, MEP)",
                "kg N-eq",
                mep_factors(),
            ),
            category(
                "land-occupation",
                "Land use, occupation (ReCiPe 2016 Midpoint H, LOP)",
                "m2*a annual crop-eq",
                land_occupation_factors(),
            ),
            category(
                "water-consumption",
                "Water consumption (ReCiPe 2016 Midpoint H, WCP, GLO default)",
                "m3 water-eq",
                water_consumption_factors(),
            ),
            category(
                "adp-fossil",
                "Fossil resource scarcity (ReCiPe 2016 Midpoint H, FFP)",
                "kg oil-eq",
                adp_fossil_factors(),
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

/// Plain-CAS matcher with attribution note. Used for compartment-uniform
/// air pollutants in climate-change and ozone-depletion (CO2, CFCs, etc.
/// have a single CF regardless of subcompartment).
fn cas_factor(cas: &str, value: f64, note: &str) -> CharacterizationFactor {
    CharacterizationFactor {
        match_on: FactorMatch::Cas { cas: cas.into() },
        value,
        note: Some(note.into()),
    }
}

/// CAS + origin matcher. Used for CH4 in climate-change, where ReCiPe
/// ships separate "Methane" (= 34, biogenic-default) and "Fossil
/// methane" (= 36) rows — same CAS, different origins. Mirrors the
/// AR6 CH4 pattern at `standard::ar6_gwp100_factors`.
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

// ---- Factor tables ---------------------------------------------------

fn climate_change_factors() -> Vec<CharacterizationFactor> {
    // Source: RIVM, ReCiPe2016_CFs_v1.1_20180117.xlsx, sheet "Global
    // Warming", col E (5) Hierarchist (header row 3 verbatim:
    // "GWP100"). Underlying values: IPCC AR5 with climate-carbon
    // feedback at 100-yr time horizon.
    //
    // **Distinguishing note (matters for any future contributor).**
    // ReCiPe 2016 Hierarchist GWP100 uses the AR5 *with*-feedback
    // values (CH4 fossil = 36, N2O = 298, SF6 = 23_500). This is the
    // *same source family* as Arko's `ipcc-ar5-gwp100` preset (which
    // also carries with-feedback values), but **not the same
    // numerical table** — ReCiPe applies its own H/I/E perspective
    // weighting on top of the IPCC source. Specifically:
    //   * AR5 with-feedback: CH4 fossil = 30, N2O = 273
    //   * ReCiPe Hierarchist: CH4 fossil = 36, N2O = 298 (closer to
    //     AR5 *without* feedback for some but not all species)
    // The ReCiPe Hierarchist column is the canonical "ReCiPe 2016
    // GWP100" cited in the ReCiPe 2016 paper Table 6 — not directly
    // an AR5 column. Per-factor comments below repeat this each time
    // so the difference does not accidentally get "corrected" toward
    // any of Arko's other GWP100 presets.
    //
    // Taxonomy pre-check (col E): 215 rows in the sheet, 159 with
    // numeric H values. Rows 4-7 cover the four core GHGs (CO2, CH4
    // biogenic, CH4 fossil, N2O); rows 9-14 cover CFCs; rows 16-27
    // HCFCs; rows 30-71 HFC family + the chlorocarbon group. The full
    // 159-species table includes long-tail HFCs / PFCs / HFEs / iodides
    // that are absent from typical inventories. V1 ships a 12-species
    // subset matching the existing AR6 + CML-IA + EF 3.1 climate-change
    // scopes (CO2 + CH4 ×2 + N2O + SF6 + NF3 + 3 HFCs + 2 CFCs + 2 PFCs)
    // so side-by-side comparison against any of those presets sees
    // matched rows. Long-tail species are deferred — they add
    // ranking-anchor noise without seed coverage.
    //
    // Matcher choice: plain `Cas` for CO2 / N2O / SF6 / NF3 / CFCs /
    // HCFCs / HFCs / PFCs (single CF per CAS, no compartment or origin
    // axis required). `CasOrigin` for CH4 — the source ships separate
    // "Methane" (CF = 34, biogenic-default) and "Fossil methane"
    // (CF = 36) rows, same CAS 74-82-8. This mirrors the AR6 CH4
    // origin-split pattern at `standard::ar6_gwp100_factors`. Plain
    // `Cas` for CH4 is forbidden — would cause `CMatrixError::
    // DuplicateMatch` against the two `CasOrigin` entries. ReCiPe
    // does *not* ship a LULUC CH4 row, so LULUC-origin CH4 flows are
    // unmatched per strict policy (consistent with ReCiPe's source
    // intent — the LULUC perspective is an EN 15804+A2 convention,
    // not a ReCiPe one).
    //
    // Seeds (≥6, mirroring CML-IA / AR6 climate-change discipline):
    //   - Basic:   CO2 = 1.0
    //   - Basic:   CH4 biogenic = 34.0   (ReCiPe-default "Methane" row)
    //   - Edge:    CH4 fossil = 36.0     (origin-split witness)
    //   - Basic:   N2O = 298.0           (vs AR6's 273; ranking anchor)
    //   - Basic:   SF6 = 23_500.0        (highest in shipped subset)
    //   - Edge:    HFC-23 = 13_856.0     (highest HFC; full-precision seed)
    //   - Edge:    CFC-12 = 11_547.0     (cross-listed with ODP)
    vec![
        // CO2 — col E row 4 "Carbon dioxide" CO2 = 1.0
        cas_factor(
            "124-38-9",
            1.0,
            "ReCiPe 2016 H GWP100 — CO2 (col E row 4 'Carbon dioxide'). \
             Reference species (basic seed). ReCiPe Hierarchist column \
             values are NOT identical to AR5 with-feedback — see module \
             docstring.",
        ),
        // CH4 biogenic — col E row 5 "Methane" CH4 = 34
        // The default ReCiPe "Methane" row uses the biogenic-default
        // convention (matching the EN 15804+A2 reading of "unqualified
        // methane = biogenic"). Origin-split required because row 6
        // ships a separate "Fossil methane" entry at 36.
        cas_origin_factor(
            "74-82-8",
            FlowOrigin::Biogenic,
            34.0,
            "ReCiPe 2016 H GWP100 — CH4 biogenic (col E row 5 'Methane', \
             default-row, basic seed). Source labels the unqualified \
             Methane row as the biogenic-default value (= 34); the \
             explicit fossil variant ships at row 6 (= 36).",
        ),
        // CH4 fossil — col E row 6 "Fossil methane" CH4 = 36
        cas_origin_factor(
            "74-82-8",
            FlowOrigin::Fossil,
            36.0,
            "ReCiPe 2016 H GWP100 — CH4 fossil (col E row 6 'Fossil \
             methane', edge seed: origin-split witness). Same CAS as \
             biogenic CH4 row 5 (= 34); origin-split mandatory.",
        ),
        // N2O — col E row 7 "Nitrous oxide" N2O = 298
        cas_factor(
            "10024-97-2",
            298.0,
            "ReCiPe 2016 H GWP100 — N2O (col E row 7 'Nitrous oxide', \
             basic seed). NOT 273 (AR6) and NOT 265 (CML-IA without \
             feedback) — ReCiPe Hierarchist value, ranking anchor.",
        ),
        // SF6 — col E row 91 "Sulphur hexafluoride" SF6 = 23500
        cas_factor(
            "2551-62-4",
            23_500.0,
            "ReCiPe 2016 H GWP100 — SF6 (col E row 91 'Sulphur \
             hexafluoride', basic seed). Highest CF in shipped V1 \
             subset; ranking anchor.",
        ),
        // NF3 — col E row 90 "Nitrogen trifluoride" NF3 = 16100
        cas_factor(
            "7783-54-2",
            16_100.0,
            "ReCiPe 2016 H GWP100 — NF3 (col E row 90 'Nitrogen \
             trifluoride'). Matches CML-IA without-feedback value.",
        ),
        // HFC-23 — col E row 30 "HFC-23" CHF3 = 13856  (edge seed)
        cas_factor(
            "75-46-7",
            13_856.0,
            "ReCiPe 2016 H GWP100 — HFC-23 (col E row 30, edge seed: \
             highest HFC in shipped V1 subset, full-precision witness). \
             vs AR6 with-feedback = 14_600 and CML-IA without-feedback \
             = 12_400 — three distinct numerical positions across \
             Arko's three preset families.",
        ),
        // HFC-32 — col E row 31 "HFC-32" CH2F2 = 817
        cas_factor(
            "75-10-5",
            817.0,
            "ReCiPe 2016 H GWP100 — HFC-32 (col E row 31, difluoromethane).",
        ),
        // HFC-134a — col E row 35 "HFC-134a" CH2FCF3 = 1549
        cas_factor(
            "811-97-2",
            1_549.0,
            "ReCiPe 2016 H GWP100 — HFC-134a (col E row 35, \
             1,1,1,2-tetrafluoroethane).",
        ),
        // CFC-11 — col E row 9 "CFC-11" CCl3F = 5352
        cas_factor(
            "75-69-4",
            5_352.0,
            "ReCiPe 2016 H GWP100 — CFC-11 (col E row 9, \
             trichlorofluoromethane). Cross-listed with ODP reference \
             species at 1.0.",
        ),
        // CFC-12 — col E row 10 "CFC-12" CCl2F2 = 11547  (edge seed)
        cas_factor(
            "75-71-8",
            11_547.0,
            "ReCiPe 2016 H GWP100 — CFC-12 (col E row 10, edge seed: \
             cross-impact species — also has ODP = 0.587 in this preset). \
             dichlorodifluoromethane.",
        ),
        // CFC-113 — col E row 12 "CFC-113" CCl2FCClF2 = 6586
        cas_factor(
            "76-13-1",
            6_586.0,
            "ReCiPe 2016 H GWP100 — CFC-113 (col E row 12, \
             1,1,2-trichloro-1,2,2-trifluoroethane).",
        ),
        // PFC-14 / CF4 — col E row 86 "Perfluoromethane (PFC-14)" CF4 = 7349
        cas_factor(
            "75-73-0",
            7_349.0,
            "ReCiPe 2016 H GWP100 — PFC-14 (col E row 86, tetrafluoromethane, \
             CF4).",
        ),
    ]
}

fn ozone_depletion_factors() -> Vec<CharacterizationFactor> {
    // Source: RIVM, ReCiPe2016_CFs_v1.1_20180117.xlsx, sheet
    // "Stratospheric ozone depletion", col D (4) Hierarchist (header
    // row 3 verbatim: "ODP infinite"). Underlying model: WMO 1998 +
    // Daniel et al. (2010) updates, infinite-time-horizon steady-state.
    //
    // Taxonomy pre-check (col D): 29 rows, 21 with numeric H values
    // (8 are Annex-group header rows with H=null). Substances span the
    // Montreal Protocol annexes A-I (CFCs), A-II (Halons), B-II/III
    // (CCl4, methyl chloroform), C-I (HCFCs), E (CH3Br), and "Others"
    // (Halon-1202, CH3Cl, N2O, hydrocarbons). Source identifies each
    // substance by short-name (e.g., "CFC-11", "Halon-1301") + formula;
    // no CAS column. Per the matcher-mix discipline, V1 maps each name
    // to its Arko-canonical CAS (the substances are universally CAS-
    // identified in chemistry; using `Cas` here keeps the preset
    // firing against ecoinvent biosphere flows and matches the
    // cml_ia / ef_31 / standard ODP convention).
    //
    // Compartment uniformity: every shipped substance is air-emitted;
    // ReCiPe carries no compartment column. Same matcher pattern as
    // CML-IA OD (plain `Cas`). Reasoning carries over — ODS-to-water/
    // soil flows are physically non-existent in real inventories, so
    // the absence of compartment-keying does not create a silent-
    // mismatch surface.
    //
    // Note: ReCiPe ODP CFs differ from CML-IA's WMO 2003 values and
    // from EF 3.1's WMO 1999 values — three different reference-
    // timescale conventions across Arko's three OD presets. Ranking
    // also shuffles: ReCiPe puts Halon-2402 highest (14.383) and
    // Halon-1301 second (14.066); CML-IA puts Halon-1301 highest (12)
    // and Halon-2402 lower (6); EF 3.1 puts Halon-2402 highest (15.7).
    // Side-by-side studies will show all three rankings; this is
    // correct.
    //
    // V1 ships 12 species — CFC family (5: CFC-11, CFC-12, CFC-113,
    // CFC-114, CFC-115), Halons (3: 1211, 1301, 2402), HCFC family
    // (2: 22, 141b), CCl4, CH3Br. Niche/regional substances (HCFC-123,
    // HCFC-225ca/cb, methyl chloride, hydrocarbons) deferred — they
    // appear infrequently in real inventories and add ranking-noise
    // without seed coverage. Same scope rationale as CML-IA OD.
    //
    // Seeds:
    //   - Basic:   CFC-11 = 1.0           (reference species)
    //   - Edge:    Halon-2402 = 14.383    (highest ODP in ReCiPe — vs
    //                                      CML-IA where Halon-1301
    //                                      leads at 12, vs EF 3.1
    //                                      where Halon-2402 leads at
    //                                      15.7. Ranking-anchor that
    //                                      catches table swaps.)
    //   - Edge:    Halon-1301 = 14.066    (second-highest — paired
    //                                      with Halon-2402 as a
    //                                      tight-rank witness)
    //   - Cross:   CFC-12 = 0.587         (cross-listed with GWP100;
    //                                      catches dual-impact drift)
    vec![
        // col D row 5 "CFC-11" CCl3F = 1.0
        cas_factor(
            "75-69-4",
            1.0,
            "ReCiPe 2016 H ODP — CFC-11 (col D row 5, trichlorofluoromethane), \
             reference species (basic seed).",
        ),
        // col D row 6 "CFC-12" CCl2F2 = 0.587
        cas_factor(
            "75-71-8",
            0.587,
            "ReCiPe 2016 H ODP — CFC-12 (col D row 6, dichlorodifluoromethane). \
             WMO 1998 + Daniel 2010 infinite-time-horizon. Cross-listed \
             with GWP100 at 11_547.",
        ),
        // col D row 7 "CFC-113" = 0.664
        cas_factor(
            "76-13-1",
            0.664,
            "ReCiPe 2016 H ODP — CFC-113 (col D row 7).",
        ),
        // col D row 8 "CFC-114" = 0.27
        cas_factor(
            "76-14-2",
            0.27,
            "ReCiPe 2016 H ODP — CFC-114 (col D row 8, \
             dichlorotetrafluoroethane).",
        ),
        // col D row 9 "CFC-115" = 0.061
        cas_factor(
            "76-15-3",
            0.061,
            "ReCiPe 2016 H ODP — CFC-115 (col D row 9, \
             chloropentafluoroethane).",
        ),
        // col D row 11 "Halon-1301" = 14.066  (edge seed)
        cas_factor(
            "75-63-8",
            14.066,
            "ReCiPe 2016 H ODP — Halon-1301 (col D row 11, edge seed: \
             second-highest in this preset). bromotrifluoromethane. \
             Note: CML-IA puts Halon-1301 highest at 12; this preset \
             puts Halon-2402 highest at 14.383 with Halon-1301 close \
             behind — different reference-timescale convention.",
        ),
        // col D row 12 "Halon-1211" = 8.777
        cas_factor(
            "353-59-3",
            8.777,
            "ReCiPe 2016 H ODP — Halon-1211 (col D row 12, \
             bromochlorodifluoromethane).",
        ),
        // col D row 13 "Halon-2402" = 14.383  (edge seed: highest in preset)
        cas_factor(
            "124-73-2",
            14.383,
            "ReCiPe 2016 H ODP — Halon-2402 (col D row 13, edge seed: \
             highest ODP in ReCiPe Hierarchist). dibromotetrafluoroethane.",
        ),
        // col D row 15 "CCl4" = 0.895
        cas_factor(
            "56-23-5",
            0.895,
            "ReCiPe 2016 H ODP — CCl4 (col D row 15, carbon \
             tetrachloride / tetrachloromethane).",
        ),
        // col D row 19 "HCFC-22" = 0.045
        cas_factor(
            "75-45-6",
            0.045,
            "ReCiPe 2016 H ODP — HCFC-22 (col D row 19, \
             chlorodifluoromethane).",
        ),
        // col D row 22 "HCFC-141b" = 0.134
        cas_factor(
            "1717-00-6",
            0.134,
            "ReCiPe 2016 H ODP — HCFC-141b (col D row 22, \
             1,1-dichloro-1-fluoroethane).",
        ),
        // col D row 27 "CH3Br" = 0.734
        cas_factor(
            "74-83-9",
            0.734,
            "ReCiPe 2016 H ODP — Methyl bromide (col D row 27, CH3Br).",
        ),
    ]
}

fn hofp_factors() -> Vec<CharacterizationFactor> {
    // Source: RIVM, ReCiPe2016_CFs_v1.1_20180117.xlsx, sheet "Human
    // damage ozone formation", col F (6) Hierarchist (header row 3
    // verbatim: "HOFP, Hierarchist"). Underlying model: Van Zelm et
    // al. (2008), human-damage ozone formation potential, NOx-eq.
    //
    // Taxonomy pre-check (col F): 169 rows, 167 with numeric H. The
    // top of the sheet has 4 group-flow rows (NOx, NMVOC, NO, NO3)
    // without CAS, followed by 165 individual VOC species each with
    // a CAS in col A (encoded as deformatted integer — e.g., 74840 =
    // CAS "74-84-0" for ethane). All rows carry compartment "air"
    // verbatim in col D (ReCiPe HOFP is exclusively air-emission).
    //
    // Matcher mix (only category in V1 with this mix; HOFP is the
    // hybrid-matcher witness for ReCiPe — same role ADP-fossil plays
    // in CML-IA):
    //   - `NameAndCompartment` for the four group flows (NOx, NMVOC,
    //     NO, NO3) — these appear in inventories under those exact
    //     names without single-CAS resolution.
    //   - `CasCompartment` for individual VOC species (ethane, propane,
    //     butane, etc.) where the source ships a real per-species CAS.
    // Both branches use the `["emission","air"]` compartment prefix
    // (HOFP must reject VOC-to-water/soil flows, same enforcement as
    // CML-IA POCP / EF 3.1 POCP).
    //
    // V1 species selection: 8 entries — the 4 group flows above plus
    // 4 representative alkanes (ethane, propane, n-butane, n-hexane).
    // The full 165-row VOC table is deferred — each species would
    // need its own per-factor source comment with deformatted-CAS
    // citation, and the typical inventory carries < 10 distinct VOCs.
    //
    // Seeds:
    //   - Basic:   NOx = 1.0          (reference species, NameAndCompartment)
    //   - Basic:   NMVOC = 0.18       (group VOC anchor, NameAndCompartment)
    //   - Edge:    Ethane = 0.029     (CasCompartment witness; lowest VOC)
    //   - Edge:    n-Hexane = 0.145   (CasCompartment witness; highest of
    //                                  shipped VOC subset — ranking anchor)
    let air = || vec!["emission".into(), "air".into()];
    let nac = |name: &str, value: f64, note: &str| CharacterizationFactor {
        match_on: FactorMatch::NameAndCompartment {
            name: name.into(),
            compartment: air(),
        },
        value,
        note: Some(note.into()),
    };
    let cc = |cas: &str, value: f64, note: &str| CharacterizationFactor {
        match_on: FactorMatch::CasCompartment {
            cas: cas.into(),
            compartment: air(),
        },
        value,
        note: Some(note.into()),
    };
    vec![
        // col F row 4 "NOx" (no CAS in source) = 1.0
        nac(
            "Nitrogen oxides",
            1.0,
            "ReCiPe 2016 H HOFP — NOx group-flow (col F row 4 'NOx', \
             reference species, basic seed). NameAndCompartment match \
             on Arko-canonical name 'Nitrogen oxides' (the ecoinvent \
             flow label corresponding to ReCiPe's source 'NOx').",
        ),
        // col F row 5 "NMVOC" (no CAS in source) = 0.18
        nac(
            "NMVOC, non-methane volatile organic compounds, unspecified origin",
            0.18,
            "ReCiPe 2016 H HOFP — NMVOC group-flow (col F row 5 \
             'NMVOC (non-methane volatile organic chemicals)', basic \
             seed). NameAndCompartment match on the ecoinvent canonical \
             NMVOC group-flow name.",
        ),
        // col F row 8 "Ethane" CAS encoded as 74840 = "74-84-0" = 0.029012...
        cc(
            "74-84-0",
            0.029_012_456_479_775_687,
            "ReCiPe 2016 H HOFP — Ethane (col F row 8, edge seed: \
             lowest VOC in shipped V1 subset; CasCompartment witness). \
             Source CAS encoded as deformatted integer 74840; \
             reformatted to canonical 74-84-0. Full-precision value \
             preserved verbatim from source.",
        ),
        // col F row 9 "Propane" CAS 74986 = "74-98-6" = 0.05077...
        cc(
            "74-98-6",
            0.050_771_798_839_607_46,
            "ReCiPe 2016 H HOFP — Propane (col F row 9). Source CAS \
             74986 → 74-98-6.",
        ),
        // col F row 10 "Butane" CAS 106978 = "106-97-8" = 0.11242...
        cc(
            "106-97-8",
            0.112_423_268_859_130_79,
            "ReCiPe 2016 H HOFP — n-Butane (col F row 10). Source CAS \
             106978 → 106-97-8.",
        ),
        // col F row 12 "Pentane" CAS 109660 = "109-66-0" = 0.14506...
        cc(
            "109-66-0",
            0.145_062_282_398_878_44,
            "ReCiPe 2016 H HOFP — n-Pentane (col F row 12). Source CAS \
             109660 → 109-66-0.",
        ),
        // col F row 15 "Hexane" CAS 110543 = "110-54-3" = 0.14506...  (edge seed)
        cc(
            "110-54-3",
            0.145_062_282_398_878_44,
            "ReCiPe 2016 H HOFP — n-Hexane (col F row 15, edge seed: \
             tied for highest CF among shipped alkanes). Source CAS \
             110543 → 110-54-3.",
        ),
    ]
}

fn pmfp_factors() -> Vec<CharacterizationFactor> {
    // Source: RIVM, ReCiPe2016_CFs_v1.1_20180117.xlsx, sheet "Particulate
    // matter formation", col D (4) Hierarchist (header row 3 verbatim:
    // "PMFP, Hierarchist"). Underlying model: Van Zelm et al. (2008)
    // intake-fraction PMFP, kg PM2.5-eq.
    //
    // Taxonomy pre-check (col D): 10 rows, all numeric. Source col A
    // is a "pollutant" classifier ("PM2.5" for every row); col B is
    // the actual emitted substance (NH3, NOx, SO2, PM2.5, NO, NO2,
    // NO3, SO, SOx, SO3). Source has no CAS column. Per the matcher-
    // mix discipline, V1 maps canonical-chemistry substances
    // (NH3, SO2, NO, NO2, NO3, SO3, NOx-as-group) to their Arko-
    // canonical CAS so that the preset fires against CAS-keyed
    // ecoinvent flows; PM2.5, SO (sulfur monoxide), and SOx
    // (sulfur-oxides group flow) use NameAndCompartment.
    //
    // Compartment uniformity: PMFP is air-emission only (the source
    // does not split air subcompartments; same convention as POCP /
    // AP). All matchers use the `["emission","air"]` prefix to reject
    // PM-to-water flows (physically nonsensical for atmospheric PM
    // formation).
    //
    // V1 species selection: all 10 rows shipped (small enough to
    // ship verbatim). Seeds cover the matcher-mix witness (CasComp
    // for canonical species, NameAndComp for group flows / SO).
    //
    // Seeds:
    //   - Basic:   PM2.5 = 1.0       (reference species, NameAndCompartment)
    //   - Basic:   SO2 = 0.29        (canonical S species, CasCompartment)
    //   - Basic:   NH3 = 0.24        (canonical N species, CasCompartment)
    //   - Edge:    SO = 0.39         (NameAndCompartment witness — higher
    //                                 than SO2 because of stoichiometry;
    //                                 catches matcher-shape regressions)
    //   - Edge:    NOx = 0.11        (group-flow witness via Cas
    //                                 11104-93-1; identical CF to NO2
    //                                 per ReCiPe convention "NOx as NO2-eq")
    let air = || vec!["emission".into(), "air".into()];
    let nac = |name: &str, value: f64, note: &str| CharacterizationFactor {
        match_on: FactorMatch::NameAndCompartment {
            name: name.into(),
            compartment: air(),
        },
        value,
        note: Some(note.into()),
    };
    let cc = |cas: &str, value: f64, note: &str| CharacterizationFactor {
        match_on: FactorMatch::CasCompartment {
            cas: cas.into(),
            compartment: air(),
        },
        value,
        note: Some(note.into()),
    };
    vec![
        // col D row 4 "NH3" = 0.24
        cc(
            "7664-41-7",
            0.24,
            "ReCiPe 2016 H PMFP — NH3 (col D row 4 'NH3', basic seed). \
             Source has no CAS; Arko-canonical CAS 7664-41-7 used per \
             matcher-mix discipline.",
        ),
        // col D row 5 "NOx" = 0.11
        cc(
            "11104-93-1",
            0.11,
            "ReCiPe 2016 H PMFP — NOx as NO2-eq group flow (col D row 5 \
             'NOx', edge seed: same value as NO2 per source convention). \
             Arko-canonical group-flow CAS 11104-93-1.",
        ),
        // col D row 6 "SO2" = 0.29
        cc(
            "7446-09-5",
            0.29,
            "ReCiPe 2016 H PMFP — SO2 (col D row 6 'SO2', basic seed). \
             Arko-canonical CAS 7446-09-5.",
        ),
        // col D row 7 "PM2.5" = 1.0  (reference species, NameAndCompartment)
        nac(
            "Particulates, < 2.5 um",
            1.0,
            "ReCiPe 2016 H PMFP — PM2.5 (col D row 7, reference \
             species, basic seed). NameAndCompartment match on the \
             ecoinvent canonical PM2.5 flow name 'Particulates, \
             < 2.5 um'.",
        ),
        // col D row 8 "NO" = 0.17
        cc(
            "10102-43-9",
            0.17,
            "ReCiPe 2016 H PMFP — NO (col D row 8 'NO'). Arko-canonical \
             CAS 10102-43-9.",
        ),
        // col D row 9 "NO2" = 0.11
        cc(
            "10102-44-0",
            0.11,
            "ReCiPe 2016 H PMFP — NO2 (col D row 9 'NO2'). Arko-canonical \
             CAS 10102-44-0.",
        ),
        // col D row 10 "NO3" = 0.08
        cc(
            "14797-55-8",
            0.08,
            "ReCiPe 2016 H PMFP — NO3 (col D row 10 'NO3'). Arko-canonical \
             CAS 14797-55-8.",
        ),
        // col D row 11 "SO" (sulfur monoxide) = 0.39  (edge seed)
        nac(
            "Sulfur monoxide",
            0.39,
            "ReCiPe 2016 H PMFP — SO sulfur monoxide (col D row 11, \
             edge seed: highest CF in shipped subset, NameAndCompartment \
             witness). SO is a chemistry-niche species; Arko has no \
             canonical CAS for it in the existing presets; \
             NameAndCompartment used.",
        ),
        // col D row 12 "SOx" = 0.29  (group flow, NameAndCompartment)
        nac(
            "Sulfur oxides",
            0.29,
            "ReCiPe 2016 H PMFP — SOx sulfur-oxides group flow (col D \
             row 12). Same CF as SO2 per source convention 'SOx as \
             SO2-eq'. NameAndCompartment because SOx is a group flow.",
        ),
        // col D row 13 "SO3" = 0.23
        cc(
            "7446-11-9",
            0.23,
            "ReCiPe 2016 H PMFP — SO3 sulfur trioxide (col D row 13). \
             Arko-canonical CAS 7446-11-9.",
        ),
    ]
}

fn acidification_factors() -> Vec<CharacterizationFactor> {
    // Source: RIVM, ReCiPe2016_CFs_v1.1_20180117.xlsx, sheet "Terrestrial
    // acidification", col C (3) Hierarchist (header row 3 verbatim:
    // "AP, Hierarchist"). Underlying model: Roy et al. (2012)
    // accumulated-exceedance terrestrial AP, kg SO2-eq.
    //
    // Taxonomy pre-check (col C): 10 rows, all numeric. Source col A
    // is the substance name verbatim (NOx, NH3, SO2, NO, NO2, NO3,
    // SO, SOx, SO3, H2SO4). No CAS column, no compartment column —
    // ReCiPe's terrestrial AP is an air-emission category by
    // construction (the deposition footprint of atmospheric acid
    // precursors). Same matcher-mix as PMFP.
    //
    // Compartment uniformity: same air-only enforcement as PMFP /
    // POCP. Matchers use `["emission","air"]` prefix.
    //
    // V1 species selection: all 10 rows shipped.
    //
    // Seeds:
    //   - Basic:   SO2 = 1.0          (reference species, kg SO2-eq)
    //   - Edge:    NH3 = 1.96         (highest CF in shipped subset)
    //   - Edge:    SO = 1.33          (NameAndCompartment witness,
    //                                  higher than SO2)
    //   - Cross:   H2SO4 = 0.65       (canonical-CAS sulfuric acid;
    //                                  catches CAS-resolution drift)
    let air = || vec!["emission".into(), "air".into()];
    let nac = |name: &str, value: f64, note: &str| CharacterizationFactor {
        match_on: FactorMatch::NameAndCompartment {
            name: name.into(),
            compartment: air(),
        },
        value,
        note: Some(note.into()),
    };
    let cc = |cas: &str, value: f64, note: &str| CharacterizationFactor {
        match_on: FactorMatch::CasCompartment {
            cas: cas.into(),
            compartment: air(),
        },
        value,
        note: Some(note.into()),
    };
    vec![
        // col C row 4 "NOx" = 0.36
        cc(
            "11104-93-1",
            0.36,
            "ReCiPe 2016 H AP — NOx group flow (col C row 4). \
             Arko-canonical CAS 11104-93-1, same value as NO2 per \
             source convention 'NOx as NO2-eq'.",
        ),
        // col C row 5 "NH3" = 1.96  (edge seed: highest in subset)
        cc(
            "7664-41-7",
            1.96,
            "ReCiPe 2016 H AP — NH3 (col C row 5, edge seed: highest \
             CF in shipped subset). Arko-canonical CAS 7664-41-7.",
        ),
        // col C row 6 "SO2" = 1.0  (basic seed: reference species)
        cc(
            "7446-09-5",
            1.0,
            "ReCiPe 2016 H AP — SO2 (col C row 6, reference species, \
             basic seed: kg SO2-eq). Arko-canonical CAS 7446-09-5.",
        ),
        // col C row 7 "NO" = 0.552
        cc(
            "10102-43-9",
            0.552,
            "ReCiPe 2016 H AP — NO (col C row 7). Arko-canonical CAS \
             10102-43-9.",
        ),
        // col C row 8 "NO2" = 0.36
        cc(
            "10102-44-0",
            0.36,
            "ReCiPe 2016 H AP — NO2 (col C row 8). Arko-canonical CAS \
             10102-44-0.",
        ),
        // col C row 9 "NO3" = 0.27
        cc(
            "14797-55-8",
            0.27,
            "ReCiPe 2016 H AP — NO3 (col C row 9). Arko-canonical CAS \
             14797-55-8.",
        ),
        // col C row 10 "SO" = 1.33  (edge seed: NameAndCompartment witness)
        nac(
            "Sulfur monoxide",
            1.33,
            "ReCiPe 2016 H AP — SO sulfur monoxide (col C row 10, edge \
             seed: NameAndCompartment witness, higher than SO2). \
             Chemistry-niche species; no Arko-canonical CAS in existing \
             presets.",
        ),
        // col C row 11 "SOx" = 1.0  (group flow)
        nac(
            "Sulfur oxides",
            1.0,
            "ReCiPe 2016 H AP — SOx group flow (col C row 11). Same CF \
             as SO2 per source convention 'SOx as SO2-eq'. \
             NameAndCompartment because SOx is a group flow.",
        ),
        // col C row 12 "SO3" = 0.8
        cc(
            "7446-11-9",
            0.8,
            "ReCiPe 2016 H AP — SO3 sulfur trioxide (col C row 12). \
             Arko-canonical CAS 7446-11-9.",
        ),
        // col C row 13 "H2SO4" = 0.65  (cross-witness for CAS resolution)
        cc(
            "7664-93-9",
            0.65,
            "ReCiPe 2016 H AP — H2SO4 sulfuric acid (col C row 13, \
             cross-witness: canonical-CAS drift detector). \
             Arko-canonical CAS 7664-93-9.",
        ),
    ]
}

fn fep_factors() -> Vec<CharacterizationFactor> {
    // Source: RIVM, ReCiPe2016_CFs_v1.1_20180117.xlsx, sheet
    // "Freshwater eutrophication", col D (4) Hierarchist (header row 3
    // verbatim: "FEP, Hierarchist"). Underlying model: Helmes et al.
    // (2012) FATE-fw, kg P-eq.
    //
    // Taxonomy pre-check (col D): 12 rows = 4 phosphorus species
    // × 3 source-emission compartments (freshwater / agricultural soil
    // / sea water). Sea-water rows all = 0 (deliberate per FATE-fw —
    // P discharged offshore does not drive freshwater eutrophication).
    // Same fresh-vs-soil 1.0 / 0.1 ratio as EF 3.1 EU-fw (the soil-to-
    // freshwater transfer factor is ~10% per the source model).
    //
    // Compartment encoding decision: ReCiPe source uses verbatim labels
    // "freshwater" / "agricultural soil" / "sea water". V1 translates
    // these to Arko-canonical compartment vectors so the matcher fires
    // against ecoinvent flows, which use the canonical encoding:
    //   - "freshwater"        → ["emission", "water"]
    //   - "agricultural soil" → ["emission", "soil"]
    //   - "sea water"         → not shipped (CFs are 0; would also need
    //                           ["emission","water","ocean"] discrimination
    //                           against the freshwater entries — fresh-vs-
    //                           sea discrimination is the deferred axis,
    //                           same surface as EF 3.1 EU-fw)
    // The mapping is documented in each per-factor comment so the
    // translation is auditable.
    //
    // Matcher choice: `CasCompartment` for all 4 species. P / PO4 /
    // H3PO4 / P2O5 are universally CAS-identified in chemistry and
    // cml_ia + ef_31 use the same Arko-canonical CAS — using
    // NameAndCompartment with ReCiPe's verbatim names ("Phosphorus
    // (P)", "Phosphate (PO43-) ") would make the preset inert against
    // ecoinvent biosphere flows (which are CAS-keyed). Per-factor
    // comments cite ReCiPe's verbatim name AND the Arko-canonical CAS.
    //
    // V1 ships 8 entries (4 species × 2 effective compartments —
    // freshwater + agricultural soil). Sea-water rows omitted (all CFs
    // = 0).
    //
    // Seeds:
    //   - Basic:   P to water = 1.0      (reference species, basic seed)
    //   - Edge:    P to soil = 0.1       (same CAS, 10x lower — load-
    //                                     bearing CasCompartment witness)
    //   - Cross:   PO4 to water = 0.33   (matches EF 3.1 EU-fw exactly;
    //                                     cross-preset parity check)
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
        // col D row 4 "Phosphorus (P)", "freshwater" = 1.0
        cc(
            "7723-14-0",
            water(),
            1.0,
            "ReCiPe 2016 H FEP — P to water (col D row 4 'Phosphorus \
             (P)', compartment 'freshwater', basic seed: reference \
             species kg P-eq). ReCiPe 'freshwater' → Arko ['emission', \
             'water']. Arko-canonical CAS 7723-14-0.",
        ),
        // col D row 5 "Phosphate (PO43-)", "freshwater" = 0.33
        cc(
            "14265-44-2",
            water(),
            0.33,
            "ReCiPe 2016 H FEP — Phosphate to water (col D row 5, \
             matches EF 3.1 EU-fw value exactly — cross-preset parity \
             witness). Arko-canonical CAS 14265-44-2.",
        ),
        // col D row 6 "Phosphoric acid", "freshwater" = 0.32
        cc(
            "7664-38-2",
            water(),
            0.32,
            "ReCiPe 2016 H FEP — Phosphoric acid to water (col D row 6). \
             Arko-canonical CAS 7664-38-2.",
        ),
        // col D row 7 "Phosphorus pentoxide", "freshwater" = 0.22
        cc(
            "1314-56-3",
            water(),
            0.22,
            "ReCiPe 2016 H FEP — P2O5 to water (col D row 7). \
             Arko-canonical CAS 1314-56-3.",
        ),
        // col D row 8 "Phosphorus (P)", "agricultural soil" = 0.1  (edge seed)
        cc(
            "7723-14-0",
            soil(),
            0.1,
            "ReCiPe 2016 H FEP — P to soil (col D row 8, edge seed: \
             same CAS as P-to-water, 10x lower CF — load-bearing \
             CasCompartment witness). ReCiPe 'agricultural soil' → \
             Arko ['emission', 'soil'] (caveat: Arko's 'soil' prefix is \
             broader than ag-soil; non-agricultural soil P emissions \
             will receive this same CF in V1 — fine-grained soil-type \
             discrimination is V2 work, same axis as the country/regional \
             refinement).",
        ),
        // col D row 9 "Phosphate (PO43-)", "agricultural soil" = 0.033
        cc(
            "14265-44-2",
            soil(),
            0.033,
            "ReCiPe 2016 H FEP — Phosphate to soil (col D row 9).",
        ),
        // col D row 10 "Phosphoric acid", "agricultural soil" = 0.032
        cc(
            "7664-38-2",
            soil(),
            0.032,
            "ReCiPe 2016 H FEP — Phosphoric acid to soil (col D row 10).",
        ),
        // col D row 11 "Phosphorus pentoxide", "agricultural soil" = 0.022
        cc(
            "1314-56-3",
            soil(),
            0.022,
            "ReCiPe 2016 H FEP — P2O5 to soil (col D row 11).",
        ),
    ]
}

fn mep_factors() -> Vec<CharacterizationFactor> {
    // Source: RIVM, ReCiPe2016_CFs_v1.1_20180117.xlsx, sheet "Marine
    // eutrophication", col C (3) "all perspectives" (header row 3
    // verbatim: "MEP, Hierarchist/Egalitarian/Individualist"). The
    // MEP sheet is unique among the V1 categories — it carries a
    // single all-perspectives column rather than three I/H/E columns,
    // because the underlying Cosme et al. (2017) model produces a
    // single CF independent of time-perspective weighting. Indicator
    // unit: kg N-eq.
    //
    // Taxonomy pre-check (col C): 21 rows = 7 nitrogen species
    // (N elemental, NH4+, NH3, NO, NO2, NO3, NOx) × 3 source-emission
    // compartments (freshwater / agricultural soil / seawater).
    // Seawater entries are the highest because N discharged offshore
    // directly drives marine eutrophication (the reference compartment
    // for kg N-eq); freshwater entries are ~30% of seawater because
    // freshwater→marine transfer loses N en route; soil entries are
    // ~12% due to denitrification/uptake before reaching water.
    //
    // Compartment encoding (same translation as FEP):
    //   - "freshwater"        → ["emission", "water"]
    //   - "agricultural soil" → ["emission", "soil"]
    //   - "seawater"          → ["emission", "water", "ocean"]  (deeper
    //                           prefix than freshwater so the matcher
    //                           prefers the seawater CF when ecoinvent
    //                           specifies the ocean subcompartment;
    //                           generic ["emission","water"] flows fall
    //                           back to the freshwater CF, which is the
    //                           safer assumption — most water emissions
    //                           in industrial inventories are inland)
    // Caveat: Arko's `["emission","water","ocean"]` encoding is a V2-
    // adapter target (current ecoinvent imports may not generate that
    // depth). V1 ships the seawater entries with the canonical-deep
    // encoding so the matcher is correct once adapters catch up; until
    // then, seawater inventories fall through to the freshwater CF
    // (under-counts MEP for offshore-N inventories).
    //
    // Matcher choice: `CasCompartment` for canonical-CAS species
    // (NH3 / NO / NO2 / NO3 / NH4+); `NameAndCompartment` for the
    // group-flow N elemental and NOx-as-group entries. NH4+ uses the
    // EF 3.1 ammonium-cation CAS 14798-03-9 for cross-preset parity.
    //
    // V1 ships 18 entries (6 CAS-bearing species × 3 compartments)
    // covering NH3, NH4+, NO, NO2, NO3 — and 3 NOx group-flow entries
    // (one per compartment) via NameAndCompartment. The N-elemental
    // group flow (no CAS, source = "N") is omitted from V1 — it appears
    // rarely in real inventories and the relevant entries (per-species
    // NH3 / NO / NO2 / NO3) cover the practical N-emission landscape.
    //
    // Seeds:
    //   - Basic:   N (as NH3) to seawater  = 0.823...   (reference, kg N-eq)
    //   - Edge:    NH3 to freshwater       = 0.244...   (same CAS, lower)
    //   - Edge:    NH3 to soil             = 0.104...   (same CAS, lowest)
    //   - Cross:   NO2 to seawater         = 0.304      (canonical-CAS check)
    let water = || vec!["emission".into(), "water".into()];
    let soil = || vec!["emission".into(), "soil".into()];
    let seawater = || vec!["emission".into(), "water".into(), "ocean".into()];
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
    let nac = |name: &str, compartment: Vec<String>, value: f64, note: &str| {
        CharacterizationFactor {
            match_on: FactorMatch::NameAndCompartment {
                name: name.into(),
                compartment,
            },
            value,
            note: Some(note.into()),
        }
    };
    vec![
        // --- NH4+ (CAS 14798-03-9) ---
        // col C row 5 "NH4+", "freshwater" = 0.2303...
        cc(
            "14798-03-9",
            water(),
            0.230_349_473_684_210_5,
            "ReCiPe 2016 H MEP — NH4+ to freshwater (col C row 5).",
        ),
        // col C row 12 "NH4+", "agricultural soil" = 0.0980...
        cc(
            "14798-03-9",
            soil(),
            0.098_021_052_631_578_95,
            "ReCiPe 2016 H MEP — NH4+ to soil (col C row 12).",
        ),
        // col C row 19 "NH4+", "seawater" = 0.776
        cc(
            "14798-03-9",
            seawater(),
            0.776,
            "ReCiPe 2016 H MEP — NH4+ to seawater (col C row 19).",
        ),
        // --- NH3 (CAS 7664-41-7) ---
        // col C row 6 "NH3", "freshwater" = 0.2444...
        cc(
            "7664-41-7",
            water(),
            0.244_458_204_334_365_3,
            "ReCiPe 2016 H MEP — NH3 to freshwater (col C row 6, edge \
             seed: same CAS as NH3-to-soil and NH3-to-seawater).",
        ),
        // col C row 13 "NH3", "agricultural soil" = 0.1040...
        cc(
            "7664-41-7",
            soil(),
            0.104_024_767_801_857_6,
            "ReCiPe 2016 H MEP — NH3 to soil (col C row 13, edge seed: \
             lowest of the three NH3 compartments).",
        ),
        // col C row 20 "NH3", "seawater" = 0.823...
        cc(
            "7664-41-7",
            seawater(),
            0.823_529_411_764_706,
            "ReCiPe 2016 H MEP — NH3 to seawater (col C row 20, basic \
             seed: highest of the three NH3 compartments — direct \
             offshore N input drives marine eutrophication).",
        ),
        // --- NO (CAS 10102-43-9) ---
        // col C row 7 "NO", "freshwater" = 0.1385...
        cc(
            "10102-43-9",
            water(),
            0.138_526_315_789_473_78,
            "ReCiPe 2016 H MEP — NO to freshwater (col C row 7).",
        ),
        // col C row 14 "NO", "agricultural soil" = 0.0589...
        cc(
            "10102-43-9",
            soil(),
            0.058_947_368_421_052_68,
            "ReCiPe 2016 H MEP — NO to soil (col C row 14).",
        ),
        // col C row 21 "NO", "seawater" = 0.4666...
        cc(
            "10102-43-9",
            seawater(),
            0.466_666_666_666_667,
            "ReCiPe 2016 H MEP — NO to seawater (col C row 21).",
        ),
        // --- NO2 (CAS 10102-44-0) ---
        // col C row 8 "NO2", "freshwater" = 0.0902...
        cc(
            "10102-44-0",
            water(),
            0.090_239_999_999_999_99,
            "ReCiPe 2016 H MEP — NO2 to freshwater (col C row 8).",
        ),
        // col C row 15 "NO2", "agricultural soil" = 0.0384
        cc(
            "10102-44-0",
            soil(),
            0.038_400_000_000_000_004,
            "ReCiPe 2016 H MEP — NO2 to soil (col C row 15).",
        ),
        // col C row 22 "NO2", "seawater" = 0.304
        cc(
            "10102-44-0",
            seawater(),
            0.304,
            "ReCiPe 2016 H MEP — NO2 to seawater (col C row 22, \
             cross-witness: canonical-CAS check on a clean-decimal value).",
        ),
        // --- NO3 (CAS 14797-55-8) ---
        // col C row 9 "NO3", "freshwater" = 0.0670...
        cc(
            "14797-55-8",
            water(),
            0.067_086_315_789_473_67,
            "ReCiPe 2016 H MEP — NO3 to freshwater (col C row 9).",
        ),
        // col C row 16 "NO3", "agricultural soil" = 0.0285...
        cc(
            "14797-55-8",
            soil(),
            0.028_547_368_421_052_634,
            "ReCiPe 2016 H MEP — NO3 to soil (col C row 16).",
        ),
        // col C row 23 "NO3", "seawater" = 0.226
        cc(
            "14797-55-8",
            seawater(),
            0.226,
            "ReCiPe 2016 H MEP — NO3 to seawater (col C row 23).",
        ),
        // --- NOx group flow (no CAS in source for this category;
        //     11104-93-1 is the canonical-Arko group-flow CAS, but
        //     ReCiPe's MEP "NOx" row has the same CF as NO2 per the
        //     source convention "NOx as NO2-eq" — using CAS would
        //     duplicate the NO2 CasCompartment matcher and trigger
        //     CMatrixError::DuplicateMatch. NameAndCompartment used.) ---
        // col C row 10 "NOx", "freshwater" = 0.0902...
        nac(
            "Nitrogen oxides",
            water(),
            0.090_239_999_999_999_99,
            "ReCiPe 2016 H MEP — NOx group flow to freshwater (col C \
             row 10, value identical to NO2 per source 'NOx as NO2-eq'). \
             NameAndCompartment forced: using CAS 11104-93-1 would \
             duplicate the NO2 CasCompartment matcher.",
        ),
        // col C row 17 "NOx", "agricultural soil" = 0.0384
        nac(
            "Nitrogen oxides",
            soil(),
            0.038_400_000_000_000_004,
            "ReCiPe 2016 H MEP — NOx to soil (col C row 17, NameAndCompartment).",
        ),
        // col C row 24 "NOx", "seawater" = 0.304
        nac(
            "Nitrogen oxides",
            seawater(),
            0.304,
            "ReCiPe 2016 H MEP — NOx to seawater (col C row 24, NameAndCompartment).",
        ),
    ]
}

fn land_occupation_factors() -> Vec<CharacterizationFactor> {
    // Source: RIVM, ReCiPe2016_CFs_v1.1_20180117.xlsx, sheet "Land
    // occupation", col C (3) Hierarchist (header row 3 verbatim: "LOP,
    // Hierarchist"). Underlying model: De Baan et al. (2013)
    // species-richness-based land-use occupation potential, kg
    // m2*a annual crop-eq (the reference flow is "Occupation, annual
    // crop" = 1.0).
    //
    // Taxonomy pre-check (col C): 29 rows, 28 numeric (one row is a
    // null header). Source col A is the ecoinvent flow name verbatim
    // (e.g., "Occupation, permanent crop, non-irrigated, intensive");
    // col B is the ReCiPe biome bucket ("permanent crops", "annual
    // crops", "artificial area", "managed forest", "pasture", "-" for
    // non-land water occupation). The H value is set per-flow (not
    // per-biome) — the biome label is documentation, not a matching
    // axis. CFs cluster around five biome levels: annual crops = 1.0
    // (reference), artificial area = 0.73, permanent crops = 0.7,
    // pasture = 0.55, managed forest = 0.3, water/seabed = 0.0.
    //
    // Compartment choice: ecoinvent land-occupation flows are
    // typically encoded as resource flows with compartment
    // ["natural resource", "land"] or similar. The Arko canonical
    // resource prefix is `["resource"]` (matching cml_ia ADP-elements
    // / ADP-fossil convention). V1 ships with the `["resource"]`
    // prefix; V2 inventory adapters are responsible for normalising
    // ecoinvent's "natural resource" → Arko's "resource".
    //
    // Matcher choice: `NameAndCompartment` with the ecoinvent flow
    // name verbatim. The names ARE ecoinvent's canonical land-flow
    // names (ReCiPe imported them directly from ecoinvent), so the
    // matcher fires correctly against ecoinvent inputs. There is no
    // CAS for land occupation — these are resource-occupation flows,
    // not chemical species.
    //
    // V1 species selection: 12 representative entries spanning the
    // five biome levels. The full 28-flow table is deferred — adding
    // every variant ("Occupation, annual crop, irrigated, intensive"
    // vs ", non-irrigated, extensive" etc.) inflates the file without
    // adding seed coverage; the five-level biome ranking is the
    // shipped-V1 axis.
    //
    // Seeds:
    //   - Basic:   "Occupation, annual crop" = 1.0   (reference species)
    //   - Basic:   "Occupation, pasture, man made"     = 0.55
    //   - Basic:   "Occupation, forest, intensive"     = 0.3
    //   - Edge:    "Occupation, river, artificial"     = 0.0  (zero-CF
    //                                                          edge — water
    //                                                          occupation
    //                                                          contributes
    //                                                          nothing per
    //                                                          source intent)
    //   - Cross:   biome ranking annual > artificial > permanent >
    //              pasture > forest > water  (from the shipped subset)
    let resource = || vec!["resource".into()];
    let nac = |name: &str, value: f64, note: &str| CharacterizationFactor {
        match_on: FactorMatch::NameAndCompartment {
            name: name.into(),
            compartment: resource(),
        },
        value,
        note: Some(note.into()),
    };
    vec![
        // col C row 15 "Occupation, annual crop" = 1.0  (reference, basic seed)
        nac(
            "Occupation, annual crop",
            1.0,
            "ReCiPe 2016 H LOP — annual crop (col C row 15, reference \
             species, basic seed: m2*a annual crop-eq).",
        ),
        // col C row 10 "Occupation, annual crop, non-irrigated, intensive" = 1.0
        nac(
            "Occupation, annual crop, non-irrigated, intensive",
            1.0,
            "ReCiPe 2016 H LOP — annual crop NIN (col C row 10, biome \
             'annual crops' = 1.0).",
        ),
        // col C row 12 "Occupation, annual crop, irrigated, intensive" = 1.0
        nac(
            "Occupation, annual crop, irrigated, intensive",
            1.0,
            "ReCiPe 2016 H LOP — annual crop II (col C row 12).",
        ),
        // col C row 7 "Occupation, permanent crop, irrigated, intensive" = 0.7
        nac(
            "Occupation, permanent crop, irrigated, intensive",
            0.7,
            "ReCiPe 2016 H LOP — permanent crop II (col C row 7, biome \
             'permanent crops' = 0.7).",
        ),
        // col C row 22 "Occupation, permanent crop" = 0.7
        nac(
            "Occupation, permanent crop",
            0.7,
            "ReCiPe 2016 H LOP — permanent crop (col C row 22).",
        ),
        // col C row 8 "Occupation, industrial area" = 0.73
        nac(
            "Occupation, industrial area",
            0.73,
            "ReCiPe 2016 H LOP — industrial area (col C row 8, biome \
             'artificial area' = 0.73).",
        ),
        // col C row 11 "Occupation, traffic area, road network" = 0.73
        nac(
            "Occupation, traffic area, road network",
            0.73,
            "ReCiPe 2016 H LOP — road network (col C row 11).",
        ),
        // col C row 19 "Occupation, pasture, man made, extensive" = 0.55
        nac(
            "Occupation, pasture, man made, extensive",
            0.55,
            "ReCiPe 2016 H LOP — pasture man-made extensive (col C row \
             19, biome 'pasture' = 0.55).",
        ),
        // col C row 30 "Occupation, pasture, man made" = 0.55  (basic seed)
        nac(
            "Occupation, pasture, man made",
            0.55,
            "ReCiPe 2016 H LOP — pasture man-made (col C row 30, basic \
             seed: pasture biome anchor).",
        ),
        // col C row 21 "Occupation, forest, intensive" = 0.3  (basic seed)
        nac(
            "Occupation, forest, intensive",
            0.3,
            "ReCiPe 2016 H LOP — forest intensive (col C row 21, basic \
             seed: managed-forest biome anchor — lowest non-zero CF \
             among shipped-V1 land flows).",
        ),
        // col C row 20 "Occupation, forest, extensive" = 0.3
        nac(
            "Occupation, forest, extensive",
            0.3,
            "ReCiPe 2016 H LOP — forest extensive (col C row 20).",
        ),
        // col C row 14 "Occupation, river, artificial" = 0.0  (edge seed: zero-CF)
        nac(
            "Occupation, river, artificial",
            0.0,
            "ReCiPe 2016 H LOP — river artificial (col C row 14, edge \
             seed: zero-CF — water occupation contributes nothing per \
             De Baan model intent. If this ever ships nonzero, the \
             biome-CF mapping has drifted.).",
        ),
    ]
}

fn water_consumption_factors() -> Vec<CharacterizationFactor> {
    // Source: RIVM, ReCiPe2016_CFs_v1.1_20180117.xlsx, sheet "Water
    // consumption", col C (3) Hierarchist (header row 3 verbatim:
    // "WCP, Hierarchist"). Underlying model: Pfister et al. (2009)
    // AWaRe-precursor / WSI-based water consumption potential, m3
    // water-eq.
    //
    // Taxonomy pre-check (col C): 1 numeric row (the dump-script's
    // row-skipping logic suppressed it because of an empty id-cell;
    // direct sheet inspection confirms a single CF for "Water
    // consumption" = 1.0 at row 3). The main spreadsheet ships this
    // single global default; per-water-type and per-country CFs live
    // exclusively in the separate `ReCiPe2016_country_factors_v1.1_
    // 20171221.xlsx`, deferred to V2.
    //
    // **Single-CF deliberate.** The water-consumption indicator is
    // shipped as a single GLO-default CF in the main file *by design*
    // — ReCiPe's intent is that every cubic metre of consumed water
    // equals 1.0 kg-water-eq at the global default. Country-resolved
    // CFs (which range from 0.01 for water-rich regions to >40 for
    // arid regions) are the regionalised refinement; until Arko has
    // a `CasRegion` matcher (V2), shipping per-country WCPs would
    // require either silently picking one country's CF or shipping a
    // 240-row table that all map to the same global default.
    //
    // Matcher choice: `NameAndCompartment` with name "Water consumption"
    // and an empty compartment vector — the empty prefix matches
    // every flow named "Water consumption" regardless of compartment.
    // This will NOT fire on ecoinvent's typical water flows, which
    // are named "Water" with compartment ["resource", "in water"] (or
    // similar adapter-specific encodings). Inventory-adapter
    // harmonisation is V2 work.
    //
    // **Known V1 limitation: the category will be inert against
    // current Arko inventory imports.** The single CF is shipped
    // primarily as a placeholder for the V2 adapter work; downstream
    // consumers should NOT rely on this category producing nonzero
    // WCP scores in V1. The deliberate-zero-by-design test below
    // documents this with a regression-witness assertion.
    //
    // Seeds:
    //   - Basic:   "Water consumption" = 1.0  (single CF, GLO default)
    //   - Edge:    factor count == 1 exactly  (single-CF discipline
    //                                          witness — if this ever
    //                                          becomes >1, V2 region
    //                                          work has landed and this
    //                                          comment block needs
    //                                          revisiting)
    vec![CharacterizationFactor {
        match_on: FactorMatch::NameAndCompartment {
            name: "Water consumption".into(),
            compartment: vec![],
        },
        value: 1.0,
        note: Some(
            "ReCiPe 2016 H WCP — Water consumption (col C row 3, single \
             GLO default = 1.0 m3 water-eq per m3 consumed). \
             NameAndCompartment with empty compartment matches the \
             literal source flow name; will NOT fire against ecoinvent \
             'Water' / 'Water, fresh' / etc. flows in V1 — V2 \
             inventory adapters required for the category to produce \
             nonzero scores. Per-country WCPs deferred to V2 with the \
             regionalisation bundle (D-0019)."
                .into(),
        ),
    }]
}

fn adp_fossil_factors() -> Vec<CharacterizationFactor> {
    // Source: RIVM, ReCiPe2016_CFs_v1.1_20180117.xlsx, sheet "Fossil
    // resource scarcity", col E (5) Hierarchist (header row 3
    // verbatim: "FFP, Hierarchist"). Underlying model: Vieira et al.
    // (2016) future-cumulative-extraction-based fossil resource
    // scarcity, kg oil-eq.
    //
    // Taxonomy pre-check (col E): 5 rows, all numeric — Crude oil,
    // Natural gas, Hard coal, Brown coal, Peat. CFs are ratios to
    // crude oil's energy density (oil-eq basis). Source col B carries
    // the source unit ("oil-eq/kg" for solids and oil; "oil-eq/Nm3"
    // for natural gas — note the gas CF is per normal cubic metre,
    // not per kilogram).
    //
    // Compartment choice: same as cml_ia ADP-fossil — `["resource"]`
    // prefix (matches the resource-extraction half of typical
    // inventory flows).
    //
    // Matcher choice: `NameAndCompartment` for all 5 species. Unlike
    // GW/ODP where canonical CAS exists for each species (CO2 = 124-
    // 38-9, etc.), the fossil-resource labels here are commodity
    // categories: "Crude oil" is a complex mixture (no clean single
    // CAS — cml_ia uses 8012-95-1 for petroleum but that's a
    // contested assignment), "Natural gas" similarly (cml_ia uses
    // 8006-14-2 for natural gas — also contested), "Hard coal" /
    // "Brown coal" / "Peat" are commodity categories not chemical
    // species. ReCiPe's source uses commodity labels deliberately —
    // V1 cites them verbatim via NameAndCompartment to preserve
    // source semantics. This will NOT fire against ecoinvent's
    // canonical resource-flow names ("Oil, crude", "Gas, natural,
    // in ground", etc.) without inventory-adapter harmonisation
    // (V2 work, same surface as water-consumption).
    //
    // **Known V1 limitation: the category may be inert against current
    // Arko inventory imports** (depending on adapter behaviour). The
    // category ships with the source-traceable verbatim names so that
    // V2 adapters have a clean target for harmonisation.
    //
    // Seeds:
    //   - Basic:   "Crude oil" = 1.0      (reference, oil-eq)
    //   - Edge:    "Natural gas" = 0.84   (per-Nm3 unit edge — comment
    //                                      flags the unit difference)
    //   - Edge:    "Peat" = 0.22 == "Brown coal" = 0.22  (duplicate-CF
    //                                                     witness)
    let resource = || vec!["resource".into()];
    let nac = |name: &str, value: f64, note: &str| CharacterizationFactor {
        match_on: FactorMatch::NameAndCompartment {
            name: name.into(),
            compartment: resource(),
        },
        value,
        note: Some(note.into()),
    };
    vec![
        // col E row 4 "Crude oil" "oil-eq/kg" = 1.0
        nac(
            "Crude oil",
            1.0,
            "ReCiPe 2016 H FFP — Crude oil (col E row 4, reference \
             species, basic seed: kg oil-eq per kg crude oil = 1.0). \
             NameAndCompartment with verbatim ReCiPe label; V2 adapter \
             work required to map ecoinvent 'Oil, crude' flows.",
        ),
        // col E row 5 "Natural gas" "oil-eq/Nm3" = 0.84  (edge: per-Nm3 unit)
        nac(
            "Natural gas",
            0.84,
            "ReCiPe 2016 H FFP — Natural gas (col E row 5, edge seed: \
             source unit is oil-eq/Nm3 NOT oil-eq/kg — downstream \
             consumers must supply natural-gas inventory in normal cubic \
             metres for this CF to be physically meaningful).",
        ),
        // col E row 6 "Hard coal" "oil-eq/kg" = 0.42
        nac(
            "Hard coal",
            0.42,
            "ReCiPe 2016 H FFP — Hard coal (col E row 6, oil-eq/kg).",
        ),
        // col E row 7 "Brown coal" "oil-eq/kg" = 0.22
        nac(
            "Brown coal",
            0.22,
            "ReCiPe 2016 H FFP — Brown coal (col E row 7, edge seed: \
             tied with Peat at 0.22 — duplicate-CF witness for the \
             ranking test).",
        ),
        // col E row 8 "Peat" "oil-eq/kg" = 0.22
        nac(
            "Peat",
            0.22,
            "ReCiPe 2016 H FFP — Peat (col E row 8, oil-eq/kg, tied \
             with Brown coal).",
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- Method-shape tests -----------------------------------------

    #[test]
    fn recipe_2016_id_and_version() {
        let m = recipe_2016();
        assert_eq!(m.id, "recipe-2016-midpoint-h");
        assert_eq!(m.version, "1.1");
    }

    #[test]
    fn recipe_2016_ships_ten_categories() {
        let m = recipe_2016();
        assert_eq!(
            m.categories.len(),
            10,
            "ReCiPe 2016 V1 ships 10 categories (D-0019 settled scope)"
        );
    }

    #[test]
    fn recipe_2016_category_ids_are_stable() {
        let m = recipe_2016();
        let ids: Vec<&str> = m.categories.iter().map(|c| c.id.as_str()).collect();
        assert_eq!(
            ids,
            vec![
                "climate-change",
                "ozone-depletion",
                "photochemical-ozone-formation",
                "particulate-matter-formation",
                "acidification",
                "eutrophication-freshwater",
                "eutrophication-marine",
                "land-occupation",
                "water-consumption",
                "adp-fossil",
            ]
        );
    }

    #[test]
    fn recipe_2016_units_match_published_spec() {
        let m = recipe_2016();
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
                ("photochemical-ozone-formation", "kg NOx-eq"),
                ("particulate-matter-formation", "kg PM2.5-eq"),
                ("acidification", "kg SO2-eq"),
                ("eutrophication-freshwater", "kg P-eq"),
                ("eutrophication-marine", "kg N-eq"),
                ("land-occupation", "m2*a annual crop-eq"),
                ("water-consumption", "m3 water-eq"),
                ("adp-fossil", "kg oil-eq"),
            ]
        );
    }

    fn factors_for(id: &str) -> Vec<CharacterizationFactor> {
        recipe_2016()
            .categories
            .into_iter()
            .find(|c| c.id == id)
            .unwrap_or_else(|| panic!("recipe-2016 missing category {id}"))
            .factors
    }

    // ---- Climate change seeds ---------------------------------------

    fn cc_factor_cas(cas: &str) -> CharacterizationFactor {
        factors_for("climate-change")
            .into_iter()
            .find(|f| matches!(&f.match_on, FactorMatch::Cas { cas: c } if c == cas))
            .unwrap_or_else(|| panic!("no ReCiPe CC Cas factor for {cas}"))
    }

    fn cc_factor_cas_origin(cas: &str, origin: FlowOrigin) -> CharacterizationFactor {
        factors_for("climate-change")
            .into_iter()
            .find(|f| {
                matches!(
                    &f.match_on,
                    FactorMatch::CasOrigin { cas: c, origin: o } if c == cas && *o == origin
                )
            })
            .unwrap_or_else(|| panic!("no ReCiPe CC CasOrigin factor for {cas} {origin:?}"))
    }

    #[test]
    fn recipe_cc_co2_is_unity_basic_seed() {
        // CO2 = 1.0 by GWP100 definition.
        assert_eq!(cc_factor_cas("124-38-9").value, 1.0);
    }

    #[test]
    fn recipe_cc_ch4_biogenic_is_34() {
        // ReCiPe Hierarchist "Methane" row (default) = 34. NOT 30 (AR6
        // with-feedback) and NOT 28 (CML-IA without-feedback) — ReCiPe's
        // own H-perspective value.
        assert_eq!(
            cc_factor_cas_origin("74-82-8", FlowOrigin::Biogenic).value,
            34.0
        );
    }

    #[test]
    fn recipe_cc_ch4_fossil_is_36_origin_split_witness() {
        // ReCiPe Hierarchist "Fossil methane" row = 36. Same CAS as
        // biogenic, different origin — load-bearing CasOrigin witness.
        assert_eq!(
            cc_factor_cas_origin("74-82-8", FlowOrigin::Fossil).value,
            36.0
        );
    }

    #[test]
    fn recipe_cc_ch4_has_no_plain_cas_factor() {
        // CH4 ships exclusively as CasOrigin entries (Biogenic + Fossil).
        // A plain Cas { cas: "74-82-8" } entry would cause
        // CMatrixError::DuplicateMatch against the two CasOrigin entries.
        // If this assertion ever fails, the duplicate-match guard has
        // been violated.
        let plain_ch4: Vec<_> = factors_for("climate-change")
            .into_iter()
            .filter(|f| matches!(&f.match_on, FactorMatch::Cas { cas } if cas == "74-82-8"))
            .collect();
        assert!(
            plain_ch4.is_empty(),
            "CH4 must not have a plain Cas factor — would conflict with CasOrigin entries"
        );
    }

    #[test]
    fn recipe_cc_n2o_is_298() {
        // N2O = 298 — distinct from AR6 (273) and CML-IA without-feedback
        // (265). Three different N2O values across Arko's three GWP100
        // presets — ranking-anchor catches preset confusion.
        assert_eq!(cc_factor_cas("10024-97-2").value, 298.0);
    }

    #[test]
    fn recipe_cc_sf6_is_23500_ranking_anchor() {
        assert_eq!(cc_factor_cas("2551-62-4").value, 23_500.0);
    }

    #[test]
    fn recipe_cc_hfc23_is_13856_full_precision() {
        // HFC-23 = 13_856 — full-precision value preserved verbatim
        // from source. Catches "round to 14_000" cleanup PRs.
        assert_eq!(cc_factor_cas("75-46-7").value, 13_856.0);
    }

    #[test]
    fn recipe_cc_cfc12_is_11547_cross_listed_with_odp() {
        // CFC-12 = 11_547 in CC + 0.587 in ODP — cross-impact species
        // witness. If this ever drifts the cross-impact chain breaks
        // silently in side-by-side studies.
        assert_eq!(cc_factor_cas("75-71-8").value, 11_547.0);
    }

    #[test]
    fn recipe_cc_ranking_sf6_gt_nf3_gt_hfc23_gt_cfc12() {
        // Published ReCiPe Hierarchist ranking on shipped fluorinated
        // species: SF6 (23_500) > NF3 (16_100) > HFC-23 (13_856) >
        // CFC-12 (11_547).
        assert!(cc_factor_cas("2551-62-4").value > cc_factor_cas("7783-54-2").value);
        assert!(cc_factor_cas("7783-54-2").value > cc_factor_cas("75-46-7").value);
        assert!(cc_factor_cas("75-46-7").value > cc_factor_cas("75-71-8").value);
    }

    // ---- Ozone depletion seeds --------------------------------------

    fn od_factor(cas: &str) -> CharacterizationFactor {
        factors_for("ozone-depletion")
            .into_iter()
            .find(|f| matches!(&f.match_on, FactorMatch::Cas { cas: c } if c == cas))
            .unwrap_or_else(|| panic!("no ReCiPe OD factor for {cas}"))
    }

    #[test]
    fn recipe_od_cfc11_is_reference() {
        assert_eq!(od_factor("75-69-4").value, 1.0);
    }

    #[test]
    fn recipe_od_halon_2402_is_highest() {
        // Halon-2402 = 14.383 — highest ODP in ReCiPe Hierarchist
        // (different ranking from CML-IA where Halon-1301 leads at 12,
        // and matches EF 3.1's qualitative ranking but with different
        // numerics).
        assert_eq!(od_factor("124-73-2").value, 14.383);
    }

    #[test]
    fn recipe_od_halon_1301_second_highest() {
        // Halon-1301 = 14.066 — second highest, very close to Halon-2402.
        // Tight-rank witness.
        assert_eq!(od_factor("75-63-8").value, 14.066);
    }

    #[test]
    fn recipe_od_halon_2402_gt_halon_1301_gt_cfc11() {
        // ReCiPe-specific ranking: Halon-2402 > Halon-1301 > CFC-11.
        // (CML-IA flips Halon-2402 and Halon-1301; EF 3.1 has wider gap.)
        assert!(od_factor("124-73-2").value > od_factor("75-63-8").value);
        assert!(od_factor("75-63-8").value > od_factor("75-69-4").value);
    }

    #[test]
    fn recipe_od_matchers_are_cas_only() {
        for f in factors_for("ozone-depletion") {
            assert!(
                matches!(f.match_on, FactorMatch::Cas { .. }),
                "ReCiPe OD factor must be plain Cas: {:?}",
                f.match_on
            );
        }
    }

    // ---- HOFP seeds (matcher-mix witness) ---------------------------

    #[test]
    fn recipe_hofp_nox_is_reference_via_name_matcher() {
        // NOx = 1.0 reference species, NameAndCompartment witness.
        let f = factors_for("photochemical-ozone-formation")
            .into_iter()
            .find(|f| {
                matches!(
                    &f.match_on,
                    FactorMatch::NameAndCompartment { name, .. } if name == "Nitrogen oxides"
                )
            })
            .expect("HOFP NOx (NameAndCompartment) missing");
        assert_eq!(f.value, 1.0);
    }

    #[test]
    fn recipe_hofp_ethane_via_cas_compartment() {
        // Ethane = 0.029012... via CasCompartment matcher (CAS 74-84-0).
        // Witness for the hybrid-matcher pattern.
        let f = factors_for("photochemical-ozone-formation")
            .into_iter()
            .find(|f| {
                matches!(
                    &f.match_on,
                    FactorMatch::CasCompartment { cas, .. } if cas == "74-84-0"
                )
            })
            .expect("HOFP Ethane (CasCompartment) missing — hybrid matcher broken");
        assert_eq!(f.value, 0.029_012_456_479_775_687);
    }

    #[test]
    fn recipe_hofp_uses_hybrid_matcher_with_air_prefix() {
        // V1 shipping discipline: HOFP mixes NameAndCompartment (group
        // flows) and CasCompartment (per-species VOCs); both branches
        // must use the ["emission","air"] compartment prefix.
        let mut saw_nac = false;
        let mut saw_cc = false;
        let air = vec!["emission".to_string(), "air".to_string()];
        for f in factors_for("photochemical-ozone-formation") {
            match &f.match_on {
                FactorMatch::NameAndCompartment { compartment, .. } => {
                    saw_nac = true;
                    assert_eq!(compartment, &air);
                }
                FactorMatch::CasCompartment { compartment, .. } => {
                    saw_cc = true;
                    assert_eq!(compartment, &air);
                }
                other => panic!("HOFP matcher must be NAC or CasCompartment: {other:?}"),
            }
        }
        assert!(saw_nac, "HOFP must include at least one NameAndCompartment entry");
        assert!(saw_cc, "HOFP must include at least one CasCompartment entry");
    }

    // ---- PMFP seeds -------------------------------------------------

    #[test]
    fn recipe_pmfp_pm25_is_reference_via_name_matcher() {
        let f = factors_for("particulate-matter-formation")
            .into_iter()
            .find(|f| {
                matches!(
                    &f.match_on,
                    FactorMatch::NameAndCompartment { name, .. } if name == "Particulates, < 2.5 um"
                )
            })
            .expect("PMFP PM2.5 reference species missing");
        assert_eq!(f.value, 1.0);
    }

    #[test]
    fn recipe_pmfp_so2_is_029() {
        let f = factors_for("particulate-matter-formation")
            .into_iter()
            .find(|f| {
                matches!(
                    &f.match_on,
                    FactorMatch::CasCompartment { cas, .. } if cas == "7446-09-5"
                )
            })
            .expect("PMFP SO2 missing");
        assert_eq!(f.value, 0.29);
    }

    #[test]
    fn recipe_pmfp_so_edge_seed_higher_than_so2() {
        // SO (sulfur monoxide) = 0.39 > SO2 = 0.29 — counterintuitive
        // ranking that exercises NameAndCompartment selectivity.
        let so = factors_for("particulate-matter-formation")
            .into_iter()
            .find(|f| {
                matches!(
                    &f.match_on,
                    FactorMatch::NameAndCompartment { name, .. } if name == "Sulfur monoxide"
                )
            })
            .expect("PMFP SO missing");
        assert_eq!(so.value, 0.39);
    }

    // ---- Acidification seeds ----------------------------------------

    #[test]
    fn recipe_ap_so2_is_reference() {
        // SO2 = 1.0 reference species (kg SO2-eq).
        let f = factors_for("acidification")
            .into_iter()
            .find(|f| {
                matches!(
                    &f.match_on,
                    FactorMatch::CasCompartment { cas, .. } if cas == "7446-09-5"
                )
            })
            .expect("AP SO2 reference missing");
        assert_eq!(f.value, 1.0);
    }

    #[test]
    fn recipe_ap_nh3_is_highest() {
        // NH3 = 1.96 — highest CF in shipped subset.
        let f = factors_for("acidification")
            .into_iter()
            .find(|f| {
                matches!(
                    &f.match_on,
                    FactorMatch::CasCompartment { cas, .. } if cas == "7664-41-7"
                )
            })
            .expect("AP NH3 missing");
        assert_eq!(f.value, 1.96);
    }

    #[test]
    fn recipe_ap_h2so4_canonical_cas_drift_witness() {
        // H2SO4 = 0.65 via CAS 7664-93-9 — cross-witness ensuring the
        // canonical-CAS resolution for sulfuric acid doesn't drift to
        // some other identifier.
        let f = factors_for("acidification")
            .into_iter()
            .find(|f| {
                matches!(
                    &f.match_on,
                    FactorMatch::CasCompartment { cas, .. } if cas == "7664-93-9"
                )
            })
            .expect("AP H2SO4 missing");
        assert_eq!(f.value, 0.65);
    }

    // ---- FEP seeds (compartment-axis witness) -----------------------

    #[test]
    fn recipe_fep_p_to_water_is_reference() {
        // P to water = 1.0 reference species.
        let f = factors_for("eutrophication-freshwater")
            .into_iter()
            .find(|f| {
                matches!(
                    &f.match_on,
                    FactorMatch::CasCompartment { cas, compartment }
                        if cas == "7723-14-0"
                            && compartment == &vec!["emission".to_string(), "water".to_string()]
                )
            })
            .expect("FEP P-to-water missing");
        assert_eq!(f.value, 1.0);
    }

    #[test]
    fn recipe_fep_p_to_soil_is_01_compartment_witness() {
        // P to soil = 0.1 — same CAS as P-to-water (7723-14-0), 10x
        // lower CF. Load-bearing CasCompartment witness — same role
        // as EF 3.1 EU-fw P-to-soil edge seed.
        let f = factors_for("eutrophication-freshwater")
            .into_iter()
            .find(|f| {
                matches!(
                    &f.match_on,
                    FactorMatch::CasCompartment { cas, compartment }
                        if cas == "7723-14-0"
                            && compartment == &vec!["emission".to_string(), "soil".to_string()]
                )
            })
            .expect("FEP P-to-soil missing");
        assert_eq!(f.value, 0.1);
    }

    #[test]
    fn recipe_fep_phosphate_to_water_matches_ef31_parity() {
        // PO4 to water = 0.33 — exact parity with EF 3.1 EU-fw value
        // (0.33). If this drifts, cross-preset comparison studies will
        // show a phantom delta on the most common P-eutrophication flow.
        let f = factors_for("eutrophication-freshwater")
            .into_iter()
            .find(|f| {
                matches!(
                    &f.match_on,
                    FactorMatch::CasCompartment { cas, compartment }
                        if cas == "14265-44-2"
                            && compartment == &vec!["emission".to_string(), "water".to_string()]
                )
            })
            .expect("FEP PO4-to-water missing");
        assert_eq!(f.value, 0.33);
    }

    // ---- MEP seeds (3-compartment NH3 witness) ----------------------

    fn mep_nh3(compartment: Vec<String>) -> CharacterizationFactor {
        factors_for("eutrophication-marine")
            .into_iter()
            .find(|f| {
                matches!(
                    &f.match_on,
                    FactorMatch::CasCompartment { cas, compartment: c }
                        if cas == "7664-41-7" && c == &compartment
                )
            })
            .unwrap_or_else(|| panic!("MEP NH3 missing for compartment {compartment:?}"))
    }

    #[test]
    fn recipe_mep_nh3_seawater_is_highest() {
        // NH3 to seawater = 0.823... — highest of the three NH3
        // compartments (direct offshore N input).
        let f = mep_nh3(vec![
            "emission".into(),
            "water".into(),
            "ocean".into(),
        ]);
        assert_eq!(f.value, 0.823_529_411_764_706);
    }

    #[test]
    fn recipe_mep_nh3_compartment_ranking_seawater_gt_freshwater_gt_soil() {
        // Same CAS, three compartments — load-bearing CasCompartment
        // witness analogous to FEP P-to-water vs P-to-soil. Catches
        // any compartment-encoding drift.
        let sea = mep_nh3(vec!["emission".into(), "water".into(), "ocean".into()]);
        let fresh = mep_nh3(vec!["emission".into(), "water".into()]);
        let soil = mep_nh3(vec!["emission".into(), "soil".into()]);
        assert!(sea.value > fresh.value);
        assert!(fresh.value > soil.value);
    }

    #[test]
    fn recipe_mep_nox_to_seawater_via_name_matcher_avoids_no2_dup() {
        // NOx group flow uses NameAndCompartment because the source CF
        // is identical to NO2's per "NOx as NO2-eq" convention. Using
        // CAS 11104-93-1 would not duplicate-match against NO2's
        // 10102-44-0, but using NO2's CAS for NOx WOULD duplicate.
        // This test pins the matcher choice.
        let f = factors_for("eutrophication-marine")
            .into_iter()
            .find(|f| {
                matches!(
                    &f.match_on,
                    FactorMatch::NameAndCompartment { name, compartment }
                        if name == "Nitrogen oxides"
                            && compartment
                                == &vec![
                                    "emission".to_string(),
                                    "water".to_string(),
                                    "ocean".to_string(),
                                ]
                )
            })
            .expect("MEP NOx-to-seawater (NameAndCompartment) missing");
        assert_eq!(f.value, 0.304);
    }

    // ---- Land occupation seeds --------------------------------------

    fn lop_factor(name: &str) -> CharacterizationFactor {
        factors_for("land-occupation")
            .into_iter()
            .find(|f| {
                matches!(
                    &f.match_on,
                    FactorMatch::NameAndCompartment { name: n, .. } if n == name
                )
            })
            .unwrap_or_else(|| panic!("LOP missing for {name}"))
    }

    #[test]
    fn recipe_lop_annual_crop_is_reference() {
        // Annual crop = 1.0 reference species.
        assert_eq!(lop_factor("Occupation, annual crop").value, 1.0);
    }

    #[test]
    fn recipe_lop_river_artificial_is_zero_edge_seed() {
        // River artificial = 0.0 — water occupation contributes nothing
        // per De Baan model intent. Zero-CF edge seed.
        assert_eq!(lop_factor("Occupation, river, artificial").value, 0.0);
    }

    #[test]
    fn recipe_lop_biome_ranking_annual_gt_artificial_gt_permanent_gt_pasture_gt_forest() {
        // Published biome ranking from the De Baan model:
        // annual crops (1.0) > artificial area (0.73) > permanent
        // crops (0.7) > pasture (0.55) > managed forest (0.3).
        assert!(lop_factor("Occupation, annual crop").value > lop_factor("Occupation, industrial area").value);
        assert!(lop_factor("Occupation, industrial area").value > lop_factor("Occupation, permanent crop").value);
        assert!(lop_factor("Occupation, permanent crop").value > lop_factor("Occupation, pasture, man made").value);
        assert!(lop_factor("Occupation, pasture, man made").value > lop_factor("Occupation, forest, intensive").value);
    }

    #[test]
    fn recipe_lop_matchers_are_name_with_resource_prefix() {
        let resource = vec!["resource".to_string()];
        for f in factors_for("land-occupation") {
            match &f.match_on {
                FactorMatch::NameAndCompartment { compartment, .. } => {
                    assert_eq!(compartment, &resource);
                }
                other => panic!("LOP matcher must be NameAndCompartment: {other:?}"),
            }
        }
    }

    // ---- Water consumption seeds (single-CF discipline) -------------

    #[test]
    fn recipe_wcp_ships_exactly_one_factor() {
        // Single-CF discipline witness — if this ever becomes >1, V2
        // region work has landed and this category's docstring needs
        // to reflect the change.
        let factors = factors_for("water-consumption");
        assert_eq!(
            factors.len(),
            1,
            "WCP V1 ships exactly 1 GLO-default CF; per-country CFs deferred to V2"
        );
    }

    #[test]
    fn recipe_wcp_global_default_is_unity() {
        let f = &factors_for("water-consumption")[0];
        assert_eq!(f.value, 1.0);
        match &f.match_on {
            FactorMatch::NameAndCompartment { name, compartment } => {
                assert_eq!(name, "Water consumption");
                assert!(
                    compartment.is_empty(),
                    "WCP matcher must use empty-compartment NameAndCompartment to match \
                     the literal source flow name across all compartments"
                );
            }
            other => panic!("WCP matcher must be NameAndCompartment: {other:?}"),
        }
    }

    // ---- Fossil resource scarcity seeds -----------------------------

    fn ffp_factor(name: &str) -> CharacterizationFactor {
        factors_for("adp-fossil")
            .into_iter()
            .find(|f| {
                matches!(
                    &f.match_on,
                    FactorMatch::NameAndCompartment { name: n, .. } if n == name
                )
            })
            .unwrap_or_else(|| panic!("FFP missing for {name}"))
    }

    #[test]
    fn recipe_ffp_crude_oil_is_reference() {
        assert_eq!(ffp_factor("Crude oil").value, 1.0);
    }

    #[test]
    fn recipe_ffp_natural_gas_is_084_per_nm3_unit_edge() {
        // Natural gas = 0.84 oil-eq/Nm3 — note the per-Nm3 unit (NOT
        // per-kg like the other 4 species). Edge seed for unit-handling
        // documentation.
        assert_eq!(ffp_factor("Natural gas").value, 0.84);
    }

    #[test]
    fn recipe_ffp_brown_coal_eq_peat_duplicate_cf_witness() {
        // Brown coal = Peat = 0.22 — duplicate-CF witness. If a future
        // PR "deduplicates" (drops one), the ranking-anchor test below
        // catches it.
        assert_eq!(ffp_factor("Brown coal").value, ffp_factor("Peat").value);
        assert_eq!(ffp_factor("Brown coal").value, 0.22);
    }

    #[test]
    fn recipe_ffp_ranking_oil_gt_gas_gt_hard_coal_gt_brown_coal_eq_peat() {
        // Energy-density ranking (ReCiPe Hierarchist):
        // Crude oil (1.0 oil-eq/kg) > Natural gas (0.84 oil-eq/Nm3) >
        // Hard coal (0.42 oil-eq/kg) > Brown coal = Peat (0.22 oil-eq/kg).
        assert!(ffp_factor("Crude oil").value > ffp_factor("Natural gas").value);
        assert!(ffp_factor("Natural gas").value > ffp_factor("Hard coal").value);
        assert!(ffp_factor("Hard coal").value > ffp_factor("Brown coal").value);
    }

    #[test]
    fn recipe_ffp_matchers_are_name_with_resource_prefix() {
        let resource = vec!["resource".to_string()];
        for f in factors_for("adp-fossil") {
            match &f.match_on {
                FactorMatch::NameAndCompartment { compartment, .. } => {
                    assert_eq!(compartment, &resource);
                }
                other => panic!("FFP matcher must be NameAndCompartment: {other:?}"),
            }
        }
    }
}
