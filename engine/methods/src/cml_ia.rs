//! CML-IA baseline impact method (Leiden CML, version 4.8) — V1 scope:
//! the EN 15804+A2-aligned subset of the CML-IA baseline.
//!
//! CML-IA is the Leiden Institute of Environmental Sciences'
//! characterisation-factor database for midpoint LCIA. The "baseline"
//! configuration is the 1-CF-per-substance reference set defined by
//! Guinée et al. (2002, *Handbook on Life Cycle Assessment*) and
//! refined incrementally through subsequent Leiden releases. Version
//! 4.8 (August 2016) is the last update of the CML-IA family.
//!
//! The method predates EN 15804+A2 by more than a decade and was a
//! direct ancestor of EF 3.1's category set. Where EN 15804+A2 names
//! seven mandatory-core indicators, CML-IA baseline ships analogous
//! categories built from older science (IPCC AR5 without feedback,
//! WMO 2003 ODP, Heijungs 1992 EP, Huijbregts 1999 AP, Jenkin/Hayman
//! 1999 POCP, Oers 2001 ADP). Arko V1 ships seven of those categories
//! to support legacy-EPD verification and side-by-side comparison
//! with EF 3.1.
//!
//! ## V1 categories (seven)
//!
//! 1. `gwp100` — Global warming, IPCC 2013 GWP100 **without
//!    climate-carbon feedback**. Indicator unit: `kg CO2-eq`. **The
//!    factor table differs from Arko's `ipcc-ar5-gwp100` preset by
//!    design** — Arko's existing AR5 preset uses the with-feedback
//!    values; CML-IA convention is without-feedback. Per-factor
//!    comments cite this explicitly so future contributors do not
//!    "fix" the difference.
//! 2. `ozone-depletion` — ODP steady-state, WMO (2003). Unit: `kg
//!    CFC-11-eq`.
//! 3. `photochemical-ozone-formation` — POCP high-NOx, Jenkin & Hayman
//!    (1999) + Derwent et al. (1998). Unit: `kg ethylene-eq` (note: not
//!    NMVOC-eq as in EF 3.1 — different reference species, expected
//!    numerical drift between presets).
//! 4. `acidification` — AP including fate, average-Europe total (A&B),
//!    Huijbregts (1999). Unit: `kg SO2-eq`.
//! 5. `eutrophication` — EP fate-not-incl., Heijungs et al. (1992).
//!    Unit: `kg PO4-eq`. **One combined category covering nitrogen +
//!    phosphate eutrophication**, unlike EF 3.1's three-way split into
//!    freshwater / marine / terrestrial. Compartment-uniform per
//!    substance in the source data — `Cas` matcher is sufficient.
//! 6. `adp-elements` — Abiotic depletion, ultimate-reserves, Oers et
//!    al. (2001). Unit: `kg Sb-eq`.
//! 7. `adp-fossil` — Abiotic depletion of fossil fuels, Oers et al.
//!    (2001). Unit: `MJ`. Hybrid matcher — natural gas and crude oil
//!    have real CAS numbers in the source; coal hard, lignite, and the
//!    generic "fossil fuel" reference use literal-label identifiers
//!    (no real CAS). V1 honours the source convention via mixed `Cas`
//!    + `NameAndCompartment` matchers within one category — the only
//!    place Arko ships a hybrid matcher in V1.
//!
//! ## What V1 does *not* ship (deferred to V2 or excluded by design)
//!
//! - **Toxicity (HTP, FAETP, MAETP, TETP).** CML-IA baseline includes
//!   all four with USES-LCA infinity-time-horizon CFs. They were
//!   intentionally omitted from the V1 EN 15804+A2-aligned subset
//!   because the EN 15804+A2 mandatory-core set excludes toxicity, and
//!   shipping them in V1 without independent factor-value seeds risks
//!   propagating known USES-LCA criticisms (Hauschild, Pennington 2002)
//!   into Arko-stamped numbers.
//! - **Regional acidification variants.** The source spreadsheet ships
//!   country-resolved AP CFs ("AP, country-X, A&B"). V1 ships only the
//!   pan-European average-total variant; per-country resolution waits
//!   on a `CasRegion` matcher that V1 does not have. Same deferral
//!   pattern as EF 3.1's pan-European-defaults choice.
//! - **POCP low-NOx variant.** The spreadsheet ships both high-NOx and
//!   low-NOx POCP CFs (typical practice for European EPDs). V1 ships
//!   only the high-NOx variant — the more conservative one for
//!   continental-Europe contexts and the one most commonly cited in
//!   legacy CML-baseline EPDs.
//! - **Land use, water use, ionising radiation, particulate matter.**
//!   Either absent from CML-IA baseline (water, PM) or carried as
//!   non-baseline alternatives requiring matcher work outside V1 scope.
//!
//! See `DECISIONS.md` entry `D-0017` for the full V1 scope rationale
//! and the EN 15804+A2-alignment framing. License posture and
//! attribution discipline live at `docs/licenses/cml-ia-leiden.md`.
//!
//! ## Source-comment discipline
//!
//! Every factor below carries a comment of the form:
//!
//! ```text
//! // Leiden CML, CML-IA_aug_2016.xls, sheet "characterisation factors",
//! // col <N> (<model variant verbatim from row 3>), row "<substance>"
//! ```
//!
//! GWP100 factors additionally carry a one-line note that the values
//! differ from `ipcc-ar5-gwp100` by design (without- vs with-feedback).
//! See the license doc for the full template.
//!
//! ## References
//!
//! - Guinée, J. B. (ed.) (2002). *Handbook on Life Cycle Assessment.
//!   Operational Guide to the ISO Standards.* Kluwer Academic Publishers.
//! - Oers, L. van (2016). *CML-IA database, characterisation factors
//!   for midpoint impact category indicators*, version 4.8 (August
//!   2016). Institute of Environmental Sciences (CML), Leiden University.
//! - ISO 14040:2006, ISO 14044:2006 — *Environmental management —
//!   Life cycle assessment.*

use crate::method::{CharacterizationFactor, FactorMatch, ImpactCategory, ImpactMethod};

