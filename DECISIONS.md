# Arko · Decision Log

Append-only log of significant architectural, strategic, and licensing decisions.
Every entry carries a date, the decision, and the reasoning behind it. When a
decision is later reversed, add a new entry with a back-reference; never delete
the old one.

Format: newest-first. Dates are `YYYY-MM-DD`, local to the author.

---

## 2026-04-23 · `D-0023` — i18n library: `next-intl` for Phase 2

**Context:** [`D-0020`](#d-0020) commits to EN + ES UI from week 11.
The library choice locks the translator workflow, the message-key
shape, and the per-screen developer ergonomics for the next 14+ weeks.
Three real candidates: `next-intl` (Next.js 15 native, App Router-
aware), `react-i18next` (most battle-tested, framework-agnostic), and
Lingui (most ergonomic compile-time extraction, smaller community).

**Decision:** Use **`next-intl`** for Phase 2.

**Reasoning, in descending weight:**

1. **Native Next.js 15 / App Router integration.** No bridge layer
   between RSC boundaries and translation lookups; `next-intl` is
   designed for App Router server components. `react-i18next` requires
   provider scaffolding around server/client boundaries that Next.js
   13+ made awkward.
2. **Zero runtime translation overhead.** Messages bundled per locale
   per route segment; only the active-locale bundle hits the wire.
   Matters for Phase 2's WASM-engine-in-browser story (page weight is
   already the bottleneck).
3. **Two-locale scope makes ergonomic differences negligible.**
   Lingui's compile-time extraction shines at 6+ locales; at EN + ES
   the manual key maintenance is trivial. Picking the heavier tool
   for headroom we don't yet need is premature.
4. **Imanol-network translators don't need a special pipeline.** JSON
   message bundles in `messages/en.json` and `messages/es.json` are
   reviewable by any Spanish-speaking collaborator without setup.

**Consequences:**

- Adds `next-intl` dependency at week-11 scaffold time.
- Locale files committed to `apps/web/messages/{en,es}.json` from
  day 1; every PR that adds a screen also adds both locale files.
- Locale switch via URL prefix (`/en/...`, `/es/...`) — `next-intl`
  default routing.

**Open items:**

- Default locale (Spanish for Basque-market positioning, or English
  for global engine narrative consistency)? Defer to week 11 when
  the marketing-vs-product framing for the homepage is clearer.

---

## 2026-04-23 · `D-0022` — Tier feature gating: Studio / Team / Enterprise three-axis split (seats × studies × EPDs)

