# Arko · Decision Log

Append-only log of significant architectural, strategic, and licensing decisions.
Every entry carries a date, the decision, and the reasoning behind it. When a
decision is later reversed, add a new entry with a back-reference; never delete
the old one.

Format: newest-first. Dates are `YYYY-MM-DD`, local to the author.

---

## 2026-04-19 · `D-0012` — EF "reference package" vs EF "LCI datasets": license bifurcation

**Context:** `D-0010` listed "the JRC EF reference package" as one of
three V1 *foreground-free* databases under a single permissive
license. A primary-source license deep-read on 2026-04-19
([docs/licenses/jrc-ef.md](docs/licenses/jrc-ef.md)) revealed that
"EF data" is two structurally different artefacts under two different
licenses, wearing the same name:

1. **EF reference package (infrastructure):** flows, flow properties,
   unit groups, LCIA methods, characterisation factors for the 16 EF
   impact categories. Published by JRC. Reusable under EC Decision
   2011/833/EU with attribution — effectively CC-BY 4.0. This is the
   bundle `ef_reference_package_resolver_smoke.rs` resolved 94,062
   flows against (CHANGELOG 2026-04-19).
2. **EF 3.1 LCI datasets (background processes):** the ~20 000-process
   inventory bundle hosted at the Sphera EF node
   (`lcdn.thinkstep.com`) and the ecoinvent-hosted Chemicals Part 1
   lot. Published by Sphera, ecoinvent, CEPE, FEFAC — not JRC — under
   restrictive per-licensor EULAs (Sphera §6: *"You are not allowed
   to use the DATASET for any other purpose than the PERMITTED USE
   […] including for commercial, non-commercial or educational
   purposes."*). Permitted uses limited to PEFCR/OEFSR compliance or
   narrow EU-policy implementation (batteries, PV, ESPR, etc.).
   General PEF/OEF study user term **expired 2025-12-31**. This is
   the bundle `ef_carpet_calc_smoke.rs` pulled the carpet process
   from.

The two artefacts have always been distinct; `D-0010` conflated them
because the Week 5 test plan treated "EF" as a single target. The
infrastructure bundle is genuinely permissive; the LCI bundle is
not.

**Decision:** Refine `D-0010`.

1. **EF reference package (infrastructure) stays on the V1
   foreground-free list.** Reader validation against it is the 2026-
   04-19 94k-flow resolver smoke; that evidence is unchanged.
2. **EF 3.1 LCI datasets are explicitly excluded from V1.** Same
   posture as ecoinvent under `D-0005`: not shipped, not bundled,
   not hosted. Reconsideration deferred to V2+ via an appropriate
   licensing relationship, if that ever makes sense for Arko's
   shape.
3. **Phase 1 exit criterion "three free databases importable"
   tightens.** The two confirmed V1 foreground-free databases are
   ÖKOBAUDAT 2024-I and the EF reference package infrastructure.
   The third slot reopens; `D-0010`'s ProBas candidate stands pending
   a primary-source license check (same discipline as this entry).
   USDA LCA Commons is an equally defensible alternative also
   pending a license check before reader work begins on it.
4. **Internal engineering smoke tests against the EF LCI bundle**
   (maintainer-downloaded, no redistribution, no commercial path,
   results recorded only in internal CHANGELOG) remain defensible
   under Sphera's "dataset/software developer" category — but that
   posture has weakened since 2025-12-31 term expiry for general
   PEF/OEF users. Future smoke entries should note the license
   posture explicitly.

**Reasoning:** Leaving `D-0010` uncorrected would ship a Phase 1
exit claim that silently depends on a restricted data bundle for
one-third of its evidentiary weight. A future auditor, contributor,
or future-maintainer reading the decision log would come away with
the wrong mental model of what's safe to ship. Naming the
bifurcation now — before the claim enters marketing copy, blog
posts, or docs-site content — is much cheaper than retroactively
unwinding it.

The general pattern worth remembering: when a licensing claim feels
loosely sourced, ask whether the thing being claimed about is one
artefact or several artefacts masquerading under a shared name.

**Alternatives rejected:**

- *Amend `D-0010` in place:* the decision log is append-only for
  provenance reasons. A new entry that back-references is the
  correct shape.
- *Frame this as a `D-0010` "reversal condition" trigger:*
  `D-0010`'s reversal condition is specifically about Agribalyse
  republishing without ecoinvent dependency. This bifurcation isn't
  that; it's a refinement of the conceptual model, not a reversal.
- *Keep both EF artefacts as a single V1 slot with a footnote:*
  the footnote approach depends on readers always reading it. Split
  naming is harder to misread.

**Impact on public-facing copy:** safe to say *"Arko parses the
ILCD format used by the EF reference package"* and *"Arko ships the
16 EF 3.1 impact-category characterisation factors as a method
preset, reused under EC Decision 2011/833/EU."* Not safe to say
*"Arko ships EF 3.1 LCI datasets"*, *"Arko supports EF 3.1 out of
the box"* (ambiguous, reads as LCI support), or *"Arko is an open
EF 3.1 platform."* Full do/don't list and attribution templates in
`docs/licenses/jrc-ef.md`.

**Cross-references:** `D-0005` (no ecoinvent in V1), `D-0010`
(foreground-free vs background-ecoinvent-dependent), CHANGELOG
entries 2026-04-19 for the two EF smokes.

---

## 2026-04-19 · `D-0011` — KarbonGarbi paused; Arko is the primary Goibix product; pause trigger retired

**Context:** `D-0006` framed Arko as a first-class product but preserved a
pause trigger ("if KarbonGarbi has <2 paying customers by 2026-09-30,
freeze Arko until it has ≥3"), under the assumption that KarbonGarbi was
the near-term revenue engine funding Arko's 14-month build. Imanol's
2026-04-19 feedback on KarbonGarbi exposed that the current positioning
isn't commercially viable without an ESG-expansion pivot — itself a
significant future project. The user considered and confirmed (not in
the moment, overnight and under direct pushback) that the right move is
to invert: KarbonGarbi goes into graceful suspension, Arko becomes the
primary Goibix product.

**Decision:**

1. **KarbonGarbi enters graceful suspension.** 13 phases complete, no
   new feature development, no active sales push. Code archived/
   documented; the option to resume as "KarbonGarbi ESG" is preserved
   but not scheduled. No KarbonGarbi work — features, refactors, GTM —
   unless the user explicitly re-opens the project.
2. **Arko is the primary Goibix product.** Customer development,
   marketing, revenue path, and brand weight all move to Arko. The
   first Arko paying customer (~Month 12–14 of the execution guide) is
   also Goibix's first revenue. Runway shifts from "KarbonGarbi MRR
   funds Arko" to "day job + savings carry Arko to first customer."
3. **Pause trigger retired.** The `D-0006` reversal condition ("if
   KarbonGarbi <2 paying by 2026-09-30, freeze Arko until ≥3") no
   longer applies. It was constructed for a parallel-tracks world that
   no longer exists.
4. **Weekly split retired.** The Mon–Wed Arko / Thu–Fri KarbonGarbi
   rhythm in the execution guide is obsolete. All working sessions go
   to Arko.
5. **Definition of done revised.** The 14-month target collapses to
   "Arko ≥1 paying customer (€149+ MRR); calc spec cited externally;
   not working 60+ hr weeks; Imanol still a friend." The KarbonGarbi
   MRR clause is removed.
6. **GTM track pulled forward.** Marketing / customer-development was
   nominally deferred to Phase 5 (~Month 10+). With Arko as primary
   product it starts now: register `arko.earth` and `arko.eus` this
   month; reserve `@arko` handles on X / LinkedIn / Bluesky / GitHub
   next month; stand up a one-page "Arko: open LCA engine, private
   beta Q3 2026, email to be notified" site in Month 3; begin a
   ~monthly technical-decisions blog cadence from Month 4
   ("Why we wrote our own LCA calculation engine"; "Understanding EF
   3.1"; "The ecoinvent licensing landscape and why it matters for
   open LCA"). This is Sunday writing time, not development time.
7. **Imanol relationship reframed.** Imanol's feedback on KarbonGarbi
   is honored, not discarded. His role for Arko remains informal
   LCA-methodology advisor. A short professional transition note will
   be sent this week naming the shift explicitly (not oversold, not
   rushed).

**Rationale:** This is the honest version of the situation, not the
optimistic one. KarbonGarbi in its current form needs the ESG pivot to
be commercially viable; that pivot is a major project; running it while
also building Arko dilutes both. Arko addresses a real disruption
window — SimaPro's captive market, no viable open alternative — with a
defensible, differentiated product already partway built (Phase 1 Week
4 shipped, engine parses real data). Betting on Arko is a real bet and
not without risk; but so was KarbonGarbi, and the Imanol data point
showed the edge of that bet. Arko is defensible; the open-EU-database
licensing path is being navigated properly (`D-0005`, `D-0010`); the
engine is real. The content / search-footprint compounding effect of
starting the GTM track 10 months earlier than originally planned is
material.

**Alternatives rejected:**
- *Pivot KarbonGarbi to ESG in-flight:* the ESG expansion is itself a
  significant project; attempting it while also building Arko produces
  two half-finished products instead of one finished one.
- *Keep KarbonGarbi in active sales mode without feature work:* would
  dribble attention without producing either revenue or credibility.
  Graceful suspension is the cleaner stance.
- *Do both on a stricter calendar split:* tried already (`D-0006`);
  the Imanol feedback invalidated the economic premise, not the
  scheduling one.

**Reversal condition:** If at any point in the next ~3 months the user
judges that KarbonGarbi needed the ESG pivot *sooner* rather than
later, treat that as a legitimate revisit. A new entry would document
the re-opening and what changed.

**Scope notes:**
- Phase 1 Week 5 technical work (EF reference package ingestion, ILCD
  reader generalisation) is unchanged by this decision.
- `docs/arko-execution-guide.md` is the canonical roadmap; this entry
  supersedes the pause-trigger and weekly-split sections of it.

---

## 2026-04-19 · `D-0010` — V1 database slate refined: foreground-free vs background-ecoinvent-dependent

**Context:** `D-0005` named **ÖKOBAUDAT, Agribalyse, and the EF
reference package** as the three open EU databases shipping in V1.
Weekend research (2026-04-19) while preparing Phase 1 Week 5's
generalisation test revealed that this slate conflates two
structurally different kinds of "free" database:

- **Foreground-free:** the publisher distributes the LCI bundle itself
  as downloadable ILCD / native XML, standalone, under a permissive
  license. Consuming it requires only the reader. **ÖKOBAUDAT** and
  the **JRC EF reference package** fit this definition.
- **Background-ecoinvent-dependent:** the publisher distributes
  *foreground* modelling but the only way to consume the full LCIs is
  through a tool (SimaPro, Brightway, openLCA) that bundles **ecoinvent**
  as the background system. The "database" is really a foreground
  layer that only produces sensible results when paired with a
  licensed ecoinvent background. **Agribalyse full LCIs** fit this
  definition. The ADEME-hosted DATAVERSE drop
  (`AGRIBALYSE3.2_Tableur...xlsx`) is **pre-computed EF 3.1 impact
  results**, not ILCD inventory — useful as a published reference
  corpus for `arko-differential`, but not something the reader can
  ingest.

**Decision:** Refine `D-0005`'s three-database slate to:

1. **ÖKOBAUDAT** (foreground-free; ILCD+EPD v1.2; construction
   sector). Primary. Pipeline smoke 2,970 / 3,075 clean at
   Phase 1 Week 4.
2. **EF reference package** (foreground-free; ILCD; closest to
   canonical spec; cross-sector). Primary generalisation test target
   for Week 5.
3. **A third foreground-free database, TBD.** The original plan named
   Agribalyse; that slot is now open. **ProBas** (Umweltbundesamt,
   German federal) is the current candidate to evaluate. The "three
   free databases" exit criterion in `D-0005` stands, but the
   identity of the third is deferred until a foreground-free
   candidate is validated.

Agribalyse is **not dropped from the ecosystem**, only from V1 as a
runtime-ingestible source: the DATAVERSE impact-results Excel is a
legitimate Phase 1 Week 8–10 asset for the `arko-differential` §14
conformance corpus (~2,500 published reference values,
CC-BY-4.0 Etalab 2.0). That use case is background-ecoinvent-agnostic
because it compares published totals against our computed totals, not
our LCI import against theirs.

**Rationale:** The distinction matters because `D-0005`'s commitment
was "no ecoinvent in V1." A database whose only consumable LCI path
runs through ecoinvent-bundled tools *is* ecoinvent-in-V1 in all but
name. Keeping Agribalyse in the V1 slate would either quietly violate
`D-0005` or silently ship a broken import path. Naming the two kinds
of "free" separately lets the open-EU-database claim stay honest
without shrinking the ecosystem footprint.

**Alternatives rejected:**
- *Keep Agribalyse on the V1 list and accept the ecoinvent
  dependency quietly:* violates `D-0005` and the open positioning
  narrative. Non-starter.
- *Drop the "three free databases" target entirely:* overcorrects.
  Two is a workable generalisation signal but three is meaningfully
  stronger; the target should stand, only the third name is deferred.
- *Carve out an ecoinvent-dependent path "for testing only":* silent
  coupling of the test rig to a commercial license is a licensing
  time-bomb. No.

**Reversal condition:** If ADEME or a successor publishes a genuinely
ecoinvent-independent Agribalyse LCI distribution, reopen this
entry and consider re-adding Agribalyse to the V1 slate. Until then,
`arko-io-ilcd` does not carry an Agribalyse code path.

**Scope notes:**
- The Phase 1 Week 5 smoke-test stub at
  `engine/io-ilcd-linker/tests/agribalyse_smoke.rs` is removed in the
  same commit as this entry. `ef_reference_smoke.rs` remains as the
  primary generalisation signal for Week 5.
- The DATAVERSE drop (`C:\Users\hical\Downloads\dataverse_files`) is
  retained locally for Phase 1 Week 8–10 conformance-corpus use.

---

## 2026-04-19 · `D-0009` — ILCD+EPD v1.2 support: stage-stratified, inline-unit-priority, warn-don't-silently-default

**Context:** Phase 1 Week 4's first real-data checkpoint fed a full
ÖKOBAUDAT 2024-I bundle (3,075 processes) through the `arko-io-ilcd` →
`arko-io-ilcd-linker` pipeline. Initial pass rate: 0/3,075. Every
dataset hit the same `<exchangeDirection>` requirement on the reference
flow — routine in the ILCD+EPD v1.2 DIN EN 15804 superset used by
ÖKOBAUDAT, EPD Norge, and Environdec construction-industry EPDs.
Extending support was unavoidable: those publishers are the primary
data source for KarbonGarbi's first market (Basque industry) and the
engine's credibility depends on parsing their feeds honestly.

**Decision:** Support ILCD+EPD v1.2 in the same `ProcessDataset` type
used for vanilla ILCD, with four invariants holding:

1. **Preserve stage stratification.** Exchange-level EPD amounts flow
   through as `Vec<EpdModuleAmount { module, scenario, amount }>` on
   the `Exchange` type — not flattened, not summed. The A1-A3 / A4 /
   A5 / B1-B7 / C1-C4 / D topology survives down to the downstream
   calc layer unchanged. Module D negatives pass through verbatim per
   EN 15804+A2 sign convention.
2. **Inline `<epd:referenceToUnitGroupDataSet>` takes priority** over
   the flow → flow-property → unit-group chain. Recorded on
   `TypedExchange.unit_source` as `UnitResolutionSource::EpdInline` vs
   `FlowChain` so auditors can tell the two apart. When both paths
   resolve and disagree on UUID, a
   `BridgeWarning::UnitGroupDisagreement` is emitted — preference is
   fixed (inline wins), but practitioners must see the disagreement.
   When the inline UUID references a JRC core-reference-data file not
   shipped locally (routine for ÖKOBAUDAT), we fall back to the inline
   `<common:shortDescription>` as the unit label and emit a
   `BridgeWarning::InlineUnitGroupUnresolved`.
3. **Missing `<exchangeDirection>` warns, does not silently default.**
   When absent (routine on ILCD+EPD reference-flow exchanges), the
   parser fills in `Output` by EN 15804 convention AND emits a typed
   `ParseWarning::InferredDirection { is_reference_flow }`. Silent
   defaulting would mask genuinely malformed waste-treatment datasets
   whose reference flow was intended as Input. Warning on the
   reference flow is the conventional case; warning on a non-reference
   flow is a strong signal of a bug.
4. **Empty `<epd:amount>` text = INA** (indicator not assessed, per
   EN 15804+A2) — dropped from `epd_modules` rather than fabricated as
   zero. Distinct from a published `0` value. If every module on an
   exchange is INA, the exchange still parses (scalar amount defaults
   to 0.0) because the publisher's *intent* to declare it as an EPD-
   indicator row is recorded via the `<c:other>` block presence.

Module-attribute namespace tolerance: parsers match `module` /
`scenario` by local name, so `<epd:amount epd:module="A1-A3">` and
`<epd:amount module="A1-A3">` both parse (ÖKOBAUDAT vintages disagree;
the iai.kit.edu/EPD/2013 spec uses unprefixed).

**Rationale:** *Silent wrongness is worse than loud refusal.* The
temptation at step (3) was to just default missing directions to
Output and move on — the test would have passed, ÖKOBAUDAT would have
loaded, and the engine would have been subtly wrong on any dataset
whose reference flow is an Input (waste-treatment processes, recycling
credits). The structured `ParseWarning` / `BridgeWarning` enums let
callers route anomalies to logs, telemetry, CI gates, or UI pills —
whatever fits their audit posture — without the engine picking a
routing policy for them. The inline-vs-chain preference at step (2)
likewise mirrors how openLCA, GaBi, and SimaPro treat the EPD
extension: the publisher's inline authority wins, but the chain walk
remains available for audit. Stage stratification at step (1) is the
load-bearing one — flattening A1-A3 + C + D into a single scalar is
cheap at parse time and catastrophic at interpretation time; downstream
calc code cannot reconstruct what was collapsed.

**Alternatives rejected:**
- *Tolerance patch:* make `<exchangeDirection>` optional with a
  silent default. Gets ÖKOBAUDAT loading without crashing but
  produces semantically wrong columns on waste-treatment datasets.
  Violates the engine's core credibility claim.
- *Separate `EpdProcessDataset` type:* parallel ILCD vs ILCD+EPD
  hierarchies would double the maintenance surface. The EPD
  extension is a strict superset of vanilla ILCD; using one type
  with `#[serde(default, skip_serializing_if)]` fields keeps the API
  flat and preserves JSON round-trip compatibility for vanilla
  fixtures (verified: all pre-existing tests pass unchanged).
- *Default-to-zero on INA empty amounts:* would count "this
  indicator was not assessed for this EPD" as "this indicator is
  zero for this EPD". That conflation is precisely the bug EN
  15804+A2's INA convention exists to prevent.

**Scope notes:**
- ÖKOBAUDAT 2024-I pipeline smoke: 2,970 / 3,075 (96.6%) processes
  resolve cleanly end-to-end; 56,430 exchanges typed; 7 distinct
  reference units (MJ, kg, m³, qm, pcs., m, a). The remaining 105
  failures are bundle-side data gaps (missing flow XMLs, flows with
  no quantitative reference and no inline unit ref) — the engine
  classifies these as `LinkError::Io` / `FlowHasNoUnitDerivation` and
  the smoke test accepts them as publisher-side issues.
- Synthetic fixture at
  [`engine/io-ilcd-linker/tests/fixtures/epd_minimal_bundle/`](engine/io-ilcd-linker/tests/fixtures/epd_minimal_bundle/)
  is hand-crafted (not an ÖKOBAUDAT slice — CC-BY-ND-3.0-DE). It
  exercises every EPD-specific path including the Module D negative,
  the inline-vs-chain unit disagreement, and the INA convention.
- The real-ÖKOBAUDAT smoke test at
  [`engine/io-ilcd-linker/tests/oekobaudat_smoke.rs`](engine/io-ilcd-linker/tests/oekobaudat_smoke.rs)
  remains `#[ignore]`-gated behind `OEKOBAUDAT_BUNDLE=...` per the
  ND clause.

**Reversal condition:** If a later ILCD+EPD version breaks the
stage-stratification invariant (e.g. reintroduces a collapsed
"total" field that must be authoritative), revisit (1). If any EPD
consumer case study shows that silent inline-UUID fallback to
`shortDescription` is masking real errors, tighten (2) to hard-fail
instead of warning.

---

## 2026-04-19 · `D-0007` — Ship both AR6 and AR5 GWP100 (Option D); AR6 is default

**Decision:** v0.0.1 ships **two** IPCC GWP100 presets as first-class
citizens of the standard registry: `ipcc-ar6-gwp100` (recommended
default for new studies) and `ipcc-ar5-gwp100` (legacy-verification
parity for EPDs authored under the prior assessment). The engine adds
a `FlowOrigin { Unspecified, Fossil, NonFossil }` classifier on
`FlowMeta` and a matching `FactorMatch::CasOrigin` variant so AR6's
fossil-vs-non-fossil CH4 split (`29.8` / `27.0`, AR6 WG1 Ch7 T7.15) is
represented explicitly. AR5's single CH4 value (`28`, AR5 WG1 Ch8
T8.A.1 without c-c feedback) stays as an origin-agnostic plain-CAS
match.

**Reasoning:**
- **Option A (ship AR6 only)** would have broken legacy-verification
  workflows consultancies rely on to re-audit pre-2022 EPDs at the
  bit level. Unacceptable given our "no vendor hostage" commitment
  (C10).
- **Option B (ship AR5 only)** fossilizes the engine against the
  current IPCC position and undermines the spec-first credibility
  story.
- **Option C (ship one "IPCC GWP100" preset that flips on a flag)**
  couples the assessment-report choice to a global config knob,
  which blurs audit trails and is exactly the kind of hidden
  behaviour SimaPro gets criticized for.
- **Option D (separate presets, AR6 default, shared infra)** is the
  honest position: each preset's `(id, version)` is the provenance
  token, the CH4 origin split is surfaced in the data model rather
  than buried in a comment, and the audit log shows which preset was
  applied. Most work; cleanest outcome.

The `FlowOrigin::Unspecified` default means existing JSON fixtures
round-trip unchanged. Under AR6 an unspecified-origin CH4 flow
surfaces in `CMatrixBuild::unmatched_flows` rather than silently
inheriting the fossil factor — the conservative-correct choice when
information is missing. A regression guard for this is vector
`l1_ch4_non_fossil_origin_split` alongside the existing fossil
vector, so any engine that confuses the two fails differential parity.

**Reversal condition:** If verification-partner feedback during Phase 5
shows that dual presets confuse users more than they help, collapse
back toward Option A and ship an AR5-compatibility shim as a separate
crate rather than a registry entry.

**Scope notes:**
- N2O values verified in the same pass: AR6 = `273`, AR5 = `265`
  (AR6 WG1 Ch7 T7.15 and AR5 WG1 Ch8 T8.A.1 respectively). No drift
  against the tables.
- Downstream seed vectors `l1_two_process_independent`
  (`h=[2.98]`) and `l1_coupled_two_process` (`h=[35.84]`) recomputed
  against the corrected fossil-CH4 factor.

---

## 2026-04-19 · `D-0006` — Arko upgraded from side-quest to primary long-term play

**Decision:** Arko is a first-class product, not a side project. Apply the same
production-quality bar as KarbonGarbi. Budget: 14 months to first paying
customer per [`docs/arko-execution-guide.md`](docs/arko-execution-guide.md).

**Reasoning:** After Imanol's 2026-04-19 SimaPro walkthrough, the scope of the
captive market is clear: enterprises and consultancies rely on SimaPro because
no viable open alternative exists. Even modest displacement is a large
business. Spec-first + AI-native is a defensible position a closed
Pascal/Delphi incumbent cannot match.

**Reversal condition:** `D-0007` — KarbonGarbi <2 paying customers by
2026-09-30 triggers an Arko freeze until KarbonGarbi has ≥3.

---

## 2026-04-19 · `D-0005` — V1 ships open EU databases only; no ecoinvent

**Decision:** V1 ships with ÖKOBAUDAT, Agribalyse, and EF reference packages.
No ecoinvent integration in V1, not even for testing. ecoinvent licensing is
deferred to V2.

**Reasoning:** ecoinvent licensing is expensive and per-seat; shipping it in
V1 creates a cost floor that kills the open positioning. Open EU databases
cover enough of the PEF/EPD regulatory use cases to prove product viability.
Differential parity against Brightway + OpenLCA can be established on open
data alone.

---

## 2026-04-19 · `D-0004` — Solver threshold policy (engine scope-v0.1)

**Decision:** Dense LU/QR via `arko-solvers-dense` for systems with <100 unknowns.
Sparse LU via `arko-solvers-sparse` (faer backend) for 100–10,000 unknowns.
UMFPACK path deferred to a later crate for 10,000+ unknowns.

**Reasoning:** faer's sparse LU handles the mid-range cleanly and stays
Apache-2.0–compatible. UMFPACK is GPL-compatible-but-fiddly and only pays off
above ~10k unknowns; most studies sit below that. Splitting the dense path
keeps small-study hot paths allocation-light.

---

## 2026-04-19 · `D-0003` — Uncertainty: MT Monte Carlo only in v0.1

**Decision:** v0.1 ships Mersenne Twister Monte Carlo in `arko-uncertainty`.
Sobol' quasi-Monte-Carlo is deferred to v0.2.

**Reasoning:** Determinism + reproducibility is the differentiator; MT with a
published seed policy satisfies that contract. Sobol' adds variance-reduction
value but needs careful dimensional-allocation plumbing the solver pipeline
doesn't yet expose. Ship the correct thing first; add QMC when the plumbing
is there.

---

## 2026-04-19 · `D-0008` — Toolchain pin bumped from 1.83 → 1.95 (current stable)

**Decision:** Amend D-0002. [`rust-toolchain.toml`](rust-toolchain.toml) channel
moves from `1.83` to `1.95` (current stable as of the knowledge cutoff).
faer stays pinned to `0.20`; no other version pins change.

**Reasoning:** First `cargo build --workspace` attempt on 2026-04-19 failed
with `feature edition2024 is required` inside `blake3 1.8.4` (a direct
dependency of `arko-core` for canonical hashing). An interim bump to 1.85
unblocked blake3 but surfaced a second MSRV failure in `constant_time_eq
0.4.3` (transitive via blake3) requiring 1.95. Rather than pinning
individual transitives down and fighting a rolling battle as the ecosystem
continues to bump — cargo's MSRV-aware resolver activates on 1.84+, so
pre-resolver pins are brittle — the honest move is to sit on current
stable and let the resolver do its job. D-0002 already flagged this exact
class of drift as foreseeable ("Caveat: faer has been API-unstable across
minor versions"). This is the documented reversal.

**Reversal condition:** If faer 0.20 stops building on 1.95, or we later
want reproducible builds across a narrower rustc band, revisit with a new
entry that bumps both the channel and faer together.

---

## 2026-04-19 · `D-0002` — Rust toolchain pinned to 1.83; faer 0.20

**Decision:** Workspace pinned in [`rust-toolchain.toml`](rust-toolchain.toml)
to channel `1.83` with `rustfmt`, `clippy`, `wasm32-unknown-unknown`. faer
pinned to `0.20` in the workspace `Cargo.toml`.

**Reasoning:** 1.83 has the const-generic + trait features the solver code
uses; newer nightly features aren't needed. faer 0.20 is the current stable
line that the sparse and dense crates were written against.

**Caveat:** faer has been API-unstable across minor versions in the past.
Phase 0 exit must include a green `cargo test --workspace` run; until that
happens, the `sp_lu` / `solve_in_place` call sites in
[`engine/solvers-sparse/src/lib.rs`](engine/solvers-sparse/src/lib.rs) are
compile-untested on this host.

---

## 2026-04-19 · `D-0001` — Dual-license policy

**Decision:** Engine (`engine/`) is Apache-2.0. Web/services are proprietary
(Goibix S.L.). Specs (`specs/`) and docs (`docs/`) are CC-BY-4.0.

**Reasoning:** Open primitives create reimplementation pressure and win
credibility (commitments C3, C9). A closed product layer is how Goibix makes
money. CC-BY on specs lets anyone quote, fork, and reimplement the math
contract — that is the credibility lever, and it does not compete with the
hosted product.

See [`LICENSE`](LICENSE) for the full policy text.

---

## Template for future entries

```markdown
## YYYY-MM-DD · `D-NNNN` — <short title>

**Decision:** <one or two sentences stating the decision>

**Reasoning:** <why; what alternatives were rejected; what trade-off was accepted>

**Reversal condition:** <if any — what signal would cause us to revisit>
```