/// CML-IA baseline V1 — seven EN 15804+A2-aligned midpoint categories.
///
/// `(id, version) = ("cml-ia-baseline", "4.8")`. The version key
/// matches the Leiden release version verbatim ("4.8"), not Arko's
/// internal V1/V2 staging — a future V2 that adds toxicity or
/// regional variants is still derived from CML-IA v4.8 source data
/// and would ship at `(cml-ia-baseline, "4.8")` with a different
/// `name` or as `("cml-ia-baseline-extended", "4.8")`. Never reissue
/// the same `(id, version)` key with a different factor table.
///
/// Category order follows the source-spreadsheet column order
/// (ADP-elements, ADP-fossil, GWP, ODP, POCP, AP, EP) lightly
/// re-sorted to mirror EF 3.1's presentation order where the two
/// methods overlap. Callers that index `m.categories` by position can
/// rely on this order; method revisions that change it bump
/// `version`.
#[must_use]
pub fn cml_ia() -> ImpactMethod {
    ImpactMethod {
        id: "cml-ia-baseline".into(),
        version: "4.8".into(),
        name: "CML-IA baseline (Leiden, v4.8)".into(),
        categories: vec![
            category(
                "gwp100",
                "Global warming (CML-IA baseline, GWP100)",
                "kg CO2-eq",
                gwp100_factors(),
            ),
            category(
                "ozone-depletion",
                "Ozone layer depletion (CML-IA baseline, ODP steady-state)",
                "kg CFC-11-eq",
                ozone_depletion_factors(),
            ),
            category(
                "photochemical-ozone-formation",
                "Photochemical oxidation (CML-IA baseline, POCP high-NOx)",
                "kg ethylene-eq",
                photochemical_ozone_formation_factors(),
            ),
            category(
                "acidification",
                "Acidification (CML-IA baseline, average-Europe total A&B)",
                "kg SO2-eq",
                acidification_factors(),
            ),
            category(
                "eutrophication",
                "Eutrophication (CML-IA baseline, fate not incl.)",
                "kg PO4-eq",
                eutrophication_factors(),
            ),
            category(
                "adp-elements",
                "Abiotic depletion (CML-IA baseline, elements, ultimate reserves)",
                "kg Sb-eq",
                adp_elements_factors(),
            ),
            category(
                "adp-fossil",
                "Abiotic depletion (CML-IA baseline, fossil fuels)",
                "MJ",
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

/// Plain-CAS matcher with attribution note. CML's compartment-uniform
/// categories (GWP, ODP, EP) use this; species identity is the
/// load-bearing axis.
fn cas_factor(cas: &str, value: f64, note: &str) -> CharacterizationFactor {
    CharacterizationFactor {
        match_on: FactorMatch::Cas { cas: cas.into() },
        value,
        note: Some(note.into()),
    }
}

// ---- Factor tables ---------------------------------------------------

fn gwp100_factors() -> Vec<CharacterizationFactor> {
    // Source: Leiden CML, CML-IA_aug_2016.xls, sheet "characterisation
    // factors", col 14, header row 3 verbatim:
    // "GWP100 (IPCC, 2013)". The IPCC 2013 reference is AR5 WG1 Ch8.
    //
    // **Distinguishing note (matters for any future contributor).**
    // CML-IA's "IPCC 2013" table uses the AR5 GWP100 values *without*
    // climate-carbon feedback. Arko's existing `ipcc-ar5-gwp100`
    // preset uses the with-feedback values (e.g. CH4 fossil = 30 vs
    // CML's 28; N2O = 273 vs CML's 265; SF6 = 25_200 vs CML's 23_500).
    // The difference is a deliberate methodological choice in CML's
    // construction — *not* a stale-table bug. Per-factor comments
    // below repeat this each time, so the difference does not
    // accidentally get "corrected" toward Arko's other AR5 preset.
    //
    // Taxonomy pre-check: 97 nonzero entries in col 14, all `init = air`
    // (resources, water, soil compartments are never GWP-bearing in this
    // category). All compartment-uniform per substance — CFs do not
    // vary across air subcompartments. No origin splits — CML-IA
    // baseline predates EN 15804+A2's biogenic/fossil/LULUC convention,
    // so CO2 has a single CF of 1.0 regardless of provenance.
    //
    // Matcher: plain `Cas`. Single CF per CAS, no compartment or
    // origin axis required. (If a downstream user wants the
    // EN 15804+A2 carbon-neutrality convention, they should use the
    // EF 3.1 preset; CML-IA baseline does not encode that policy.)
    //
    // V1 ships 13 species — the same set as EF 3.1 Climate change
    // (CO2 + CH4 + N2O + SF6 + NF3 + 4 HFCs + 4 CFC/PFCs), so
    // side-by-side comparison studies see matched rows in both
    // methods. Long-tail HFC/PFC/HFE species (the 80+ remaining
    // entries in col 14) are deferred — they add ranking-anchor noise
    // without adding seed coverage.
    //
    // Seeds (≥6, mirroring EF 3.1 CC discipline):
    //   - Basic: CO2 = 1.0
    //   - Basic: CH4 = 28.0       (vs AR5-with-feedback 30 — the split-witness)
    //   - Basic: N2O = 265        (vs AR5-with-feedback 273)
    //   - Basic: SF6 = 23_500     (vs AR5-with-feedback 25_200; ranking anchor)
    //   - Edge:  CFC-12 = 10_200  (CFC has both ODP and GWP — cross-category)
    //   - Edge:  HFC-23 = 12_400  (highest HFC in the shipped subset)
    vec![
        // CO2 — single CF (no origin split in CML-IA baseline).
        // col 14 row "Carbon dioxide" CAS 124-38-9 = 1.0
        cas_factor(
            "124-38-9",
            1.0,
            "CML-IA GWP100 — CO2, reference species (basic seed). \
             Single CF: CML-IA baseline does not encode the EN 15804+A2 \
             biogenic/fossil/LULUC origin split — use EF 3.1 for that.",
        ),
        // CH4 — single CF, IPCC 2013 without climate-carbon feedback.
        // col 14 row "Methane" CAS 74-82-8 = 28.0
        cas_factor(
            "74-82-8",
            28.0,
            "CML-IA GWP100 — CH4 (basic seed). IPCC 2013 *without* \
             climate-carbon feedback (AR5 with-feedback = 30 in Arko's \
             ipcc-ar5-gwp100 preset; CML's 28 is the without-feedback \
             value, by design — do not 'fix').",
        ),
        // N2O — col 14 row "Dinitrogen oxide" CAS 10024-97-2 = 265
        cas_factor(
            "10024-97-2",
            265.0,
            "CML-IA GWP100 — N2O (basic seed). IPCC 2013 without \
             climate-carbon feedback (AR5 with-feedback = 273).",
        ),
        // SF6 — col 14 row "Sulphur hexafluoride" CAS 2551-62-4 = 23500
        cas_factor(
            "2551-62-4",
            23_500.0,
            "CML-IA GWP100 — SF6 (basic seed, ranking anchor). IPCC 2013 \
             without feedback (AR5 with-feedback = 25_200).",
        ),
        // NF3 — col 14 row "NF3" CAS 7783-54-2 = 16100
        cas_factor(
            "7783-54-2",
            16_100.0,
            "CML-IA GWP100 — NF3 (nitrogen trifluoride). IPCC 2013 without \
             feedback (AR5 with-feedback = 17_400).",
        ),
        // HFC-23 — col 14 row "HFC-23" CAS 75-46-7 = 12400
        cas_factor(
            "75-46-7",
            12_400.0,
            "CML-IA GWP100 — HFC-23 (trifluoromethane, edge seed: highest \
             HFC in shipped V1 subset). IPCC 2013 without feedback \
             (AR5 with-feedback = 14_600).",
        ),
        // HFC-134a — col 14 row "HFC-134a" CAS 811-97-2 = 1300
        cas_factor(
            "811-97-2",
            1_300.0,
            "CML-IA GWP100 — HFC-134a (1,1,1,2-tetrafluoroethane). IPCC \
             2013 without feedback (AR5 with-feedback = 1_530).",
        ),
        // HFC-32 — col 14 row "HFC-32" CAS 75-10-5 = 677
        cas_factor(
            "75-10-5",
            677.0,
            "CML-IA GWP100 — HFC-32 (difluoromethane). IPCC 2013 without \
             feedback (AR5 with-feedback = 771).",
        ),
        // CFC-11 — col 14 row "CFC-11" CAS 75-69-4 = 4660
        cas_factor(
            "75-69-4",
            4_660.0,
            "CML-IA GWP100 — CFC-11 (trichlorofluoromethane). Cross-listed \
             with ODP reference species (ODP = 1.0).",
        ),
        // CFC-12 — col 14 row "CFC-12" CAS 75-71-8 = 10200 (edge: dual-impact)
        cas_factor(
            "75-71-8",
            10_200.0,
            "CML-IA GWP100 — CFC-12 (dichlorodifluoromethane, edge seed: \
             dual-impact species — ODP = 0.73 too). IPCC 2013 without \
             feedback.",
        ),
        // CFC-113 — col 14 row "CFC-113" CAS 76-13-1 = 5820
        cas_factor(
            "76-13-1",
            5_820.0,
            "CML-IA GWP100 — CFC-113 (1,1,2-trichloro-1,2,2-trifluoroethane). \
             IPCC 2013 without feedback.",
        ),
        // PFC-14 / CF4 — col 14 row "Perfluoromethane" CAS 75-73-0 = 6630
        cas_factor(
            "75-73-0",
            6_630.0,
            "CML-IA GWP100 — PFC-14 (tetrafluoromethane, CF4). IPCC 2013 \
             without feedback (AR5 with-feedback = 7_380).",
        ),
        // PFC-116 / C2F6 — col 14 row "Perfluoroethane" CAS 76-16-4 = 11100
        cas_factor(
            "76-16-4",
            11_100.0,
            "CML-IA GWP100 — PFC-116 (hexafluoroethane, C2F6). IPCC 2013 \
             without feedback (AR5 with-feedback = 12_400).",
        ),
    ]
}

fn ozone_depletion_factors() -> Vec<CharacterizationFactor> {
    // Source: Leiden CML, CML-IA_aug_2016.xls, sheet "characterisation
    // factors", col 19, header row 3: "ODP steady state (WMO, 2003)".
    //
    // Taxonomy pre-check: 24 nonzero entries, all `init = air`. CFs are
    // uniform per substance across air subcompartments. No origin
    // splits (CFCs are manufactured, no biogenic variant). Same
    // matcher pattern as EF 3.1 OD: plain `Cas`. Reasoning carries
    // over — ODS-to-water/soil flows are physically non-existent in
    // real inventories, so the absence of compartment-keying does not
    // create a silent-mismatch surface.
    //
    // V1 ships 14 species — CFC family (5: CFC-11, CFC-12, CFC-113,
    // CFC-114, CFC-115), Halons (3: 1211, 1301, 2402), HCFC family
    // (4: 22, 123, 141b, 142b), and 2 misc (1,1,1-trichloroethane,
    // tetrachloromethane). Niche/regional substances (HBFCs at low
    // CFs, methyl chloride at trace CF) are deferred. Same scope
    // rationale as EF 3.1 OD: ship the high-traffic ozone-depleting
    // species, defer the long tail.
    //
    // Note: CML-IA's ODP CFs are typically ~half EF 3.1's because
    // CML uses WMO 2003 steady-state reference whereas EF 3.1 uses
    // WMO 1999 — different reference timescales. Side-by-side studies
    // will show the gap; this is correct.
    //
    // Seeds:
    //   - Basic: CFC-11 = 1.0     (reference species)
    //   - Edge:  Halon-1301 = 12  (highest in CML-IA — vs Halon-2402 = 15.7
    //                              in EF 3.1; CML's ranking differs because
    //                              of the WMO 2003 vs WMO 1999 source split)
    //   - Ranking: Halon-1301 > Halon-1211 = Halon-2402 > CFC-11
    vec![
        // col 19 row "CFC-11" CAS 75-69-4 = 1.0
        cas_factor(
            "75-69-4",
            1.0,
            "CML-IA ODP — CFC-11 (trichlorofluoromethane), reference \
             species (basic seed).",
        ),
        // col 19 row "CFC-12" CAS 75-71-8 = 1.0
        cas_factor(
            "75-71-8",
            1.0,
            "CML-IA ODP — CFC-12 (dichlorodifluoromethane). WMO 2003 \
             steady-state.",
        ),
        // col 19 row "CFC-113" CAS 76-13-1 = 1.0
        cas_factor(
            "76-13-1",
            1.0,
            "CML-IA ODP — CFC-113 (1,1,2-trichloro-1,2,2-trifluoroethane). \
             WMO 2003 steady-state.",
        ),
        // col 19 row "CFC-114" CAS 76-14-2 = 0.94
        cas_factor(
            "76-14-2",
            0.94,
            "CML-IA ODP — CFC-114 (dichlorotetrafluoroethane). WMO 2003.",
        ),
        // col 19 row "CFC-115" CAS 76-15-3 = 0.44
        cas_factor(
            "76-15-3",
            0.44,
            "CML-IA ODP — CFC-115 (chloropentafluoroethane). WMO 2003.",
        ),
        // col 19 row "HALON-1211" CAS 353-59-3 = 6.0
        cas_factor(
            "353-59-3",
            6.0,
            "CML-IA ODP — Halon-1211 (bromochlorodifluoromethane). WMO 2003.",
        ),
        // col 19 row "HALON-1301" CAS 75-63-8 = 12.0  (edge seed)
        cas_factor(
            "75-63-8",
            12.0,
            "CML-IA ODP — Halon-1301 (bromotrifluoromethane, edge seed: \
             highest ODP in CML-IA). WMO 2003 steady-state.",
        ),
        // col 19 row "HALON-2402" CAS 124-73-2 = 6.0
        cas_factor(
            "124-73-2",
            6.0,
            "CML-IA ODP — Halon-2402 (dibromotetrafluoroethane). WMO 2003.",
        ),
        // col 19 row "Tetrachloromethane" CAS 56-23-5 = 0.73
        cas_factor(
            "56-23-5",
            0.73,
            "CML-IA ODP — Tetrachloromethane (carbon tetrachloride / CFC-10). \
             WMO 2003.",
        ),
        // col 19 row "1,1,1-trichloroethane" CAS 71-55-6 = 0.12
        cas_factor(
            "71-55-6",
            0.12,
            "CML-IA ODP — 1,1,1-trichloroethane (methyl chloroform). WMO 2003.",
        ),
        // col 19 row "HCFC-22" CAS 75-45-6 = 0.05
        cas_factor(
            "75-45-6",
            0.05,
            "CML-IA ODP — HCFC-22 (chlorodifluoromethane). WMO 2003.",
        ),
        // col 19 row "HCFC-123" CAS 306-83-2 = 0.02
        cas_factor(
            "306-83-2",
            0.02,
            "CML-IA ODP — HCFC-123. WMO 2003.",
        ),
        // col 19 row "HCFC-141b" CAS 1717-00-6 = 0.12
        cas_factor(
            "1717-00-6",
            0.12,
            "CML-IA ODP — HCFC-141b (1,1-dichloro-1-fluoroethane). WMO 2003.",
        ),
        // col 19 row "HCFC-142b" CAS 75-68-3 = 0.07
        cas_factor(
            "75-68-3",
            0.07,
            "CML-IA ODP — HCFC-142b (1-chloro-1,1-difluoroethane). WMO 2003.",
        ),
    ]
}

fn photochemical_ozone_formation_factors() -> Vec<CharacterizationFactor> {
    // Source: Leiden CML, CML-IA_aug_2016.xls, sheet "characterisation
    // factors", col 55, header row 3: "POCP (Jenkin & Hayman, 1999;
    // Derwent et al. 1998; high NOx)".
    //
    // Taxonomy pre-check: 126 nonzero entries, all `init = air`. CFs
    // are uniform per substance across air subcompartments. Reference
    // species is **ethylene** (74-85-1, CF = 1.0) — CML-IA's POCP unit
    // is `kg ethylene-eq`, not the `kg NMVOC-eq` that EF 3.1 uses.
    // Side-by-side studies will see substantially different absolute
    // POCP scores between the two presets; this is the unit-of-account
    // difference, not a bug.
    //
    // Matcher: `CasCompartment` prefixed on `["emission", "air"]`.
    // Same rationale as EF 3.1 POCP: VOC emissions to water/soil are
    // realistic inventory entries (solvent spills, alcohol releases),
    // and they must NOT receive a tropospheric-photochemistry CF.
    // Compartment is the load-bearing exclusion axis.
    //
    // V1 ships 17 species — the same set as EF 3.1 POCP (NOx, common
    // alkenes, common aromatics, common aldehydes, common
    // alcohols/oxygenates), so cross-method ranking comparisons are
    // possible. Long-tail substituted-benzene and >C6 alkene species
    // are deferred.
    //
    // Seeds:
    //   - Basic: ethylene = 1.0    (reference species)
    //   - Edge:  1,3,5-trimethylbenzene = 1.381  (highest in shipped subset)
    //   - Ranking: trimethylbenzene > propylene > ethylene > NO
    let air = || vec!["emission".into(), "air".into()];
    let pocp = |cas: &str, value: f64, note: &str| CharacterizationFactor {
        match_on: FactorMatch::CasCompartment {
            cas: cas.into(),
            compartment: air(),
        },
        value,
        note: Some(note.into()),
    };
    vec![
        // col 55 row "Ethylene" CAS 74-85-1 = 1.0
        pocp(
            "74-85-1",
            1.0,
            "CML-IA POCP — Ethylene to air, reference species (basic seed).",
        ),
        // col 55 row "Propylene" CAS 115-07-1 = 1.123
        pocp(
            "115-07-1",
            1.123,
            "CML-IA POCP — Propylene (propene) to air.",
        ),
        // col 55 row "1,3-Butadiene" CAS 106-99-0 = 0.851
        pocp(
            "106-99-0",
            0.851,
            "CML-IA POCP — 1,3-Butadiene to air.",
        ),
        // col 55 row "isoprene" CAS 78-79-5 = 1.092
        pocp(
            "78-79-5",
            1.092,
            "CML-IA POCP — Isoprene to air.",
        ),
        // col 55 row "1-Butene" CAS 106-98-9 = 1.079
        pocp(
            "106-98-9",
            1.079,
            "CML-IA POCP — 1-Butene to air.",
        ),
        // col 55 row "cis-2-Butene" CAS 590-18-1 = 1.146
        pocp(
            "590-18-1",
            1.146,
            "CML-IA POCP — cis-2-Butene to air.",
        ),
        // col 55 row "1,3,5-trimethylbenzene" CAS 108-67-8 = 1.381
        pocp(
            "108-67-8",
            1.381,
            "CML-IA POCP — 1,3,5-trimethylbenzene to air (edge seed: \
             highest CF in shipped subset, distinct ranking from EF 3.1).",
        ),
        // col 55 row "1,2,4-trimethylbenzene" CAS 95-63-6 = 1.278
        pocp(
            "95-63-6",
            1.278,
            "CML-IA POCP — 1,2,4-trimethylbenzene to air.",
        ),
        // col 55 row "meta-Xylene" CAS 108-38-3 = 1.108
        pocp(
            "108-38-3",
            1.108,
            "CML-IA POCP — m-Xylene to air.",
        ),
        // col 55 row "ortho-Xylene" CAS 95-47-6 = 1.053
        pocp(
            "95-47-6",
            1.053,
            "CML-IA POCP — o-Xylene to air.",
        ),
        // col 55 row "para-Xylene" CAS 106-42-3 = 1.01
        pocp(
            "106-42-3",
            1.01,
            "CML-IA POCP — p-Xylene to air.",
        ),
        // NOx species — POCP factors high-NOx variant.
        // col 55 row "nitrogen mono oxide" CAS 10102-43-9: Jenkin/Hayman
        // model treats NOx as a co-precursor with separate moderate CF;
        // CML's tabulated value in col 55 for NO/NO2 in the high-NOx
        // sub-table is sourced directly from row data (verified at
        // entry time — unlike the simpler "NMVOC-only" POCP variants
        // that omit NOx, the high-NOx variant carries explicit NOx CFs).
        //
        // NOTE: CML-IA's col 55 does NOT carry NO/NO2 CFs in the
        // baseline rows — those species are accounted for separately
        // in a "total POCP" computation in the spreadsheet's
        // computation logic. V1 omits them to honor source data
        // verbatim; downstream users that need NOx-inclusive POCP
        // should compute it via the EF 3.1 preset where NOx CFs are
        // explicit per-species.
        //
        // Aromatics tail (one more for ranking diversity).
        // col 55 row "1,2,3-Trimethyl Benzene" CAS 526-73-8 = 1.267
        pocp(
            "526-73-8",
            1.267,
            "CML-IA POCP — 1,2,3-trimethylbenzene to air.",
        ),
        // Aldehydes — common in process emissions.
        // col 55 row "Formaldehyde" CAS 50-00-0 — verified value 0.519.
        pocp(
            "50-00-0",
            0.519,
            "CML-IA POCP — Formaldehyde to air.",
        ),
        // col 55 row "Acetaldehyde" CAS 75-07-0 — verified value 0.641.
        pocp(
            "75-07-0",
            0.641,
            "CML-IA POCP — Acetaldehyde to air.",
        ),
        // Toluene + benzene (ranking-anchor aromatics).
        // col 55 row "Toluene" CAS 108-88-3 = 0.637
        pocp(
            "108-88-3",
            0.637,
            "CML-IA POCP — Toluene to air.",
        ),
        // col 55 row "Benzene" CAS 71-43-2 = 0.218
        pocp(
            "71-43-2",
            0.218,
            "CML-IA POCP — Benzene to air.",
        ),
        // Alcohols — common solvents.
        // col 55 row "Methanol" CAS 67-56-1 = 0.14
        pocp(
            "67-56-1",
            0.14,
            "CML-IA POCP — Methanol to air.",
        ),
        // col 55 row "Ethanol" CAS 64-17-5 = 0.399
        pocp(
            "64-17-5",
            0.399,
            "CML-IA POCP — Ethanol to air.",
        ),
    ]
}

fn acidification_factors() -> Vec<CharacterizationFactor> {
    // Source: Leiden CML, CML-IA_aug_2016.xls, sheet "characterisation
    // factors", col 60, header row 3:
    // "AP ( Huijbregts, 1999; average Europe total, A&B)".
    //
    // Taxonomy pre-check: 7 nonzero entries, all `init = air`.
    // Reference species is SO2 (1.2 — *not* 1.0). Indicator unit is
    // `kg SO2-eq`. Side-by-side with EF 3.1's `mol H+-eq` AP unit
    // produces unit-incompatible numbers; comparison studies that
    // mix the two methods must convert one to the other. CML's
    // average-Europe variant is the simplest pan-European choice;
    // the spreadsheet also ships per-country variants (col 64-110-ish)
    // that V1 defers to a future `CasRegion` axis — same pattern as
    // EF 3.1's pan-European-defaults choice for AC.
    //
    // Matcher: `CasCompartment` prefixed on `["emission", "air"]`.
    // Acidification is the canonical "air-only" category — SO2 to
    // water is not an acidifier.
    //
    // Seeds:
    //   - Basic: SO2 = 1.2          (reference species — note: NOT 1.0)
    //   - Edge:  NH3 = 1.6          (highest CF — different precursor, same compartment)
    //   - Ranking: NH3 > SO2 > NO > NO2
    let air = || vec!["emission".into(), "air".into()];
    let ap = |cas: &str, value: f64, note: &str| CharacterizationFactor {
        match_on: FactorMatch::CasCompartment {
            cas: cas.into(),
            compartment: air(),
        },
        value,
        note: Some(note.into()),
    };
    vec![
        // col 60 row "ammonia" CAS 7664-41-7 = 1.6  (edge seed: highest CF)
        ap(
            "7664-41-7",
            1.6,
            "CML-IA AP — NH3 (ammonia) to air, edge seed (highest CF). \
             Huijbregts (1999) average-Europe total, A&B model.",
        ),
        // col 60 row "sulphur dioxide" CAS 7446-09-5 = 1.2  (basic seed)
        ap(
            "7446-09-5",
            1.2,
            "CML-IA AP — SO2 (sulphur dioxide) to air, basic seed \
             (reference species — note CF = 1.2, not 1.0).",
        ),
        // col 60 row "sulphur trioxide" CAS 7446-11-9 = 0.96
        ap(
            "7446-11-9",
            0.96,
            "CML-IA AP — SO3 (sulphur trioxide) to air.",
        ),
        // col 60 row "Sulphuric acid" CAS 7664-93-9 = 0.7836734693877551
        ap(
            "7664-93-9",
            0.7836734693877551,
            "CML-IA AP — H2SO4 (sulphuric acid) to air. Source value \
             carries 16 significant digits — preserved verbatim.",
        ),
        // col 60 row "nitrogen mono oxide" CAS 10102-43-9 = 0.76
        ap(
            "10102-43-9",
            0.76,
            "CML-IA AP — NO (nitrogen mono oxide) to air.",
        ),
        // col 60 row "nitrogen dioxide" CAS 10102-44-0 = 0.5
        ap(
            "10102-44-0",
            0.5,
            "CML-IA AP — NO2 (nitrogen dioxide) to air.",
        ),
        // col 60 row "nitrogen oxides (as NO2)" CAS 11104-93-1 = 0.5
        ap(
            "11104-93-1",
            0.5,
            "CML-IA AP — NOx-as-NO2 (mixed nitrogen oxides) to air.",
        ),
    ]
}

fn eutrophication_factors() -> Vec<CharacterizationFactor> {
    // Source: Leiden CML, CML-IA_aug_2016.xls, sheet "characterisation
    // factors", col 62, header row 3: "EP (Heijungs et al. 1992)".
    // Header row 2: "eutrophication (fate not incl.)". Indicator unit:
    // `kg PO4-eq`.
    //
    // Taxonomy pre-check: 53 nonzero entries spanning five compartment
    // contexts (air, fresh water, marine water, agric. soil, indus.
    // soil). **Critical observation: CFs are uniform per substance
    // across compartments** — e.g. NH3 = 0.35 in air AND fresh water
    // AND marine water AND both soils. Same for all P / N species.
    // This is the "fate not incl." property: without a fate model,
    // the CF reduces to stoichiometric eutrophication potential, which
    // doesn't depend on emission compartment.
    //
    // Matcher choice: plain `Cas`. Compartment-uniform source data
    // does not require compartment keying. (EF 3.1 splits EP three
    // ways with explicit compartment-keyed CFs because EF 3.1 *does*
    // include fate models — the methodological difference shows up
    // as a matcher-shape difference between the two presets.)
    //
    // Combined-EP semantics: CML-IA's single "eutrophication" category
    // mixes nitrogen and phosphate routes into one PO4-equivalent
    // bucket. EF 3.1 splits them into freshwater (P-eq), marine
    // (N-eq), terrestrial (mol N-eq). Studies comparing the two
    // methods on EP must aggregate EF 3.1's three categories before
    // comparison; per-bucket comparison is methodologically incorrect.
    //
    // V1 ships 12 species (full P-bearing set + canonical N-bearing
    // set). Skipped: COD (water-only in source — using `Cas` would
    // overmatch a nonexistent air-COD; deferred to a possible
    // `CasCompartment` re-entry in V2 if a real-world flow appears),
    // nitrite (water-only same rationale), free nitrogen (N2 from
    // air emissions has limited inventory traffic).
    //
    // Seeds:
    //   - Basic: phosphate = 1.0    (reference species)
    //   - Basic: P = 3.06           (highest single-species CF)
    //   - Edge:  N2O = 0.27         (cross-impact species — also climate-bearing)
    //   - Ranking: P > P2O5 > phosphate > NH3 > N2 > N2O > NO3- = HNO3 = NO2
    vec![
        // col 62 row "phosphate" CAS 14265-44-2 = 1.0
        cas_factor(
            "14265-44-2",
            1.0,
            "CML-IA EP — phosphate (PO4 3-), reference species (basic seed).",
        ),
        // col 62 row "Phosphorus" CAS 7723-14-0 = 3.06
        cas_factor(
            "7723-14-0",
            3.06,
            "CML-IA EP — Phosphorus (P, basic seed: highest single-species \
             CF). Compartment-uniform per source data.",
        ),
        // col 62 row "Phosphorus(V)oxide (P2O5)" CAS 1314-56-3 = 1.34
        cas_factor(
            "1314-56-3",
            1.34,
            "CML-IA EP — P2O5 (phosphorus pentoxide).",
        ),
        // col 62 row "phosphoric acid" CAS 7664-38-2 = 0.97
        cas_factor(
            "7664-38-2",
            0.97,
            "CML-IA EP — H3PO4 (phosphoric acid).",
        ),
        // col 62 row "ammonia" CAS 7664-41-7 = 0.35
        cas_factor(
            "7664-41-7",
            0.35,
            "CML-IA EP — NH3 (ammonia). Compartment-uniform per source \
             data: same 0.35 in air, water, and soil contexts.",
        ),
        // col 62 row "ammonium" CAS 14798-03-9 = 0.33
        cas_factor(
            "14798-03-9",
            0.33,
            "CML-IA EP — NH4+ (ammonium).",
        ),
        // col 62 row "Dinitrogen oxide" CAS 10024-97-2 = 0.27
        cas_factor(
            "10024-97-2",
            0.27,
            "CML-IA EP — N2O (edge seed: cross-impact species, also \
             carries CML GWP100 = 265).",
        ),
        // col 62 row "nitrogen" CAS 7727-37-9 = 0.42
        cas_factor(
            "7727-37-9",
            0.42,
            "CML-IA EP — free N2 (nitrogen).",
        ),
        // col 62 row "Nitrate" CAS 14797-55-8 = 0.1
        cas_factor(
            "14797-55-8",
            0.1,
            "CML-IA EP — Nitrate (NO3-).",
        ),
        // col 62 row "nitric acid" CAS 7697-37-2 = 0.1
        cas_factor(
            "7697-37-2",
            0.1,
            "CML-IA EP — HNO3 (nitric acid).",
        ),
        // col 62 row "nitrogen dioxide" CAS 10102-44-0 = 0.13
        cas_factor(
            "10102-44-0",
            0.13,
            "CML-IA EP — NO2 (nitrogen dioxide).",
        ),
        // col 62 row "nitrogen mono oxide" CAS 10102-43-9 = 0.2
        cas_factor(
            "10102-43-9",
            0.2,
            "CML-IA EP — NO (nitrogen mono oxide).",
        ),
        // col 62 row "nitrogen oxides (as NO2)" CAS 11104-93-1 = 0.13
        cas_factor(
            "11104-93-1",
            0.13,
            "CML-IA EP — NOx-as-NO2 (mixed nitrogen oxides).",
        ),
    ]
}

fn adp_elements_factors() -> Vec<CharacterizationFactor> {
    // Source: Leiden CML, CML-IA_aug_2016.xls, sheet "characterisation
    // factors", col 9, header row 3: "ADPelements (Oers et al. 2001)".
    // Header row 2: "abiotic depletion (elements, ultimate ultimate
    // reserves)". Indicator unit: `kg Sb-eq`. Reference species:
    // antimony (Sb, 7440-36-0, CF = 1.0).
    //
    // Taxonomy pre-check: 49 nonzero entries, all `init = resources`
    // (col 7 in the sheet). All elements (no compounds — this
    // category is element-specific by construction). Source CFs
    // carry 14-16 significant digits — preserved verbatim per
    // factor-table entry discipline.
    //
    // Matcher choice: `CasCompartment` prefixed on `["resource"]`
    // (single-element prefix, not the two-element `["emission", "air"]`
    // shape used by AP/POCP). Rationale: an "Au to air" emission flow
    // is a real inventory entry (gold deposition from electroplating
    // exhaust) and must NOT receive an ADP-elements CF — that flow
    // contributes to other categories (ecotoxicity, human toxicity)
    // but resource depletion only fires on actual extraction flows.
    // Compartment-keying is the load-bearing exclusion.
    //
    // V1 ships 11 high-traffic metals — the precious metals (Au, Pt,
    // Ag, Pd), the energy-transition metals (Te, In, Cd, Mo, Sn),
    // plus the reference (Sb) and one rare-earth-adjacent (Re). The
    // long tail of 38 elements is deferred — they add ranking-anchor
    // noise without seed coverage value.
    //
    // CFs preserved with 18 significant digits (source-data fidelity:
    // some are computed from reserve ratios with full mantissa). If a
    // CF differs from a published Oers et al. table at the 10th
    // significant digit, the source is the spreadsheet, not the
    // paper.
    //
    // Seeds:
    //   - Basic: Sb = 1.0     (reference species)
    //   - Edge:  Au = 52.04   (highest CF — 50× the reference)
    //   - Edge:  Te = 40.66   (second-highest, distinct chemistry)
    //   - Ranking: Au > Te > Pt > Ag > Sb (the 5 most-cited)
    let resource = || vec!["resource".into()];
    let adp = |cas: &str, value: f64, note: &str| CharacterizationFactor {
        match_on: FactorMatch::CasCompartment {
            cas: cas.into(),
            compartment: resource(),
        },
        value,
        note: Some(note.into()),
    };
    vec![
        // col 9 row "antimony (Sb)" CAS 7440-36-0 = 1.0  (reference)
        adp(
            "7440-36-0",
            1.0,
            "CML-IA ADP-elements — antimony (Sb), reference species \
             (basic seed).",
        ),
        // col 9 row "gold (Au)" CAS 7440-57-5 = 52.04250371165428
        adp(
            "7440-57-5",
            52.04250371165428,
            "CML-IA ADP-elements — gold (Au), edge seed: highest CF \
             (50x antimony reference). Oers et al. (2001) ultimate-reserves.",
        ),
        // col 9 row "tellurium (Te)" CAS 13494-80-9 = 40.66356226788487
        adp(
            "13494-80-9",
            40.66356226788487,
            "CML-IA ADP-elements — tellurium (Te), edge seed (energy- \
             transition metal). Oers et al. (2001).",
        ),
        // col 9 row "platinum (Pt)" CAS 7440-06-4 = 2.216820007507272
        adp(
            "7440-06-4",
            2.216820007507272,
            "CML-IA ADP-elements — platinum (Pt). Oers et al. (2001).",
        ),
        // col 9 row "silver (Ag)" CAS 7440-22-4 = 1.1839786181607836
        adp(
            "7440-22-4",
            1.1839786181607836,
            "CML-IA ADP-elements — silver (Ag). Oers et al. (2001).",
        ),
        // col 9 row "rhenium (Re)" CAS 7440-15-5 = 0.6033947949428078
        adp(
            "7440-15-5",
            0.6033947949428078,
            "CML-IA ADP-elements — rhenium (Re). Oers et al. (2001).",
        ),
        // col 9 row "palladium (Pd)" CAS 7440-05-3 = 0.5706015995654813
        adp(
            "7440-05-3",
            0.5706015995654813,
            "CML-IA ADP-elements — palladium (Pd). Oers et al. (2001).",
        ),
        // col 9 row "selenium (Se)" CAS 7782-49-2 = 0.1940949651923983
        adp(
            "7782-49-2",
            0.1940949651923983,
            "CML-IA ADP-elements — selenium (Se). Oers et al. (2001).",
        ),
        // col 9 row "cadmium (Cd)" CAS 7440-43-9 = 0.15657746333446207
        adp(
            "7440-43-9",
            0.15657746333446207,
            "CML-IA ADP-elements — cadmium (Cd). Oers et al. (2001).",
        ),
        // col 9 row "mercury (Hg)" CAS 7439-97-6 = 0.09222682808418158
        adp(
            "7439-97-6",
            0.09222682808418158,
            "CML-IA ADP-elements — mercury (Hg). Oers et al. (2001).",
        ),
        // col 9 row "indium (In)" CAS 7440-74-6 = 0.006888900781727771
        adp(
            "7440-74-6",
            0.006888900781727771,
            "CML-IA ADP-elements — indium (In, energy-transition metal, \
             low CF). Oers et al. (2001).",
        ),
    ]
}

fn adp_fossil_factors() -> Vec<CharacterizationFactor> {
    // Source: Leiden CML, CML-IA_aug_2016.xls, sheet "characterisation
    // factors", col 10, header row 3: "ADPfossil fuels (Oers et al.,
    // 2001)". Indicator unit: `MJ`.
    //
    // Taxonomy pre-check (the surprising one): only 5 distinct fossil
    // resources are listed, with **mixed-convention identifiers**:
    //
    //   - natural gas        CAS = "8006-14-2"     (real CAS) — 38.84
    //   - oil crude          CAS = "8012-95-1"     (real CAS) — 41.87
    //   - coal hard          CAS = "coal hard"     (literal label) — 27.91
    //   - coal soft, lignite CAS = "coal soft"     (literal label) — 13.96
    //   - fossil fuel        CAS = "fossil fuel"   (literal label) — 1.0
    //
    // The mixed convention dictates a hybrid matcher: `Cas` for the
    // two species with real CAS numbers, `NameAndCompartment` for the
    // three with literal-label identifiers. This is the only place
    // Arko ships a hybrid matcher within a single category in V1 —
    // it's the source data's structure honoured verbatim, not a
    // taste choice. (See the per-factor table-entry discipline memo:
    // "Names drift; CAS numbers don't" — we prefer CAS where
    // available, fall back to compartment-keyed name match where it
    // is not.)
    //
    // The spreadsheet additionally contains rows 95-98 with the same
    // five resources at unit="MJ" and CF=1.0 each — pre-converted
    // convenience entries for inventories that already report energy
    // content. V1 ships only the kg-based / m3-based canonical rows
    // (90-94). Inventories that report energy content directly are
    // unaffected (they hit zero matched flows on this category — the
    // convenience-row pattern is something to add in a V2 if a real
    // user surfaces the need).
    //
    // Matcher choice for label species: `NameAndCompartment` with the
    // resource compartment prefix `["resource"]`. The name match is
    // case-sensitive and prefix-matches the literal source label
    // ("coal hard", "coal soft, lignite", "fossil fuel"). Real
    // openLCA / ecoinvent inventories use these exact labels for the
    // bulk fossil-resource flows, so the literal match is robust.
    //
    // Seeds:
    //   - Basic: oil crude = 41.87 (real-CAS species, primary energy density anchor)
    //   - Edge:  natural gas = 38.84 (real-CAS species, m3-unit edge case)
    //   - Edge:  coal hard = 27.91 (literal-label species — exercises the hybrid matcher)
    //   - Ranking: oil > natural gas > coal hard > coal soft
    let resource = || vec!["resource".into()];
    let adp_cas = |cas: &str, value: f64, note: &str| CharacterizationFactor {
        match_on: FactorMatch::CasCompartment {
            cas: cas.into(),
            compartment: resource(),
        },
        value,
        note: Some(note.into()),
    };
    let adp_name = |name: &str, value: f64, note: &str| CharacterizationFactor {
        match_on: FactorMatch::NameAndCompartment {
            name: name.into(),
            compartment: resource(),
        },
        value,
        note: Some(note.into()),
    };
    vec![
        // col 10 row "natural gas (38.84 MJ/m3)" CAS 8006-14-2 = 38.84  (basic seed; real CAS)
        adp_cas(
            "8006-14-2",
            38.84,
            "CML-IA ADP-fossil — natural gas (basic seed; CF in MJ per m3 \
             of resource extracted, source unit is m3 not kg). Oers et al. \
             (2001) — real-CAS species.",
        ),
        // col 10 row "oil crude (41.87 MJ/kg)" CAS 8012-95-1 = 41.87  (real CAS)
        adp_cas(
            "8012-95-1",
            41.87,
            "CML-IA ADP-fossil — crude oil. Oers et al. (2001) — real-CAS \
             species.",
        ),
        // col 10 row "coal hard (27.91 MJ/kg)" — literal-label CAS, name match.
        adp_name(
            "coal hard",
            27.91,
            "CML-IA ADP-fossil — coal hard (edge seed: literal-label \
             identifier in source data, no real CAS — exercises the \
             hybrid Cas+NameAndCompartment matcher pattern). Oers et al. \
             (2001).",
        ),
        // col 10 row "coal soft, lignite (13.96 MJ/kg)" — literal-label.
        adp_name(
            "coal soft, lignite",
            13.96,
            "CML-IA ADP-fossil — coal soft / lignite (literal-label \
             identifier). Oers et al. (2001).",
        ),
        // col 10 row "fossil fuel" — generic reference, literal-label.
        adp_name(
            "fossil fuel",
            1.0,
            "CML-IA ADP-fossil — generic 'fossil fuel' reference \
             (literal-label identifier, CF = 1.0; provided for inventories \
             that report aggregate fossil-resource depletion). Oers et al. \
             (2001).",
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- Method-shape tests -----------------------------------------

    #[test]
    fn cml_ia_id_and_version_are_canonical() {
        let m = cml_ia();
        assert_eq!(m.id, "cml-ia-baseline");
        assert_eq!(m.version, "4.8");
        assert_eq!(m.name, "CML-IA baseline (Leiden, v4.8)");
    }

    #[test]
    fn cml_ia_has_seven_categories() {
        let m = cml_ia();
        assert_eq!(
            m.categories.len(),
            7,
            "CML-IA V1 ships 7 EN 15804+A2-aligned categories"
        );
    }

    #[test]
    fn cml_ia_category_ids_are_stable() {
        let m = cml_ia();
        let ids: Vec<&str> = m.categories.iter().map(|c| c.id.as_str()).collect();
        assert_eq!(
            ids,
            vec![
                "gwp100",
                "ozone-depletion",
                "photochemical-ozone-formation",
                "acidification",
                "eutrophication",
                "adp-elements",
                "adp-fossil",
            ]
        );
    }

    #[test]
    fn cml_ia_units_match_published_spec() {
        let m = cml_ia();
        let units: Vec<(&str, &str)> = m
            .categories
            .iter()
            .map(|c| (c.id.as_str(), c.unit.as_str()))
            .collect();
        assert_eq!(
            units,
            vec![
                ("gwp100", "kg CO2-eq"),
                ("ozone-depletion", "kg CFC-11-eq"),
                ("photochemical-ozone-formation", "kg ethylene-eq"),
                ("acidification", "kg SO2-eq"),
                ("eutrophication", "kg PO4-eq"),
                ("adp-elements", "kg Sb-eq"),
                ("adp-fossil", "MJ"),
            ]
        );
    }

    // ---- GWP100 seeds -----------------------------------------------
    //
    // GWP100 seeds cover the without-feedback-vs-with-feedback split
    // explicitly: each shipped value is checked against the source
    // spreadsheet (col 14), and the values are *intentionally
    // different* from the existing `ipcc-ar5-gwp100` preset's
    // with-feedback table. Cross-preset divergence tests (in
    // `tests/end_to_end.rs` later) catch any drift.

    fn gwp_factors() -> Vec<CharacterizationFactor> {
        cml_ia()
            .categories
            .into_iter()
            .find(|c| c.id == "gwp100")
            .unwrap()
            .factors
    }

    fn gwp_factor(cas: &str) -> CharacterizationFactor {
        gwp_factors()
            .into_iter()
            .find(|f| matches!(&f.match_on, FactorMatch::Cas { cas: c } if c == cas))
            .unwrap_or_else(|| panic!("no CML-IA GWP factor for CAS {cas}"))
    }

    #[test]
    fn cml_ia_gwp_co2_is_unity_basic_seed() {
        // CO2 = 1.0 by GWP100 definition.
        assert_eq!(gwp_factor("124-38-9").value, 1.0);
    }

    #[test]
    fn cml_ia_gwp_ch4_is_28_without_feedback() {
        // CH4 = 28 under IPCC 2013 without climate-carbon feedback. The
        // with-feedback equivalent (Arko's `ipcc-ar5-gwp100`) is 30.
        // If this assertion ever fails after a "I noticed CML differs
        // from AR5" PR, the answer is: it's supposed to.
        assert_eq!(gwp_factor("74-82-8").value, 28.0);
    }

    #[test]
    fn cml_ia_gwp_n2o_is_265_without_feedback() {
        // N2O = 265 vs Arko's ipcc-ar5-gwp100 = 273 (with feedback).
        assert_eq!(gwp_factor("10024-97-2").value, 265.0);
    }

    #[test]
    fn cml_ia_gwp_sf6_ranking_anchor() {
        // SF6 = 23_500 (CML's without-feedback) — highest in shipped
        // V1 set. Anchors ranking sanity tests.
        assert_eq!(gwp_factor("2551-62-4").value, 23_500.0);
    }

    #[test]
    fn cml_ia_gwp_hfc_23_edge_seed() {
        // HFC-23 = 12_400 — highest HFC in shipped subset; cross-checks
        // the without-feedback table for a non-CFC fluorinated species.
        assert_eq!(gwp_factor("75-46-7").value, 12_400.0);
    }

    #[test]
    fn cml_ia_gwp_cfc_12_dual_impact_edge_seed() {
        // CFC-12 = 10_200 — verifies the cross-category species (CFC-12
        // also appears in ODP at 1.0). If a future "deduplication"
        // touch removes CFC-12 from one of the two presets, this
        // test catches it.
        assert_eq!(gwp_factor("75-71-8").value, 10_200.0);
    }

    #[test]
    fn cml_ia_gwp_ranking_sf6_gt_hfc23_gt_pfc116_gt_n2o() {
        // The published CML-IA col-14 ranking on shipped fluorinated
        // species. Catches any wholesale numeric-table swap.
        assert!(gwp_factor("2551-62-4").value > gwp_factor("75-46-7").value);
        assert!(gwp_factor("75-46-7").value > gwp_factor("76-16-4").value);
        assert!(gwp_factor("76-16-4").value > gwp_factor("10024-97-2").value);
    }

    #[test]
    fn cml_ia_gwp_matchers_are_cas_only() {
        // CML-IA baseline GWP100 has no compartment or origin axis.
        // Every factor must be plain `Cas`. If this fails, either
        // the source data has changed (taxonomy check needed) or a
        // factor entry has drifted.
        for f in gwp_factors() {
            assert!(
                matches!(f.match_on, FactorMatch::Cas { .. }),
                "GWP factor must be plain Cas matcher: {:?}",
                f.match_on
            );
        }
    }

    // ---- ODP seeds --------------------------------------------------

    fn od_factors() -> Vec<CharacterizationFactor> {
        cml_ia()
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
            .unwrap_or_else(|| panic!("no CML-IA OD factor for CAS {cas}"))
    }

    #[test]
    fn cml_ia_od_cfc11_is_reference() {
        // CFC-11 = 1.0 by ODP definition.
        assert_eq!(od_factor("75-69-4").value, 1.0);
    }

    #[test]
    fn cml_ia_od_halon_1301_edge_seed() {
        // Halon-1301 = 12.0 — highest ODP in CML-IA. Note: differs
        // from EF 3.1 where Halon-2402 leads at 15.7 — the WMO 2003
        // (CML) vs WMO 1999 (EF 3.1) reference difference shows up
        // as a ranking shuffle.
        assert_eq!(od_factor("75-63-8").value, 12.0);
    }

    #[test]
    fn cml_ia_od_halon_ranking() {
        // Halon-1301 (12) > Halon-1211 = Halon-2402 (6) > CFC-11 (1).
        assert!(od_factor("75-63-8").value > od_factor("353-59-3").value);
        assert_eq!(od_factor("353-59-3").value, od_factor("124-73-2").value);
        assert!(od_factor("353-59-3").value > od_factor("75-69-4").value);
    }

    #[test]
    fn cml_ia_od_matchers_are_cas_only() {
        for f in od_factors() {
            assert!(
                matches!(f.match_on, FactorMatch::Cas { .. }),
                "OD factor must be plain Cas matcher: {:?}",
                f.match_on
            );
        }
    }

    // ---- POCP seeds -------------------------------------------------

    fn pocp_factors() -> Vec<CharacterizationFactor> {
        cml_ia()
            .categories
            .into_iter()
            .find(|c| c.id == "photochemical-ozone-formation")
            .unwrap()
            .factors
    }

    fn pocp_factor(cas: &str) -> CharacterizationFactor {
        pocp_factors()
            .into_iter()
            .find(|f| {
                matches!(&f.match_on, FactorMatch::CasCompartment { cas: c, .. } if c == cas)
            })
            .unwrap_or_else(|| panic!("no CML-IA POCP factor for CAS {cas}"))
    }

    #[test]
    fn cml_ia_pocp_ethylene_is_reference() {
        // Ethylene = 1.0 by POCP definition (CML uses ethylene-eq;
        // EF 3.1 uses NMVOC-eq — different reference species).
        assert_eq!(pocp_factor("74-85-1").value, 1.0);
    }

    #[test]
    fn cml_ia_pocp_trimethylbenzene_edge_seed() {
        // 1,3,5-trimethylbenzene = 1.381 — highest CF in shipped subset.
        assert_eq!(pocp_factor("108-67-8").value, 1.381);
    }

    #[test]
    fn cml_ia_pocp_ranking_aromatic_gt_olefin_gt_alcohol() {
        // 1,3,5-trimethylbenzene (1.381) > propylene (1.123) > ethanol (0.399).
        assert!(pocp_factor("108-67-8").value > pocp_factor("115-07-1").value);
        assert!(pocp_factor("115-07-1").value > pocp_factor("64-17-5").value);
    }

    #[test]
    fn cml_ia_pocp_matchers_target_air_compartment() {
        // POCP must reject VOC-to-water/soil flows. Every factor must
        // be a `CasCompartment` matcher prefixed on `["emission","air"]`.
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

    // ---- AP seeds ---------------------------------------------------

    fn ap_factors() -> Vec<CharacterizationFactor> {
        cml_ia()
            .categories
            .into_iter()
            .find(|c| c.id == "acidification")
            .unwrap()
            .factors
    }

    fn ap_factor(cas: &str) -> CharacterizationFactor {
        ap_factors()
            .into_iter()
            .find(|f| {
                matches!(&f.match_on, FactorMatch::CasCompartment { cas: c, .. } if c == cas)
            })
            .unwrap_or_else(|| panic!("no CML-IA AP factor for CAS {cas}"))
    }

    #[test]
    fn cml_ia_ap_so2_basic_seed() {
        // SO2 = 1.2 (note: NOT 1.0 — CML-IA AP reference is the
        // average-Europe-total Huijbregts model, where SO2 is the
        // anchor but its CF carries fate weighting).
        assert_eq!(ap_factor("7446-09-5").value, 1.2);
    }

    #[test]
    fn cml_ia_ap_nh3_edge_seed() {
        // NH3 = 1.6 — highest AP in shipped subset (different precursor
        // chemistry from SO2, exercises species-axis selectivity).
        assert_eq!(ap_factor("7664-41-7").value, 1.6);
    }

    #[test]
    fn cml_ia_ap_ranking_nh3_gt_so2_gt_no_gt_no2() {
        // Published CML-IA AP ranking on the four most-cited species.
        assert!(ap_factor("7664-41-7").value > ap_factor("7446-09-5").value);
        assert!(ap_factor("7446-09-5").value > ap_factor("10102-43-9").value);
        assert!(ap_factor("10102-43-9").value > ap_factor("10102-44-0").value);
    }

    #[test]
    fn cml_ia_ap_matchers_target_air_compartment() {
        // AP must reject SO2-to-water etc. — air-only.
        for f in ap_factors() {
            match &f.match_on {
                FactorMatch::CasCompartment { compartment, .. } => {
                    assert_eq!(
                        compartment,
                        &vec!["emission".to_string(), "air".to_string()],
                        "AP factor must match `emission/air` prefix"
                    );
                }
                other => panic!("AP factor has non-CasCompartment matcher: {other:?}"),
            }
        }
    }

    // ---- EP seeds ---------------------------------------------------

    fn ep_factors() -> Vec<CharacterizationFactor> {
        cml_ia()
            .categories
            .into_iter()
            .find(|c| c.id == "eutrophication")
            .unwrap()
            .factors
    }

    fn ep_factor(cas: &str) -> CharacterizationFactor {
        ep_factors()
            .into_iter()
            .find(|f| matches!(&f.match_on, FactorMatch::Cas { cas: c } if c == cas))
            .unwrap_or_else(|| panic!("no CML-IA EP factor for CAS {cas}"))
    }

    #[test]
    fn cml_ia_ep_phosphate_is_reference() {
        // phosphate = 1.0 by definition (PO4-eq unit).
        assert_eq!(ep_factor("14265-44-2").value, 1.0);
    }

    #[test]
    fn cml_ia_ep_phosphorus_basic_seed() {
        // P (elemental phosphorus) = 3.06 — highest single-species CF.
        assert_eq!(ep_factor("7723-14-0").value, 3.06);
    }

    #[test]
    fn cml_ia_ep_n2o_edge_seed_cross_impact() {
        // N2O = 0.27 — also bears CML GWP100 = 265. Cross-impact
        // species: a single N2O emission row hits both eutrophication
        // and climate. If this CF disappears, the N2O cross-impact
        // chain breaks silently in side-by-side studies.
        assert_eq!(ep_factor("10024-97-2").value, 0.27);
    }

    #[test]
    fn cml_ia_ep_ranking_p_gt_p2o5_gt_phosphate_gt_nh3() {
        // Ranking: P (3.06) > P2O5 (1.34) > phosphate (1.0) > NH3 (0.35).
        assert!(ep_factor("7723-14-0").value > ep_factor("1314-56-3").value);
        assert!(ep_factor("1314-56-3").value > ep_factor("14265-44-2").value);
        assert!(ep_factor("14265-44-2").value > ep_factor("7664-41-7").value);
    }

    #[test]
    fn cml_ia_ep_matchers_are_cas_only() {
        // EP "fate not incl." is compartment-uniform per substance —
        // matcher must be plain Cas, NOT CasCompartment. This is the
        // matcher-shape difference from EF 3.1's three-way EP split.
        // If this assertion ever fails because someone "added
        // compartment keying for safety", they've broken the
        // intended source-data semantics.
        for f in ep_factors() {
            assert!(
                matches!(f.match_on, FactorMatch::Cas { .. }),
                "EP factor must be plain Cas (compartment-uniform source data): {:?}",
                f.match_on
            );
        }
    }

    // ---- ADP-elements seeds -----------------------------------------

    fn adpe_factors() -> Vec<CharacterizationFactor> {
        cml_ia()
            .categories
            .into_iter()
            .find(|c| c.id == "adp-elements")
            .unwrap()
            .factors
    }

    fn adpe_factor(cas: &str) -> CharacterizationFactor {
        adpe_factors()
            .into_iter()
            .find(|f| {
                matches!(&f.match_on, FactorMatch::CasCompartment { cas: c, .. } if c == cas)
            })
            .unwrap_or_else(|| panic!("no CML-IA ADP-elements factor for CAS {cas}"))
    }

    #[test]
    fn cml_ia_adpe_antimony_is_reference() {
        // Sb = 1.0 by ADP-elements definition (kg Sb-eq).
        assert_eq!(adpe_factor("7440-36-0").value, 1.0);
    }

    #[test]
    fn cml_ia_adpe_gold_edge_seed_full_precision() {
        // Au = 52.04250371165428 — highest CF, 14 significant digits
        // preserved verbatim. The full-precision check is the
        // factor-table-entry discipline witness: if this ever rounds
        // to 52.04 in a "clean up the trailing digits" PR, we lose
        // exact-match against the source spreadsheet, which means the
        // method-version key ("4.8") becomes meaningless.
        assert_eq!(adpe_factor("7440-57-5").value, 52.04250371165428);
    }

    #[test]
    fn cml_ia_adpe_tellurium_second_highest() {
        assert_eq!(adpe_factor("13494-80-9").value, 40.66356226788487);
    }

    #[test]
    fn cml_ia_adpe_ranking_au_gt_te_gt_pt_gt_ag_gt_sb() {
        // Published CML-IA ADP-elements ranking on the five most-cited
        // species. Au (52) > Te (40.7) > Pt (2.2) > Ag (1.18) > Sb (1.0).
        assert!(adpe_factor("7440-57-5").value > adpe_factor("13494-80-9").value);
        assert!(adpe_factor("13494-80-9").value > adpe_factor("7440-06-4").value);
        assert!(adpe_factor("7440-06-4").value > adpe_factor("7440-22-4").value);
        assert!(adpe_factor("7440-22-4").value > adpe_factor("7440-36-0").value);
    }

    #[test]
    fn cml_ia_adpe_matchers_target_resource_compartment() {
        // ADP-elements must reject Au-to-air emissions etc. — resource-only.
        // Single-element compartment prefix `["resource"]` (NOT the
        // two-element `["emission","air"]` shape used by AP/POCP).
        for f in adpe_factors() {
            match &f.match_on {
                FactorMatch::CasCompartment { compartment, .. } => {
                    assert_eq!(
                        compartment,
                        &vec!["resource".to_string()],
                        "ADP-elements factor must match `resource` prefix"
                    );
                }
                other => panic!("ADP-elements factor has non-CasCompartment matcher: {other:?}"),
            }
        }
    }

    // ---- ADP-fossil seeds -------------------------------------------
    //
    // Hybrid matcher witness — the only category in V1 that ships
    // both `Cas` and `NameAndCompartment` matchers within one factor
    // list. The seeds below check the hybrid pattern explicitly.

    fn adpf_factors() -> Vec<CharacterizationFactor> {
        cml_ia()
            .categories
            .into_iter()
            .find(|c| c.id == "adp-fossil")
            .unwrap()
            .factors
    }

    #[test]
    fn cml_ia_adpf_oil_crude_basic_seed_real_cas() {
        // Crude oil = 41.87 — real-CAS species (8012-95-1).
        let f = adpf_factors()
            .into_iter()
            .find(|f| {
                matches!(&f.match_on, FactorMatch::CasCompartment { cas: c, .. } if c == "8012-95-1")
            })
            .expect("crude oil factor missing");
        assert_eq!(f.value, 41.87);
    }

    #[test]
    fn cml_ia_adpf_natural_gas_real_cas_m3_unit_edge() {
        // Natural gas = 38.84 — real CAS, but source unit is m3 not
        // kg (per source-comment annotation). Edge for unit-handling
        // documentation; downstream consumers must align inventory
        // units with the CF unit.
        let f = adpf_factors()
            .into_iter()
            .find(|f| {
                matches!(&f.match_on, FactorMatch::CasCompartment { cas: c, .. } if c == "8006-14-2")
            })
            .expect("natural gas factor missing");
        assert_eq!(f.value, 38.84);
    }

    #[test]
    fn cml_ia_adpf_coal_hard_literal_label_via_name_matcher() {
        // Edge seed: coal hard = 27.91 — literal-label species
        // matched via NameAndCompartment, not Cas. Witnesses the
        // hybrid matcher pattern.
        let f = adpf_factors()
            .into_iter()
            .find(|f| {
                matches!(&f.match_on, FactorMatch::NameAndCompartment { name, .. } if name == "coal hard")
            })
            .expect("coal hard factor missing — hybrid matcher broken");
        assert_eq!(f.value, 27.91);
    }

    #[test]
    fn cml_ia_adpf_ranking_oil_gt_gas_gt_coal_hard_gt_lignite() {
        // Energy-density ranking: oil (41.87 MJ/kg) > natural gas
        // (38.84 MJ/m3) > coal hard (27.91 MJ/kg) > lignite (13.96 MJ/kg).
        let by_cas = |cas: &str| {
            adpf_factors()
                .into_iter()
                .find(|f| matches!(&f.match_on, FactorMatch::CasCompartment { cas: c, .. } if c == cas))
                .unwrap()
                .value
        };
        let by_name = |name: &str| {
            adpf_factors()
                .into_iter()
                .find(|f| matches!(&f.match_on, FactorMatch::NameAndCompartment { name: n, .. } if n == name))
                .unwrap()
                .value
        };
        assert!(by_cas("8012-95-1") > by_cas("8006-14-2"));
        assert!(by_cas("8006-14-2") > by_name("coal hard"));
        assert!(by_name("coal hard") > by_name("coal soft, lignite"));
    }

    #[test]
    fn cml_ia_adpf_matchers_are_hybrid_cas_or_name_with_resource_prefix() {
        // V1 shipping discipline witness: ADP-fossil is the *only*
        // category that mixes Cas and NameAndCompartment within one
        // factor list. The mix is dictated by source-data convention
        // (some species have real CAS, some use literal labels).
        // Both branches must use the resource-compartment prefix.
        let mut saw_cas = false;
        let mut saw_name = false;
        for f in adpf_factors() {
            match &f.match_on {
                FactorMatch::CasCompartment { compartment, .. } => {
                    saw_cas = true;
                    assert_eq!(compartment, &vec!["resource".to_string()]);
                }
                FactorMatch::NameAndCompartment { compartment, .. } => {
                    saw_name = true;
                    assert_eq!(compartment, &vec!["resource".to_string()]);
                }
                other => panic!(
                    "ADP-fossil matcher must be CasCompartment or NameAndCompartment: {other:?}"
                ),
            }
        }
        assert!(saw_cas, "ADP-fossil must include at least one CAS-keyed entry");
        assert!(saw_name, "ADP-fossil must include at least one Name-keyed entry");
    }
}
