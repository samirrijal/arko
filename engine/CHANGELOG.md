# Engine changelog

All notable changes to the Arko calculation engine are recorded here.
Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
Versioning follows the calc specification (`specs/calc/`) — engine
releases track the spec version they implement.

## [Unreleased]

### Added — CML-IA baseline v4.8 V1 preset registered; 4/4 standard-registry method presets (2026-04-22)

Closes the Phase-1 method-preset exit criterion. `MethodRegistry::standard()`
now ships **four** presets — AR6 (default for new climate studies),
AR5 (legacy parity, *with* climate-carbon feedback), EF 3.1
(EN 15804+A2 mandatory-core, shippable-EPD floor), and **CML-IA
baseline v4.8** (legacy-EPD verification + side-by-side with EF 3.1,
GWP100 *without* climate-carbon feedback). Seven coupled changes
landing in one commit because the matcher work, the factor entry,
the registry registration, and the source-attribution discipline
together constitute one shippable preset.

**1. New `engine/methods/src/cml_ia.rs` module** — public
`cml_ia()` returning `ImpactMethod` keyed `("cml-ia-baseline", "4.8")`.
Version key matches the Leiden release verbatim (4.8), not Arko's
internal V1/V2 staging — a future V2 that adds toxicity or regional
variants is still derived from CML-IA v4.8 source data and would
ship at the same `(id, version)` key with a different `name`, never
reissued with a different factor table. Module-level doc preamble
(~80 lines) explains the V1 EN 15804+A2-alignment scope, the source-
comment discipline, the without-vs-with-feedback distinction, and
references the seven model citations (Guinée 2002, Oers 2001/2016,
WMO 2003, Huijbregts 1999, Heijungs 1992, Jenkin/Hayman 1999,
Derwent 1998).

**2. Seven category factor tables populated** from direct inspection
of `CML-IA_aug_2016.xls` sheet "characterisation factors":

- `gwp100` — 13 species, plain `Cas`. CO2 = 1.0, CH4 = 28 (vs
  AR5-with-feedback 30), N2O = 265 (vs 273), SF6 = 23_500 (vs
  25_200), plus NF3, HFC-23/-134a/-32, CFC-11/-12/-113, PFC-14
  (CF4), PFC-116 (C2F6). Per-factor source comments cite col 14,
  the IPCC-2013-without-feedback model variant, and the
  with-feedback Arko equivalent so the difference is documented at
  every site (anti-"helpful fix" guard).
- `ozone-depletion` — 14 species, plain `Cas`. CFC-11 = 1.0
  (reference), Halon-1301 = 12.0 (highest in CML-IA — note: differs
  from EF 3.1's WMO-1999 ranking where Halon-2402 leads at 15.7).
  The WMO 2003 vs WMO 1999 source-package difference shows up as a
  ranking shuffle on the high-CF tail.
- `photochemical-ozone-formation` — 17 species,
  `CasCompartment`/air-only. Reference is **ethylene** (74-85-1 =
  1.0) — CML uses `kg ethylene-eq`, not `kg NMVOC-eq` like EF 3.1.
  Side-by-side studies will see substantially different absolute
  POCP scores between the two presets; this is the unit-of-account
  difference, not a bug.
- `acidification` — 7 species, `CasCompartment`/air-only. Reference
  is SO2 = 1.2 (NOT 1.0 — the avg-Europe-total Huijbregts variant
  carries fate weighting on the reference). NH3 = 1.6 (highest CF).
- `eutrophication` — 13 species, plain `Cas`. **Compartment-uniform
  per substance** — CML's "fate not incl." baseline doesn't vary
  CFs across air/water/soil compartments. Single combined category
  (P + N together in PO4-eq), unlike EF 3.1's three-way split into
  freshwater/marine/terrestrial. Phosphate = 1.0 reference, P =
  3.06 (highest single-species), full N-bearing set (NH3, NH4+,
  N2O, N2, NO3-, HNO3, NO2, NO, NOx, H3PO4).
- `adp-elements` — 11 high-traffic metals,
  `CasCompartment`/`["resource"]` prefix (single-element compartment
  vs the two-element `["emission","air"]` shape). Sb = 1.0 reference,
  Au = 52.04250371165428 (full 14-significant-digit precision
  preserved verbatim per factor-table-entry discipline; rounding
  this to 52.04 in a "clean up trailing digits" PR would silently
  lose source-fidelity at the version-key level).
- `adp-fossil` — 5 fossil resources, **hybrid matcher** (the only
  V1 category that mixes `Cas` and `NameAndCompartment` within one
  factor list). Real CAS for natural gas (8006-14-2 = 38.84 MJ/m3)
  and crude oil (8012-95-1 = 41.87); literal-label
  `NameAndCompartment` for "coal hard" (27.91), "coal soft, lignite"
  (13.96), and the generic "fossil fuel" reference (1.0). Driven
  by source-data convention, not taste. Witnessed by a dedicated
  seed test (`cml_ia_adpf_matchers_are_hybrid_cas_or_name_with_resource_prefix`)
  that asserts both branches are present.

**3. 38 new seed tests** in `cml_ia::tests`. Per category: ≥1 basic
seed (reference species or canonical anchor), ≥1 edge seed (highest
CF or matcher-shape edge), ≥1 ranking invariant (catches wholesale
table swaps), and 1 matcher-shape invariant (catches matcher-type
drift). The GWP100 cluster is the largest (8 tests) because the
without-vs-with-feedback split is the highest-blast-radius
silent-correctness surface. Method-shape tests at the top (id +
version + name + 7 categories + 7 unit strings) lock the public
contract.

**4. `MethodRegistry::standard()` registers CML-IA fourth**, after
EF 3.1. Test rename: `standard_registry_ships_ar5_ar6_and_ef_31`
→ `standard_registry_ships_ar5_ar6_ef31_and_cml_ia` with assertion
`r.len() == 3` → `r.len() == 4`. New test
`standard_registry_has_cml_ia_baseline` asserts the lookup path
resolves and the GWP100 category id + unit are present. The pre-
existing integration test in `tests/end_to_end.rs` bumped from
`reg.len(), 3` → `reg.len(), 4` with an updated comment naming all
four presets.

**5. `engine/methods/src/lib.rs` preamble updated** — removed the
"EF 3.1 ships with empty factor lists; data entry is a separate
landing" stale text (EF 3.1 factors landed with `D-0015`/`D-0016`)
and added a CML-IA bullet pointing at `D-0017` and the license doc.
Module declaration `pub mod cml_ia;` added in alphabetical position.

**6. License analysis doc** at
[`docs/licenses/cml-ia-leiden.md`](../docs/licenses/cml-ia-leiden.md)
— characterises the gratis-with-no-explicit-license posture honestly,
reproduces the source disclaimer verbatim, names the citation form
(with the v4.5→v4.8 row drift Leiden never updated), enumerates
what is *missing* from the source (commercial-use, redistribution,
attribution, term, patent/trademark, choice-of-law all silent),
documents Arko's 4-point rationale for V1 redistribution (factual
data; different selection/arrangement; gratis-with-no-prohibition;
attribution preserved), and lays out commercial-scale requirements
(explicit grant via Leiden outreach, Phase 2-3 task). Per-factor
source-comment template formalised in the doc, including the
GWP100-specific without-feedback note.

**7. `DECISIONS.md` entry `D-0017`** records the V1 scope rationale
including: the four scope corrections that the spreadsheet
inspection surfaced (GWP100 split, AP avg-Europe variant, EP
compartment-uniformity, toxicity-in-baseline-but-deferred); the
hybrid-matcher rationale for ADP-fossil; the EU sui generis Database
Right open question; and the V2 expansion plan.

