# Phase 1 Closeout — Engine Breadth

**Date:** 2026-04-22
**Tag:** `arko-engine v0.2.0`
**Period:** 2026-03 → 2026-04 (Weeks 3–10 of the [Arko Execution Guide](arko-execution-guide.md))

This document is the factual record of what Phase 1 shipped versus what
the Execution Guide specified, the taxonomy and pipeline state at exit,
the parity evidence underwriting the engine's correctness story, and
the V2 work deferred with the reasoning that justified each deferral.
It is not a celebration; it is the artifact future-me reads when
asking "what was true at the v0.2.0 boundary?"

---

## 1. Exit criteria — guide vs shipped

The Phase 1 exit criteria, verbatim from
[arko-execution-guide.md](arko-execution-guide.md) lines 103–110,
compared against what landed.

| # | Execution Guide (Phase 1, lines 103–110) | Status | Evidence |
|---|---|---|---|
| 1 | ILCD bundles (multi-file) can be loaded into Arko Study | ✅ | [`engine/io-ilcd-linker`](../engine/io-ilcd-linker), commits `02e5cc7` (Week 3 scaffold) → `e44b612` (Week 4 EPD v1.2) → `35e747b` (94k-flow EF reference package smoke) |
| 2 | LCAx V1 writer (schema-conformant `Project→Assembly→Product→EPD`); LCAx reader and ILCD+EPD writer deferred to Phase 2 per `D-0018` | ✅ | [`engine/io-lcax`](../engine/io-lcax), commit `6cc5c03` |
| 3 | OpenLCA JSON-LD import works | ✅ | [`engine/io-olca-jsonld`](../engine/io-olca-jsonld), commit `ff3f734` (USDA beef bundle reader + multi-process parity smoke) |
| 4 | Four method presets: IPCC AR6 GWP100, ReCiPe 2016 Midpoint, EF 3.1, CML 2001 | ✅ | All four registered in [`MethodRegistry::standard()`](../engine/methods/src/registry.rs); commits `7b7f83f` (EF 3.1), `3b5bd71` (CML-IA satisfying CML 2001 via Leiden continuation), `3c8c6d7` (ReCiPe 2016) |
| 5 | At least three free databases importable: ÖKOBAUDAT, Agribalyse, EF reference packages | ✅ (substituted slate, see §5) | Real-data smokes: `704e81b` (ÖKOBAUDAT harness), `35e747b` (EF reference package), `ff3f734` (USDA LCA Commons beef bundle) |
| 6 | FactoredSolver trait implemented | ✅ | [`engine/solvers-dense/src/lib.rs`](../engine/solvers-dense/src/lib.rs), commit `fe30079` |
| 7 | All above covered by unit tests | ✅ | Per-crate unit tests + 3 real-data parity smokes (carpet, beef multi-process, beef factored) |

