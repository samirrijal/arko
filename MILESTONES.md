# Arko · Milestones

Append-only log of dated project-level events. Strategic decisions live
in `DECISIONS.md`; engine changes live in `engine/CHANGELOG.md`; this
file tracks dated public-facing events future-you will want findable in
thirty seconds ("when did Arko first become a public artifact?" "when
was v0.1 cut?" "when did the first external contributor land?").

Format: `YYYY-MM-DD · event — link(s)`. Newest-first. One event per
bullet. Keep entries short; link out to the richer record.

---

- **2026-04-22** · **`arko-engine v0.2.0` tagged — Phase 1 (Engine Breadth) closeout** —
  retrospective at [`docs/phase-1-closeout.md`](docs/phase-1-closeout.md),
  Phase 2 scope at [`docs/phase-2-boundary-memo.md`](docs/phase-2-boundary-memo.md).
  Closes the Execution Guide Phase 1 exit criteria
  ([`docs/arko-execution-guide.md:103-110`](docs/arko-execution-guide.md#L103))
  with all seven items satisfied (named-slate 4/4, ILCD multi-file
  loader, OpenLCA JSON-LD import, LCAx V1 writer, three free databases
  importable, FactoredSolver trait, unit-test coverage). `MethodRegistry::
  standard()` ships 5 presets at the boundary (4 named-slate + AR5
  legacy-parity bonus); `FactorMatch` taxonomy at 5 variants
  (`CasCompartment` added per `D-0015`); `FlowOrigin` taxonomy at 4
  variants (`LandUseChange` added per `D-0016`). Three real-data parity
  smokes green: carpet (bit-exact), beef multi-process (1.776e-15), beef
  factored (≤1e-15 vs single-shot, transitive to numpy). Workspace at
  16 crates (Phase 1 added `io-ilcd-linker`, `io-lcax`, `io-olca-jsonld`
  to the v0.0.1 baseline of 13). V2 deferrals all carry decision-log
  entries with concrete trigger conditions: regionalisation bundle
  (`D-0019`), USEtox toxicity (`D-0017`/`D-0019`), LCAx reader +
  ILCD+EPD writer (`D-0018`), egalitarian/individualist ReCiPe
  perspectives (`D-0019`).
- **2026-04-22** · **ReCiPe 2016 Midpoint Hierarchist V1 registered — Phase 1 named-slate criterion closed (4/4)** —
  [`engine/methods/src/recipe_2016.rs`](engine/methods/src/recipe_2016.rs)
  factor table landed earlier the same day at commit `a69cbd5`
  (10 categories, 47 seed tests, 2,256 lines, full per-factor RIVM
  source-traceability comments); this commit wires the preset into
  [`MethodRegistry::standard()`](engine/methods/src/registry.rs).
  Closes the Phase-1 named-slate criterion **"four method presets
  registered"** (Execution Guide §1 line 107): IPCC AR6 (default),
  EF 3.1 (EN 15804+A2 core), CML-IA baseline 4.8 (satisfying
  CML 2001 via Leiden continuation lineage), and ReCiPe 2016
  Midpoint Hierarchist 1.1 (D-0019, GLO-only V1). Registry length
  bumps 4 → 5; the AR5 GWP100 legacy-parity bonus (with climate-
  carbon feedback, for verifying historical EPDs) accounts for the
  fifth slot — the `r.len() == 5` assertion in
  `registry::tests::standard_registry_ships_named_slate_plus_ar5_bonus`
  carries an inline comment explaining the bonus so future-readers
  don't wonder about the discrepancy. Factor-entry and registration
  intentionally split across two commits per
  `feedback_arko_factors_then_registration` — the registration
  commit is the moment that actually changes runtime behaviour
  (any external user can now resolve `recipe-2016-midpoint-h@1.1`
  through `standard()`), which is the right granularity for the
  named-slate-satisfaction event. Phase 1 remaining punch list:
  `FactoredSolver` trait + Phase 1 closeout (`v0.2.0` tag,
  retrospective doc, Phase 2 boundary memo).
- **2026-04-22** · **LCAx v3.4 writer V1 shipped — Phase 1 "EPDX"
  execution-guide bullet closed via the successor format** —
  new crate [`engine/io-lcax`](engine/io-lcax) emits
  schema-conformant LCAx v3.4 `Project` documents from an Arko
  `Study` + `Computed` pair. EPDX was archived by ocni-dtu
  2024-08-22 in favour of LCAx (same maintainers, Apache-2.0,
  `lcax_models` v3.4 on crates.io); closing the Phase-1 bullet via
  the living successor rather than a dead format. V1 shape: single
  public entry point `write_lcax_project(study, computed, metadata)`,
  `EpdDocumentMetadata` config (named for Phase-2 ILCD+EPD reuse,
  not LCAx-specific), synthetic single-Assembly/single-Product
  `Project` wrapper (schema root-type constraint), all impact values
  at `LifeCycleModule::A1A3` (truthful default until Phase 2-3
  adds stage decomposition), `standard` enum maps `ef-3.1` →
  `EN15804A2` and everything else → `UNKNOWN` with method identity
  preserved in `EPD.comment`. 9 tests (7 unit + 2 smoke exercising
  spec §16 pipeline → writer → JSON round-trip). Scope governed by
  `D-0018` (staged plan — LCAx V1 now, ILCD+EPD V2 before the
  Phase-2 EPD-renderer milestone). Phase 1 remaining punch list:
  ReCiPe 2016 Midpoint + FactoredSolver.
- **2026-04-22** · **CML-IA baseline V1 preset shipped — 4/4 method
  presets registered, Phase 1 method-preset exit criterion met** —
  [`engine/methods/src/cml_ia.rs`](engine/methods/src/cml_ia.rs)
  ships the EN 15804+A2-aligned subset of CML-IA baseline (Leiden
  CML, v4.8, August 2016) as the fourth registered preset
  alongside AR6, AR5 (legacy parity, with feedback), and EF 3.1.
  Seven categories: GWP100 *without* climate-carbon feedback (the
  intentional split from `ipcc-ar5-gwp100`'s with-feedback table),
  ozone depletion (WMO 2003 steady-state), photochemical oxidation
  (high-NOx, ethylene-eq), acidification (avg-Europe total A&B,
  Huijbregts 1999), eutrophication (combined P+N, fate not incl.,
  Heijungs 1992), ADP-elements (ultimate-reserves, Oers 2001),
  ADP-fossil (hybrid `Cas`+`NameAndCompartment` matcher honouring
  source data's mixed-convention identifiers — the only V1 category
  that mixes matcher types within one factor list). 38 new seed
  tests covering basic + edge + ranking + matcher-shape invariants
  per category. Scope governed by `D-0017` (V1 = EN 15804+A2-aligned
  subset; toxicity, regional variants, POCP low-NOx deferred to
  V2). License posture characterised as gratis-with-no-explicit-
  license; full analysis at
  [`docs/licenses/cml-ia-leiden.md`](docs/licenses/cml-ia-leiden.md).
  Closes the Phase 1 *method-preset* exit criterion (4/4 of the
  original four-preset Phase-1 plan); ReCiPe 2016 Midpoint, EPDX
  reader, and FactoredSolver remain as the rest of the Phase 1
  punch list.
- **2026-04-21** · **EF 3.1 V1 preset shipped** —
  [`engine/methods`](engine/methods) registry now ships
  three preset methods (`ipcc-ar6-gwp100`, `ipcc-ar5-gwp100`,
  `ef-3.1`), the third being Arko's first non-climate preset and
  the first to cover the EN 15804+A2 mandatory-core set: Climate
  change, Ozone depletion, Photochemical ozone formation,
  Acidification, Eutrophication freshwater/marine/terrestrial.
  `MethodRegistry::standard()` at 3/4 for Phase 1 exit (CML 2001
  and ReCiPe 2016 remaining). The CC entry shipped alongside a
  `FlowOrigin` taxonomy extension (`Fossil | Biogenic |
  LandUseChange | Unspecified`, see `D-0016`) that fixed a
  pre-existing latent silent-mis-classification bug in the ILCD
  and openLCA flow parsers ("land use change" was routing to
  `Unspecified` and silently falling out of LULUC-aware
  calculations). Migration verified semantics-preserving on both
  the EF carpet parity smoke (`max |dev| 4.654e-6`) and the USDA
  beef multi-process parity smoke (`max |dev| 1.776e-15`). EF 3.1
  scope governed by `D-0015` (V1 = mandatory-core, additional
  EN 15804+A2 indicators deferred to V2).
- **2026-04-20** · Multi-process LU-parity closure on the USDA LCA
  Commons beef cattle finishing bundle — five-process
  cow-calf-finisher subgraph, 5×5 A matrix, Arko (`DenseLuSolver`,
  nalgebra partial-pivot LU) vs independent Python reference
  (`numpy.linalg.solve`, LAPACK `dgesv`, stdlib `json` parser).
  Max abs deviation `1.776e-15`, `CrossImpl` tolerance, **PASS**.
  Closes the LU-parity gap that `ef_carpet_parity_smoke`
  deliberately did not cover (carpet is A = 1×1). First run of the
  new [`arko-io-olca-jsonld`](engine/io-olca-jsonld) reader crate
  (see `D-0014`); parity test at
  [`engine/io-olca-jsonld/tests/beef_multi_process_parity_smoke.rs`](engine/io-olca-jsonld/tests/beef_multi_process_parity_smoke.rs).
- **2026-04-19** · Bit-exact parity on EF carpet smoke against an
  independent Python reference — 0.000e0 deviation, `CrossImpl`
  tolerance, **PASS**. First external-witness parity evidence on real
  JRC EF data (process `972cd3cd`), via
  [`engine/io-ilcd-linker/tests/ef_carpet_parity_smoke.rs`](engine/io-ilcd-linker/tests/ef_carpet_parity_smoke.rs).
  Commit [`aa59407`](https://github.com/samirrijal/arko/commit/aa59407).
- **2026-04-19** · `arko-engine v0.0.1` published — first public,
  tagged, citable, pre-alpha release of the Arko calculation engine.
  Release page:
  <https://github.com/samirrijal/arko/releases/tag/v0.0.1>.
  Release commit: [`daa91c4`](https://github.com/samirrijal/arko/commit/daa91c4).
  Same day: `D-0011` strategic shift (KarbonGarbi paused, Arko primary)
  and Phase 1 Week 5 reader-level generalisation smokes landed
  (EF 3.1 single-process + 94k-flow EF reference package resolver).