**Compartment-wiring caveat (honest Phase-1-exit state).** CML's
compartment-keyed categories (POCP, AP, ADP-elements, half of
ADP-fossil) compile and register correctly and pass their
matcher-shape invariant tests, but they only bind to real flow rows
once the bridge layer extracts compartment from process exchanges
into `FlowMeta::compartment` — same caveat that applied to EF 3.1's
AC, POCP, EU-fw/m/t when those landed. Both ILCD and openLCA
readers populate `FlowMeta::compartment` as `Vec::new()` today;
extending the readers with compartment-extraction is a separate
landing (Phase 1 wiring polish, not scope-blocking the CML preset).
The categories that use plain `Cas` (CML's GWP100, ODP, EP) bind
fully today since they have no compartment requirement.

**Cross-preset numerical-divergence notes (intentional, not bugs).**
Side-by-side studies that run AR5 and CML-IA's GWP100 on the same
inventory will see divergent numbers because the factor tables differ
by the climate-carbon-feedback policy choice — Arko's `ipcc-ar5-gwp100`
ships with-feedback values (CH4 = 30, N2O = 273, SF6 = 25_200) per
the AR5 mainline table; CML's `cml-ia-baseline` GWP100 ships
without-feedback values (CH4 = 28, N2O = 265, SF6 = 23_500) per the
CML construction convention. Both are correct against their
respective reference documents. A divergence-witness end-to-end test
across the two presets is *not* added in this commit because it
would require a fresh study fixture; it can land as a follow-up if
a real-world bug surfaces that conflates the two.

Source-of-truth file inventory:

- [`engine/methods/src/cml_ia.rs`](methods/src/cml_ia.rs) — new
- [`engine/methods/src/lib.rs`](methods/src/lib.rs) — preamble
  updated, module declaration added
- [`engine/methods/src/registry.rs`](methods/src/registry.rs) —
  `standard()` registers fourth preset; assertion bumped 3→4;
  new lookup test for `cml-ia-baseline`
- [`engine/methods/tests/end_to_end.rs`](methods/tests/end_to_end.rs)
  — registry-len assertion bumped 3→4
- [`docs/licenses/cml-ia-leiden.md`](../docs/licenses/cml-ia-leiden.md)
  — new
- [`DECISIONS.md`](../DECISIONS.md) — `D-0017` entry
- [`MILESTONES.md`](../MILESTONES.md) — 2026-04-22 entry

Test summary: `cargo test -p arko-methods --lib` → 124 passed
(38 new CML-IA + 86 pre-existing); `cargo test -p arko-methods
--test end_to_end` → 6 passed.

### Added — EF 3.1 V1 preset registered; Climate change factor data lands; `FlowOrigin` taxonomy extended (2026-04-21)

Closes the 6-of-7 → 7-of-7 gap from earlier the same day and lifts
EF 3.1 from "factors entered" to "registered preset users can resolve
through `MethodRegistry::standard()`". Three coupled changes that
ship together because each depends on the others:

**1. `FlowOrigin` extended from 3 to 4 values** (`D-0016`). The prior
enum was `Unspecified | Fossil | NonFossil`. The new enum is
`Unspecified | Fossil | Biogenic | LandUseChange`. `NonFossil →
Biogenic` rename is part of the same change — "biogenic" is the
unambiguous EN 15804+A2 / EF 3.1 / IPCC AR6 term, and the rename
keeps the three-vs-four distinction visible at every call site.

The change closes a latent silent-mis-classification bug: both flow
parsers ([`engine/io-ilcd-linker/src/flow.rs`](io-ilcd-linker/src/flow.rs)
and [`engine/io-olca-jsonld/src/model.rs`](io-olca-jsonld/src/model.rs))
contained an `_ => Unspecified` fall-through for any origin tag they
did not recognise, including the literal string "land use change". A
LULUC methane flow would parse to `NonFossil` and silently match the
biogenic CF (27.0 kg CO2-eq/kg) instead of the LULUC CF (29.8). The
bug surfaced no error, no warning, no `unmatched_flows` entry — just
a wrong number. Test-first parser fixes: failing tests written first
to reproduce the bug as red, then made green by extending the
classification arm. Parser test files split into per-variant tests
(`fossil_classifies_as_fossil`,
`biogenic_and_synonyms_classify_as_biogenic`,
`land_use_change_classifies_as_land_use_change`) so each variant has
its own falsifiable witness.

AR6 preset CH4 expanded from 2 to 3 `CasOrigin` factors with explicit
`LandUseChange = 29.8` (fossil-equivalent per AR6 GWP100 footnote).
Same anti-silent-zero rationale: an unmatched LULUC CH4 is more
dangerous than the same flow producing a documented fossil-equivalent
value. Test renamed `ar6_ch4_is_split_three_ways_with_lulu_as_fossil_equivalent`.

Migration scope: 19 call sites across `engine/core`,
`engine/io-ilcd-linker`, `engine/io-olca-jsonld`, `engine/methods`,
and `engine/differential`. Compiler-driven cascade — the rename
forces the compiler to surface every site, no grep relied upon.

**2. EF 3.1 Climate change factor table populated** (14 factors,
mixed `CasOrigin` + `Cas`):

- CO2 (124-38-9) — three `CasOrigin` factors: fossil = 1.0,
  **biogenic = 0.0**, land-use-change = 1.0. The biogenic-CO2 = 0
  entry is the EN 15804+A2 carbon-neutrality convention: EPDs do
  not count biogenic CO2 emissions toward GWP100 (the carbon was
  sequestered from the atmosphere within the assessment horizon).
  This is a *policy* split layered on top of IPCC AR6's model-level
  CO2 = 1.0, which is why EF 3.1 ships it but the bare AR6 preset
  does not. Worth its own seed test
  (`ef_31_cc_co2_biogenic_is_zero_per_en15804_carbon_neutrality`)
  because the failure mode (wood/paper/biomass-derived embedded
  carbon mis-reported) is a high-blast-radius EPD bug.
- CH4 (74-82-8) — three `CasOrigin` factors mirroring AR6: fossil =
  29.8, biogenic = 27.0, land-use-change = 29.8.
- N2O (10024-97-2) = 273, SF6 (2551-62-4) = 25_200, NF3 (7783-54-2)
  = 17_400, plus HFC-134a, HFC-23, HFC-32, PFC-14 (CF4), PFC-116
  (C2F6) — all plain `Cas`, all matching AR6 since EF 3.1 CC is
  IPCC AR6 GWP100.

Source: JRC dataset `6209b35f-9447-40b5-b68c-a1099e3674a0.xml`
("Climate change"), dataSetVersion 01.00.000, dateOfLastRevision
2022-06-17. CFs are uniform across air subcompartments per substance
(no `CasCompartment` needed — same uniformity-per-compartment pattern
as the other six categories).

10 new CC seed tests added to `ef_31::tests`: per-species basic +
edge values (CO2 fossil, CO2 biogenic, CO2 LULUC, CH4 LULUC, N2O,
SF6), two ranking tests for the origin splits (`biogenic < fossil =
LULUC` for both CO2 and CH4 — catches origin-swap bugs), and two
matcher-shape invariants (only `Cas` or `CasOrigin` allowed; `CasOrigin`
reserved for CO2/CH4 — catches a regression where someone adds an
N2O fossil/biogenic split unaware that EF 3.1 doesn't differentiate).

**3. EF 3.1 registered in `MethodRegistry::standard()`.** The standard
registry now returns 3 methods (was 2: AR6 + AR5; now also `ef-3.1`
v1). `MethodRegistry::len()` test bumped from `== 2` to `== 3` with a
new descriptive assertion message; new `standard_registry_has_ef_31`
test verifies the lookup path and the 7-category shape. Phase 1 exit
slate at 3/4 method presets (CML 2001 and ReCiPe 2016 remain).

**Correctness evidence — migration is semantics-preserving.** Both
parity smokes pass after the taxonomy migration: EF carpet
(`9.180243685... kg CO2-eq/m²` vs Python reference, max |dev|
`4.654e-6`) and USDA beef multi-process (`11.234849... kg CO2-eq/kg
beef LW` vs Python reference, max |dev| `1.776e-15`). Both well within
the workspace `CrossImpl` tolerance (`eps_abs=1e-9, eps_rel=1e-6`).
The taxonomy widened without any existing calculation drifting — the
only behavioural change is that flows that were previously silently
mis-classified are now correctly classified.

### Added — EF 3.1 V1 factor data for 6 of 7 EN 15804+A2 core emission categories (2026-04-21)

Factor tables populated in
[`engine/methods/src/ef_31.rs`](methods/src/ef_31.rs) for six of the
seven EN 15804+A2 core emission indicators: Acidification (AC),
Ozone depletion (OD), Photochemical ozone formation (POCP),
Eutrophication freshwater (EU-fw), Eutrophication marine (EU-m), and
Eutrophication terrestrial (EU-t). Climate change (CC) deliberately
left as an empty factor list — see "not yet shipped" below.

**Why a multi-category landing rather than one preset commit.** The
first category entered (Acidification) surfaced a data-point worth
documenting before moving on, not a blocker: per-species CFs are
uniform across air subcompartments in the JRC source, so
`CasCompartment`'s role for AC is air-vs-non-air exclusion rather
than air-subcompartment distinction. The same uniformity-per-
compartment pattern held through POCP, EU-fw, EU-m, and EU-t — a
consistent property of the EF 3.1 pan-European defaults. Future
category additions (e.g. Particulate Matter in V2) may expose a
finer-grained subcompartment axis; the existing matcher handles that
too but will exercise it.

The Climate change category hit a taxonomy gap instead. EF 3.1 CC
splits CO2 and CH4 across three origin tags — fossil, biogenic,
land-use-change — with biogenic CO2 = 0 and land-use-change CH4 =
29.8 (grouped with fossil, not biogenic). The engine's current
`FlowOrigin` is three-valued (`Unspecified | Fossil | NonFossil`)
and routes land-use-change methane to `NonFossil`, which would map
to the biogenic CF (27.0) and produce a silently-wrong result. Path
A was taken: pause CC, land the six well-covered categories against
the existing taxonomy, and extend `FlowOrigin` in a separate session
before CC entry. (Path B, shipping CC with a TODO and fixing the
taxonomy later, was rejected because the failure mode is silent
value drift, not a loud `DuplicateMatch`.)

**What changed — per category.**

- **Acidification** (5 factors, CasCompartment on
  `["emission","air"]`). SO2 = 1.31 mol H+-eq/kg (basic seed), NH3 =
  3.02 (edge seed), SO3, NO, NO2. Source: JRC dataset
  `b5c611c6-def3-11e6-bf01-fe55135034f3.xml` dataSetVersion
  01.04.000, dateOfLastRevision 2022-06-17.
- **Ozone depletion** (19 factors, plain `Cas`). CFC-11 = 1.0
  (reference species), CFC-12 = 0.73, HCFC family, Halon-1211,
  Halon-1301, Halon-2402 = 15.7 (edge seed, highest CF). Source:
  JRC dataset `b5c629d6-def3-11e6-bf01-fe55135034f3.xml`
  dataSetVersion 02.00.012, WMO 1999 model. OD chose `Cas` over
  `CasCompartment` because ODS flows to non-air compartments are
  physically non-existent — no silent-bug surface from the looser
  matcher.
- **POCP** (17 factors, CasCompartment on `["emission","air"]`).
  NOx + reactive VOCs: ethylene = 1.69 (basic seed), 1,3,5-
  trimethylbenzene = 2.33 (edge seed, highest CF), butadiene,
  isoprene, BTEX, aldehydes, alcohols, acetone. Source: JRC
  dataset `b5c610fe-def3-11e6-bf01-fe55135034f3.xml`, Van Zelm et
  al. LOTOS-EUROS model. `CasCompartment` chosen here (vs OD's
  `Cas`) because VOC-to-water is a realistic inventory flow — the
  matcher acts as a category-scope gate.
- **Eutrophication, freshwater** (6 factors, CasCompartment on
  `["emission","water"]` or `["emission","soil"]`). Three P species
  × two compartments: P (7723-14-0) to water = 1.0 (basic seed,
  reference species), P to soil = 0.05 (edge seed — same CAS, 20×
  lower CF). This P-water/P-soil pair is the canonical
  `CasCompartment` case: the plain `Cas` variant could not
  represent it without a `DuplicateMatch` collision. Source: JRC
  dataset `b53ec18f-7377-4ad3-86eb-cc3f4f276b2b.xml`, Struijs 2008
  CARMEN/EUTREND. Known V1 limit: fresh-vs-sea-water collapse under
  the `water` prefix — the JRC source itself treats unspecified-
  water emissions as freshwater-equivalent, so the matcher matches
  the dataset's own convention. A future `CasRegion` variant would
  refine this.
- **Eutrophication, marine** (9 factors, CasCompartment on
  `["emission","air"]` or `["emission","water"]`). Six CAS-keyed N
  species: NH3 to water = 0.824 (basic seed), NH3 to air = 0.092
  (edge seed, same CAS ~9× lower), NH4+, NO3-, NO2-, NO2, NO.
  Source: JRC dataset `b5c619fa-def3-11e6-bf01-fe55135034f3.xml`
  dataSetVersion 02.00.010, Struijs 2008. Fresh/sea water CFs are
  identical in EF 3.1 EU-marine (the CARMEN matrix routes any
  water emission through the marine zone), so the `water` prefix
  collapse is physically correct here, not a V1 limit.
- **Eutrophication, terrestrial** (6 factors, CasCompartment on
  `["emission","air"]` — air-only category). NH3 = 13.47 mol
  N-eq/kg (basic seed), NH4+, NO3-, NO2-, NO2 = 4.26, NO = 6.532
  (edge seed — CAS-axis selectivity against chemically similar NO2).
  Reference unit is **mol N-eq** (not kg — Seppälä et al. 2006
  accumulated-exceedance model uses molar equivalents). Source:
  JRC dataset `b5c614d2-def3-11e6-bf01-fe55135034f3.xml`
  dataSetVersion 01.02.009.

**What did not change — Climate change stays an empty factor list.**
The `ef_31()` builder still returns a 7-category method (CC kept in
the shape as `climate-change` with `factors: vec![]`) so the scope
contract stays stable. `ef_31_has_seven_categories` still passes.
The empty CC list will be filled in the same session that extends
`FlowOrigin` and fixes the `io-ilcd-linker` LULUC methane
classification.

**Correctness evidence.** 28 tests in the `ef_31::tests` module
(up from 5 at session start): per-category basic + edge seed value
tests, ranking tests that exercise structural intuitions (NH3 > SO2
> NO2 for acidification, mesitylene > propene > methanol for POCP,
Halon > CFC for ozone depletion, water > soil/air for
eutrophications), matcher-shape invariant tests that lock each
category's compartment scope, and the preserved JSON round-trip
test. Every factor carries an inline `// source: ...` comment
citing the JRC dataset UUID, dataSetVersion, and dateOfLastRevision
— applying the factor-table-entry discipline recorded in the
author's session memory.

**Not yet shipped — registry and verification.**

- `MethodRegistry::standard()` still returns 2 methods (AR6 +
  AR5). EF 3.1 will be added in the session that lands CC — a
  6-of-7-categories registry entry is less useful than the complete
  EN 15804+A2 core set.
- No DECISIONS.md entry for the path-A choice yet. That will land
  with the `FlowOrigin` extension session, since the rationale is
  coupled to the taxonomy change.
- MILESTONES.md entry deferred to the same complete-preset landing.

> **Update (later same day):** all three deferrals closed by the
> next entry above — CC factor table populated, EF 3.1 registered in
> `MethodRegistry::standard()`, `D-0016` records the `FlowOrigin`
> extension that path A required, and the 2026-04-21 MILESTONES
> entry marks the registry hitting 3/4 Phase 1 presets.

### Added — `FactorMatch::CasCompartment` variant (2026-04-20)

Fifth variant of the flow-matching enum in
[`engine/methods/src/method.rs`](methods/src/method.rs), keyed on
`(cas, compartment)` with compartment-prefix semantics symmetric to
the existing `NameAndCompartment` variant. Origin-agnostic.

**Why.** EF 3.1 — and any LCIA method with compartment-dependent
CFs — needs to characterise the same substance differently depending
on where it's emitted. SO2 to air is an acidification flow
(CF ≈ 1.31 mol H+-eq/kg); SO2 in water is not. With only `Cas` and
`CasOrigin` available, the author's choices were (a) tag the substance
globally and get the wrong CF for water emissions, or (b) fall back to
name-based matching, which is brittle across databases that spell the
same substance differently. `CasCompartment` is the
CAS-reliable + compartment-specific combination the taxonomy was
missing.

**What changed.**

- New variant: `CasCompartment { cas: String, compartment: Vec<String> }`.
- `FactorMatch::matches()` arm: CAS equality + flow-compartment
  starts-with matcher-compartment (same prefix semantics as
  `NameAndCompartment`).
- `matcher_label()` arm in
  [`engine/methods/src/builder.rs`](methods/src/builder.rs) so
  `DuplicateMatch` errors show the new variant legibly.
- 12 new tests — 9 at the method layer (prefix match,
  deeper-path match, different-compartment rejection, shorter-path
  rejection, no-CAS rejection, wrong-CAS rejection, origin-agnostic
  behaviour, empty-compartment edge case, JSON round-trip) + 3 at the
  builder layer (route-into-matrix, disjoint compartments coexist,
  `Cas × CasCompartment` overlap hard-fails via `DuplicateMatch`).

**What did not change.**

- Builder semantics. "At most one factor per (category, flow)" is
  still the invariant. No priority/fallback ordering exists or was
  added — method authors pick one matcher per (category, flow) pair,
  and overlap is still a hard `DuplicateMatch` error.
- Existing variants. `Cas`, `CasOrigin`, `FlowId`,
  `NameAndCompartment` are unchanged; their serde tags are
  unchanged. EF 3.1 only requires a *new* variant, not surgery on
  existing ones.

**Decision record.** [`DECISIONS.md` `D-0015`](../DECISIONS.md)
carries the full rationale, alternatives considered (extending
`CasOrigin` with an optional compartment field, shipping EF 3.1 with
only existing-taxonomy categories, adding priority-ordered fallback
matching — all rejected), and the paired EF 3.1 V1 scope decision
(7 emission-based EN 15804+A2 core indicators).

### Added — openLCA JSON-LD reader crate `arko-io-olca-jsonld` (2026-04-20)

New crate at [`engine/io-olca-jsonld`](io-olca-jsonld) parses the
openLCA JSON-LD on-disk format published by the USDA LCA Commons
(and the wider Federal LCA Commons) into typed Rust structs, with an
adapter that produces the same `arko_io_ilcd_linker::TypedColumn` the
rest of the engine already consumes.

**Why a separate crate rather than extending `arko-io-ilcd`.** ILCD
XML and openLCA JSON-LD represent structurally similar content but
disagree on every concrete shape: per-object JSON files keyed by
`@id` and `@type`, UUID-addressed units instead of
`dataSetInternalID` integers, `referenceUnit: true` flags instead of
ID pointers, origin encoded as a trailing comma-qualifier
(`"Methane, biogenic"`) rather than a parenthetical. `D-0014`
captures the reasoning for standing up a separate crate: resist
premature-shared-crate promotion until a third reader (post-v0.1)
tells us what the right shared surface actually is.

**Crate shape.**

```text
reader.rs   →  parse_process / parse_flow / parse_flow_property / parse_unit_group
               (no-IO, consume JSON strings, emit native olca types)

bundle.rs   →  OlcaBundle  (on-disk directory walker + lazy loader)

adapter.rs  →  olca_to_typed_column  (the ONLY place that touches
               arko_io_ilcd_linker's TypedColumn / TypedExchange)
```

**Scope bound the crate respects.** v0.1 is scoped to the USDA LCA
Commons beef cattle finishing bundle (five-process cow-calf-finisher
subgraph) — parse what that bundle needs, leave every other openLCA
feature for the next bundle's pressure. Deliberate omissions are
enumerated in [`SUPPORTED.md`](io-olca-jsonld/SUPPORTED.md) so that
future readers can distinguish "feature I punted" from "regression":
`LCI_RESULT` semantics, `allocation` factor blocks, `parameters` +
`mathematicalRelations`, `avoidedProduct` sign-flip, cross-property
exchanges, alternate unit groups, ZIP-packaged bundles, and the
`actors` / `sources` / `categories` / `locations` / `dq_systems`
object kinds are all v0.1-unsupported by design.

**Correctness evidence.**

- 24 unit tests covering the parser (exactly one
  `quantitativeReference`, exactly one `referenceFlowProperty`,
  exactly one `referenceUnit`, wrong `@type` errors cleanly), the
  adapter (sign convention `(input, +magnitude)` and `(output,
  +magnitude)` with sign-flip left to matrix-assembly downstream,
  unit conversion `2 t → 2000 kg`, dangling `defaultProvider`
  errors rather than silently drops, origin classification
  propagates from the flow's name), CAS normalization
  (`"000074-82-8"` → `"74-82-8"`), and comma-tail origin recognition
  (`"biogenic"`, `"non-fossil"`, `"land use change"`, `"short
  cycle"`, `"from soil or biomass stocks"`).
- Real-data structural smoke at
  [`io-olca-jsonld/tests/beef_bundle_smoke.rs`](io-olca-jsonld/tests/beef_bundle_smoke.rs)
  loads all five beef processes end-to-end; verifies finishing's
  three in-bundle `defaultProvider` edges (vitamin, calf, feed),
  calf's two (vitamin, pasture), and zero in-bundle providers on
  the three leaves; asserts `olca_to_typed_column` succeeds on every
  process; verifies `Methane, biogenic`'s CAS trims to `"74-82-8"`
  and its origin classifies as `NonFossil`. Env-var gated on
  `USDA_BEEF_BUNDLE` (same pattern as `EF_REFERENCE_BUNDLE` for the
  ILCD smokes) — bundle is CC0 and *could* be committed but is kept
  on maintainer disk anyway to match the ÖKOBAUDAT smoke pattern
  and avoid dataset bloat on a code repo.

**Dev-deps** (not runtime-deps): `arko-core`, `arko-differential`,
`arko-methods`, `arko-solvers-dense`, `sprs` — all for the parity
smoke landing in the same commit.

### Added — multi-process LU-parity smoke on USDA beef bundle (2026-04-20)

New test at
[`engine/io-olca-jsonld/tests/beef_multi_process_parity_smoke.rs`](io-olca-jsonld/tests/beef_multi_process_parity_smoke.rs)
wires the five-process beef bundle through
`arko_differential::run_single_vector` with a reference value from
an independent Python implementation. Closes the LU-parity gap that
[`ef_carpet_parity_smoke`](io-ilcd-linker/tests/ef_carpet_parity_smoke.rs)
deliberately did not cover — carpet is a pre-aggregated `A = 1×1`
LCI, so `C @ b` collapses the solve; the beef bundle is a genuine
5×5 system with non-diagonal A (calf → finishing, pasture → calf,
vitamin → finishing + calf, feed → finishing), and solving it
requires LU factorization.

**Independence posture of the reference.** The Python reference at
`scratch/parity/beef_reference.py` (not shipped; maintainer-local):

- stdlib `json` parser vs Arko's `serde_json`;
- filesystem glob + dict lookups vs Arko's `OlcaBundle`;
- reimplemented unit-conversion walker vs Arko's `adapter.rs`;
- reimplemented CAS normaliser and comma-tail origin classifier vs
  Arko's `normalize_cas` + `classify_flow_origin_from_name`;
- `numpy.linalg.solve` (LAPACK `dgesv`, partial-pivot LU) vs Arko's
  `DenseLuSolver` (nalgebra partial-pivot LU). Same algorithm
  class, different implementations — the independence is real;
- shared **only** at the AR6 WG1 Ch7 Table 7.15 factor values
  (same posture as `carpet_reference.py`).

A pass exercises (a) A-matrix wiring with diagonals from reference
outputs and off-diagonals routed through `defaultProvider` edges,
(b) B-matrix assembly across UUID-distinct-but-species-identical
elementary flows (CO2 emitted to air vs water both carry CAS
124-38-9 and both characterise under AR6 regardless of compartment),
(c) CAS normalisation from the `000074-82-8` zero-padded form, (d)
origin classification from the comma-tail `", biogenic"`, and (e)
the LU solve itself against an independent linear-system solver.
Factor-value correctness is **not** proven here — that stays with
the hand-calc seed vectors in `arko_differential::seed`
(wiring-vs-factor regime per `feedback_arko_correctness_regime`).

**Tolerance class.** `ToleranceClass::CrossImpl` (`ε_abs = 1e-9`,
`ε_rel = 1e-6`). The reference did not come from Arko itself; the
observed deviation is `1.776e-15` (ulp-scale), well under the
class threshold. Tightening to `ReferenceParity` would over-claim
on a synthetic ground truth.

**Env-var gating.** `USDA_BEEF_BUNDLE=/path/to/USDA_Beef cargo test
-p arko-io-olca-jsonld --test beef_multi_process_parity_smoke --
--ignored --nocapture`.

### Observed — AR6 GWP100 parity against independent Python reference on USDA beef (2026-04-20)

Second external-witness parity on real CC0 data. Run 2026-04-20
against the USDA LCA Commons beef cattle finishing bundle (five
processes: finishing `1b97b691…`, calf `ac2816ed…`, vitamin
`9f9e378b…`, pasture `2185d89c…`, feed `efa8b1d9…`).

|                               | value                         |
|-------------------------------|-------------------------------|
| Arko impact (AR6 GWP100)      | `1.123484925812980e+01` kg CO2-eq / kg beef LW |
| Python reference (AR6 GWP100) | `1.123484925812980e+01` kg CO2-eq / kg beef LW |
| max \|deviation\|             | `1.776e-15`                   |
| max relative deviation        | `1.581e-16`                   |
| tolerance applied             | `eps_abs=1e-9, eps_rel=1e-6`  |
| verdict                       | **PASS**                      |

Species contributions (kg CO2-eq per kg beef LW, signed):

| species            | contribution |
|--------------------|--------------|
| CH4 non-fossil     | `7.504e+00`  |
| N2O                | `3.553e+00`  |
| CO2                | `1.770e-01`  |

**What this supports.** The end-to-end pipeline from openLCA JSON-LD
reader → `OlcaBundle` → `olca_to_typed_column` → `FlowMeta` bridge →
`build_c_matrix` → LU-backed `compute()` → `result.impact[0]`
produces the same AR6 GWP100 number as a reader/unit-converter/
matrix-assembler/solver stack written independently from the same
IPCC factor table. Flow-matching is correct across multi-species
inventory (16 distinct elementary flow UUIDs, 9 aggregated CAS +
origin keys that AR6 characterises), off-diagonal routing through
`defaultProvider` matches the intended DAG, and the LU solve
produces the same scaling vector to IEEE-754 ulp.

**What this still does *not* support.** Methodology correctness of
the AR6 factor values themselves (shared between implementations;
covered by `arko_differential::seed`); parity against a *third*
independent engine such as Brightway 2.5 or openLCA itself (next
follow-up now that carpet + beef are both covered); parity on
parameterised datasets, allocation-factor blocks, or `LCI_RESULT`
documents (all v0.1-unsupported per `SUPPORTED.md`).

Reproducible by setting `USDA_BEEF_BUNDLE` and running the test
under `--ignored --nocapture`. Bundle is CC0 1.0 Universal and
contains no attribution plumbing; the process UUIDs and impact
number in this record are fully reproducible from public data.

**Plausibility signal (not a correctness claim).** The `11.23 kg
CO2-eq / kg beef LW` total sits at the lower end of the 10–30
kg CO2-eq range reported across peer-reviewed beef LCA studies
(spread driven by system — grass-fed vs feedlot — region, and
allocation choices); a US cow-calf-finisher system lines up
with the lower end. The species mix also matches the expected
beef emissions profile: CH4 non-fossil dominates at 7.50
(enteric fermentation + manure), N2O at 3.55 (fertiliser +
manure N), CO2 at 0.18 (small fuel contribution). This is
order-of-magnitude sanity that the wiring isn't producing
nonsense under the independent-reference agreement — it is
**not** a second parity claim. The comma-tail origin rule
("Methane, biogenic" → `NonFossil`) is visibly exercised: had
the classifier fallen through to `Unspecified`, CH4 would not
have characterised under AR6 and the total would be ~3.7
instead of 11.23.

### Changed — Phase 1 third-slot database confirmed: USDA LCA Commons (2026-04-20)

Primary-source license read of the Federal LCA Commons landed at
[`docs/licenses/usda-lca-commons.md`](../docs/licenses/usda-lca-commons.md),
resolving the third slot opened by `D-0012`. Decision entry at
[`DECISIONS.md`](../DECISIONS.md) § `D-0013`.

**License posture:** every dataset in the Commons is dedicated to the
public domain under **CC0 1.0 Universal**, mandatory at submission per
USDA-NAL policy (LCA Commons Submission Guidelines, Final 2018-07-25).
CC0 § 2 is explicit on commercial, advertising, and promotional use.
Two non-copyright carve-outs travel with access (USDA/ARS/NAL
trademark restriction in advertising; Appendix A AS-IS + indemnity);
neither blocks any Arko-ship operation. Strictly more permissive than
the JRC EF reference package (no attribution legally required, no
term expiry, no redistribution ceiling).

**Phase 1 exit slate resolves to:**

1. ÖKOBAUDAT 2024-I — CC-BY-ND-3.0-DE with attribution (imported).
2. JRC EF reference package — EC Decision 2011/833/EU reuse with
   attribution (imported).
3. **USDA LCA Commons — CC0 1.0 Universal, unrestricted (to be
   imported).**

ProBas (Umweltbundesamt) is not rejected, just deferred to a future
fourth-slot question if it ever enters the Arko critical path.

**Format:** LCA Commons publishes in ILCD XML (openLCA-compatible per
Submission Guidelines) — the existing `arko-io-ilcd` reader handles
parsing without net-new format engineering. Import work proper is
incremental from the ÖKOBAUDAT / EF reader surface, not a new crate.

**Phase 2 `arko-license` preset roadmap:** add `usda_lca_commons_cc0`
encoding no legal attribution requirement, unrestricted commercial use
and redistribution, no term expiry, USDA/ARS/NAL trademark carve-out,
AS-IS warranty disclaimer. Serves as the "baseline free" calibration
for the preset registry.

**Hosting ToS flag (not a Phase 1 blocker):** Appendix A's indemnity
runs from "the user" to the Government; when Arko is the immediate
accessor serving customers downstream, the hosting ToS needs
pass-through language. Checklist item for the first paid Arko
hosted-data customer; legal review recommended before that point.

### Added — cross-implementation parity smoke on real EF data (Phase 1, Week 5)

New test at
[`engine/io-ilcd-linker/tests/ef_carpet_parity_smoke.rs`](io-ilcd-linker/tests/ef_carpet_parity_smoke.rs)
wires the carpet LCI through `arko-differential::run_single_vector`
with a reference value produced by an **independent** Python
implementation. Closes the "we produce a finite number, but is it the
right one" gap that [`ef_carpet_calc_smoke`](io-ilcd-linker/tests/ef_carpet_calc_smoke.rs)
deliberately left open.

**Independence posture of the reference.** The Python reference at
`scratch/parity/carpet_reference.py` (not shipped; maintainer-local)
uses a different XML parser (`lxml` vs Arko's `roxmltree` / `quick-xml`),
a different CAS matcher (plain Python dict vs Arko's
`FactorMatch::CasOrigin`), a reimplemented basename-parenthetical
origin rule, and plain `sum()` arithmetic vs Arko's sparse matvec + LU
solve. The two paths share **only** the AR6 WG1 Ch7 Table 7.15 factor
values, which are tabulated IPCC constants. A pass therefore
exercises flow-matching, amount extraction, CF lookup, and arithmetic
wiring against an independently-written engine — not the factor
values themselves (those stay covered by the hand-calc seed vectors
in `arko-differential::seed`).

**Tolerance class.** `ToleranceClass::CrossImpl`
(`ε_abs = 1e-9`, `ε_rel = 1e-6`). The reference did not come from
Arko itself, so `ReferenceParity` would over-claim; `CrossImpl` is
the right level for this kind of witness.

**Scope bound the test respects.** Carpet is a pre-aggregated LCI
result (`A = 1×1`), so the parity check is
**flow-matching + characterization**, not LU factorization. The
LU path stays covered by the hand-calc `l1_coupled_two_process` seed
vector; a multi-process ÖKOBAUDAT parity vector is the next
follow-up. Explicitly called out in the test module docs so the
claim this evidence produces is properly bounded.

**Dev-dep change.** `arko-io-ilcd-linker` now takes
`arko-differential` as a dev-dependency (was previously only
`arko-core` / `arko-methods` / `arko-solvers-dense` / `sprs`).

### Observed — AR6 GWP100 parity against independent Python reference (Phase 1, Week 5)

First external-witness parity on real JRC EF data. Run 2026-04-19
against the same carpet LCI as the prior calc smoke (process
`972cd3cd-25bf-4b70-96e9-eab4bed329f7`).

|                               | value                         |
|-------------------------------|-------------------------------|
| Arko impact (AR6 GWP100)      | `9.180243685487213e0` kg CO2-eq / m² |
| Python reference (AR6 GWP100) | `9.180243685487213e0` kg CO2-eq / m² |
| max \|deviation\|             | `0.000e0`                     |
| max relative deviation        | `0.000e0`                     |
| tolerance applied             | `eps_abs=1e-9, eps_rel=1e-6`  |
| verdict                       | **PASS**                      |

Bit-exact agreement on the summed impact. Both implementations
matched the same 44 flows out of 20,288 elementary flows (one CO2,
one CH4-fossil, etc. each contributing in the same order), and
IEEE-754 double arithmetic happens to accumulate to the same word
under both paths. A deviation of exactly zero is more than the
`CrossImpl` class requires; it does **not** prove the two paths are
mathematically equivalent, only that on this particular input they
agreed to the last bit.

**What this supports.** The end-to-end pipeline from ILCD reader
→ linker `TypedColumn` → `FlowMeta` bridge → `build_c_matrix`
→ `compute()` → `result.impact[0]` produces the same AR6 GWP100
number as a reader/matcher/arithmetic stack written independently
from the same IPCC factor table. Flow-matching is correct at
scale (20k-flow inventory), amount extraction is correct across
the 44 matched cells, and arithmetic wiring doesn't introduce
subtle pipeline losses.

**What this still does *not* support.** Methodology correctness of
the AR6 factor values themselves (not tested here; shared between
implementations); parity on multi-process systems with LU
factorization (carpet is a pre-aggregated 1×1 LCI); parity across
multiple EF processes (still N=1); parity against a *third*
independent engine such as Brightway 2.5 or OpenLCA. The last item
is explicitly deferred: `D-0012` reminds us the EF 3.1 LCI bundle is
not a redistributable reference dataset, so comparison artifacts
stay internal.

Reproducible by setting `EF_REFERENCE_BUNDLE` and running the test
under `--ignored --nocapture`.

**License posture.** Same Sphera-hosted LCI bundle as the prior
calc smoke; see that entry and `D-0012`. The Python reference
script and this parity smoke are internal evidence artifacts, not
redistributable content — the carpet process UUID and impact number
appearing here are internal engineering records consistent with the
maintainer-download / no-redistribution posture.

### Changed — V1 open-EU-database slate refined (Phase 1, Week 5)

Weekend-of-2026-04-19 research while preparing the Week 5 generalisation
test revealed that `D-0005`'s "three free databases" slate conflated
two structurally different kinds of free: **foreground-free** bundles
(ÖKOBAUDAT, the **EF reference package infrastructure** — flows,
flow properties, unit groups, LCIA methods; distributed as standalone
ILCD XML under EC Decision 2011/833/EU reuse terms, readable by us
directly) and **background-ecoinvent-dependent** bundles (Agribalyse
full LCIs — consumable only through SimaPro / Brightway / openLCA
with a licensed ecoinvent background). `D-0010` documents the
refinement:

- **Agribalyse removed from the V1 runtime-ingestible slate.** The
  ADEME DATAVERSE drop is pre-computed EF 3.1 impact results in Excel,
  not ILCD inventory — useful as a reference corpus for
  `arko-differential` §14 (~2,500 published values, CC-BY-4.0 Etalab)
  but not importable by `arko-io-ilcd`. The
  `engine/io-ilcd-linker/tests/agribalyse_smoke.rs` stub is removed in
  this commit; nothing references it.
- **EF reference package infrastructure is now the primary Week 5
  generalisation target.** Module docstring of `ef_reference_smoke.rs`
  updated accordingly.
- **Third foreground-free database named: USDA LCA Commons**
  (2026-04-20, `D-0013`). Primary-source license read landed at
  [`docs/licenses/usda-lca-commons.md`](../docs/licenses/usda-lca-commons.md).
  Every dataset in the Federal LCA Commons is dedicated to the
  public domain under **CC0 1.0 Universal**, mandatory at
  submission per USDA-NAL policy (Submission Guidelines Final
  2018-07-25). Strictly more permissive than the EF reference
  package — no legal attribution requirement, commercial use
  explicit in CC0 § 2, no term expiry, no redistribution ceiling;
  two non-copyright carve-outs (USDA/ARS/NAL trademark in
  advertising, Appendix A AS-IS + indemnity) shape marketing and
  hosting-ToS language but do not constrain the engine path.
  ProBas (Umweltbundesamt) deferred to a future fourth-slot
  question if it ever enters the Arko critical path.

`D-0005`'s "no ecoinvent in V1" commitment is preserved; `D-0010`
sharpens the working definition of what "free" means for that
commitment; `D-0012` (landed same-day) further sharpens it by
splitting the EF reference package (infrastructure, permissive) from
the EF 3.1 LCI datasets (background processes, restrictive
per-licensor EULAs — not V1-eligible).

### Observed — EF 3.1 reader-level generalisation smoke (Phase 1, Week 5)

First real-data signal from the EF reference testing, 2026-04-19. The
smoke harness at
[`engine/io-ilcd-linker/tests/ef_reference_smoke.rs`](io-ilcd-linker/tests/ef_reference_smoke.rs)
was run against an EF 3.1 background-processes export from the EC EF
node (<http://eplca.jrc.ec.europa.eu/EF-node/>), export stamp
`EF3_1_background_processes_2026-04-19T14_59_12`.

Bundle contents: 1 process (LCI result
`972cd3cd-25bf-4b70-96e9-eab4bed329f7`, landscaping synthetic turf
system), 2,443 flows, 7 flow properties, 7 unit groups. Result:
1 / 1 processes parsed; 0 engine failures; 0 bundle data gaps;
20,290 exchanges resolved through the
`flow → flowproperty → unitgroup → reference unit` chain; 7 distinct
reference units (m², m²·a, kg, m³, kBq, MJ, m³·a); runtime ≈55 s on
the maintainer Windows laptop (no resolver cache).

What this supports: `arko-io-ilcd` parses plain ILCD (EF 3.1 Process
v1.1 schema), not only the ILCD+EPD v1.2 superset used by ÖKOBAUDAT;
`arko-io-ilcd-linker` resolves 20k+ elementary-flow chains on
JRC-blessed content with zero engine error. The "is the reader
secretly tuned to ÖKOBAUDAT idioms" concern is falsified at the
reader level.

What this does **not** yet support: reader robustness on broader EF
3.1 content (N=1 here); end-to-end calculation correctness on EF 3.1
data (no calculation performed; methods layer lands in Week 6);
multi-process ILCD ingest on EF (the tested dataset is an LCI result,
i.e. a pre-aggregated node, so no process-to-process product-flow
linking was exercised). See the observation section of
[`ef_reference_smoke.rs`](io-ilcd-linker/tests/ef_reference_smoke.rs)
for the full characterisation, including the LCI-result-vs-unit-process
nuance behind the 20k exchange count.

Bundle is not redistributed (same posture as the ÖKOBAUDAT smoke —
maintainer-downloaded, `#[ignore]`-gated, no fixture committed).
Reproducible by anyone who exports the same stock from the EC EF node
and points `EF_REFERENCE_BUNDLE` at the unpacked `ILCD/` subdirectory.

**License posture.** This bundle is the **Sphera-hosted EF 3.1 LCI
datasets** (background processes), not the JRC-published EF reference
package infrastructure. The LCI datasets are governed by the Sphera
EULA (restrictive; see `D-0012` and
[`docs/licenses/jrc-ef.md`](../docs/licenses/jrc-ef.md)) — not a
permissive open-data license. Use here is internal
engineering-smoke-only: maintainer-downloaded, no redistribution,
no product path.

Next steps tracked: (b) 94k-flow resolver-only smoke on the separate
JRC EF reference package bundle for broader-than-ÖKOBAUDAT flow-parse
evidence; (c) larger-N process export from the EC EF node (or from
a free EF-compliant node per `D-0010`) for multi-process ingest
evidence.

### Observed — 94k-flow resolver smoke on JRC EF reference package (Phase 1, Week 5, step b)

Step (b) from the prior entry, landed the same day. The smoke harness
at
[`engine/io-ilcd-linker/tests/ef_reference_package_resolver_smoke.rs`](io-ilcd-linker/tests/ef_reference_package_resolver_smoke.rs)
was run against the JRC EF reference package 3.1 downloaded from
EPLCA under *Developer - Environmental Footprint* — the
infrastructure-only bundle (flows, flowproperties, unitgroups,
lciamethods, sources, contacts; zero processes by construction).

Bundle contents: **94,062 elementary-flow XML files**. Result:
94,062 / 94,062 flows parsed; 94,062 fully resolved through the
`flow → flowproperty → unitgroup → reference unit` chain;
0 publisher gaps; 0 engine failures. Flow-type distribution:
Elementary 93,993 / Waste 68 / Product 1. Complete reference-unit
set (8 distinct units): `kg` 92,802; `kBq` 964; `m²` 150; `m²·a` 76;
`MJ` 34; `m³` 29; `kg·a` 6; `m³·a` 1. Runtime 821 s on the maintainer
Windows laptop (debug build, with a test-local `CachingResolver`
wrapping the otherwise cache-less `DirectoryBundle`).

What this supports, beyond the prior entry: reader-level breadth
scales from ÖKOBAUDAT's few-thousand flows to ~94k on JRC-curated
content, with zero publisher-side cross-reference gaps — consistent
with the opening expectation that EF reference is cleaner than
ÖKOBAUDAT (which ran at 3.4% gaps on 3,075 processes).

What this does **not** add: anything about multi-process ingest on
EF (this bundle ships no processes — the N=1 single-process
characterisation from the prior entry remains the only process-level
EF evidence); anything about end-to-end calculation correctness (no
calc performed; methods layer still Week 6).

**Engine caching policy unchanged.** The cache lives in the test
file only, not in `DirectoryBundle`. `LinkResolver`'s cache-deferral
policy is still waiting for a production workload to name the shape
of caching it wants; this smoke being a test-suite artifact, not a
user workload, doesn't constitute that evidence.

Bundle is not redistributed (same posture as the single-process
smoke). Reproducible by downloading the EF reference package 3.x
zip from EPLCA and pointing `EF_REFERENCE_PACKAGE_BUNDLE` at the
unpacked `ILCD/` subdirectory.

**License posture.** This bundle is the **EF reference package
infrastructure** published by JRC — flows, flow properties, unit
groups, LCIA methods only. Reusable under EC Decision 2011/833/EU
with attribution (CC-BY 4.0 equivalent). This is the permissive
side of the EF artefact split formalised in `D-0012`; V1-eligible
per that decision.

### Observed — first end-to-end calc smoke on real EF data (Phase 1, Week 5)

A wiring-only smoke verifying that the linker's `TypedColumn` output
structurally feeds `arko-methods::build_c_matrix` and
`arko-core::pipeline::compute` to a finite impact number on real
JRC content. Harness at
[`engine/io-ilcd-linker/tests/ef_carpet_calc_smoke.rs`](io-ilcd-linker/tests/ef_carpet_calc_smoke.rs),
run against the same single-process EF 3.1 export as the prior
single-process entry (process
`972cd3cd-25bf-4b70-96e9-eab4bed329f7`, the EU+EFTA+UK 2023
synthetic-turf carpet repurposing LCI result, 1 m² reference flow).

**What was tested.** Exactly two questions: (1) does the bridge from
linker output to engine input compile and execute without panic on
real EF content at 20k-flow scale; (2) does the resulting `Study`
solve to a finite impact number under both AR6 and AR5 GWP100. The
test deliberately is **not** a methodology validation — no published
reference number exists for this exact dataset, and the smoke
asserts only `impact.is_finite()` plus shape invariants.

**Result.**

| method            | C nnz | match rate          | impact[0]                |
|-------------------|------:|---------------------|--------------------------|
| ipcc-ar6-gwp100   |    35 |  35/20288 (0.17%)   |  8.452703 kg CO2-eq / m² |
| ipcc-ar5-gwp100   |    44 |  44/20288 (0.22%)   |  9.143786 kg CO2-eq / m² |

20,288 elementary flows fed B (after excluding 1 reference product
flow + 1 waste flow). Of those, 4,326 carry a CAS number (21%) and
15,962 don't (79%) — mostly resources (air, aluminium, antimonite,
argon, ores, water) for which CAS is structurally absent at the
inventory level. The 0.17–0.22% match rate is **not** a reader bug:
of ~20k EF elementary flows, only ~30–60 are GHG species AR6/AR5
GWP100 characterizes. Hitting 35 (AR6) / 44 (AR5) intersects the
registry cleanly.

Wall-clock: typed-column build 102.8 s with the test-local
`CachingResolver` wrapper (debug build, Windows laptop); FlowMeta
construction 0.15 s; C-matrix construction 0.03–0.04 s per method;
`compute()` 0.009–0.017 s per method. Whole test 105.8 s.

**Plausibility check (informal).** Synthetic-turf-carpet systems in
the published literature run 5–30 kg CO2-eq / m². Our 8.45–9.14 kg
CO2-eq / m² lands in the lower end of that range — plausible for an
*avoided-burden / repurposing* process where the credits partially
offset the burdens. Order-of-magnitude pass; not a methodology
proof.

**The AR6 vs AR5 delta is the FlowOrigin gap, quantified.** AR6 is
9 nonzeros lighter than AR5 (35 vs 44) and 0.69 kg CO2-eq lower
(8.45 vs 9.14, ~7.5%). The 9 missing AR6 cells are almost certainly
CH4 flows — `arko-io-ilcd-linker::Flow` carries no `FlowOrigin`,
so AR6's `CasOrigin` matchers correctly skip CH4 (per the spec:
"unspecified origin does not match — missing information surfaced
rather than silently papered over"). AR5's plain `Cas` matcher,
origin-agnostic, picks them up at 28x. **The CH4 fossil/non-fossil
split that AR6 enforces costs us about 7.5% of total impact on this
process** until origin parsing lands in the linker. That's a
concrete number, not a hypothetical.

**What this does *not* support.** Methodology correctness on this
dataset (no reference value, sign convention passed verbatim with
no Input/Output flip); calc correctness across the wider EF process
catalogue (still N=1 process); FlowOrigin-aware CH4 matching; any
multi-process LU exercise on EF (LCI result is pre-aggregated, A is
1×1).

**Concrete follow-up unblocked by this smoke**: extend
`arko-io-ilcd-linker` to derive `FlowOrigin` from the ILCD flow XML
(compartment path + name heuristics like *"Methane, fossil"* /
*"Methane, biogenic"* / *"Methane, from soil or biomass stocks"*),
then re-run this smoke to verify AR6 picks the missing 9 cells back
up at 29.8 / 27.0 fossil/non-fossil. Tracked as its own piece of
work — not a drive-by patch inside this smoke.

Bundle is not redistributed; same posture as the prior two EF
entries. Reproducible by setting `EF_REFERENCE_BUNDLE` and running
the test under `--ignored --nocapture`.

**License posture.** Same as the reader-level generalisation entry
above: carpet process is from the Sphera-hosted EF 3.1 LCI datasets,
not the permissive JRC reference package infrastructure. The impact
numbers in the result table are internal evidence, not a published
or redistributable dataset. See `D-0012` and
[`docs/licenses/jrc-ef.md`](../docs/licenses/jrc-ef.md).

### Added — `FlowOrigin` parsing in `arko-io-ilcd-linker` (Phase 1, Week 5)

Closes the FlowOrigin gap surfaced and quantified by the prior
end-to-end calc smoke (~7.5% of total impact under AR6 GWP100 on
the carpet LCI result).

The publisher-side observation that drove the design: JRC EF /
ÖKOBAUDAT / ILCD-network publishers encode origin in the flow
`<baseName>` as a trailing parenthetical:

- `methane (fossil)` → `Fossil`
- `methane (biogenic)` → `NonFossil`
- `methane (land use change)` → `NonFossil`
  (LULUC is treated as non-fossil per the carbon-cycle convention)

Implementation:

- New `FlowOrigin` enum in `arko-io-ilcd-linker::flow`, parallel to
  `arko_core::meta::FlowOrigin`. Variants: `Fossil`, `NonFossil`,
  `Unspecified` (default). The mirror is deliberate — the linker is
  a reader/bridge layer and does **not** depend on the engine-side
  meta types; callers that produce `FlowMeta` from a linker `Flow`
  translate at the boundary.
- `classify_flow_origin(base_name)` parses the trailing
  parenthetical, case-insensitively. Recognised tags: `fossil`
  (→ `Fossil`); `biogenic`, `non-fossil`, `land use change`,
  `short cycle`, `from soil or biomass stocks` (→ `NonFossil`).
  Anything else falls through to `Unspecified` rather than guessing
  — matching AR6's policy of surfacing missing information rather
  than silently characterizing under the wrong factor.
- 7 unit tests cover all observed patterns plus edge cases:
  case-insensitivity inside the parens, whitespace trimming,
  unrecognised tags falling through, non-trailing parentheticals
  being ignored, last-trailing-parenthetical-wins.
- `Flow.origin` and `TypedExchange.origin` plumbed end-to-end so
  bridge code receives the classification without re-parsing.

### Observed — AR6 fossil/non-fossil split closes the carpet smoke gap (Phase 1, Week 5)

Re-running `ef_carpet_calc_smoke` after the FlowOrigin parser
landed gave the result the prior entry's analysis predicted, on
the nose.

|                                 |   before |    after |
|---------------------------------|---------:|---------:|
| AR6 nonzeros                    |       35 |   **44** |
| AR6 impact (kg CO2-eq / m²)     |  8.45270 | **9.18024** |
| AR5 impact (kg CO2-eq / m²)     |  9.14379 |  9.14379 |
| AR6 vs AR5 delta                | 0.691 kg | **0.036 kg** |
| AR6 vs AR5 relative             |     7.5% |  **0.4%** |

The previously-missing 9 cells are picked up; AR6 nonzeros now
match AR5's. AR6 impact closes most of the gap to AR5, leaving
a 0.036 kg CO2-eq / m² residual that is the **real** methodology
delta — IPCC AR6 splits fossil CH4 (29.8) from biogenic (27.0) on
physical grounds (biogenic CH4 eventually returns to CO2 via the
contemporary carbon cycle); AR5's flat 28.0 averages over that.
The remaining AR6 vs AR5 disagreement is no longer a wiring
artifact; it is the methodology.

Origin tagging on this 20,288-flow inventory: **16 fossil, 15
non-fossil, 20,257 unspecified**. The parser caught more than just
the 9 CH4 flows — fossil/biogenic CO2, CO, and other origin-split
species in the EF inventory all classified correctly. Defensive
synonym coverage pays off.

Wall-clock note: typed-column build dropped from 102.8 s to 3.4 s
on this re-run. Not a performance change — the 2,443 distinct flow
XMLs are warm in the OS page cache from the prior run. This is
also why engine-layer caching is still deferred per `LinkResolver`
policy; the OS does most of the work for typical workloads.

What this still does **not** support: methodology validation on
this dataset (no published reference; sign convention still
verbatim); calc correctness across the wider EF process catalogue
(N=1 process); multi-process LU on EF.

### Added — `arko-io-ilcd-linker` 0.0.1 (Phase 1, Week 3)

First crate of Phase 1. Resolves the cross-document refs that
`arko-io-ilcd` deliberately leaves dangling: `flowDataSet`,
`flowPropertyDataSet`, `unitGroupDataSet` — just enough of each to walk
the chain **flow → reference flow property → unit group → reference
unit**, which is the question every `A`-matrix column-builder needs to
answer ("given a flow UUID on an ILCD exchange, what unit is its
amount reported in?").

- `LinkResolver` trait — UUID-keyed lookup over Flow / FlowProperty /
  UnitGroup. The extension point for future `ZipBundle`, HTTP-backed,
  or cached resolvers; callers (e.g. future column-builders) depend on
  the trait, not the concrete backend.
- `DirectoryBundle` — the v0.1 reference implementation, backed by
  the standard EU JRC on-disk layout
  (`processes/<UUID>.xml`, `flows/<UUID>.xml`, `flowproperties/`,
  `unitgroups/`). Lazy by design: opening a bundle is free, every
  `resolve_*` re-reads and re-parses the one file it needs. ÖKOBAUDAT
  has ~1k flows and a calc touches ~50 of them, so caching is deferred
  to first profiling-driven need rather than designed in.
- `resolve_reference_unit(resolver, flow_uuid) -> ReferenceUnit` —
  the one-call chain walker. Returns UUID + name for every link in
  the chain so callers can attribute a unit without re-walking
  (useful for error messages and provenance).
- Typed model: `Flow { uuid, base_name, flow_type, cas,
  reference_flow_property_id, flow_properties }` with
  `FlowType::{Elementary, Product, Waste, Other}`;
  `FlowProperty { uuid, base_name, reference_unit_group_uuid }`;
  `UnitGroup { uuid, base_name, reference_unit_id, units }`;
  `Unit { internal_id, name, mean_value }`. `Flow::reference_flow_property()`
  and `UnitGroup::reference_unit()` are the dangling-ref-safe accessors
  (return `Option`; the chain walker converts `None` to
  `LinkError::MissingInternalId`).
- `LinkError` — `thiserror` enum carrying path context on every
  variant: `Io`, `Xml`, `UnexpectedRoot`, `MissingElement`,
  `MissingAttribute`, `InvalidText`, `MissingInternalId`. Diagnostics
  stay useful across bundles with thousands of datasets.
- Strict on missing cross-refs; permissive on unknown elements
  (same posture as `arko-io-ilcd`). `roxmltree`-based, pure-Rust,
  zero unsafe, default-namespace-transparent (ILCD's dual
  `default + common:` namespaces parse without per-element prefix
  tracking).
- Shared `crate::xml` module — `first_child`, `node_text`, `parse_int`,
  `parse_f64` (with non-finite rejection). Three parsers, one place for
  the boilerplate.
- **Integration tests against a synthetic minimal bundle**
  (`tests/fixtures/minimal_bundle/`): hand-crafted fossil-CO2 flow →
  Mass property → Units-of-mass group with `kg` as reference. Five
  tests cover per-dataset parse, `resolve_reference_unit` end-to-end
  chain walk, and `Io` error on missing flow file. Every link in the
  chain exercised with predictable values — the first Week 3 checkpoint
  from the Execution Guide ("tests against a small synthetic ILCD
  bundle you construct manually").

### Added — `arko-io-ilcd-linker` bridge (Phase 1, Week 3-4)

Closes the gap between `arko-io-ilcd` (single-process XML reader) and
the matrix-assembly code downstream: one call turns a `ProcessDataset`
into a column where every exchange carries its flow's resolved
reference unit and flow type.

- `build_typed_column(&ProcessDataset, &impl LinkResolver)
  -> Result<TypedColumn, LinkError>` — walks every exchange, resolves
  each flow + its reference-unit chain, and packs the result.
- `TypedExchange { data_set_internal_id, direction, flow_uuid,
  flow_name, flow_type, amount, reference_unit, is_reference_flow }`
  — one row per exchange, with everything a column-builder needs to
  decide A-vs-B placement and keep units dimensionally honest.
- `TypedColumn { process_uuid, process_name,
  reference_exchange_internal_id, exchanges }` — the column-builder-
  ready shape.
- `resolve_reference_unit_from_flow(resolver, &Flow)` — extracted
  helper so the bridge can reuse an already-loaded `Flow` without a
  duplicate file read. `resolve_reference_unit(resolver, uuid)` now
  delegates to it.
- `arko-io-ilcd` added as a dependency of `arko-io-ilcd-linker`. The
  bridge is the intended coupling point; future caller crates depend
  on `arko-io-ilcd-linker` and inherit `arko-io-ilcd` transitively.
- Amount semantics at v0.1: `resultingAmount` is passed through
  unchanged and labelled with the resolved reference unit. Multi-
  flow-property unit math (mass ↔ energy for a fuel) is deferred —
  lives in `arko-units`, hooks in later.
- Fail-fast: first unresolvable exchange surfaces the underlying
  `LinkError`. Per-exchange error collection for whole-bundle scans
  is a caller concern.
- **Tests**: two new integration tests against the minimal bundle
  plus a hand-built `processDataSet` fixture
  (`processes/00000000-0000-0000-0000-000000000500.xml`) — one happy
  path (CO2 output, direction + flow type + amount + unit assertions,
  `is_reference_flow == true`), one dangling-flow-ref path (confirms
  `LinkError::Io` bubble-up when the exchange's flow isn't in the
  bundle).

### Added — OEKOBAUDAT real-data smoke harness (Phase 1, Week 4)

The Week 4 checkpoint from the Execution Guide: exercise the full
`arko-io-ilcd` → `arko-io-ilcd-linker` pipeline on a real EU public
database, not just the synthetic bundle.

- New integration test `tests/oekobaudat_smoke.rs` —
  `#[ignore]`-attributed and env-var-gated. Reads
  `OEKOBAUDAT_BUNDLE=<path>`, walks every `processes/*.xml`, parses
  each, builds the typed column through the linker, and asserts
  pipeline invariants (non-empty exchanges, exactly one reference
  exchange per column, non-empty reference-unit name everywhere).
  Prints a unit-distribution and flow-type-distribution summary with
  `--nocapture` — a shape-survey of whatever the current OEKOBAUDAT
  release contains, useful for deciding which specific UUIDs are
  stable enough for stricter known-value assertions later.
- Engine vs publisher-gap classification: bridge errors that are
  `LinkError::Io` (missing flow / flow-property / unit-group XML on
  disk) or `LinkError::FlowHasNoUnitDerivation` are counted as
  **bundle data gaps** (ÖKOBAUDAT publishes ~105 processes that point
  at flow UUIDs it doesn't ship) and tolerated; anything else is an
  **engine failure** and must be zero. The test only asserts on
  engine failures, so ÖKOBAUDAT's publisher-side gaps don't mask real
  engine regressions.
- **Not committed to the repo**: OEKOBAUDAT is CC-BY-ND-3.0-DE. The
  maintainer downloads the bundle, unzips it, and points the env var
  at the root. `.gitignore` excludes
  `engine/**/tests/fixtures/external/` for the convention location.
  CI does not run this test (it's `#[ignore]`d); maintainers run it
  before tagging releases.
- Stricter known-value assertions (e.g. a named concrete-mix process
  with an expected GWP100 contribution) are deferred until LCIA
  method support lands — those tests can't be written without
  `arko-methods` consuming impact-factor tables.
- **Current smoke result against ÖKOBAUDAT 2024-I (3,075 processes):**
  2,970 / 3,075 clean (96.6%), 0 engine failures, 56,430 exchanges
  resolved across 7 reference units (MJ 29747, kg 21779, m³ 3218,
  qm 991, pcs 590, m 102, a 3). The remaining 105 are publisher-side
  data gaps — missing flow XMLs or flows with no unit derivation
  path. Flow type distribution: Other 53460, Product 2967, Waste 3.

### Added — ILCD+EPD v1.2 extension support (Phase 1, Week 4)

Extend `arko-io-ilcd` + `arko-io-ilcd-linker` to read the **ILCD+EPD
v1.2** superset used by ÖKOBAUDAT, EPD Norge, and Environdec. ÖKOBAUDAT
does not validate as plain ILCD — every process exchange uses EN 15804
stage stratification (`<epd:amount module="A1-A3">...</epd:amount>`)
and indicator flows frequently inline their unit group with
`<epd:referenceToUnitGroupDataSet>`. Without v1.2 support, 0 / 3,075
ÖKOBAUDAT processes pipelined clean.

**Guiding principle:** silent wrongness is worse than loud refusal.
Every extension path surfaces as a typed structured warning rather
than defaulting, so downstream callers route to log / telemetry / UI
as they see fit.

**Schema — `arko-io-ilcd`:**
- `EpdModuleAmount { module, scenario, amount }` captured per
  exchange — **stage stratification preserved**, not flattened.
  `module` is the EN 15804 life-cycle stage label (`A1-A3`, `B6`,
  `C4`, `D`, …). Module D amounts stay negative per EN 15804+A2
  convention (benefits beyond the system boundary); we do not flip
  signs.
- `Exchange.epd_modules: Vec<EpdModuleAmount>` — the stratified
  amount table. Empty on vanilla ILCD.
- `Exchange.epd_unit_group_uuid: Option<String>` — the inline
  `<epd:referenceToUnitGroupDataSet>` when present. Authoritative
  override of the flow chain's unit group per ÖKOBAUDAT author
  convention.
- `Exchange.epd_unit_group_short_description: Option<String>` —
  the `<common:shortDescription>` inside the inline ref, kept as a
  last-resort unit label when the unit-group XML isn't resolvable
  (JRC reference-data unit groups are frequently not shipped with
  ÖKOBAUDAT bundles).
- `Exchange.exchange_direction_inferred: bool` — set when the
  process omits `<exchangeDirection>` on the reference flow (common
  in ÖKOBAUDAT). We infer `Output` with a warning, rather than
  silently defaulting.
- `ParseWarning::InferredDirection { data_set_internal_id,
  is_reference_flow }` — typed warning on the dataset for every
  inference. Emitted with `is_reference_flow: true` when the
  omission is on the declared reference exchange.
- `ProcessDataset.warnings: Vec<ParseWarning>` — new field carrying
  all parse-time inferences. Forwarded verbatim onto the typed
  column by the bridge.
- Amount acceptance loosened: an exchange with `<epd:amount>`
  elements (even all empty — "INA", Indicator Not Assessed) is
  accepted with a scalar amount of 0.0 and the module table
  populated. Empty-text `<epd:amount>` entries are silently dropped
  (convention, not zeroed) since zeroing is quantitatively wrong.
- Namespace-tolerant attribute reads: roxmltree's
  `.attribute("module")` only matches empty-namespace attrs; added
  `attr_by_local_name` helper to read `epd:module` / `epd:scenario`
  regardless of prefix.

**Schema — `arko-io-ilcd-linker`:**
- `Flow.reference_flow_property_id: Option<i32>` (was `i32`) —
  ILCD+EPD indicator flows routinely omit `<quantitativeReference>`
  because their unit is declared inline on the exchange. Parser now
  accepts that. `Flow::reference_flow_property()` returns `None`
  when the field is absent.
- `LinkError::FlowHasNoUnitDerivation { flow_uuid }` — new variant
  returned by `resolve_reference_unit_from_flow` when the flow
  declared no unit derivation path at all (neither
  `<quantitativeReference>` nor inline unit group on any referring
  exchange). Classified as a publisher-side data gap, not an engine
  bug — the smoke test tolerates it.
- `UnitResolutionSource::{FlowChain, EpdInline}` — per-exchange
  provenance tag on `TypedExchange.unit_source`. `FlowChain` = the
  unit came from walking flow → flow-property → unit-group. `EpdInline`
  = the unit came from the exchange's `<epd:referenceToUnitGroupDataSet>`.
- `BridgeWarning::UnitGroupDisagreement { data_set_internal_id,
  flow_uuid, inline_unit_group_uuid, chain_unit_group_uuid }` —
  emitted when inline unit-group UUID differs from the one the flow
  chain walks to. **Inline wins** (authoritative), but we warn so
  the mismatch is never silent.
- `BridgeWarning::InlineUnitGroupUnresolved { data_set_internal_id,
  flow_uuid, inline_unit_group_uuid, fallback_unit_label }` —
  emitted when the inline unit group UUID is in the exchange but
  the unit-group XML is missing from the bundle. The bridge falls
  back to the inline `<common:shortDescription>` (usually
  `"MJ"`, `"kg"`, …) so the column still gets a unit label.
- `TypedColumn.parse_warnings` and `TypedColumn.bridge_warnings` —
  warnings plumbed through from parse time and accumulated during
  the bridge walk.

**Fixture — `engine/io-ilcd-linker/tests/fixtures/epd_minimal_bundle/`:**
- Hand-crafted synthetic ILCD+EPD v1.2 bundle — **not** a slice of
  ÖKOBAUDAT (CC-BY-ND-3.0-DE). Six files: 2 flows, 2 flow properties,
  2 unit groups (Mass + Energy), 1 process.
- The process is deliberately pathological:
  - Reference exchange (concrete) **omits `<exchangeDirection>`** →
    exercises `ParseWarning::InferredDirection`.
  - Indicator exchange (PERE) declares Mass in its flow chain but
    **inlines Energy/MJ** via `<epd:referenceToUnitGroupDataSet>`
    → exercises inline-priority AND
    `BridgeWarning::UnitGroupDisagreement`.
  - PERE carries 8 `<epd:amount>` entries across A1-A3, A4, A5,
    C1-C4, D with **Module D negative** (`-0.48`, scenario
    `"Recycled"`) → exercises EN 15804+A2 sign pass-through and
    scenario capture.
- 7 new integration tests in `tests/epd_bundle_tests.rs`.

**Tests remain green on vanilla ILCD:** all 11 `arko-io-ilcd` tests
and all 7 pre-existing `arko-io-ilcd-linker` bundle tests still pass
unchanged. v1.2 is a strict superset — nothing about vanilla ILCD
parse or resolution changes.

**See** `DECISIONS.md` D-0009 for the design rationale, the four
invariants (stage stratification, inline-unit priority, warn-don't-
silently-default, INA dropped not zeroed), and the alternatives
considered and rejected.

### Deferred from this crate (→ v0.2)

- Source and Contact datasets (pure provenance; off the unit-
  resolution path).
- LCIA Method datasets (belong in `arko-methods`, not here).
- Compliance / modelling metadata blocks.
- ZIP-packaged bundles — directory-backed only at v0.1. The
  `LinkResolver` trait is the extension point; a future `ZipBundle`
  plugs in without disturbing callers.
- Caching — re-reads on every call at v0.1 per the lazy-load design.

## [0.0.1] - 2026-04-19

### Added — `arko-core` 0.0.1
- Workspace scaffolded, targeting calc spec **v0.1**.
- `Study`, `ProcessMeta`, `FlowMeta`, `ImpactMeta`, `Parameter`, `Expression`
  AST, `LicenseTier`, `DerivativeRule`, `Provenance`,
  `EffectiveRestriction`, `Warning`, `EngineError` — all types from spec
  §4, §5, §11, §12, §13.
- `Study::canonical_hash` via BLAKE3 for the determinism contract (§7).
- `Solver` trait — the backend contract of §6.3.
- `pipeline::compute` — the §4.3 three-equation pipeline, with license-
  tier join (§11.2) and contributing-process tracking (§12).

### Added — `arko-solvers-dense` 0.0.1
- `DenseLuSolver` — nalgebra-backed dense LU with partial pivoting.
- Intended for `n < 100` per spec §6.3; larger systems will use the
  future `arko-solvers-sparse` (faer) and `arko-solvers-umfpack`.
- Unit tests: identity, non-square rejection, dim-mismatch rejection,
  singular detection, config echoing for provenance.

### Added — `arko-uncertainty` 0.0.1
- `Distribution` enum with all 5 families from spec §9.1: LogNormal,
  Normal, Triangular, Uniform, Beta-PERT. `Distribution::point(v)` is
  the canonical degenerate-lognormal point value.
- Every variant carries `validate()` + `sample()` — validation is
  strict and explicit (negative sd, out-of-order triangular bounds,
  geometric_sd < 1, non-finite inputs all rejected) so malformed
  distributions can't ship silent NaNs through MC runs.
- `run_monte_carlo(config, closure)` — closure-driven runner seeded
  from a Mersenne-Twister stream (`rand_mt::Mt64`). Same seed ⇒
  bit-identical `MonteCarloResult` under the §7.1 + §9.2 determinism
  contract; verified in-test.
- `DimensionStats` — mean, sd, min/max, p05/p25/p50/p75/p95, standard
  error, full sorted samples vector for histogram reuse.
- `MonteCarloConfig.convergence_threshold` drives the `converged` flag
  that surfaces `W_MC_NONCONVERGENT` per spec §13.2.
- 17 tests across `distribution.rs`, `stats.rs`, and
  `tests/monte_carlo_tests.rs` cover validation, sampling bounds,
  determinism, multi-dim consistency, seed independence, dimension-
  mismatch detection, convergence flag behavior, JSON round-trip.

### Deferred from this crate (→ v0.2)
- Sobol' low-discrepancy sampling (spec §9.2 default path). MT-only
  in v0.1. Requires a unified inverse-CDF pipeline.
- Correlated draws (pedigree-based).
- Adjoint sensitivity analysis (§9.3) — belongs with AD in
  `arko-parameters`.

### Added — `arko-solvers-sparse` 0.0.1
- `SparseLuSolver` — faer-backed sparse LU with partial pivoting and a
  COLAMD-family fill-reducing ordering.
- Deterministic matrix construction: triplets sorted by `(col, row)`
  before handing to faer, so the fill-reducing ordering is
  bit-reproducible across runs (spec §7.2).
- Intended for `100 ≤ n < 10_000` per spec §6.3.
- Cross-solver parity tests against `DenseLuSolver` on the §16 example,
  a banded 5×5, and a 20×20 diagonally-dominant sparse system — within
  §8.1 reference-parity tolerance. First down-payment on the §14
  differential harness.
- Version pinning: targets **faer 0.20**. Method names on `SparseColMat`
  (`sp_lu`, `solve_in_place`) have churned across recent faer releases;
  re-verify on upgrade.

### Added — `arko-methods` 0.0.1
- `ImpactMethod`, `ImpactCategory`, `CharacterizationFactor`,
  `FactorMatch` — the data model for a named, versioned method and
  its characterization factor table. `FactorMatch` supports CAS
  (primary), flow-id (fallback), and name+compartment (fuzzy).
- `MethodRegistry` backed by `BTreeMap` for deterministic iteration
  order (spec §7.1). `MethodRegistry::standard()` returns a batteries-
  included instance populated with the default presets.
- `build_c_matrix(method, flows) -> CMatrixBuild` — produces the
  sparse `k × m` characterization matrix plus `ImpactMeta` labels and
  a list of `unmatched_flows` for UI surfacing. Enforces one-factor-
  per-(category, flow) cell: `CMatrixError::DuplicateMatch` surfaces
  author bugs rather than silently summing. Rejects non-finite
  factors.
- Deterministic triplet sort `(row, col)` before handing to sprs so
  the resulting `CsMat` is byte-stable (§7.2).
- **Preset: IPCC AR6 GWP100 (`ipcc-ar6-gwp100` v1)** — 10 canonical
  long-lived GHGs with AR6 Table 7.15 factors: CO2, CH4 (fossil &
  non-fossil — see the AR5/AR6 split entry above), N2O, SF6, NF3,
  HFC-134a, HFC-23, HFC-32, CF4, C2F6. Each factor carries an
  attribution note for audit-log use.
- End-to-end test drives a full `Study` through `pipeline::compute`
  against this method: `0.1 kg fossil CH4 × 29.8 = 2.98 kg CO2-eq`
  to `1e-12` reference-parity tolerance (§8.1). First non-trivial
  test in the repo for the complete methods → builder → pipeline →
  impact chain.
- 18 tests across `method.rs`, `registry.rs`, `builder.rs`,
  `standard.rs`, and `tests/end_to_end.rs`.

### Added — `arko-sensitivity` 0.0.1
- `FactoredSystem { a, f_dense, scaling, generation }` — caches a
  solved `(A, f, s)` triple plus the §10.2 staleness counter.
- `update_rank_1(u, v, solver)` — pure Sherman-Morrison with
  `m = 1 + v^T·A⁻¹·u`, `|m| < 1e-12` → `E_SINGULAR`.
- `update_rank_r(u, v, solver)` — Woodbury identity with nalgebra-LU
  inversion of the `r × r` capacitance matrix; `r = 0` no-ops,
  `r = 1` delegates to `update_rank_1`.
- Convenience entry points required by §10.1: `edit_entry(i, j, δ)`,
  `replace_column(j, new_col)`, `modify_edge(i, j_from, j_to, value)`
  — the last one is a rank-2 update per the spec's "Adding/removing
  an edge" clause.
- `refactor(a, f, solver)` — full recompute path that **resets**
  `generation` to 0 per §10.2.
- `SMW_EPS = 1e-12` — determinant / `m` threshold; tight enough to
  not false-positive on §8.1 problems, loose enough that a truly
  singular update surfaces as `E_SINGULAR` rather than NaN.
- Deterministic `A` rebuild: triplets sorted by `(row, col)` before
  handing to sprs so duplicate-entry accumulation is bit-stable
  (spec §7.2).
- Generation counter invariants verified: bumps on success, not on
  failure or on no-op paths (`δ = 0`, `r = 0`, zero-column replace).
- 17 tests split across `factored.rs`/`updates.rs` inline and
  `tests/smw_parity.rs`. Every update path is verified against a
  full-refactor ground truth within §8.1 tolerance; additional
  coverage for singular-detect, dimension-mismatch, JSON round-trip,
  near-singular pivots, and sequential composition.

### Added — `arko-license` 0.0.1
- `fire_rules(study, scaling)` — walks every process with `|s[j]| >
  EPS_PRESENCE`, evaluates each `DerivativeRule` trigger, and returns
  `Vec<FiredRule>` in byte-stable order (process index then rule
  declaration — matches pipeline's §11.2 join order for determinism).
- `authorize(intent, study, computed, now)` — combines three signals
  into one `Authorization`: base `EffectiveRestriction` flag for the
  intent, worst-outcome across fired derivative rules (`Blocked >
  Watermark > Warn > Allowed`), and expiry check against the caller-
  provided `now`. Fired rules are returned verbatim even on `Blocked`
  so callers can write the audit log required by the §11 spec.
- `Intent::{Publish, Share, Export}` and `Outcome::{Allowed, Warn,
  Watermark, Blocked}` — the `Ord` impl on `Outcome` is deliberate so
  downstream code can use `.max()` across a rule set.
- `presets::ecoinvent_v3`, `sphera_strict`, `open_cc_by`, `custom_user`
  — conservative std-library presets with doc-commented disclaimers
  that they are not legal advice. Seed content for the future
  `specs/license/` ratification.
- **Guiding principle preserved** from `specs/license/README.md`:
  the solve is always legal. This crate only runs at publish/share/
  export time, never during `pipeline::compute`.
- 22 tests across `fire.rs`, `authorize.rs`, `presets.rs`, and
  `tests/authorize_tests.rs` cover every branch: base-flag
  dispatch per intent, threshold boundary (fires at `==`, not below),
  sign-independence of `ScalingGe`, worst-outcome ordering, expiry
  (before/after caller-provided `now`), zero-contribution processes
  skipped, preset end-to-end against a real solve, JSON round-trip.

### Added — `arko-differential` 0.0.1
- **The conformance harness of spec §14.** Ship the *framework* — test-
  vector data model, runner, property checks, report — at v0.0.1. The
  ≥10,000-vector corpus is a populate-over-time operation; the loader,
  runner, and reporter do not need to change as it grows.
- `TestVector { id, level, description, study, expected_h,
  tolerance_class, notes }` — the §14.1 unit of conformance: a fully-
  specified `Study` paired with the expected impact vector and the
  tolerance class under which parity is checked.
- `ConformanceLevel::{L1Basic, L2Full, L3Elite}` with a total order so
  `highest_level_passed` can compare levels monotonically (§14.2).
- `ToleranceClass::{ReferenceParity, CrossImpl}` implementing §8.1's
  `max(ε_abs, ε_rel · |want|)` with the exact (1e-12, 1e-9) vs
  (1e-9, 1e-6) pairs the spec mandates.
- `run_conformance(vectors, solver, config) -> ConformanceReport` —
  drives a `Solver` through every vector, collecting pass / fail /
  engine-error / shape-mismatch verdicts, timings, and worst
  deviations per vector. Deterministic output order = input order.
- `ConformanceReport` is the §14.4 required output: `engine_version`,
  `spec_version`, `solver_name`, `started_at`, `total_ms`, summary
  counts, `highest_level_passed`, and `per_vector`. Round-trips
  through `serde_json` for on-disk `conformance-report.json`.
- `VectorVerdict::{Pass, Fail, ShapeMismatch, EngineError}` with
  `is_pass/is_fail/is_error` classifiers; `Pass` carries
  `max_abs_deviation` + `max_rel_deviation` for regression dashboards,
  `Fail` surfaces the first component that exceeded tolerance plus the
  tolerance actually applied (useful when triaging).
- **§14.3 property tests** — four of five shipped at v0.0.1:
  - `check_scaling_identity` — `A·s ≈ f` to §8.1 parity.
  - `check_idempotent_recompute` — two solves produce bit-identical
    `(s, g, h)` per §7.1 determinism.
  - `check_block_diagonal_independence` — stacking two disjoint
    studies yields the concatenation of their individual scaling
    vectors.
  - `check_sherman_morrison_parity` — the `arko-sensitivity` rank-1
    path matches a full refactor within §8.1.
  - Parameter-rewrite equivalence is deferred to v0.2 (needs
    parameter-expression evaluation wired into `pipeline::compute`).
- `load_vector_directory(dir)` — reads every `*.json` file from a
  directory in sorted filename order (reproducible iteration per §7.1).
  `VectorLoadError` includes the offending path so malformed vectors
  are easy to locate in a corpus of hundreds.
- `seed_vectors()` — five hand-crafted seed vectors covering L1
  (identity-single-impact, two-process-independent, coupled-upper-
  triangular forcing real back-substitution), L2 (allocation metadata
  round-trip), and L3 (determinism-tagged variant). Self-contained
  fixtures so the crate ships with working test material even before
  the on-disk corpus is populated.
- End-to-end conformance tests drive `run_conformance` against the
  seed corpus with `DenseLuSolver`: every vector passes,
  `highest_level_passed == L3Elite`, per-vector order preserved, and
  the report JSON contains every §14.4 required top-level key.
- Property integration tests exercise each `check_*` function against
  seed studies.

### Added — `arko-parameters` 0.0.1
- Expression evaluator for the restricted arithmetic of spec §5.1.
- Kahn's-algorithm topological sort with deterministic tie-breaking.
- Cycle detection → `E_PARAM_CYCLE`; unresolved refs → `E_PARAM_UNRESOLVED`;
  non-finite results (sqrt of negative, div-by-zero, log of non-positive)
  → `E_PARAM_NONFINITE`.
- Public `walk_deps` / `eval_expr` helpers for reuse by matrix-entry
  evaluators.

### Added — `arko-io-ecospold2` 0.0.1
- `parse_dataset(xml)` — reads a single ecospold2 XML document (both
  `<activityDataset>` and `<childActivityDataset>` envelopes accepted).
- Typed model: `ActivityDataset`, `Activity`, `Geography`,
  `IntermediateExchange`, `ElementaryExchange`, `Direction`. Everything
  round-trips through `serde_json` so parsed datasets can be cached.
- `roxmltree` backend — pure-Rust, zero-unsafe, default-namespace-aware.
- Permissive on unknown elements (silently ignored), strict on
  structural violations (missing attributes, ambiguous direction,
  non-finite amounts).
- Integration tests cover: reference-product detection, technosphere-
  input identification, elementary-exchange compartment parsing, JSON
  round-trip, error cases (bad root, missing envelope, both directions,
  child-dataset envelope acceptance).

### Added — `arko-units` 0.0.1
- **UCUM-subset parser** covering the unit strings LCA data actually
  uses: SI bases, SI-derived named units (N, Pa, J, W, Wh, Hz, L,
  bar), and LCA-common composites (`t.km`, `kg.m`, `MJ`, `kWh`,
  `GJ`). Grammar: `.`/`*` multiplication, `/` division
  (left-associative per UCUM 4.2.2), `^` explicit exponent, `m2`
  implicit-positive exponent. Unicode middle-dot `·` and `×`
  normalize to `.`.
- **Full SI prefix set** `Y..y` (1e24..1e-24) including unicode `μ`
  and the ASCII `µ`/`u` micro aliases. Two-character `da` (deka)
  works correctly under greedy longest-match.
- **Semantic tag carrier** — anything after the first ASCII space
  (trimmed and whitespace-collapsed) rides as `tag`. `kg` and
  `kg CO2-eq` parse to the same 7D dimension but are reported **not
  commensurable** because their tags differ. Conversion between
  them is the characterization-factor problem; it is *not* a unit
  problem and must not silently succeed.
- `Dimension` — a 7×`i8` tuple over L/M/T/N/Θ/I/J with `add`/`sub`/
  `scale` for compound-unit arithmetic. `Display` impl emits the
  canonical `L^a M^b T^c N^d Θ^e I^f J^g` form for diagnostics.
- `ParsedUnit { source, dimension, scale_to_si, tag }` — the
  public analyzed form; JSON-serializable for on-disk method
  definitions and round-trippable under `serde_json`.
- Public API: `parse`, `commensurable`, `conversion_factor`,
  `convert`, `convert_str` (string-level convenience), plus
  `check_compatibility(a: &arko_core::Unit, b: &arko_core::Unit)`
  — the hook `arko-validation` uses.
- **Mass atom is `g`, not `kg`.** `kg` parses as prefix `k` +
  atom `g`, net scale 1.0 kg. Tonne `t` = 1000 kg is a distinct
  atom. This mirrors the UCUM convention and avoids double-counting
  the prefix when users write `Mg` (megagram).
- End-to-end conversion identities verified in
  `tests/conversion_tests.rs`: `1 kWh = 3.6 MJ`, `1 t.km = 10^6 kg.m`,
  `1 yr = 365.25 d`, `1 L = 10^-3 m3`, plus every round-trip
  (`MJ → kWh → MJ ≈ identity within 1e-15`).
- **Not covered at v0.0.1 (→ v0.2)**: affine conversions (°C ↔ K),
  UCUM curly-brace annotations (`kg{dry}`), non-metric units (lb,
  ft, BTU), and dimensional analysis across parameterized matrix
  entries. Parser error messages surface the offending identifier
  so unknown units get routed to the right follow-up crate rather
  than silent acceptance.

### Added — `specs/license/v0.1.md`
- First formal draft of the **license-tier policy language**, the
  semantic layer on top of the storage types ratified in
  `specs/calc/v0.1.md` §11. The reference implementation
  (`engine/license/`) targets this draft.
- Sections: guiding principle (solve is always legal), `Intent`
  enumeration, three-signal authorization decision (base flag →
  derivative rules → expiry, most-severe-wins ordering on
  `Outcome`), trigger semantics (`ScalingGe { threshold }` with
  abs-value `≥` semantics, `Always`), action → outcome mapping,
  audit-log requirements (append-only, operator-attestation for
  `warn`/`watermark`, no overrides at the API layer), preset
  catalogue, conformance, determinism contract, error handling,
  forward-compatibility.
- `specs/license/README.md` and `specs/README.md` updated to point
  at the new draft and remove the "Not started" tag.
- v1.0 ratification gated on: (1) preset EULA verification by
  counsel, (2) one independent implementation passing the
  forthcoming conformance corpus, (3) audit-log validation against
  a real-world reporting workflow.

### Added — `arko-io-ilcd` 0.0.1
- ILCD `<processDataSet>` reader — parses EU JRC ILCD process datasets,
  the format underlying **PEF** (Product Environmental Footprint) and
  most European EPDs (EPD-Norge, GlobalEPD/AENOR, DAPcons, EPD Italy,
  ECO Platform members). The single most consequential format outside
  ecoinvent for KarbonGarbi's Spanish industrial market.
- Typed model: `ProcessDataset`, `ProcessInformation`,
  `QuantitativeReference`, `Exchange`, `Direction`. Round-trips via
  `serde_json` so parsed datasets can be cached.
- Captured fields per dataset: dataset UUID, base name, treatment-
  standards-routes qualifier, ISO/region location code, reference
  year, the `dataSetInternalID` of the declared reference flow, and
  per-exchange: internal ID, linked flow UUID/URI, English short
  description, direction, `meanAmount`, `resultingAmount`, parameter
  binding (`<referenceToVariable>`), data-derivation status.
- `roxmltree`-based parser; `has_tag_name` matches local name only,
  so the dual `default + common:` namespace declarations on every
  ILCD root parse without per-element prefix tracking.
- Permissive on unknown / EPD-extension elements (modelling metadata,
  validation, compliance declarations, bundled `<LCIAResults>`),
  strict on structural violations: missing UUID, dangling
  `<referenceToReferenceFlow>` (`MissingReferenceFlow { id }`),
  unknown `<exchangeDirection>`, non-finite amounts.
- Resulting-amount fallback: when `<resultingAmount>` is absent (very
  common in published EPDs) the reader copies `<meanAmount>` forward
  so downstream column-builders have one canonical amount field.
- 11 integration tests over a Spanish CEM I cement fixture: 6
  positive (envelope parse, reference-product resolution,
  parameterized input, elementary emission, JSON round-trip,
  mean-only fallback) and 5 negative (bad root, missing UUID,
  dangling `<referenceToReferenceFlow>`, unknown direction, `NaN`
  amount).

### Deferred from this crate (→ v0.2)
- Flow, FlowProperty, UnitGroup, Source, Contact, LCIA Method
  datasets — separate readers, future crates.
- Full modelling-and-validation / administrative-information
  metadata blocks (preserved-as-strings would bloat the model).
- `<mathematicalRelations>` parameter expressions — name binding
  only at v0.1; expression evaluation belongs in `arko-parameters`.
- Bundled `<LCIAResults>` blocks in published EPDs — they are
  *outputs*, not inputs, and we recompute from B and C.
- Cross-document linking against flow / unit-group catalogues —
  belongs in a future `arko-io-ilcd-linker`.

### Added — `arko-validation` 0.0.1
- `validate(study)` — enforces the §6.1 construction-order checks that
  are implementable in v0.1: parameter DAG well-formedness (delegated to
  `arko-parameters`), license-tier reference resolution, matrix shape
  consistency, user-allocation factor sum ≈ 1.
- Unit-consistency check (§6.1 step 3) now wired to `arko-units` —
  every `ProcessMeta::reference_unit`, `FlowMeta::unit`, and
  `ImpactMeta::unit` must parse under the UCUM-subset grammar;
  unparseable inputs surface as `E_UNIT_INCOMPATIBLE` naming the
  offending field. Cross-dimensional consistency (flows ↔ impacts via
  characterization) and the §6.1 step-6 well-conditioning heuristic
  remain deferred.

### Tests
- `engine/core/tests/minimal_example.rs` — the §16 worked example end
  to end: constructs the two-process steel/electricity study, solves
  `A·s=f`, computes `g = B·s`, `h = C·g`, and asserts
  `s = [1.0, 0.5]`, `g = [2.05]`, `h = [2.05]` within §8.1 tolerance.
- Also asserts the §11.2 effective-restriction join and the §12
  contributing-processes list.

### CI
- `.github/workflows/engine.yml`: `cargo fmt --check`, `cargo clippy
  -D warnings`, `cargo test` on Linux / macOS / Windows.
- `.github/workflows/specs.yml`: markdownlint + lychee link-check on
  `specs/**/*.md`.

### Added — AR5/AR6 GWP100 split (2026-04-19, pre-`v0.0.1` tag)

**Decision:** D-0007 (Option D) in `DECISIONS.md`. Ship both assessment
reports as first-class presets so new studies default to current
science (AR6) and legacy EPD re-verification stays bit-exact on AR5.

**Schema — `arko-core`:**
- `FlowOrigin { Unspecified, Fossil, NonFossil }` added to
  `core::meta`. `#[default]` is `Unspecified`, so existing JSON
  fixtures round-trip unchanged and legacy blobs with no `origin`
  key deserialize cleanly.
- `FlowMeta.origin` added with
  `#[serde(default, skip_serializing_if = "FlowOrigin::is_unspecified")]`
  so the wire format is backward-compatible.
- `FlowOrigin` re-exported from `arko_core::` prelude.
- Three new unit tests in `core/src/meta.rs`: default, JSON skip,
  legacy-blob round-trip.

**Schema — `arko-methods`:**
- `FactorMatch::CasOrigin { cas, origin }` variant added alongside the
  existing `Cas { cas }`. Matches CAS **and** exact `FlowOrigin`:
  a flow with `Unspecified` origin does **not** match a `CasOrigin`
  factor (regression guard against silently applying fossil factors
  to unknown-origin CH4).
- `FactorMatch::matches()` updated; `builder::matcher_label()` updated.
- Four new unit tests exercising `CasOrigin` match/reject, CAS
  mismatch rejection, plain `Cas` remaining origin-agnostic, and
  JSON round-trip including the new tag.

**Method library — `arko-methods::standard`:**
- `ipcc_ar6_gwp100()` — now splits CH4 into two `CasOrigin` factors:
  fossil `29.8` and non-fossil `27.0` per AR6 WG1 Ch7 T7.15.
  **N2O verified 273** (no drift vs table). All fluorinated-gas
  factors (SF6 `25_200`, NF3 `17_400`, HFC-134a `1_530`, HFC-23
  `14_600`, HFC-32 `771`, CF4 `7_380`, C2F6 `12_400`) reconfirmed
  against AR6 Table 7.15 — **no drift detected**.
- `ipcc_ar5_gwp100()` — new preset. CH4 single-valued `28`, N2O
  `265`, SF6 `23_500`, NF3 `16_100`, HFC-134a `1_300`, HFC-23
  `12_400`, HFC-32 `677`, CF4 `6_630`, C2F6 `11_100` per AR5 WG1
  Ch8 T8.A.1 (column without climate-carbon feedback). CH4 stays
  as a plain `Cas` match — AR5 did not differentiate fossil vs
  non-fossil.
- `MethodRegistry::standard()` now returns both presets; `len() == 2`.
- Nine new unit tests across `standard.rs` (AR6 CH4 split, AR5 CH4
  single-valued + origin-agnostic, AR6 N2O = 273, AR5 N2O = 265,
  distinct `(id, version)` keys).

**Seed corpus — `arko-differential::seed`:**
- `l1_two_process_independent`: CH4 flow tagged `FlowOrigin::Fossil`;
  CF updated `27.9 → 29.8`; `expected_h: [2.79] → [2.98]`.
- `l1_coupled_two_process`: CH4 flow tagged fossil; CF updated
  `27.9 → 29.8`; `expected_h: [34.32] → [35.84]` (re-derived from
  `s = [4, 4]`, `g = [12, 0.8]`, `h = 12·1 + 0.8·29.8`).
- **New vector** `l1_ch4_non_fossil_origin_split`: paired regression
  guard using `FlowOrigin::NonFossil` and `27.0`, `expected_h = [2.70]`.
  An engine that confuses the origin split will silently compute
  `2.98` here and fail.

**End-to-end test — `arko-methods::tests::end_to_end`:**
- `full_pipeline_with_real_method_and_real_numbers`: CH4 flow tagged
  fossil; `expected_h: 2.79 → 2.98`.
- `build_c_matrix_against_ipcc_flows`: CH4 flow tagged fossil so the
  `CasOrigin{fossil}` matcher fires; assertions unchanged structurally.
- **New test** `ar6_rejects_unspecified_origin_ch4_as_unmatched`:
  asserts that an `Unspecified`-origin CH4 flow surfaces in
  `unmatched_flows` rather than silently inheriting the fossil GWP.
- **New test** `ar5_and_ar6_agree_where_they_should_and_disagree_where_they_should`:
  cross-method consistency guard. A single study with 0.06 kg fossil
  CH4 + 0.04 kg non-fossil CH4 is computed under both presets —
  AR6 must return `2.868` (split-weighted, and **not** `2.98` or
  `2.70`), AR5 must return `2.8` (origin-agnostic), and the two
  results must differ. Catches the class of bug where the CH4 split
  silently collapses to a single origin in dispatch.
- `standard_registry_contains_ipcc_gwp100_and_it_resolves` updated
  for `reg.len() == 2`.

**Other construction sites updated for the new field** (each uses
`origin: Default::default()` since they test unrelated machinery):
- `engine/core/tests/minimal_example.rs`
- `engine/validation/src/lib.rs`
- `engine/license/tests/authorize_tests.rs`
- `engine/methods/src/builder.rs` test helper.

**Resolved from Pending:** CH4 GWP100 reconciliation (D-0007) and
N2O verification. The stale `docs/phase-0-compile-risks.md` §2a / §2b
items are now superseded — when cargo goes green the doc becomes a
postmortem per its own exit criterion.

### Fixed — Phase 0 first-green shakedown (2026-04-19)

First-ever `cargo build --workspace && cargo test --workspace` against
the live ecosystem surfaced the following issues. None changed the
engine's public contract; all are fixed in-place.

**Toolchain & resolver:**
- Bumped `rust-toolchain.toml` from `1.83` → `1.95` (D-0008). `1.83`
  predates edition 2024, which is now required by `blake3 1.8.4`
  (direct) and `constant_time_eq 0.4.3` (transitive via blake3).
  1.85 was tried as an interim step but `constant_time_eq` pushed the
  MSRV to 1.95. Rather than pin transitives down individually, sit
  on current stable.

**Code fixes:**
- `core::parameters::Expression` — dropped `#[serde(tag = "kind")]`.
  Internal-tag representation is incompatible with tuple variants;
  switched to default externally-tagged. No committed fixtures used
  the `{"kind": "..."}` shape.
- `units::dimension::Dimension::scale` — marked `const fn` so it can
  appear in the static `ATOMS` table entries for `Hz`, `L`, and `l`.
- `solvers-sparse` — added `use faer::prelude::SpSolver;` so
  `sp_lu().solve_in_place(...)` resolves. Confirmed §1b and §1c of
  `docs/phase-0-compile-risks.md` on the actual faer 0.20.2 surface:
  `sp_lu` works as-is, `solve_in_place` needs the trait imported.
- `sensitivity/Cargo.toml` — added missing `thiserror.workspace = true`
  dependency (was declared in source but not in the manifest).
- `validation/Cargo.toml` — added `sprs.workspace = true` as a
  dev-dependency; inline tests construct `TriMat`s.
- `uncertainty/Cargo.toml` — added `serde_json.workspace = true`
  dev-dep for the JSON-roundtrip tests.
- `license/Cargo.toml` — added `serde_json.workspace = true` dev-dep.
- `differential::vector::TestVector` — dropped unused `PartialEq`
  derive; `Study` isn't `PartialEq`, and the vector is only ever
  identified by its `id` field.
- `differential::runner::compute_highest_level` — fixed vacuous-pass
  bug. An L1+L2-only run was earning `L3Elite` because "no failures
  at level ≤ L3" is not the same as "at least one L3 vector passed."
  Now requires a vector to exist at the exact level being claimed.
- `differential::vector::tolerance_for_scales_with_large_values` —
  replaced bit-equality check with an `abs() < 1e-18` tolerance.
  `1e-9 * 1000.0 = 1.0000000000000002e-6` per IEEE-754 and the test
  was effectively asserting against rounding.
- `core/tests/minimal_example.rs` — transposed the technosphere
  triplet from `(0, 1, -0.5)` to `(1, 0, -0.5)`. The spec §16 narrative
  "1 kg steel needs 0.5 MJ electricity, so s = [1, 0.5]" only holds
  under Brightway convention (`A[product][activity]`), and the engine
  solves `A·s = f` literally without transposing. **Flagged as spec
  errata for v0.1.1** — §16's written `A = [[1.0, -0.5], [0.0, 1.0]]`
  is arithmetically inconsistent with its claimed `s`; it should read
  `A = [[1.0, 0.0], [-0.5, 1.0]]`. The seed vector
  `l1_coupled_two_process` uses the same `(0, 1, -0.5)` triplet but
  is self-consistent because its `f = [2, 4]` is chosen to make the
  literal `A·s = f` yield the expected `s = [4, 4]`.
- `units::parser::parse_unit_expr` — handle bare `"1"` as the canonical
  dimensionless literal. The tokenizer rejects leading digits (they
  aren't identifier starters), so special-cased before tokenizing.
- `methods/tests/end_to_end.rs` — removed trailing string-message
  arguments from `assert_relative_eq!` calls; the `approx` macro
  doesn't accept them (pre-existing typo, surfaced here for the first
  time). Behavior unchanged.

### Documentation — Phase 0 pre-cargo sweep
- `engine/README.md` rewritten to reflect the actual 13-crate workspace
  (was stuck on "Not started. No Cargo workspace yet").
- Root `README.md` repository-layout section updated to list the real
  crate directory names (`solvers-dense`, `solvers-sparse`, `io-ecospold2`,
  `io-ilcd`, `methods`, `sensitivity`, `units`, `validation`, `license`)
  instead of the outdated placeholder paths.
- `DECISIONS.md` seeded at repo root with six initial entries dated
  2026-04-19: dual-license policy, Rust 1.83 + faer 0.20 pin with a
  version-drift caveat, MT-only Monte Carlo in v0.1, solver thresholds
  (dense<100 / sparse 100-10k / UMFPACK 10k+), no ecoinvent in V1,
  strategic upgrade of Arko from side-quest to primary product.
- `docs/phase-0-compile-risks.md` added: pre-cargo self-audit flagging
  faer 0.20 API surface bets in `engine/solvers-sparse/src/lib.rs`
  (`try_new_from_triplets`, `sp_lu`, `solve_in_place`, `Col` index
  mutation) and an **AR6 GWP100 CH4 factor discrepancy** — shipped
  `27.9` does not match AR6 Table 7.15 (fossil `29.8` / non-fossil
  `27.0`). Downstream `engine/differential/src/seed.rs` uses 27.9 in
  `l1_two_process_independent` (`h=[2.79]`) and `l1_coupled_two_process`
  (`h=[34.32]`); both must be updated in lockstep when the CH4 factor
  is reconciled.

### Pending
- `arko-solvers-umfpack` — UMFPACK bindings for `n ≥ 10,000`.
- `arko-io-epdx`, `arko-io-openlca-jsonld`.
- `arko-io-ilcd-linker` — resolve flow / unit-group / LCIA-method
  cross-references across an ILCD bundle into a column of `A` and
  rows of `B`. The reader gives one process; the linker turns a
  zip of processes + flows + unit groups into a study.
- Populate the `arko-differential` corpus toward the §14 ≥10,000-vector
  requirement — reference dumps from Brightway 2.5, OpenLCA, SimaPro.
  The framework is in place; only the on-disk JSON fixtures need
  generating.
- Parameter-rewrite equivalence property (§14.3 fifth property) —
  blocked on parameter-expression evaluation inside `pipeline::compute`.
- Additional method presets: ReCiPe 2016 Midpoint, CML 2001 baseline,
  TRACI 2.1, EF 3.1 (16 categories).
- `FactoredSolver` trait — let `arko-sensitivity` reuse a cached
  factorization instead of re-factoring once per rank-r solve.
- License-spec **conformance corpus** (`specs/license/v0.1.md` §10).
  Generate study + computed + intent + now tuples that bracket every
  preset's threshold edge, expiry instant, and join-collapse case;
  freeze expected `Authorization` JSON for cross-implementation
  parity testing.
