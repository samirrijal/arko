# Arko · Milestones

Append-only log of dated project-level events. Strategic decisions live
in `DECISIONS.md`; engine changes live in `engine/CHANGELOG.md`; this
file tracks dated public-facing events future-you will want findable in
thirty seconds ("when did Arko first become a public artifact?" "when
was v0.1 cut?" "when did the first external contributor land?").

Format: `YYYY-MM-DD · event — link(s)`. Newest-first. One event per
bullet. Keep entries short; link out to the richer record.

---

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