**Context:** [`D-0020`](#d-0020) commits to three subscription tiers
named Studio / Team / Enterprise (per the product spec). Tier prices
are deferred (see "Open items" below — pending Imanol-network signal).
What each tier *unlocks* needs to be decided independently of price,
because the billing UI and the in-app gates can't be built without it.

**Decision:** Three-axis gating model — **users (org seats) ×
active studies × EPDs/month**, plus database tier:

| Axis | Studio | Team | Enterprise |
|---|---|---|---|
| Users / org | 1 | 5 | unlimited |
| Active studies | 5 | unlimited | unlimited |
| EPDs / month | 1 | 5 | unlimited |
| Database access | Free DBs only (USDA LCA Commons, ÖKOBAUDAT, JRC EF reference) | Free DBs + premium DBs once licensed | All databases incl. ecoinvent (Phase 4+) |
| Support | Email best-effort | Email same-business-day | SLA + dedicated channel (Phase 4+) |
| SSO / SAML | — | — | Phase 4+ |

"Active studies" = studies in non-archived state. Archiving is free
and unlimited (so Studio users aren't punished for accumulating
historical work; they just have to archive before starting a 6th).

EPD throttling is per-org per-calendar-month, resets on the 1st.

**Reasoning, in descending weight:**

1. **Matches the SaaS-for-consultancies shape Imanol's network
   already understands.** SimaPro / openLCA-cloud / GaBi all gate on
   seats and study counts. Reading the gating model takes one glance.
2. **EPDs/month is the value-meter that maps to revenue.** Generating
   an EPD is the highest-value action a user takes; a consultancy
   producing 20 EPDs/year (Team-shaped) is fundamentally different
   from a single practitioner doing 1–2 EPDs/year (Studio-shaped).
   Throttling here matches actual willingness-to-pay slope.
3. **Free-DB-only at Studio matches license reality.** USDA + ÖKOBAUDAT
   + JRC EF cover the construction/agriculture/food spans that
   single-practitioner studies typically need. Premium databases
   (ecoinvent et al.) carry per-seat licensing costs Arko can't
   subsidize at €49/mo.
4. **SSO behind Enterprise leaves the door open for Phase 4 SAML
   work.** Putting SSO in Team would force the implementation forward
   into Phase 2; gating it to Enterprise keeps the Phase 4 placement
   intact (per [`D-0020`](#d-0020) carve-outs).

**Consequences:**

- Billing UI shows three columns with these axes; copy lives in
  `messages/{en,es}.json` from day 1.
- `subscriptions.tier` enum (`studio` / `team` / `enterprise`) drives
  feature flags read at every gated action (study creation, EPD
  generation, user invite, premium DB access).
- Need a "you've hit your monthly EPD limit, upgrade or wait until
  the 1st" empty state — design at EPD Output screen build time.
- Trial users (14-day, per [`D-0020`](#d-0020)) get **Team-tier**
  access during trial — not Enterprise — to avoid setting an
  expectation the trial-ender can't keep on Studio.

**Open items:**

- Tier prices (€/month, monthly vs annual discount): unresolved,
  pending Imanol-network price-test conversation. Comparable
  anchors: SimaPro analyst ~€2,500/yr, openLCA Nexus DBs €1k–€5k/yr.
  Naïve seed: Studio €49/mo, Team €199/mo, Enterprise "contact us" —
  but lock only after Imanol input. Filed separately when decided.
- Whether premium DBs are bundled in Team vs sold à la carte. Lean:
  bundled (simpler SKU; ecoinvent licensing is the only one that
  forces à la carte, and that's Phase 4).
- Annual discount magnitude (10%? 20%?) — defer until pricing lands.

---

## 2026-04-23 · `D-0021` — Phase 2 timeline shape: stretch (shape A) over rescope (shape B)

**Context:** [`D-0020`](#d-0020) added ~3–5 weeks of scope to Phase 2
(billing + orgs + i18n). The boundary memo
([`docs/phase-2-boundary-memo.md`](docs/phase-2-boundary-memo.md) §1,
§5, §8) framed two timeline shapes:

- **Shape A:** stretch Phase 2 from 14 weeks → 17–20 weeks; ship all
  five screens (Process Browser, Study Builder, Calc Runner,
  Contribution Analysis, Scenario Comparison) + EPD output + the
  D-0020 scope. v0.3.0 tags late Q4 2026.
- **Shape B:** keep the 14-week window; defer Scenario Comparison to
  Phase 3. v0.3.0 tags Q3 2026 with four screens + EPD output.

**Decision:** **Shape A — stretch the timeline.** All five screens
plus EPD output plus D-0020 scope ship at v0.3.0.

**Reasoning, in descending weight:**

1. **Scenario Comparison is the screen that demonstrates Arko's
   thesis.** "Pin two studies side-by-side, see which design choices
   move the impact" is the answer to *why use Arko over a
   spreadsheet* for design-stage practitioners. Shipping Phase 2
   without it leaves a hole in the sales conversation that Imanol's
   network would notice immediately.
2. **The v0.3.0 demo to Imanol (week 24, or stretched-equivalent)
   needs the full thesis.** Per
   [`feedback_imanol_arko_session_prep`](../C--Users-hical-Desktop-karbongarbi/memory/feedback_imanol_arko_session_prep.md):
   the closing question is "1–10, how close is this to something
   you'd actually use?" Shipping a four-screen Arko forces him to
   imagine the fifth screen — that imagination penalty is worth
   3–6 weeks of slip.
3. **Phase 3 already has its own scope** (multiple EPD templates,
   verifier workflow, PEF, lock+audit+signature). Adding Scenario
   Comparison to Phase 3 would push *Phase 3's* exit out, not
   compress Phase 2's. Net schedule cost is roughly equal; the
   product cost differs.
4. **Pricing the stretch:** ~3–5 weeks of additional Phase 2 work
   pushes v0.3.0 from late Q3 2026 to late Q4 2026 / early Q1 2027.
   Phase 5 (first paying customer) shifts the same amount. Acceptable
   given that the alternative is shipping a weaker Phase 2.

**Consequences:**

- Phase 2 window: weeks 11 → ~31 (was 11 → 24).
- Phase 3 window shifts: was weeks 25–34, now ~32–41.
- Phase 4, Phase 5 shift the same ~6–8 weeks.
- Boundary memo §1 deliverables table stays as-is (5 screens + EPD).
- Boundary memo §8 hinge stays as-is (5 screens, not 4).

**Open items:**

- Whether the ~6–8 week downstream slip changes the Phase 5
  "first paying customer" quarter for Imanol's expectations.
  Surface in the next Imanol conversation as context, not as ask.

---

## 2026-04-22 · `D-0020` — Phase 2 scope expansion: billing (Redsys), org/project hierarchy with role management, EN+ES UI all pulled forward from Phase 4 to Phase 2

**Context:** The Execution Guide places billing, GDPR, legal pages, and
admin panel in Phase 4 (`docs/arko-execution-guide.md:357`, "All of
KarbonGarbi's hard-won infrastructure patterns applied"). Phase 5 exit
is "first paying customer." Same-day Phase 2 product spec landed
([`docs/phase-2-product-spec.md`](docs/phase-2-product-spec.md))
articulates a six-screen + EPD output product where the canonical user
journey — *signup → org → project → study → calc → contribution → EPD
generation → operator submission* — requires billing and org
infrastructure to be present from Phase 2 launch. Without billing, no
paid customers between Phase 2 and Phase 5; without orgs, no
consultancies (the primary customer shape per
[`feedback_product_vs_business`](../C--Users-hical-Desktop-karbongarbi/memory/feedback_product_vs_business.md));
without ES UI alongside EN, half the Basque target market is gated.

**Decision:** Pull three categories of work from Phase 4 → Phase 2:

1. **Billing — Redsys integration**, three tiers (Studio/Team/
   Enterprise), monthly/annual subscriptions, free 14-day trial,
   in-app plan management. Same Redsys pattern as KarbonGarbi
   (HMAC_SHA256_V1, COF recurring, `pycryptodome`) per
   [`feedback_redsys_billing`](../C--Users-hical-Desktop-karbongarbi/memory/feedback_redsys_billing.md).
   No Stripe (initial market = Basque via Imanol's network; "Arko =
   international" is long-term positioning, not Phase 2 reality).
2. **Org/project hierarchy with role management** — `org → projects →
   studies` nesting; owner/editor/viewer roles; org-level subscription
   attachment (subscriptions belong to orgs, not users).
3. **EN + ES UI from week 11** — i18n scaffolding (next-intl or
   equivalent) baked in from project setup; both locales shipped at
   v0.3.0; copy decisions made per screen alongside implementation.

**Reasoning, in descending weight:**

1. **Billing is GTM-critical for Phase 5 (first paying customer).**
   Phase 5 exit is "first paying customer." Without billing
   infrastructure in Phase 2, there's a Phase 4 cliff where billing has
   to be built in parallel with polish + launch + landing-page work.
   Building it in Phase 2 means it bakes for two phases (3 and 4)
   before the first customer touches it; building it in Phase 4 means
   it ships untested into the Phase 5 cutover.
2. **Orgs are a prerequisite for billing.** Subscriptions attach to
   organizations, not users — consultancies (the
   [`feedback_product_vs_business`](../C--Users-hical-Desktop-karbongarbi/memory/feedback_product_vs_business.md)
   target shape: "50 Imanols + 5 consultancies") need multi-user
   accounts. Building billing without orgs would force a painful
   retrofit — every paying customer would need migration. Building
   orgs without billing would build half the surface twice. They
   couple naturally; pull both or pull neither.
3. **ES UI is half the Basque addressable market.** Imanol's network
   is Spanish-speaking. Shipping EN-only at v0.3.0 cuts the
   addressable Basque market in half *during the time when Imanol-
   shaped feedback is most actionable*. Adding i18n to a fully-built
   UI later costs ~2x the upfront i18n scaffolding cost (every screen
   needs key extraction, every copy decision needs revisiting).
4. **Redsys (not Stripe) is the right gateway for the Basque market.**
   Per
   [`feedback_redsys_billing`](../C--Users-hical-Desktop-karbongarbi/memory/feedback_redsys_billing.md).
   All major Basque/Spanish industries pay through Redsys; Stripe
   reads as foreign to the Spanish B2B market. The KarbonGarbi
   Redsys integration code is reusable. Revisit if/when a non-Spanish
   customer base materializes — that's a future `D-00xx`, not a
   day-1 design assumption.
5. **The Execution Guide's Phase 4 placement of billing/admin was
   conservative.** Tech Spec v2.0 framed Phase 4 as "all of
   KarbonGarbi's hard-won infrastructure patterns applied" — that
   meant *reuse*, not *build from scratch*. Reuse from KarbonGarbi
   (Redsys integration, org schema, role management) is faster than
   the Phase 4 budget assumed; pulling it into Phase 2 spreads the
   work across more weeks rather than concentrating it at launch.
6. **The minimum working flow in the product spec is the right
   acceptance criterion for v0.3.0.** Six steps end-to-end:
   *signup → org → project → study → calc → contribution → EPD*. Steps
   1 and 7 (signup, EPD operator submission) require billing and
   account infrastructure. Phase 2 v0.3.0 cannot demo the canonical
   flow without these.

**Consequences:**

- **Phase 2 scope grows by ~3-5 weeks of additional work** (Redsys + orgs
  + i18n on top of the 5-screen + EPD output build the Execution Guide
  budgeted). Phase 2 timeline either:
  - **Stretch:** 14 weeks → ~17-20 weeks (decision shape A)
  - **Rescope:** defer Scenario Comparison to Phase 3, which absorbs
    naturally alongside Monte Carlo / sensitivity UIs that already
    target Phase 3 (decision shape B)

  Phase 2 week 11 is the right moment to lock A or B; both are
  defensible. Phase 2 week-1 work (week 11 itself) is the same under
  either choice.
- **Phase 4 timeline shrinks correspondingly.** Phase 4 becomes pure
  polish (Monte Carlo / sensitivity UIs surface, landing page, docs
  site, CloudNativePG migration). The "all of KarbonGarbi's
  infrastructure patterns" Phase 4 line is partially closed-out by
  Phase 2 itself.
- **Postgres schema in `docs/phase-2-boundary-memo.md` §3 grows.** Add
  tables: `organizations`, `projects`, `org_members` (with role enum:
  `owner` / `editor` / `viewer`), `subscriptions`, `redsys_customers`
  (mirrors the KarbonGarbi `redsys_identifier` column pattern from
  [`feedback_redsys_billing`](../C--Users-hical-Desktop-karbongarbi/memory/feedback_redsys_billing.md)).
- **Boundary memo §1 (scope) reframes** to point at the product spec
  as canonical scope source; boundary memo retains its architectural
  + decisions + week-1-spike + hand-off role.
- **Imanol session at week 14** adds an org-shape question: "show him
  the org/project hierarchy and ask if it matches consultancy
  workflows" alongside the existing Process Browser questions.
- **`feedback_redsys_billing` memory updated** today to cover both
  KarbonGarbi *and* Arko (was previously KarbonGarbi-only); the
  "don't assume Arko = international until user explicitly opens up
  the geography" guard added so future-me doesn't repeat the original
  Stripe assumption that prompted this decision.

**Open items:**

- **Phase 2 timeline shape — stretch (A) vs rescope (B).** Lock at
  week 11 (Phase 2 day 1). Decision shape lives in the boundary memo's
  week-1 work list as `D-00xx`.
- **Tier feature gating.** Studio / Team / Enterprise — which features
  gate behind which tier (study count, project count, advanced
  comparison, EPD generation, multi-user, etc.). Defer to Phase 2
  week 11-12 product decision alongside billing implementation.
- **Tier prices.** Set in Phase 2 week 11-12; informed by Imanol-style
  consultancy rate context.
- **i18n source-of-truth format.** JSON files vs a translations service
  (Crowdin, etc.) — defer to week 11; JSON files first by default
  (the Execution Guide's "no premature infrastructure" posture).
- **Org-level billing vs per-user billing.** Decision lands in the
  Redsys subscription model: each org has one subscription, role
  changes don't trigger billing changes. Per-user pricing (e.g., $X
  per editor seat) is a tier-config detail, not an architecture
  question.
- **GDPR / legal pages.** Originally bundled with Phase 4 billing
  work; should they pull forward too? **Yes** — billing without legal
  pages (privacy policy, ToS, cookie consent) cannot accept paying
  customers in the EU. Add to Phase 2 scope; not breaking out as a
  separate `D-00xx` because they're a logical billing-bundle item
  rather than a standalone decision.
- **Admin panel.** Defer to Phase 4 with the original guide placement.
  Admin needs are diagnostic / support-shaped, not blocking the
  customer journey. Phase 2 ships with raw Postgres access for me;
  Phase 4 wraps it in UI.

**Back-references:**

- [`docs/phase-2-product-spec.md`](docs/phase-2-product-spec.md) —
  the canonical product scope this decision adapts the timeline to.
- [`docs/phase-2-boundary-memo.md`](docs/phase-2-boundary-memo.md) —
  architectural decisions, week-1 spikes, hand-off (rewritten to
  reference the product spec).
- [`feedback_redsys_billing`](../C--Users-hical-Desktop-karbongarbi/memory/feedback_redsys_billing.md) —
  Redsys-as-billing-gateway feedback memory, scope updated today to
  cover Arko as well as KarbonGarbi.
- [`feedback_product_vs_business`](../C--Users-hical-Desktop-karbongarbi/memory/feedback_product_vs_business.md) —
  "50 Imanols + 5 consultancies" target customer shape that motivates
  the org/project hierarchy.

---

## 2026-04-22 · `D-0019` — ReCiPe 2016 Midpoint Hierarchist V1 scoped to 10 categories with GLO-only matchers; `CasRegion` + per-process pipeline restructure deferred to V2

**Context:** ReCiPe 2016 Midpoint Hierarchist is the fourth and final
named-slate Phase-1 method preset (the other three — AR6, EF 3.1,
CML-IA baseline 4.8 — landed at `D-0015`/`D-0016`/`D-0017`). After
shipping CML-IA, the method-preset exit criterion was misframed as
"met (registry at 4)" because the registry happened to count to 4;
re-reading the Execution Guide (`arko/docs/arko-execution-guide.md:107`)
clarified that the criterion names a *specific slate* — IPCC AR6,
ReCiPe 2016 Midpoint, EF 3.1, CML 2001 — and AR5 is a legacy-parity
bonus not on it. Three of four named slate were shipped (AR6 ✓,
EF 3.1 ✓, CML 2001 satisfied via CML-IA baseline v4.8's Leiden
continuation lineage ✓); ReCiPe was still required for formal exit.

`D-0015` and `D-0017` both flagged ReCiPe 2016 as the most likely
forcing function for a `CasRegion` matcher variant. ReCiPe ships
country-specific CFs for a subset of midpoint categories that EF 3.1
V1 and CML-IA V1 deferred behind the same anticipated `CasRegion`
gate — `D-0015` deferred EF 3.1 water use pending "regionalised
matcher aligning with ReCiPe 2016", `D-0017` deferred CML-IA per-
country AP/POCP with the explicit note "ReCiPe 2016 may force
`CasRegion` on regionalised midpoints". This entry was drafted to
land that variant alongside the preset; layered scoping then
discovered a pipeline-level constraint that reframes V1 scope
(see "Layered-scoping discoveries" below). The variant-extension
pattern after `CasCompartment` (D-0015) and `FlowOrigin` (D-0016)
does not apply here: `CasRegion` is part of a coupled regionalisation
bundle deferred to V2, not a third taxonomy extension landing
alongside ReCiPe V1.

ReCiPe 2016 ships **18 midpoint categories** in three cultural
perspectives (Hierarchist, Egalitarian, Individualist); only
Hierarchist is in scope for V1.

**Layered-scoping discoveries — three layers of contact, three
sequential refinements:**

1. **Primary-source verification (RIVM xlsx).** Three RIVM xlsx
   versions exist: `ReCiPe2016_CFs_v1.0_20161004.xlsx`,
   `_v1.1_20170929.xlsx`, and `_v1.1_20180117.xlsx` (the latter is
   the post-erratum canonical version `brightway-lca/bw_recipe_2016`
   reads). Country-specific CFs ship as a *separate* xlsx; the main
   file ships GLO defaults for all 18 categories. Five of 18
   categories ship country-specific CFs in the companion xlsx:
   photochemical ozone formation (split human + ecosystems),
   particulate matter formation, terrestrial acidification,
   freshwater eutrophication, water consumption. Land occupation
   is **not** country-regionalised — it uses biome/land-type
   categorisation, which the existing `CasCompartment` matcher
   pattern already absorbs. RIVM's downloads page documents no
   explicit license terms — gratis-with-no-explicit-license,
   structurally weaker than JRC EF / USDA LCA Commons, mirroring
   the CML-IA Leiden posture (`docs/licenses/cml-ia-leiden.md`).
   `FlowOrigin` extension not required: ReCiPe ships single CFs
   per substance regardless of biogenic/fossil/LULUC provenance —
   same convention as AR5/AR6, opposite of EF 3.1.
2. **Data-model audit (where region lives).** The initial scoping
   plan added `region: Option<String>` to `FlowMeta`. Pre-
   implementation audit surfaced that `engine/core/src/meta.rs`
   already carries `geography: Option<String>` on `ProcessMeta`
   with the comment "Full regionalized impact is deferred to v0.3;
   this is informational" — the field was placed deliberately
   anticipating the regionalisation forcing function. Region is
   genuinely process-level: a Spanish electricity process emits
   CO2 in the Spanish atmosphere; a German cement plant emits SO2
   in the German atmosphere. Region is a property of the activity
   (the process column), not the substance (the flow row).
   Encoding region on `FlowMeta` would conflate "what was emitted"
   with "where the emitting happened" and orphan the existing
   `ProcessMeta.geography` field. Corrected before any code landed.
3. **Pipeline-depth check (whether region can be plumbed).** With
   region established as process-level, the next layer asked: can
   the matcher *receive* process geography at dispatch time?
   Reading `engine/core/src/pipeline.rs` answered: **no, not in the
   current pipeline shape.** The calculation is `g = B · s` (Eq. 2)
   then `h = C · g` (Eq. 3). After Eq. 2, per-process information
   collapses — `g` is a flow-aggregated vector with no provenance
   back to which process emitted what fraction. A single `C` matrix
   applied to `g` cannot express region-dependent CFs because the
   process columns are gone. Honest regional CF support requires
   restructuring Eq. 3 from `h = C · g` to
   `h = Σₚ Cₚ · B[:,p] · s[p]` — a per-process `C` build +
   per-process impact accumulation. That's a calc-pipeline change,
   not a methods-crate cascade. The "matcher signature change
   cascading through ~9 sites" estimate (the layer-2 conclusion)
   was wrong because no caller in the current pipeline has process
   geography to plumb.

The pattern at work: each layer of contact with the actual system
surfaced new information. These are sequential discoveries in the
same investigation, not repeated mistakes — the alternative
(committing to layer-1 or layer-2 assumptions and discovering the
pipeline constraint mid-implementation) would have cost much more
than three scope refinements in scoping.

**Decision:** Ship `("recipe-2016-midpoint-h", "1.1")` with **10
categories**, all using existing `Cas` (and `CasCompartment` where
appropriate) matchers, with **GLO defaults from the main RIVM xlsx**
for all 10 categories. The five regionalised-in-source categories
use their GLO values for V1; country-specific overrides defer to V2.

*EN 15804+A2 alignment subset (7) — parity with EF 3.1 V1 and
CML-IA V1:*
1. Global warming (GWP100, Hierarchist 100-year) — `kg CO2-eq`
2. Stratospheric ozone depletion — `kg CFC-11-eq`
3. Photochemical ozone formation, human health — `kg NOx-eq`
   *(GLO default — country-specific CFs deferred to V2)*
4. Terrestrial acidification — `kg SO2-eq`
   *(GLO default — country-specific CFs deferred to V2)*
5. Freshwater eutrophication — `kg P-eq`
   *(GLO default — country-specific CFs deferred to V2)*
6. Marine eutrophication — `kg N-eq`
7. Fossil resource scarcity — `kg oil-eq`

*ReCiPe-distinctive subset (3) — categories that justify ReCiPe's
presence in the named slate beyond duplicating EN 15804 coverage:*
8. Particulate matter formation — `kg PM2.5-eq`
   *(GLO default — country-specific CFs deferred to V2)*
9. Land occupation — `m²·a`
10. Water consumption — `m³`
    *(GLO default — country-specific CFs deferred to V2)*

Method `name` field is `"ReCiPe 2016 Midpoint (Hierarchist, v1.1)"`.
Per-factor source comments cite the
`ReCiPe2016_CFs_v1.1_20180117.xlsx` post-erratum version specifically,
plus sheet name and row. Comments on the five regionalised-in-source
categories explicitly note "GLO default from main xlsx;
country-specific CFs from companion xlsx deferred to V2" so future-me
reading the factor doesn't wonder why the country xlsx isn't being
read. Water-consumption factors carry an extra line flagging the
brightway-hardcoded precedent.

**Practitioner-default parity is the correctness story.** A study
run through ReCiPe 2016 V1 in Arko produces the same numbers
`bw_recipe_2016` produces — the most-used Python ReCiPe port reads
*only* the main xlsx and treats ReCiPe as global-only (its
`bw_recipe_2016/categories/water.py` even hardcodes water with the
explicit comment "Provided data is effectively useless, do it
ourselves" rather than parsing the country xlsx). Arko V1 matching
brightway's default behaviour is defensible practitioner parity,
not partial implementation. Regionalisation as opt-in becomes a V2
addition driven by real user demand.

**V2 regionalisation bundle (deferred together):** The pipeline-
depth discovery means regionalisation isn't a single-axis taxonomy
extension; it's a coupled bundle that should land together when a
real user asks for it:

- `FactorMatch::CasRegion` matcher variant (design specifics
  revisited at V2 — `String` vs typed `Region` payload, fallback
  chain shape — when the pipeline restructure clarifies the dispatch
  surface).
- Per-process `C` build + per-process impact accumulation (Eq. 3
  restructure: `h = Σₚ Cₚ · B[:,p] · s[p]`). Affects
  `engine/core/src/pipeline.rs`, `Study`, possibly `Computed`. The
  honest "regional CFs work" path; Phase-2-shaped because it
  changes the engine's fundamental computation pattern.
- Reader-side region extraction verification (low risk: both
  `arko-io-ilcd-linker` and `arko-io-olca-jsonld` already populate
  `ProcessMeta.geography` natively from ILCD
  `LocationOfOperationSupplyOrProduction` and openLCA `location.code`
  respectively; the V2 work is end-to-end verification of the
  regional dispatch, not new parser logic).
- The 5 country-specific CF tables (POCP-h, particulate matter,
  terrestrial acidification, freshwater eutrophication, water
  consumption — V1 ships their GLO defaults from the main xlsx;
  V2 layers country overrides on top). Region vocabulary
  verification (ISO 3166-1 alpha-2/alpha-3, watershed IDs)
  happens during V2 factor entry alongside the country tables.
- `ProcessMeta.geography` promotion from "informational, deferred
  to v0.3" to load-bearing (stays informational in V1; the
  comment is unchanged).

This is a coherent V2 bundle: when a user requests regionalised
LCA, the work is to restructure the pipeline (Eq. 3), introduce
`CasRegion`, verify reader region extraction end-to-end, and
ingest the country-specific CFs as overrides on top of GLO
baselines. Cleanly scoped V2 work driven by user demand rather
than speculative infrastructure. Both parity smokes (EF carpet,
beef multi-process) remain bit-exact for V1 — no taxonomy or
pipeline changes, no fixture changes.

**V2 expansion roadmap — four reasoning buckets:**

*Niche or covered-elsewhere (2):*
- Ionising radiation — niche, CML-IA V1 also skipped
- Mineral resource scarcity (Cu-eq) — methodology contested; CML-IA
  ships ADP-elements (ultimate-reserves) which covers the same
  practitioner intent at a less-criticised baseline

*EN 15804+A2 alignment boundary (1):*
- Photochemical ozone formation, ecosystems — splits POCP further
  than EN 15804+A2's single POCP indicator requires; ship the
  EN 15804-aligned human-health variant in V1 and offer the second
  variant in V2 once user demand surfaces

*USEtox cluster — separate scoping decision (5):*
- Human carcinogenic toxicity, human noncarcinogenic toxicity,
  freshwater ecotoxicity, marine ecotoxicity, terrestrial
  ecotoxicity. USEtox introduces receptor-compartment and
  time-horizon design questions distinct from regionalisation.
  V2 toxicity will need its own factor-table-entry-discipline pass
  with paired EF 3.1 toxicity values for cross-witnessing
  (same shape as the EF 3.1 / CML-IA toxicity deferrals).

*Regionalisation bundle (the V2 trigger for `CasRegion` + pipeline
restructure):* country-specific CFs for the 5 regionalised-in-source
categories shipped here in V1 with GLO defaults. See "V2
regionalisation bundle" above for the full coupled-work list. Real
user demand is the trigger to land this bundle; speculative
infrastructure ahead of demand is the wrong shape given the
pipeline-restructure cost.

**Reasoning, in descending weight:**

1. **Standards reference is academic-default + regionalised-CF-
   uniqueness, not formal endorsement.** GLAM (UNEP Life Cycle
   Initiative) and the JRC PEF method recommendation (Commission
   Recommendation 2021/2279) are *not* ReCiPe endorsements — JRC's
   formal recommendation is the EF method (which Arko ships as
   EF 3.1). ReCiPe's institutional weight is (a) the most-cited
   single LCIA method in academic LCA practice (5000+ citations on
   the Springer 2017 paper) and (b) the only widely-adopted method
   shipping native regionalised CFs for water, particulate matter,
   acidification, eutrophication, and POCP. The 10-category V1 set
   reflects this: 7 categories cover EN 15804+A2 alignment for
   parity with EF 3.1 / CML-IA; 3 (particulate matter, land,
   water) cover ReCiPe-distinctive territory that justifies its
   presence in the named slate beyond being a third copy of the
   EN 15804 subset. This framing answers "why ship ReCiPe at all
   if EF 3.1 and CML already cover the EN 15804 set" with a
   concrete value proposition.
2. **Pipeline-depth constraint forces V1 = GLO-only.** Region as
   process-level data is correct (layer-2 discovery); the matcher
   needing process context to dispatch is correct (layer-2
   implication); the calc pipeline collapsing per-process
   information at Eq. 2 means the matcher *cannot receive* process
   context at the C-build step in the current pipeline shape
   (layer-3 discovery). Honest regional CF support requires
   restructuring Eq. 3 — a Phase-2-shaped change to the engine's
   fundamental computation pattern, not a methods-crate cascade.
   GLO-only V1 ships ReCiPe with practitioner-default semantics;
   the regionalisation bundle defers to V2 when a real user
   requests it.
3. **`CasRegion` deferral is V2 architectural cleanup, not
   regression.** Three prior preset decisions (D-0015, D-0017,
   plus the inline comment in `engine/methods/src/ef_31.rs`)
   deferred regional CF support pending a forcing function. The
   forcing function turned out to be larger than a matcher
   variant — the pipeline restructure is the real work. Deferring
   the variant *with* the pipeline change keeps the variant from
   being inert (added but unused) for the duration. Two taxonomy
   extensions in the prior preset cycles (`CasCompartment`,
   `FlowOrigin`) is a defensible pattern; `CasRegion` joining them
   in V2 alongside the pipeline restructure is the right shape.
4. **Hierarchist V1 only; Egalitarian and Individualist V2-or-never.**
   Hierarchist is the standard reference (scientific consensus on
   time frame and mechanism plausibility), the published ReCiPe
   default, and the perspective most academic LCA studies cite.
   Egalitarian (precautionary, long-term) and Individualist
   (short-term, optimistic) use the *same* matcher infrastructure
   with *different* factor tables — V2 expansion would be pure
   factor-table addition with no design change. Clean V2 boundary.
5. **Per-factor source comments cite v1.1_20180117 specifically.**
   Three RIVM versions exist; the post-erratum one is the
   community-default. Citing the version (not just "the RIVM xlsx")
   prevents future-me re-deriving values from a different version
   and silently producing diverging numbers. Same per-factor
   source-comment template as EF 3.1 and CML-IA.
6. **Brightway parity does double duty.** Originally cited as
   evidence for three-tier fallback (region → GLO → unmatched).
   With V1 = GLO-only, the same precedent is now evidence for
   GLO-only-as-default-implementation. Same source, two related
   claims: the most-used Python ReCiPe port treats regionalisation
   as opt-in additive, and ships GLO-only as the baseline
   experience. Arko V1 matching that posture is consistent
   practitioner-default behavior, not partial implementation.
7. **Water consumption is the documented scope risk.** The
   `bw_recipe_2016` port hardcodes this category — its
   `bw_recipe_2016/categories/water.py` carries the comment
   "Provided data is effectively useless, do it ourselves" —
   rather than parsing the RIVM xlsx. The most-used Python port
   gave up on programmatic extraction for water specifically.
   Arko V1 ships water with GLO defaults from the main xlsx; the
   companion country-specific xlsx for water defers with the rest
   of the regionalisation bundle to V2. Flagging so future-me
   knows the friction is expected, not a source-reading bug.

**Consequences:**

- `MethodRegistry::standard()` ships **5 methods** at Phase 1 exit:
  AR6, AR5 (legacy parity, documented bonus), EF 3.1, CML-IA
  baseline 4.8, ReCiPe 2016 Midpoint Hierarchist 1.1. The named
  slate is 4-of-4 satisfied (AR6, EF 3.1, CML 2001 via CML-IA
  baseline, ReCiPe 2016 Midpoint); the registry count of 5
  includes AR5 as the legacy-parity bonus. Registration commit
  should explain `r.len() == 5` so future-me reading the assertion
  doesn't wonder why 5 instead of 4.
- `FactorMatch` taxonomy stays at **5 variants** for Phase 1
  (`Cas`, `CasOrigin`, `CasCompartment`, `FlowId`,
  `NameAndCompartment`). No `CasRegion` in V1; deferred to V2
  alongside the pipeline restructure.
- `FactorMatch::matches` signature **unchanged**. No mirroring
  cascade. No reader changes. Both parity smokes (EF carpet, beef
  multi-process) stay bit-exact — no taxonomy or pipeline changes.
- `ProcessMeta.geography` stays informational ("deferred to v0.3"
  comment unchanged). Promotion to load-bearing happens in V2 when
  the regionalisation bundle lands.
- Per-factor source-comment template extended with the
  `v1.1_20180117` version line and (for the 5
  regionalised-in-source categories) the explicit "GLO default;
  country-specific CFs from companion xlsx deferred to V2" line.
  Water consumption carries the additional brightway-hardcoded-
  precedent flag. Documented at `docs/licenses/recipe-2016-rivm.md`
  for future preset entries derived from the same source.
- License posture is "gratis with no explicit license terms" —
  same as CML-IA baseline. Defensible for V1 (factual data,
  different selection/arrangement, attribution preserved);
  commercial-scale distribution would need explicit grant. Full
  analysis at `docs/licenses/recipe-2016-rivm.md` (template
  borrows from `cml-ia-leiden.md`).
- Phase 1 method-preset exit criterion fully closed: 4-of-4 named
  slate (AR6, EF 3.1, CML 2001 via CML-IA baseline, ReCiPe 2016).
  Remaining Phase-1 punch-list: FactoredSolver + Phase 1 closeout
  (`arko-engine v0.2.0` tag + retrospective doc + Phase 2 boundary
  memo). LCAx reader and ILCD+EPD writer remain as Phase 2 V2
  items per `D-0018`.
- Session sequence accelerated by collapsing the planned
  taxonomy-extension session into a non-session. Original plan
  was three sessions (taxonomy + factors + registration); V1 =
  GLO-only collapses to two (factors against existing matchers +
  registration).

**Open items:**

- **Water consumption factor entry — known scope risk.** Plan
  extra time per the brightway-hardcoded precedent. May require
  manual transcription rather than programmatic extraction even
  for the GLO-default values. Initial seed-test coverage may be
  tighter than other V1 categories at first register; expand as
  parsing edge cases are understood.
- **POCP-ecosystems V2 trigger.** Surface user demand from real
  users; ship in V2 alongside any other splits-beyond-EN-15804
  variants if a single-POCP-axis preset shows up insufficient.
- **USEtox toxicity slate scoping (V2).** Separate scoping decision
  before factor entry. Will need paired EF 3.1 + CML-IA + ReCiPe
  toxicity values for cross-witnessing (three independent factor
  tables on the same matcher infrastructure). Same scope-discipline
  pattern as V1 presets; do not collapse into "V2 is one bucket".
- **Regionalisation bundle (V2).** `CasRegion` matcher + per-process
  `C` build + Eq. 3 restructure (`h = Σₚ Cₚ · B[:,p] · s[p]`) +
  reader region-extraction verification + 5 country-specific CF
  tables (POCP-h, particulate matter, terrestrial acidification,
  freshwater eutrophication, water consumption). Cleanly scoped
  V2 work driven by real user demand for region-aware LCA. Region
  vocabulary verification (ISO 3166-1 alpha-2/alpha-3, watershed
  IDs) happens during V2 implementation alongside factor entry
  for the country-specific tables.
- **RIVM outreach (Phase 2-3).** Contact RIVM (ReCiPe coordination
  via `recipe@rivm.nl` or the document author chain) to request
  explicit grant for the redistributed CFs. Not blocking V1;
  hygiene step for commercial-scale shipping. Same pattern as the
  pending Leiden outreach in `D-0017`.
- **Egalitarian / Individualist perspectives (V2-or-never).** Same
  matcher infrastructure, different factor tables. Pure factor-
  table addition; no design change. Track demand from real users;
  ship if and when surfaced.

---

## 2026-04-22 · `D-0018` — Phase-1 "EPDX" bullet closed via LCAx V1 writer; ILCD+EPD writer staged for Phase 2

**Context:** The Phase-1 execution guide listed "EPDX read/write works"
as a Week 5-6 bullet. Scope discovery for that work surfaced that the
EPDX name has been archived by its upstream authors (ocni-dtu, 2024-08-22)
in favour of **LCAx** — the actively-maintained successor, same
maintainers, Apache-2.0, `lcax_models` v3.4 on crates.io. At the same
time, preset-scoping discovery clarified that the EPD ecosystem actually
hosts **four** distinct formats with overlapping-but-not-identical
purposes:

| Format | Role | License | Active? |
|---|---|---|---|
| **LCAx** (ocni-dtu) | Open exchange format for LCA results/EPDs/assemblies; successor to EPDX | Apache-2.0 | ✅ |
| **ILCD+EPD** (ILCD format + EPD extension) | The actual submission format for Environdec/IBU/EPD International | proprietary-ish | ✅ |
| **EPDX** (ocni-dtu) | Predecessor to LCAx | Apache-2.0 | 🔴 archived 2024-08-22 |
| **openEPD** (Building Transparency) | Competing ecosystem, buildings-focused | Apache-2.0 | ✅ |

Closing the Phase-1 bullet required choosing (a) which format(s) to
ship, (b) what minimum-viable shape, and (c) reader, writer, or both.

**Decision:** *Staged plan.* Ship an **LCAx v3.4 writer-only (V1)** now
to close the Phase-1 bullet via the living successor format. Schedule
the **ILCD+EPD writer (V2)** for Phase 2 *before* the EPD renderer
milestone — ILCD+EPD is the format program operators actually accept,
so Phase 2's "practitioner submits an EPD" workflow needs it.

Reasons, in descending weight:

1. **The Phase-1 bullet's original "EPDX" target is archived.** Shipping
   against a dead format to satisfy a planning-document bullet is the
   kind of execution drift Arko's decision-log discipline exists to
   prevent. The living successor has the same maintainers, the same
   Apache-2.0 license, and a Rust `lcax_models` crate on crates.io —
   the upgrade path is literally "type `lcax_models` where you would
   have typed `epdx`".
2. **LCAx is writer-meaningful in isolation; ILCD+EPD isn't read-or-write-
   meaningful before Phase 2.** A standalone LCAx document
   demonstrates "Arko produces a real schema-conformant digital EPD
   artifact" — the end-to-end story Phase 1 needs to tell. An
   ILCD+EPD writer is only useful against a program operator's
   submission workflow, and that workflow is a Phase-2-UI concern.
3. **Phase-2 config reuse is real.** The ~10 EPD-domain metadata fields
   LCAx requires (RSL, publishedDate, validUntil, version,
   declaredUnit, subtype, location, projectPhase, softwareInfo,
   formatVersion) are the same set ILCD+EPD will need. The writer's
   config struct is deliberately named `EpdDocumentMetadata`, not
   `LcaxWriterConfig`, so the Phase-2 ILCD+EPD writer can consume the
   same type.
4. **Reader is future-V2.** Phase 1's deliverable is "import data →
   build study → run calculation → **export** EPD"; ingesting
   existing LCAx documents into Arko studies is a separate problem
   (different invariants, different failure modes) that Phase 1
   doesn't need to solve.

**Writer shape (V1):** A synthetic single-Assembly / single-Product
Project wrapping one EPD. The schema requires `Project` as the root
type; standalone Product/EPD documents are not valid LCAx. The wrapper
is real but thin — one input (`product_name`) collapses to four
schema-level `.name` fields.

**Three refinements landed with the decision:**

1. **`EpdDocumentMetadata` naming.** Chosen deliberately over
   `WriterConfig` / `LcaxWriterConfig` because the Phase-2 ILCD+EPD
   writer will consume the same shape. Portable EPD-domain concept,
   not LCAx-specific glue.

2. **`standard` enum mapping — preservation via `EPD.comment`.** LCAx
   constrains `EPD.standard` to `EN15804A1 | EN15804A2 | UNKNOWN`.
   Of Arko's four V1 method presets, only `ef-3.1` maps cleanly to
   `EN15804A2`. The other three (`ipcc-ar6-gwp100`, `ipcc-ar5-gwp100`,
   `cml-ia-baseline`) predate or sit outside EN 15804 and emit as
   `UNKNOWN` — *not* because the methodology is unknown but because
   the schema enum has no matching variant. To keep methodology
   discoverable, the writer always embeds `"Generated by Arko using
   method <id>@<version>"` in `EPD.comment`. Downstream consumers
   can pattern-match `comment` rather than `standard` when
   `standard == UNKNOWN`. This is the least-bad option: the
   alternatives (lying about `standard`, dropping the method
   entirely, refusing to emit) all fail worse.

3. **Stage defaulting to `A1A3`.** Arko's `Study` doesn't model
   life-cycle stages — the engine produces one number per impact
   category. LCAx requires every impact value to land under a
   `LifeCycleModule` (`a0`–`d`). V1 emits all values at `A1A3`
   (cradle-to-gate) because that's the truthful stage for the
   unit-process LCAs Arko fixtures currently represent (carpet
   cradle-to-gate, beef finishing). Real stage decomposition is a
   Phase 2-3 question (requires either per-stage process tagging in
   `Study` or per-stage sub-calculations); the writer picks the most
   honest single-stage default until then.

**Implementation:** New crate `engine/io-lcax`, dependency on
`lcax_models = "3.4"` + `lcax_core = "3.4"` (the former for types, the
latter for `Country`). Single public entry point
`write_lcax_project(study, computed, metadata) -> Result<Project,
WriteError>`. `(α)` defaults-everywhere config (no core-type changes);
the `(β)` first-class `EpdMetadata` on `Study` is a Phase-2 decision
driven by UI needs, not a Phase-1 blocker. 9 tests total (7 unit + 2
smoke including JSON round-trip through `serde_json` → `Project`
proving schema-shape conformance).

**Outcome:** Phase 1 "EPDX" execution-guide bullet closed. Four
remaining Phase-1 items: ReCiPe 2016 Midpoint preset, FactoredSolver,
LCAx reader (V2), ILCD+EPD writer (V2). Of these only ReCiPe and
FactoredSolver block Phase-1 exit; the two V2 items move to Phase 2
planning.

**References:** `engine/io-lcax/src/lib.rs` (scope + mapping),
`engine/io-lcax/src/writer.rs` (writer + comment preservation),
`engine/io-lcax/src/metadata.rs` (config shape + Phase-2 reuse note),
`engine/io-lcax/tests/minimal_writer_smoke.rs` (round-trip evidence).

---

## 2026-04-22 · `D-0017` — CML-IA baseline V1 scoped to the EN 15804+A2-aligned subset; toxicity, regional variants, and POCP low-NOx deferred

**Context:** `D-0011` made Arko the primary product; `D-0015` set the
EN 15804+A2-aligned-subset framing for EF 3.1 V1 (mandatory-core only,
toxicity deferred); `D-0016` extended the taxonomy to support EF 3.1
CC's biogenic/LULUC origin splits. CML-IA baseline (Leiden CML, v4.8,
August 2016) was named the fourth Phase-1 method preset, the second
non-climate one, and Arko's first legacy-EPD verification method.
CML-IA predates EN 15804+A2 by more than a decade and was a direct
ancestor of EF 3.1's category set; it remains the most-cited LCIA
method in pre-2020 European EPDs. Picking the V1 scope required
choosing which of CML-IA's many baseline categories to ship and how
to handle the methodological differences from EF 3.1 (which had
already shipped at `D-0015`/`D-0016`).

The taxonomy-axis check ran first per the preset-scoping discipline.
CML-IA's CF structure was inspected via direct inspection of
`CML-IA_aug_2016.xls`, sheet "characterisation factors". Findings:

1. **No `CasRegion` axis required for V1** — all shipped categories
   use a single pan-European-default CF set. The spreadsheet ships
   per-country variants for AP and POCP; V1 ships only the
   pan-European total, defers per-country to a future `CasRegion`
   matcher. (Open: ReCiPe 2016 may force `CasRegion` on regionalised
   midpoints — the same deferral pattern would absorb it.)
2. **No new `FactorMatch` variant required** — the existing 5
   variants (`Cas`, `CasOrigin`, `CasCompartment`, `FlowId`,
   `NameAndCompartment`) cover every CML-IA V1 category. The only
   surprise was ADP-fossil (see point 4).
3. **No `FlowOrigin` extension required** — CML-IA baseline predates
   the EN 15804+A2 carbon-neutrality convention. CO2 has a single
   CF of 1.0 regardless of provenance; CH4 has a single CF
   regardless of origin. Origin splits are an EF 3.1 / EPD-policy
   layer, not a CML-IA property.
4. **ADP-fossil ships a hybrid matcher.** Source data uses real CAS
   for natural gas (8006-14-2) and crude oil (8012-95-1) but
   literal-label identifiers ("coal hard", "coal soft", "fossil
   fuel") for the other three fossil resources. V1 honours this
   mixed convention via mixed `Cas` + `NameAndCompartment` matchers
   within one factor list. ADP-fossil is the only V1 category that
   ships a hybrid matcher — driven by source-data structure, not a
   taste choice.

Direct inspection of the spreadsheet also surfaced four scope
corrections from the initial reading-by-headers assumption:

- **GWP100 in CML uses IPCC 2013 *without* climate-carbon feedback,**
  whereas Arko's existing `ipcc-ar5-gwp100` preset uses the
  with-feedback values (CH4 = 30 vs CML's 28; N2O = 273 vs 265;
  SF6 = 25_200 vs 23_500). The two AR5-derived presets are
  intentionally distinct factor tables and per-factor source
  comments cite this each time so the difference does not
  accidentally get "fixed" toward each other.
- **Acidification baseline is the average-Europe-total Huijbregts
  variant** (col 60 in the source spreadsheet), not the simpler
  non-regional one. Reference species is SO2 with CF = 1.2 (NOT
  1.0) — the average-Europe model carries fate weighting on the
  reference. Per-country variants exist (col 64+) and are deferred.
- **Eutrophication is compartment-uniform per substance** in
  CML-IA's "fate not incl." baseline (col 62). The matcher choice
  is `Cas`, not `CasCompartment` — the source data does not vary
  CFs across air/water/soil compartments for any P or N species.
  This is the matcher-shape difference from EF 3.1's three-way EP
  split (which ships compartment-keyed CFs because EF 3.1 includes
  fate models).
- **Toxicity (HTP, FAETP, MAETP, TETP) is in CML-IA baseline** at
  USES-LCA infinity-time-horizon CFs. V1 omits these even though
  they are nominally part of "the baseline" because the EN 15804+A2
  mandatory-core set excludes toxicity, and shipping USES-LCA-derived
  CFs at V1 risks propagating known criticisms (Hauschild,
  Pennington 2002) into Arko-stamped numbers without independent
  factor-value seeds. Toxicity is deferred to V2 alongside the EF
  3.1 toxicity expansion — same scope-discipline pattern.

**Decision:** Ship `("cml-ia-baseline", "4.8")` with seven
EN 15804+A2-aligned categories: GWP100, ozone-depletion,
photochemical-ozone-formation (high-NOx), acidification (avg-Europe
total A&B), eutrophication (combined P+N, fate not incl.),
ADP-elements (ultimate-reserves), ADP-fossil (hybrid matcher).
Method `name` field is `"CML-IA baseline (Leiden, v4.8)"`. Indicator
units follow the source: `kg CO2-eq`, `kg CFC-11-eq`, `kg
ethylene-eq` (NOT NMVOC-eq), `kg SO2-eq`, `kg PO4-eq`, `kg Sb-eq`,
`MJ`. Per-factor source comments cite source file + sheet + column +
model-variant header + substance row, with an extra line for GWP100
factors noting the without-vs-with-feedback split.

V2 expansion will likely include: the toxicity quartet (HTP, FAETP,
MAETP, TETP) per the EN 15804+A2 optional-reporting set; per-country
AP and POCP variants behind a `CasRegion` matcher; and the POCP
low-NOx variant for studies in NOx-limited atmospheric regimes. None
of these block Phase 1 exit.

**Reasoning:**

- **Legacy-EPD verification is a real Phase-1 use-case, not just
  scope padding.** Pre-2020 European EPDs are dominated by CML-IA
  baseline citations. Without a CML-IA preset, Arko cannot verify
  those numbers, which is a hard requirement for any consultancy
  workflow that touches an existing EPD library. The scope choice
  ("EN 15804+A2-aligned subset of CML-IA baseline") makes the
  preset specifically useful for *modern-format-meets-legacy-method*
  verification — the highest-traffic legacy-EPD shape.
- **Side-by-side with EF 3.1 is the V1 differentiator.** Two
  presets that share categories but use different reference species
  (ethylene-eq vs NMVOC-eq for POCP), different ranking shifts (WMO
  2003 vs WMO 1999 ODP putting Halon-1301 vs Halon-2402 at the top),
  different EP semantics (combined vs split), and different GWP
  conventions (without- vs with-feedback) lets Arko users see how
  much of a study's bottom-line result depends on the LCIA-method
  choice. That insight is the core practitioner value of having
  multiple comparable presets in one tool.
- **Shipping toxicity at V1 would create false confidence.** The
  USES-LCA model that CML-IA toxicity uses has documented limitations
  (infinity-time-horizon makes results dominated by long-lived
  metals; fate models predate modern atmospheric chemistry).
  Arko-stamped toxicity numbers without independent factor-value
  seeds risk being cited as authoritative when they are
  decade-old-with-known-issues. V2 toxicity will need its own
  factor-table-entry-discipline pass with paired EF 3.1 toxicity
  values for cross-witnessing.
- **Hybrid matcher in ADP-fossil is the right pattern, not a smell.**
  The source data is mixed-convention. Forcing all five fossil
  species into one matcher type would either invent CAS numbers
  (silent-correctness risk) or rename the species (drifts from
  source). The hybrid is documented at the call site, witnessed by
  a dedicated seed test (`cml_ia_adpf_matchers_are_hybrid_...`),
  and called out in this decision so future-me does not "clean it
  up". Names drift; CAS numbers don't — but when source data uses
  names, we use names.
- **Compartment-wiring caveat applies to CML's CasCompartment
  categories** (POCP, AP, ADP-elements, half of ADP-fossil) just as
  it does to EF 3.1. The ILCD and openLCA flow readers populate
  `FlowMeta::compartment` as `Vec::new()` until the bridge layer
  extracts compartment from process exchanges. So at Phase 1 exit,
  the compartment-keyed matchers compile and register correctly,
  pass their matcher-shape invariant tests, but only bind to real
  flow rows when reader compartment-extraction lands. This is a
  documented Phase-1-exit honest-state, not a regression introduced
  by CML-IA.

**Consequences:**

- `MethodRegistry::standard()` ships **4 methods** at Phase 1 exit
  (AR6, AR5, EF 3.1, CML-IA baseline 4.8). The Phase-1 method-preset
  exit criterion (4-of-4 from the original Phase-1 plan) is met.
- The remaining Phase 1 punch-list items (ReCiPe 2016 Midpoint
  preset, EPDX reader, FactoredSolver) are independent of method-
  preset count — Phase 1 exits when those land, not when CML-IA
  shipped.
- Per-factor source-comment template extended with the GWP100
  without-vs-with-feedback distinguishing line. Documented at
  `docs/licenses/cml-ia-leiden.md` for future preset entries
  derived from the same source.
- License posture is "gratis with no explicit license terms" —
  defensible for V1 (factual data, different selection/arrangement,
  attribution preserved); commercial-scale distribution would need
  explicit grant. Full analysis at `docs/licenses/cml-ia-leiden.md`.
- The EU sui generis Database Right is the only theoretical
  rights-residue concern (factual data is not copyrightable; CFs
  are factual), and Arko's selection-and-arrangement is materially
  different from the source. Risk-disclosure in the license doc;
  not a blocker.

**Open items:**

- Leiden outreach (Phase 2-3): contact Lauran van Oers
  (`oers@cml.leidenuniv.nl`, listed contact in spreadsheet) to
  request explicit grant for the redistributed CFs. Not blocking
  V1; a hygiene step for commercial-scale shipping.
- `arko-license` preset: register `cml_ia_leiden_gratis` once the
  license-tier crate gains a "gratis-no-explicit-grant" tier
  variant.
- POCP low-NOx variant: track demand from real users; ship
  alongside the per-country AP variants in V2 if a single-CF-axis
  preset shows up insufficient.

---

## 2026-04-21 · `D-0016` — `FlowOrigin` taxonomy extended from 3 to 4 values; `NonFossil` renamed `Biogenic`, `LandUseChange` added

**Context:** The pre-D-0016 `FlowOrigin` enum was three-valued —
`Unspecified | Fossil | NonFossil` — adopted at AR6 preset time when the
only origin distinction shipped was AR6's CH4 fossil/non-fossil split
(29.8 vs 27.0). Starting on **EF 3.1 Climate change** factor entry
(see `D-0015` for EF 3.1 scope) surfaced a semantic gap: EF 3.1 CC
distinguishes **three** origin classes for CO2 and CH4 — fossil,
biogenic, **and land-use-change** — with values that do not collapse:

- CO2: fossil = 1.0, biogenic = 0.0, land-use-change = 1.0
- CH4: fossil = 29.8, biogenic = 27.0, land-use-change = 29.8

The naming "NonFossil" was already imprecise for AR6 (where it meant
"biogenic short-loop carbon"), but the imprecision was harmless while
LULUC was outside the registry's reach. Once EF 3.1 CC enters, the
imprecision becomes a silent-correctness bug: a methane flow tagged
"land use change" in an ILCD inventory would parse to `NonFossil` and
match the biogenic CF (27.0) instead of the LULUC CF (29.8). The
calculation would not error, would not warn, would not surface in
`unmatched_flows` — it would just be wrong.

The bug surface was already present in the codebase pre-EF-3.1 even
without LULUC CFs in any preset: both ILCD and openLCA flow parsers
contained an `_ => Unspecified` fall-through for any origin tag they
did not recognise, including the literal string "land use change".
Audit of `arko_io_ilcd_linker::flow::classify_flow_origin` and the
mirror in `arko_io_olca_jsonld::model` confirmed the silent
mis-classification predates this decision; the taxonomy extension
fixes it.

**Decision:** Extend `FlowOrigin` (and its parallel reader-side enum
in `arko-io-ilcd-linker`) from three to four variants:

```rust
pub enum FlowOrigin {
    Unspecified,
    Fossil,
    Biogenic,         // renamed from NonFossil
    LandUseChange,    // new variant
}
```

The `NonFossil → Biogenic` rename is part of the same change. "Biogenic"
is the unambiguous EN 15804+A2 / EF 3.1 / IPCC AR6 term for short-loop
biospheric carbon; "non-fossil" was a domain-incorrect umbrella that
implied LULUC was a kind of biogenic, which contradicts how all three
standards account for it (LULUC carbon is typically counted *with*
fossil at GWP100, not with biogenic). Renaming alongside the variant
add keeps the three-value vs four-value distinction visible at every
call site rather than letting the new semantics hide behind an old name.

Both flow parsers updated to map "land use change" → `LandUseChange`.
Test-first: failing parser tests written before the rename, so the
silent-mis-classification bug was reproduced as a red test before the
fix made it green. AR6 preset's CH4 also expanded from 2 to 3
`CasOrigin` factors with explicit `LandUseChange = 29.8`
(fossil-equivalent per AR6 GWP100 footnote) — same anti-silent-zero
rationale: an unmatched LULUC CH4 is more dangerous than the same
flow producing a documented fossil-equivalent value.

**Reasoning:**

- **Domain correctness > backwards-compatibility ergonomics.** The
  rename touches 19 call sites. Compiler-driven cascade catches them
  all in one pass; the alternative (deprecating `NonFossil` as an
  alias for `Biogenic` and adding `LandUseChange` separately) would
  leave the imprecise name in code permanently for no benefit since
  the engine has no external consumers yet.
- **Explicit equivalence > implicit fallback for LULUC = fossil.**
  Both AR6 and EF 3.1 happen to assign LULUC CH4 the same numerical
  value as fossil CH4 (29.8). It would be tempting to model LULUC as
  "fall back to fossil's CF if no LULUC entry exists." Rejected: the
  fall-back hides the equivalence from code reviewers and from
  EPD verifiers reading the table; an explicit `LandUseChange`
  factor with `value = 29.8` and a `// LULUC = fossil-equivalent`
  comment makes the policy decision visible at the data layer.
- **Verified semantics-preserving.** The EF carpet parity smoke
  (`max |dev| = 4.654e-6` against the Python reference) and the USDA
  beef multi-process LU parity smoke (`max |dev| = 1.776e-15`) both
  pass after the migration. The taxonomy widened without any existing
  calculation drifting — the only behavioural change is that flows
  that were previously silently mis-classified are now correctly
  classified.

**Reversal condition:** None plausible. A future origin distinction
the standards bring in (e.g. "blue carbon" oceanic uptake/release)
would extend the enum to a fifth variant, not revert this one.

**Out of scope for this entry:** EF 3.1 CC factor entry itself, and
the registration of EF 3.1 in `MethodRegistry::standard()`. Both
land in the same commit but are scope choices governed by `D-0015`,
not new decisions.

---

## 2026-04-20 · `D-0015` — `FactorMatch::CasCompartment` variant added; EF 3.1 V1 scoped to the 7 emission-based EN 15804+A2 core indicators

**Context:** Phase 1 exit slate requires four method presets (AR6
GWP100, EF 3.1, CML 2001, ReCiPe 2016). Shipping AR6 first locked in
the existing `FactorMatch` taxonomy — four variants: `Cas`,
`CasOrigin`, `FlowId`, `NameAndCompartment`. Starting on EF 3.1
surfaced a category of factor that neither of the CAS variants nor
the name-based variant can express cleanly: **CAS-keyed, compartment-
specific** characterization factors.

Concrete example the existing taxonomy gets wrong: **EF 3.1
Acidification** scores SO2 emitted to air at CF ≈ 1.31 mol H+-eq/kg,
but SO2 dissolved in water has CF = 0 for acidification (it's counted
under eutrophication pathways if anywhere). With only `Cas` available,
the author has to choose: either tag SO2 globally with 1.31 (wrong
for water emissions) or leave SO2 unmatched and rely on
`NameAndCompartment` to pick it up by name (brittle across databases
where the substance name might be "Sulfur dioxide" vs "Sulphur dioxide"
vs "SO2"). CAS is the reliable cross-database key; compartment is the
orthogonal axis the CF depends on; the taxonomy needed a variant that
combines both.

**Decision:** Add a fifth `FactorMatch` variant:

```rust
CasCompartment { cas: String, compartment: Vec<String> }
```

Semantics: flow's CAS must equal `cas` **and** the flow's compartment
path must start with the matcher's compartment (prefix match, same
as `NameAndCompartment`). Origin-agnostic. An empty `compartment`
vector reduces to plain CAS matching (redundant with `Cas` — method
authors should prefer the simpler variant; the code doesn't reject the
redundancy but the discipline is documented).

**Concrete behaviour — see the tests that define it:**

- Basic prefix match: `CasCompartment { cas: "7446-09-5",
  compartment: ["emission", "air"] }` matches an SO2 flow in
  `emission/air/urban` (flow's compartment is longer than matcher's —
  prefix semantics per
  [`cas_compartment_matches_flow_with_deeper_path`](engine/methods/src/method.rs))
- Disjoint compartments coexist: same CAS, compartment `["air"]` and
  compartment `["water"]` in the same category match two different
  flows and both land in the matrix (per
  [`disjoint_cas_compartment_factors_coexist_in_one_category`](engine/methods/src/builder.rs))
- Wrong compartment rejected: SO2 matcher on `["emission", "air"]`
  does not match an SO2 flow in `emission/water` (per
  [`cas_compartment_rejects_different_compartment`](engine/methods/src/method.rs) —
  this is the load-bearing property the variant exists to enforce)
- Overlap with `Cas` hard-fails: a plain `Cas` factor and a
  `CasCompartment` factor matching the same flow in the same category
  triggers `CMatrixError::DuplicateMatch` (per
  [`cas_and_cas_compartment_matching_same_flow_is_duplicate_error`](engine/methods/src/builder.rs))

**Reasoning:**

1. **Load-bearing for EF 3.1 core categories.** Of the 7 EN 15804+A2
   core emission indicators EF 3.1 V1 will ship (see scope decision
   below), four — Acidification, Eutrophication-freshwater,
   Eutrophication-marine, Eutrophication-terrestrial — have
   compartment-dependent CFs. Without this variant, those four
   categories cannot be expressed against CAS-keyed flows. With it,
   they fall out naturally.
2. **Generalises forward to ReCiPe 2016.** ReCiPe 2016 has the same
   compartment dependence across acidification and eutrophication
   midpoints. Landing the variant now, once, against EF 3.1's factor
   entry is strictly cheaper than landing it twice (once per method).
3. **Orthogonal to `CasOrigin`.** AR6 CH4 needs fossil/non-fossil
   split (origin axis); EF 3.1 SO2 needs air/water split (compartment
   axis). No current preset needs **both** axes simultaneously — but
   when one eventually does, the right move is a composite variant
   (`CasOriginCompartment`) rather than extending either existing
   variant with an optional field. Deferring that shape until a real
   preset demands it avoids premature design.
4. **Duplicate-match discipline unchanged.** The builder's "at most
   one factor per (category, flow)" invariant continues to hard-fail
   on authorship bugs. No priority-ordering or "most-specific wins"
   logic was added — method authors pick one matcher per flow per
   category and the builder guards that discipline.

**Non-feature: there is no specificity ordering.** It's tempting to
read the enum variant declaration order
(`Cas` → `CasOrigin` → `CasCompartment` → `FlowId` → `NameAndCompartment`)
as a priority chain where more-specific matchers "win" over
less-specific ones. **This is not what the builder does.** Each
factor is tested against each flow independently; two matches within
the same category hard-fail via `DuplicateMatch`. The enum order is
documentation, not priority. Recording this explicitly because the
"most-specific wins" mental model is the first intuition a future
contributor will have, and it would quietly produce wrong behaviour
if they relied on it.

**EF 3.1 V1 scope (paired decision):** EF 3.1 will ship the **7
emission-based core indicators of EN 15804+A2**:

1. Climate change (`CasOrigin` — reuses AR6 factor table)
2. Ozone depletion (`Cas`)
3. Photochemical ozone formation (`Cas`)
4. Acidification (`CasCompartment`)
5. Eutrophication, freshwater (`CasCompartment`)
6. Eutrophication, marine (`CasCompartment`)
7. Eutrophication, terrestrial (`CasCompartment`)

Deferred to V2: particulate matter (subcompartment-dense; EN 15804+A2
annex "additional indicator"); ionising radiation, human tox cancer,
human tox non-cancer, ecotoxicity freshwater, land use, resource use
fossils, resource use minerals/metals (also "additional indicators"
in EN 15804+A2). **Water use deferred pending a regionalised
matcher** (`CasRegion` or compartment-with-region variant) that aligns
with ReCiPe 2016's regionalised midpoints — doing it twice is waste;
doing it once when the regionalisation shape is known is right.

This scope produces a shippable EPD floor: any EN 15804+A2 EPD that
relies only on **core** indicators (the mandatory set) is fully
serviceable by Arko's EF 3.1 V1 preset. EPDs that additionally report
"additional indicators" (optional reporting category) will need V2.
Framing the scope around the standard's core/additional split, rather
than picking a subset of categories by feel, makes the V1-vs-V2
boundary defensible to verifiers.

**Alternatives considered:**

- *Extend `CasOrigin` with an optional `compartment` field.* Rejected:
  breaks the variant's tight semantics (origin-only matching) and
  forces every existing `CasOrigin` factor to carry an explicit `None`
  in its JSON serialization.
- *Ship EF 3.1 with only the categories that fit the existing
  taxonomy (CC + OD + POCP).* Rejected: produces a preset that cannot
  generate an EN 15804+A2 EPD because the eutrophication and
  acidification core indicators would be missing. Shippable-EPD floor
  is the right correctness bar for a core preset.
- *Build a priority-ordered fallback chain in the builder
  ("CasCompartment wins over Cas").* Rejected: changes matching
  semantics for all existing factors, not just new ones. Duplicate-
  error discipline is strictly safer — method authorship bugs surface
  loudly. Priority fallback would let bugs compile.

**Evidence the decision produced:**

- New variant shipped at
  [`engine/methods/src/method.rs`](engine/methods/src/method.rs) with
  `matches()` arm.
- Nine method-level tests covering prefix match, deeper-path match,
  compartment rejection, shorter-path rejection, no-CAS rejection,
  wrong-CAS rejection, origin-agnostic match, empty-compartment
  edge, JSON round-trip — all in
  [`engine/methods/src/method.rs`](engine/methods/src/method.rs)
  `tests` module under the `CasCompartment` heading.
- Three builder-level tests covering route-into-matrix, disjoint
  coexistence, and `Cas × CasCompartment` duplicate error — in
  [`engine/methods/src/builder.rs`](engine/methods/src/builder.rs).
- `matcher_label()` arm added so `DuplicateMatch` error messages show
  `CAS 7446-09-5 in ["emission", "air"]` rather than falling off a
  non-exhaustive match.

**Non-decision carried forward:** whether a composite
`CasOriginCompartment` variant is needed. Defer until a real preset
has a substance whose CF depends on both axes simultaneously.

**Cross-references:** `D-0007` (AR6 + AR5 preset pair —
`CasOrigin` precedent), the CH4 origin/compartment example above.

---

## 2026-04-20 · `D-0014` — openLCA JSON-LD reader lives in a new crate, not in `arko-io-ilcd`

**Context:** Phase 1 had three foreground-free databases named after
`D-0013`. Two of the three (ÖKOBAUDAT, JRC EF) publish in ILCD XML and
plug into the existing [`arko-io-ilcd`](engine/io-ilcd) reader. The
third — USDA LCA Commons — publishes in **openLCA JSON-LD**, a
structurally-similar but syntactically-different serialization that
the ILCD reader cannot decode: per-object JSON under `processes/`,
`flows/`, `flow_properties/`, `unit_groups/`; UUID-addressed units
instead of `dataSetInternalID` integers; `referenceUnit: true` flags
instead of ID pointers; origin encoded as a comma-tail
(`"Methane, biogenic"`) rather than a parenthetical
(`"methane (biogenic)"`).

The question when starting Week-6 of Phase 1: extend `arko-io-ilcd`
to dual-read XML *and* JSON-LD, or stand up a separate crate?

**Decision:** Separate crate — `arko-io-olca-jsonld` — scoped to the
USDA beef cattle finishing bundle (five-process cow-calf-finisher
subgraph), with a narrow adapter boundary producing the same
`TypedColumn` the ILCD linker already consumes.

**Reasoning:**

1. **Two formats, one linker.** ILCD and openLCA JSON-LD disagree on
   every concrete shape while representing the same conceptual
   model. A dual-reader crate would need two parallel parser trees
   and a lot of shape-adaptation boilerplate in a single module; the
   adapter layer is the single place where shapes collapse into one
   typed column. A dedicated JSON-LD reader plus a single-purpose
   adapter keeps that collapse point explicit.
2. **Resist premature-shared-crate promotion.** Shared-types
   abstraction needs ≥ 3 data points (ILCD, openLCA JSON-LD, and one
   more) to reveal the right shared surface. Two is too few; forcing
   unification now would pick the wrong axis and need to be
   undone. Duplicated parser plumbing in two crates is the
   temporarily-right trade.
3. **Boundary discipline.** Parser emits pure `OlcaProcess` / etc.
   native types; the adapter is the **only** place that touches
   `arko_io_ilcd_linker::{TypedColumn, TypedExchange, FlowType,
   ReferenceUnit}`. A parse failure on a USDA bundle can be
   diagnosed without also untangling the matrix-bridge concerns.
4. **Scope matches the data under pressure.** v0.1 reader targets
   the beef bundle (`SUPPORTED.md` enumerates what the bundle needs
   and what the crate deliberately punts: `LCI_RESULT` semantics,
   `allocation` factor blocks, parameter expressions,
   `avoidedProduct` sign-flip, cross-property exchanges, ZIP
   packaging, `actors` / `sources` / `categories` / `locations` /
   `dq_systems` object kinds). Extensions land when the next bundle
   demands them, not speculatively.

**Evidence the decision produced:**

- Crate lands with 24 unit tests covering parser + adapter + CAS
  normalization + origin classification.
- Structural smoke test ([`engine/io-olca-jsonld/tests/beef_bundle_smoke.rs`](engine/io-olca-jsonld/tests/beef_bundle_smoke.rs))
  loads all five beef processes, verifies the 3-edge finishing DAG
  and 2-edge calf DAG, trims the zero-padded `000074-82-8` CAS to
  the canonical `74-82-8`, classifies `Methane, biogenic` as
  `NonFossil` via the comma-tail rule.
- Multi-process parity smoke
  ([`engine/io-olca-jsonld/tests/beef_multi_process_parity_smoke.rs`](engine/io-olca-jsonld/tests/beef_multi_process_parity_smoke.rs))
  matches an independent Python reference (`numpy.linalg.solve`,
  stdlib `json`, reimplemented CAS/origin rules) on AR6 GWP100 to
  max abs deviation `1.776e-15` — essentially ulp agreement,
  `CrossImpl` tolerance class is well above what the test actually
  requires but the right *claim* for this evidence.

**Non-decision carried forward:** when/whether to promote shared
types between `arko-io-ilcd` and `arko-io-olca-jsonld` stays
open. Answer when the third reader (ecospold-2, or whatever comes
after) tells us what the right shared surface actually is.

---

## 2026-04-20 · `D-0013` — USDA LCA Commons named the Phase 1 third-slot foreground-free database

**Context:** `D-0012` (2026-04-19) reopened the third slot on the
Phase 1 "three free databases importable" exit criterion after the
EF LCI bundle was reclassified as restricted. Two candidates were
named pending primary-source license checks: ProBas
(Umweltbundesamt) and USDA LCA Commons. The same
EF-bifurcation-style discipline that produced
[`docs/licenses/jrc-ef.md`](docs/licenses/jrc-ef.md) was applied to
the USDA side on 2026-04-20; full primary-source analysis lives at
[`docs/licenses/usda-lca-commons.md`](docs/licenses/usda-lca-commons.md).

**The license read, in one line:** every dataset in the Federal LCA
Commons is dedicated to the public domain under **CC0 1.0
Universal**, mandatory at submission per USDA-NAL policy (*"USDA-NAL
is requiring that all datasets submitted to the LCA Commons be
placed in the public domain under the terms of the Creative Commons
Zero, Public Domain Dedication License (CC0 1.0 Universal (CC0
1.0))."* — LCA Commons Submission Guidelines, 2018-07-25 Final,
§ "Placing Your Data in the Public Domain"). CC0 § 2 waives
copyright "for any purpose whatsoever, including without limitation
commercial, advertising or promotional purposes."

Two non-copyright carve-outs travel with access but neither blocks
any Arko-ship operation:

1. **Trademark-style restriction** on USDA/ARS/NAL names in
   advertising/endorsement copy (CC0 § 4(a) explicitly preserves
   trademark).
2. **Appendix A AS-IS warranty disclaimer + indemnity** from user
   to Government for claims arising from data use.

Strictly more permissive than the JRC EF reference package:

| Axis                  | JRC EF ref package              | USDA LCA Commons               |
| --------------------- | ------------------------------- | ------------------------------ |
| License               | EC Decision 2011/833/EU (CC-BY equiv.) | CC0 1.0 Universal       |
| Attribution required? | Yes (legal)                     | No (encouraged, not legal)     |
| Commercial use?       | Yes                             | Yes (explicit in CC0 § 2)      |
| Redistribution?       | Yes with attribution            | Yes, unrestricted              |
| Term expiry?          | None (ref package)              | None                           |

**Decision:**

1. **USDA LCA Commons takes the third slot** on the Phase 1
   foreground-free database list. The Phase 1 exit criterion
   "three free databases importable" resolves to:

   1. ÖKOBAUDAT 2024-I — CC-BY-ND style with attribution (imported).
   2. JRC EF reference package — EC Decision 2011/833/EU reuse with
      attribution (imported).
   3. **USDA LCA Commons — CC0 1.0 Universal, unrestricted (to be
      imported).**

2. **ProBas (Umweltbundesamt) deferred, not rejected.** A future
   fourth-slot candidate; a primary-source license read is still
   owed if/when it enters the Arko critical path. No Phase 1
   dependency on it.

3. **`arko-license` preset roadmap.** Add a
   `usda_lca_commons_cc0` preset encoding: no legal attribution
   requirement, unrestricted commercial use, unrestricted
   redistribution, no term expiry, USDA/ARS/NAL trademark carve-out,
   AS-IS warranty disclaimer. This becomes the most permissive
   preset in the crate and serves as the "baseline free"
   calibration. Phase 2 work, not Phase 1.

4. **Hosting ToS language flagged for legal review before first
   paid hosted-data customer.** Appendix A's indemnity runs from
   "the user" (whoever accesses the data) to the Government. When
   Arko is the immediate accessor and serves customers downstream,
   the hosting ToS needs pass-through language so the indemnity
   resolves correctly. Not a Phase 1 blocker; a checklist item for
   the first paid Arko customer.

**Rationale:** The whole point of the `D-0012` discipline was that
a "free database" claim silently depending on a restricted bundle
would poison the Phase 1 exit. The USDA read produces the opposite
outcome for its candidate — CC0 is the cleanest possible license
posture, letting Arko ship fixtures, bundle the data, serve it via
SaaS, cite it in release notes, and publish parity smokes against
it without qualification. Taking the third slot on that basis is
the low-risk, license-honest move.

The format alignment is also convenient: the Submission Guidelines
confirm LCA Commons publishes in ILCD XML (openLCA-compatible), so
the existing `arko-io-ilcd` reader handles the parsing work
without net-new format engineering.

**Alternatives rejected:**

- *Hold the third slot open through Phase 1:* risks the Phase 1
  exit narrative reading as "two free databases, third TBD"
  instead of "three free databases." The claim is crisper if the
  slot is filled by exit time.
- *Fill the slot with ProBas instead:* requires the same
  primary-source license read first, and ProBas is a narrower data
  lot in a less-structured distribution. USDA's CC0 + ILCD-XML
  combination is strictly cleaner on both legal and engineering
  axes.
- *Treat USDA LCA Commons as a V2+ addition and ship Phase 1 with
  two databases:* weakens the generalisation signal that the
  three-database target exists to produce.

**Reversal condition:** If USDA-NAL changes the submission policy
(e.g. moves away from CC0 for new submissions, or imposes
dataset-specific restrictions that contradict the Submission
Guidelines), revisit this entry. If the first LCA Commons reader
import surfaces a structural mismatch between the Submission
Guidelines-declared format and the actual distributed artefacts,
revisit (3) and potentially the slot choice itself.

**Cross-references:** `D-0005` (V1 open EU databases only), `D-0010`
(foreground-free vs background-ecoinvent-dependent), `D-0012`
(EF reference package vs LCI bifurcation; slot reopened here).

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