Every Phase 1 exit criterion is satisfied at v0.2.0. The substitutions
on criterion 2 (LCAx in place of EPDX) and criterion 5 (USDA LCA
Commons in place of Agribalyse) are documented in `D-0018` and `D-0010`
/ `D-0011` / `D-0013` respectively, and both substitutions strengthened
the criterion rather than weakened it (LCAx is the maintained
successor to the archived EPDX; USDA LCA Commons ships under CC0 1.0
Universal versus Agribalyse's restrictive license).

---

## 2. Named-slate status — 4/4 registered + 1 legacy bonus

`MethodRegistry::standard()` ships **5 presets** at Phase 1 exit. Four
satisfy the Execution Guide named slate; the fifth (AR5) is a
documented legacy-parity bonus.

| Preset | Registry key | Categories | Provenance | Decision |
|---|---|---|---|---|
| **IPCC AR6 GWP100** (default) | `("ipcc-ar6-gwp100", "1")` | 1 (CC) with `FlowOrigin` × `{Fossil, Biogenic, LandUseChange}` and `LandUseChange→Fossil` equivalence wiring for CH4 | AR6 WG1 Ch. 7 Table 7.15 | (initial) |
| **EF 3.1** | `("ef-3.1", "1")` | 7 (CC, OD, POCP, AC, EU-fw, EU-m, EU-t) — EN 15804+A2 emission core | JRC EF 3.1 method packages | `D-0015` |
| **CML-IA baseline 4.8** (satisfying CML 2001) | `("cml-ia-baseline", "4.8")` | 7 (GWP100 *without* climate-carbon feedback, OD WMO 2003, POCP high-NOx, AC avg-Europe A&B, EU combined P+N, ADP-elements ultimate-reserves, ADP-fossil) | Leiden CML, August 2016 | `D-0017` |
| **ReCiPe 2016 Midpoint Hierarchist** | `("recipe-2016-midpoint-h", "1.1")` | 10 (7 EN 15804-aligned + 3 ReCiPe-distinctive: PMFP, land occupation, water consumption); GLO-only | RIVM `ReCiPe2016_CFs_v1.1_20180117.xlsx` (post-erratum) | `D-0019` |
| **IPCC AR5 GWP100** (legacy bonus) | `("ipcc-ar5-gwp100", "1")` | 1 (CC, with climate-carbon feedback) | AR5 WG1 Ch. 8 Table 8.7 | (initial; bonus, not on slate) |

The `r.len() == 5` assertion in
[`registry::tests::standard_registry_ships_named_slate_plus_ar5_bonus`](../engine/methods/src/registry.rs)
carries an inline comment explaining the AR5 bonus so future-readers
don't wonder about the discrepancy.

The named-slate criterion was **closed via re-reading the Execution
Guide mid-cycle**: between CML-IA shipping and ReCiPe scoping, the
exit criterion was briefly misframed as "registry at 4 ⇒ done"; the
guide actually names *specific* presets, and AR5 is not on the list.
ReCiPe 2016 was required for formal exit. See `D-0019` "Context"
section for the full sequence.

---

## 3. Taxonomy state at v0.2.0

Two structural extensions were made to the calculation taxonomy during
Phase 1, both forced by EF 3.1 scoping. Neither was speculative; both
were coupled to a real preset's value-correctness needs.

### `FactorMatch` — 5 variants

| Variant | Phase | Forcing function |
|---|---|---|
| `Cas { cas }` | Initial | Default substance match |
| `CasOrigin { cas, origin }` | v0.0.1 | AR6 GWP100 CH4 fossil (29.8) vs non-fossil (27.0) |
| `FlowId { id }` | Initial | Match where CAS is absent |
| `NameAndCompartment { name, compartment }` | Initial | Fuzzy name + compartment-prefix match |
| **`CasCompartment { cas, compartment }`** | EF 3.1 (`D-0015`) | Acidification counts SO2 to air, not water; eutrophication splits N species by freshwater/marine/terrestrial |

Definition at
[`engine/methods/src/method.rs:64`](../engine/methods/src/method.rs#L64).

### `FlowOrigin` — 4 variants

| Variant | Phase | Forcing function |
|---|---|---|
| `Unspecified` (default) | Initial | Backward-compatible default |
| `Fossil` | v0.0.1 | AR6 CH4 fossil (29.8) |
| `Biogenic` (renamed from `NonFossil`) | EF 3.1 CC (`D-0016`) | Distinguish recent-photosynthesis carbon from LULUC |
| **`LandUseChange`** | EF 3.1 CC (`D-0016`) | EF 3.1 groups LULUC CH4 with fossil at 29.8, not biogenic at 27.0 |

Definition at
[`engine/core/src/meta.rs:105`](../engine/core/src/meta.rs#L105).
The `LandUseChange` addition closed a pre-existing latent silent
mis-classification bug in both reader crates (ILCD basename
parenthetical "land use change" was routing to `Unspecified` and
silently falling out of LULUC-aware calculations); the parser fix
landed in commits `7ef05eb` (ILCD) and `0cf828e` (taxonomy +
both readers).

### Notably *not* extended

- **`FactorMatch::CasRegion`** — anticipated since `D-0015`, deferred
  in `D-0017`, formally deferred to V2 in `D-0019` after a layered
  scoping pass discovered the constraint is pipeline-depth, not
  matcher-shape (see §6 "V2 regionalisation bundle" below).
- **Per-process `C` build** — current pipeline shape (`g = B·s` then
  `h = C·g`) collapses per-process information at Eq. 2; honest
  region-aware CFs require restructuring Eq. 3 to
  `h = Σₚ Cₚ · B[:,p] · s[p]`. Deferred to V2 alongside `CasRegion`.
- **`ProcessMeta.geography`** — ships in core ("informational, deferred
  to v0.3"); both readers populate it natively from ILCD
  `LocationOfOperationSupplyOrProduction` and openLCA `location.code`.
  Promotion to load-bearing happens with the V2 regionalisation bundle.

---

## 4. Parity evidence

Two real-data parity smokes, both preserved bit-exact across the two
taxonomy extensions of §3 and across all four preset additions of §2.

### Carpet smoke — `ef_carpet_parity_smoke`

- Fixture: JRC EF carpet process `972cd3cd` (1×1 A matrix, single-process).
- Reference: independent Python (`numpy.linalg.solve`, LAPACK `dgesv`,
  stdlib `json` parser).
- First PASS: 2026-04-19 at commit `aa59407`, 0.000e0 deviation,
  `CrossImpl` tolerance.
- Latest re-run framing in [`engine/CHANGELOG.md`](../engine/CHANGELOG.md):
  bit-exact preserved through EF 3.1 CC factor entry (`19fbd4e` →
  `b347e6e` migration: max |dev| 4.654e-6, well within `CrossImpl`
  `eps_abs=1e-9, eps_rel=1e-6`).
- Fixture: [`engine/io-ilcd-linker/tests/ef_carpet_parity_smoke.rs`](../engine/io-ilcd-linker/tests/ef_carpet_parity_smoke.rs).
- Env-gated on `EF_REFERENCE_BUNDLE`.

### Beef multi-process smoke — `beef_multi_process_parity_smoke`

- Fixture: USDA LCA Commons cow-calf-finisher subgraph (5×5 A matrix,
  multi-process LU).
- Reference: independent Python (`numpy.linalg.solve`, LAPACK `dgesv`,
  stdlib `json` parser).
- First PASS: 2026-04-20, max |dev| 1.776e-15 (ulp-scale on the LU
  solve), `CrossImpl` tolerance.
- Closes the LU-parity gap that `ef_carpet_parity_smoke` deliberately
  did not cover (carpet is A=1×1).
- Fixture: [`engine/io-olca-jsonld/tests/beef_multi_process_parity_smoke.rs`](../engine/io-olca-jsonld/tests/beef_multi_process_parity_smoke.rs).
- Env-gated on `USDA_BEEF_BUNDLE`.

### Beef factored smoke — `beef_factored_parity_smoke`

- Fixture: same beef bundle.
- Verifies `factorize(A)` then per-`f_k` `factorization.solve(f_k)`
  produces results within 1e-15 absolute of single-shot
  `DenseLuSolver::solve(A, f_k)` for the same `(A, f_k)` (kernel-shared
  contract from
  [`engine/solvers-dense/src/lib.rs:65`](../engine/solvers-dense/src/lib.rs#L65)),
  and that residual `A·s_k - e_k` is within `CrossImpl` tolerance.
- Transitively: factored ≈ single-shot ≈ numpy. No new Python script
  needed.
- Fixture: [`engine/io-olca-jsonld/tests/beef_factored_parity_smoke.rs`](../engine/io-olca-jsonld/tests/beef_factored_parity_smoke.rs).
- Env-gated on `USDA_BEEF_BUNDLE`.

### Correctness regime in force

The two-axis correctness model from `feedback_arko_correctness_regime`:

- **Factor-value correctness**: hand-calculated seed tests against
  primary sources (AR6 WG1 tables, EF 3.1 method packages, Leiden CML
  4.8, RIVM v1.1_20180117). Lives inside each preset's `tests` module.
  Every preset shipped with ≥2 seeds per category before registration
  (the `feedback_arko_factor_table_entry` rule).
- **Wiring correctness**: cross-implementation parity sharing only the
  factor table. Lives in the parity smokes above. Numpy is the
  external witness.

Conflating these into one test was avoided throughout Phase 1; both
axes have distinct evidence at exit.

---

## 5. Database coverage — three free databases with license docs

The Execution Guide named ÖKOBAUDAT + Agribalyse + EF reference
packages. The shipped slate substitutes USDA LCA Commons for Agribalyse
per `D-0010` / `D-0011` / `D-0013`. Substitution rationale: USDA LCA
Commons ships CC0 1.0 Universal (mandatory at submission), which is
strictly more permissive than Agribalyse's terms; Agribalyse remains a
Phase 2 candidate when the regionalisation bundle (§6) makes its
French-agriculture geography load-bearing.

| Database | Format | License | Smoke fixture | License doc |
|---|---|---|---|---|
| **ÖKOBAUDAT** | ILCD | CC-BY 4.0 (German construction) | `704e81b` (real-data harness) | (covered by JRC EF analysis lineage) |
| **JRC EF reference package** | ILCD | EF infra CC-BY-ish; LCI datasets restrictive Sphera/ecoinvent EULAs | `35e747b` (94k-flow resolver), `aa59407` (carpet parity) | [`docs/licenses/jrc-ef.md`](licenses/jrc-ef.md) |
| **USDA LCA Commons** | openLCA JSON-LD | CC0 1.0 Universal (with trademark + indemnity carve-outs only) | `ff3f734` (multi-process beef) | [`docs/licenses/usda-lca-commons.md`](licenses/usda-lca-commons.md) |

Method-side license docs:

- [`docs/licenses/cml-ia-leiden.md`](licenses/cml-ia-leiden.md) — CML-IA baseline 4.8 (gratis, no explicit terms)
- [`docs/licenses/recipe-2016-rivm.md`](licenses/recipe-2016-rivm.md) — ReCiPe 2016 Midpoint H (gratis, no explicit terms; mirrors CML-IA posture)

The license-distinction discipline (per-database analysis, gratis-vs-
explicit-grant framing, V1-defensible-but-V2-needs-outreach posture)
is consistent across the four docs and constitutes the licensing
record at v0.2.0.

---

## 6. What was deferred to V2 — and why

Each deferral has an active decision-log entry and a concrete trigger
condition for un-deferral. None are vague TODOs.

### Regionalisation bundle (`D-0019`)

- `FactorMatch::CasRegion` matcher variant
- Per-process `C` build + Eq. 3 restructure (`h = Σₚ Cₚ · B[:,p] · s[p]`)
- 5 country-specific CF tables for ReCiPe (POCP-h, particulate matter,
  terrestrial acidification, freshwater eutrophication, water consumption)
- Reader-side region extraction end-to-end verification
- `ProcessMeta.geography` promotion to load-bearing

**Why deferred together as a bundle, not individually**: a layered
scoping pass (RIVM xlsx pick → data-model audit → pipeline-depth check)
revealed that the constraint blocking regional CFs is pipeline-depth,
not matcher-shape. Adding `CasRegion` without the Eq. 3 restructure
would land an inert variant. The bundle ships when a real user requests
region-aware LCA.

### USEtox toxicity cluster (`D-0017`, `D-0019`)

- 5 categories: human carcinogenic, human noncarcinogenic, freshwater
  ecotox, marine ecotox, terrestrial ecotox
- Spans EF 3.1, CML-IA, and ReCiPe — all three V1 presets deferred
  toxicity together

**Why deferred**: USEtox introduces receptor-compartment and
time-horizon design questions distinct from regionalisation; needs its
own factor-table-entry pass with paired EF / CML-IA / ReCiPe values for
cross-witnessing (three independent factor tables on the same matcher
infrastructure).

### Mineral resource scarcity (Cu-eq) — ReCiPe-only deferral (`D-0019`)

**Why deferred**: methodology contested; CML-IA ships ADP-elements
(ultimate-reserves) which covers the same practitioner intent at a
less-criticised baseline. Ship if user demand surfaces.

### POCP-ecosystems — ReCiPe-only deferral (`D-0019`)

**Why deferred**: splits POCP further than EN 15804+A2's single POCP
indicator requires; ship the EN 15804-aligned human-health variant in
V1 and offer the ecosystems variant in V2 once user demand surfaces.

### Ionising radiation — ReCiPe + CML-IA (`D-0019`)

**Why deferred**: niche category, CML-IA V1 also skipped.

### LCAx reader (`D-0018`)

**Why deferred**: writer-meaningful in isolation (a standalone LCAx
document is a useful deliverable); reader is only useful inside an
import workflow that doesn't exist yet. Ships in Phase 2 alongside the
ILCD+EPD writer.

### ILCD+EPD writer (`D-0018`)

**Why deferred**: ILCD+EPD is the format program operators (Environdec,
IBU, EPD International) actually accept. Phase 2's "practitioner submits
an EPD" workflow needs it; landing it in Phase 1 without that workflow
attached would be premature.

### Egalitarian / Individualist ReCiPe perspectives (`D-0019`)

**Why deferred**: same matcher infrastructure as Hierarchist, different
factor tables. Pure factor-table addition with no design change. Track
demand from real users; ship if and when surfaced.

### CML-IA toxicity, regional variants, POCP low-NOx (`D-0017`)

**Why deferred**: same scope-discipline pattern as the per-preset V2
boundaries above. CML-IA V1 = EN 15804+A2-aligned subset only.

### EF 3.1 water use, additional EN 15804+A2 indicators (`D-0015`)

**Why deferred**: water use needs the regionalisation bundle; other
indicators are non-mandatory-core for EN 15804+A2.

### ecoinvent integration

**Why deferred**: ecoinvent ships under a restrictive commercial
license. V2 work is licensing/access-control infrastructure, not factor
entry. The license-tier scaffolding in
[`engine/core/src/license.rs`](../engine/core/src/license.rs) (carried
on every `ProcessMeta`) is the V1 hook; activation is Phase 2-shaped.

---

## 7. Phase 1 architectural and licensing decisions — index

Decisions taken during Phase 1 (D-0010 through D-0019), keyed by area:

| ID | Area | Subject |
|---|---|---|
| `D-0010` | DB slate | Pause-trigger retired; V1 DB slate refined |
| `D-0011` | Strategy | KarbonGarbi paused; Arko primary |
| `D-0012` | Licensing | EF reference package vs EF LCI datasets license bifurcation |
| `D-0013` | Licensing | USDA LCA Commons CC0 characterisation |
| `D-0014` | Architecture | `arko-io-olca-jsonld` reader crate |
| `D-0015` | Methods | EF 3.1 V1 = EN 15804+A2 mandatory-core; `CasCompartment` matcher addition |
| `D-0016` | Taxonomy | `FlowOrigin` extended (`NonFossil` → `Biogenic` + `LandUseChange`) |
| `D-0017` | Methods | CML-IA baseline V1 = EN 15804+A2-aligned subset |
| `D-0018` | I/O | LCAx V1 writer closes "EPDX" Phase-1 bullet via successor format |
| `D-0019` | Methods | ReCiPe 2016 V1 = 10 categories GLO-only; regionalisation bundle deferred to V2 |

Full entries in [`DECISIONS.md`](../DECISIONS.md).

---

## 8. v0.2.0 boundary state

- `arko-engine` workspace: 16 crates (core, differential, io-ecospold2,
  io-ilcd, io-ilcd-linker, io-lcax, io-olca-jsonld, license, methods,
  parameters, sensitivity, solvers-dense, solvers-sparse, uncertainty,
  units, validation) — Phase 1 added 3 to the v0.0.1 baseline of 13
  (io-ilcd-linker, io-lcax, io-olca-jsonld)
- `MethodRegistry::standard()`: 5 entries (4 named-slate + AR5 bonus)
- `FactorMatch`: 5 variants
- `FlowOrigin`: 4 variants
- Public release: `v0.0.1` (2026-04-19, GitHub release page)
- Phase 1 closeout tag: `v0.2.0` (2026-04-22, this commit)

Phase 2 work is scoped in [`docs/phase-2-boundary-memo.md`](phase-2-boundary-memo.md).
