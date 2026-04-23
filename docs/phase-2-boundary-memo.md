# Phase 2 Boundary Memo — Web UI MVP

**Drafted:** 2026-04-22 (at v0.2.0 tag), §1/§3/§5 reframed for `D-0020` scope-pull-forward
**Phase 2 window per Execution Guide:** Weeks 11–24 (timeline shape A vs B per §5 / `D-0020`)
**Inputs:**
- [`phase-2-product-spec.md`](phase-2-product-spec.md) — **canonical scope source**
- [`phase-1-closeout.md`](phase-1-closeout.md) — engine state at boundary
- [`arko-execution-guide.md:178`](arko-execution-guide.md#L178) — original roadmap (parts overridden by `D-0020`)
- [`arko-tech-spec-v2.md`](arko-tech-spec-v2.md) — pragmatic compromises
- [`DECISIONS.md` `D-0020`](../DECISIONS.md) — Phase-4-to-Phase-2 scope pull-forward (billing, orgs, i18n)

This memo is the v0.2.0 → v0.3.x boundary record: the *architectural*
decisions still open at the boundary, the gaps inherited from Phase 1
that have to be addressed before screens can be built, and the
concrete week-1 work that opens Phase 2. *What* gets built lives in
[`phase-2-product-spec.md`](phase-2-product-spec.md); this memo is
*how* it gets built.

Decisions land in [`DECISIONS.md`](../DECISIONS.md) as `D-0020` onward
once committed (`D-0020` already filed for the scope expansion).

---

## 1. Scope — see the product spec

Canonical: [`phase-2-product-spec.md`](phase-2-product-spec.md). Six
deliverables (5 screens + EPD output) plus shared infrastructure
(auth, billing, orgs, projects, navigation, persistence, basic help).
The minimum working flow at the bottom of the spec is the v0.3.0
acceptance criterion.

| Deliverable | Source | Execution Guide week (original) | Boundary memo notes |
|---|---|---|---|
| Process Browser | [spec](phase-2-product-spec.md#process-browser) | 13–14 | inherits Phase 1 reader crates (ILCD, openLCA JSON-LD); §2.4 compartment-shape gap is load-bearing here |
| Study Builder | [spec](phase-2-product-spec.md#study-builder) | 15–17 | LCIA method dropdown defaults to EF 3.1 V1 (matches `D-0015` framing) |
| Calculation Runner | [spec](phase-2-product-spec.md#calculation-runner) | 18–19 | depends on §2.5 WASM compile spike; signed-manifest display reuses determinism contract from v0.0.1 |
| Contribution Analysis | [spec](phase-2-product-spec.md#contribution-analysis) | 20–21 | Sankey-primary visualization (visx Sankey or @nivo/sankey per Execution Guide note line 258) |
| Scenario Comparison | [spec](phase-2-product-spec.md#scenario-comparison) | 22 | candidate to defer to Phase 3 under timeline shape B (per `D-0020`) |
| EPD Output | [spec](phase-2-product-spec.md#epd-output) | 23–24 | LCAx writer (`io-lcax`, shipped Phase 1); Environdec submission guidance text only — no automated submission |

**Scope expansion via `D-0020` (pulled forward from Phase 4):**

- **Billing** — Redsys integration (per
  [`feedback_redsys_billing`](../../C--Users-hical-Desktop-karbongarbi/memory/feedback_redsys_billing.md)),
  three tiers (Studio/Team/Enterprise), 14-day trial, in-app plan
  management. Spec section: *The shared infrastructure → Billing*.
- **Org/project hierarchy** — `org → projects → studies` with
  owner/editor/viewer roles. Spec section: *The shared infrastructure
  → Organization/team management* + *Project structure*.
- **EN + ES UI** — i18n scaffolded from week 11; both locales shipped
  at v0.3.0. Spec section: *What Phase 2 does not include* — "EN + ES
  only" is the V1 boundary, not zero-i18n.

**Cross-cutting (from Execution Guide):**

- **Imanol-usable** for one real small study by week 24
- **WASM build** of `arko-engine` — set up week 11–12, used by Calc Runner from week 18
- **Axum API server** for studies that exceed WASM bounds — see §2.6 for day-1-vs-week-18 decision

---

## 2. Architectural decisions deferred to Phase 2 week 1

Each is a real open decision, not a foregone conclusion. The Execution
Guide's "Architecture reminder" lists defaults; Phase 2 week 1 either
ratifies those defaults with a `D-00xx` entry or revises them.

### 2.1 Auth — Keycloak vs Supabase Auth

**Default in guide ([line 203](arko-execution-guide.md#L203)):** Supabase
Auth for Phase 2; "we'll migrate to Keycloak later."

**Tension:** KarbonGarbi already runs Supabase Auth, so reusing the
pattern is the lowest-friction path. But the Master Spec v4 lists
Keycloak for Arko on sovereignty grounds (self-hostable, no vendor
lock-in for the EU-data narrative). Migrating auth mid-product is
expensive and high-blast-radius.

**Decision shape:** ratify Supabase Auth for Phase 2 with explicit
migration trigger (revenue threshold, or first enterprise customer
asking about self-hosted), OR commit to Keycloak from week 11 and
absorb the slower setup. Either is defensible; the wrong move is
deferring the decision into week 13 when screens depend on it.

### 2.2 Hosting — Vercel vs self-hosted

**Default in guide ([line 205](arko-execution-guide.md#L205)):** Vercel
for Phase 2; self-host in Phase 4.

**Tension:** same shape as 2.1 — sovereignty narrative argues against
Vercel; development velocity argues for it. KarbonGarbi infrastructure
patterns (CloudNativePG, Hetzner) exist and could be applied earlier.

**Decision shape:** ratify Vercel for Phase 2 with explicit Phase 4
migration commitment in the decision, OR set up Hetzner + Coolify-or-
equivalent in week 11–12 and run on it from day one. Quantifiable cost
of Vercel deferral: ~30–50 hours of DevOps in Phase 4 per the guide;
quantifiable cost of week-11 self-host: ~20–30 hours up front.

### 2.3 Design system — shadcn/ui + Tailwind matching KarbonGarbi

**Default in guide ([line 190](arko-execution-guide.md#L190)):** Next.js
15 + React 19 + Tailwind, "matches KarbonGarbi, reuse design system."

**Tension:** KarbonGarbi's design system has its own constraints
(rounded-full buttons, `#16A34A` primary, `gray-100` borders, no
alternating table rows — see KarbonGarbi memory). Some of these are
KarbonGarbi-specific aesthetics that won't carry over to the
practitioner-tool register Arko needs (LCA practitioners are used to
SimaPro's information density; rounded-full buttons may read as toy).

**Decision shape:** copy KarbonGarbi's tokens (colors, spacing,
typography, shadcn config) but commit explicitly to which aesthetic
choices Arko inherits versus which it diverges on. Land an
[`arko-design-system.md`](arko-design-system.md) sibling to the
KarbonGarbi guide in week 12.

### 2.4 Compartment extraction — Phase 1 gap to close

**Inherited gap:** the ILCD reader populates `FlowMeta.compartment`
from the basename parenthetical ("emissions to air, unspecified"
→ `["air", "unspecified"]`). The openLCA JSON-LD reader (D-0014, beef
bundle) populates compartment from `flow.category` differently. This
worked for Phase 1 because parity smokes only tested CC categories
where compartment doesn't differentiate — but EF 3.1 acidification and
eutrophication, both registered in `MethodRegistry::standard()`, use
`FactorMatch::CasCompartment` and dispatch on compartment paths.

**What's untested at v0.2.0:** end-to-end EF 3.1 AC/EU calculation on
the beef bundle (openLCA-source compartments through `CasCompartment`
matching). Carpet smoke is single-process CC-only and doesn't exercise
the path.

**Decision shape for Phase 2:** before Process Browser ships (week 13–14),
add a compartment-shape parity test: same substance via ILCD reader
and openLCA reader should land in the same `compartment` Vec. If not,
normalise reader-side. This is a Phase 1 carryover that becomes
load-bearing the moment the Calc Runner UI lets users pick non-CC
methods on openLCA-imported studies.

### 2.5 WASM target — engine compile-to-WASM compatibility

**Inherited unknown:** `arko-engine` has not been compiled to WASM.
`nalgebra` and `faer` both *should* compile (no platform-specific
deps), but Phase 1 never proved it. The Calc Runner exit criterion
("in-browser WASM <100ms for small studies") assumes this works.

**Decision shape:** week 11 spike — `cargo build --target
wasm32-unknown-unknown -p arko-engine`. If it fails, the failure mode
shapes Phase 2 architecture (e.g., must-have-API-server-from-day-one
versus optional-perf-fallback).

### 2.6 API server scope — Axum from day 1 or only when WASM bounds hit?

**Default in guide ([line 192](arko-execution-guide.md#L192)):** WASM
in-browser for small studies, Axum for larger.

**Tension:** the boundary between "small" and "larger" is undefined
at v0.2.0. The carpet smoke (1×1) and beef smoke (5×5) are both
trivially small; production studies can be 100s × 100s. Pre-existing
sparse solver uses faer 0.20 and may not be WASM-friendly.

**Decision shape:** decide in week 11 whether the Axum server is
day-1 critical-path or week-18 lazy-add. If WASM compiles cleanly and
small-study perf is acceptable, defer Axum to week 18. If not, Axum
from week 11.

---

## 3. Persistence — Postgres schema scope

**Default in guide ([line 194](arko-execution-guide.md#L194)):** Supabase-as-Postgres
(not Supabase-as-backend) — keeps sovereignty migration path open.

**What needs schema in Phase 2:**

*Account / billing layer (added per `D-0020`):*

- `users` (Supabase Auth's table)
- `organizations` (orgs are the subscription-attachment unit)
- `org_members` (m:n users↔orgs with `role` enum: `owner` / `editor` / `viewer`)
- `subscriptions` (org-level; tier ∈ {`studio`, `team`, `enterprise`}; status ∈ {`trial`, `active`, `past_due`, `canceled`}; trial_ends_at, current_period_end)
- `redsys_customers` (mirrors KarbonGarbi's `redsys_identifier` column pattern per [`feedback_redsys_billing`](../../C--Users-hical-Desktop-karbongarbi/memory/feedback_redsys_billing.md); stores `DS_MERCHANT_IDENTIFIER` after first payment for COF MIT charges)
- `projects` (org-scoped folders that contain studies; created by consultancies per client)

*Study / calculation layer (Execution Guide native):*

- `studies` (the JSON-serialisable `arko_core::Study`, plus metadata: name, project_id, created_by, created/updated timestamps, lock state, EPD-program-operator FK)
- `studies_versions` (immutable snapshots — Phase 3 verifier workflow needs this; ship the table in Phase 2 to avoid migration churn)
- `processes_imported` (cached/indexed view of imported processes for the Process Browser; the canonical source remains the original ILCD/openLCA bundles on disk or object storage)
- `study_runs` (calculation results with signed manifest, for reproducibility)
- `recently_viewed` (per-user) — Process Browser exit criterion

**Open question (process catalogue):** how much of the imported-process
catalogue lives in Postgres versus how much stays as parsed-on-load
from the original bundles. Indexing for full-text search needs
Postgres rows; loading a 94k-flow EF reference package every session
is unacceptable. Decision shape: pre-parse on import, write a flat row
per process to `processes_imported`, keep the bundle on disk for
re-resolution if needed.

**Open question (process catalogue scope):** are imported databases
org-scoped or system-shared? `D-00xx` candidate at week 11. Default
assumption: org-scoped (consultancy A's USDA import is independent of
consultancy B's), with system-pre-loaded databases (the three Phase 1
free DBs) shared.

---

## 4. Imanol involvement schedule

Phase 2 has three Imanol-shaped checkpoints baked into the Execution
Guide. The
[`feedback_imanol_arko_session_prep`](https://example.invalid)
discipline (raw terminal, written questions, no scripted demo) extends
to UI sessions: show the screen with whatever real data is loaded,
let him use it, write down friction verbatim.

| Week | Format | Question to write down beforehand |
|---|---|---|
| 14 | 30 min, Process Browser | "Find a process you'd typically use. Anything missing?" |
| 17 | 30 min, Study Builder pairing | "Build a small study with me; note every friction." |
| 24 | 90 min, end-to-end + EPD | "Build a small real study from scratch; generate an EPD; compare two variants." Closing: "1–10, how close is this to something you'd actually use?" |

The week-24 score is the Phase 2 → Phase 3 hinge. Under 6 means
Phase 3 includes a significant UI rework before EPD-template
multiplication starts.

---

## 5. Phase 2 week 1 (week 11) — concrete starting work

The first week of Phase 2 is architecture-lock and scaffold, not
feature work. Discharging this list closes the §2 deferred decisions
and produces a Phase 2 work surface.

### Day 1–2: spikes that gate architecture

1. **WASM compile spike** — `cargo build --target
   wasm32-unknown-unknown -p arko-engine`. Result determines §2.5 and
   §2.6.
2. **Compartment parity check** — write the §2.4 test against a
   substance present in both ILCD and openLCA fixtures; fix reader if
   needed. Output: a passing test, or a `D-00xx` deferral with reasoning.

### Day 2–3: architecture decisions land as `D-00xx` entries

3. **Auth decision** (`D-00xx`) — Supabase or Keycloak; trigger
   conditions for migration if Supabase.
4. **Hosting decision** (`D-00xx`) — Vercel or self-hosted from day 1.
5. **API server decision** (`D-00xx`) — day 1 or week 18.
6. **Design system divergence** (`D-00xx`) — which KarbonGarbi tokens
   carry over, which Arko diverges on.

### `D-0020` follow-on decisions — landed at boundary (2026-04-23)

Filed before week 11 to unblock scaffold work:

7. **Timeline shape A** — stretch Phase 2 to ~17–20 weeks; ship all
   five screens. [`D-0021`](../DECISIONS.md#d-0021).
8. **Tier feature gating** — Studio/Team/Enterprise three-axis split
   (seats × studies × EPDs/month + DB tier). [`D-0022`](../DECISIONS.md#d-0022).
9. **i18n library: `next-intl`** — Next.js 15 native, App Router-aware,
   JSON bundles per locale. [`D-0023`](../DECISIONS.md#d-0023).

**Still open at week 11 (need user / Imanol input before locking):**

10. **Default locale** — Spanish-first (Basque-market positioning) or
    English-first (engine-narrative consistency). Filed in `D-0023`
    open items; defer to week 11 marketing-vs-product framing call.

**Landed since boundary memo first draft:**

- Tier pricing: Studio €4,900/yr, Team €13,900/yr (3–5 seats),
  Enterprise contact-us. Annual-only at launch. [`D-0024`](../DECISIONS.md#d-0024).

### Day 3–5: scaffold

11. **Repo creation** — `goibix/arko-app` (Next.js 15 + TS + Tailwind +
    shadcn/ui) per the §2.3 ratified design system.
12. **Auth + org model wired** per §2.2 decision; users land in a
    default org on signup, role = `owner`.
13. **Database scaffolded** — Postgres schema for §3 tables (account
    layer + study layer), migrations in repo. Billing tables included
    even if Redsys integration isn't wired yet.
14. **i18n scaffold** — `en` and `es` locale files in repo from day 1;
    every screen built henceforth ships both locales (no English-only
    debt to retrofit later).
15. **Redsys sandbox account** — request from BBVA / partner bank in
    week 11 (provisioning lead time); integration code lands in the
    week the billing UI does, but the credentials need to exist.
16. **WASM integration** — engine compiled, loaded by a smoke page that
    runs the carpet calc in-browser and shows the result. Closes the
    §2.5 spike.
17. **Deploy pipeline** — first deployment to chosen target (Vercel or
    self-hosted), even if it's a "Hello, Arko" page.

### Day 5: Imanol scope confirmation (optional but encouraged)

18. **20-minute call with Imanol** — show him the 5-screen scope and
    the order; ask which screen he most wants to see first. The
    Execution Guide locks order (Process Browser → Study Builder → …),
    but his answer informs which features within each screen are
    must-have vs nice-to-have. Same prep discipline as the Phase 1
    exit session.

---

## 6. What Phase 2 explicitly does *not* include

Per the Execution Guide, Tech Spec v2.0, and the
[product spec's "What Phase 2 does *not* include"](phase-2-product-spec.md#what-phase-2-does-not-include):

*Engine/calc layer:*

- **Monte Carlo / sensitivity UIs** — Phase 4 (engine already supports
  them via `FactoredSolver`; UIs are in
  [`arko-execution-guide.md:359`](arko-execution-guide.md#L359))
- **ecoinvent integration** — Phase 4+ (license-access infrastructure
  is Phase 4 work; the V1 hook in `engine/core/src/license.rs` is the
  carrier)
- **License tier UI propagation** — Phase 3 (the `license_tier` field
  exists on `ProcessMeta` in v0.2.0; Phase 3 surfaces it in the UI)

*Output layer:*

- **Multiple EPD templates** — Phase 3 (Phase 2 ships Environdec only)
- **Verifier workflow** (lock + audit + signature) — Phase 3
  (`studies_versions` table ships in Phase 2 to avoid migration churn)
- **PEF report generator** — Phase 3
- **Automated EPD submission** — out (Phase 2 generates LCAx + PDF;
  user submits to operator manually)

*Account / billing layer (D-0020 carve-outs — what's still NOT in Phase 2):*

- **Admin panel / super-admin** — Phase 4 (operationally ok to manage
  via SQL or Supabase Studio in Phase 2)
- **SSO / SAML / Active Directory** — Phase 4 (email-password +
  password reset only in Phase 2)
- **Locales beyond EN + ES** — V2+ (per spec: "EN + ES only" is V1
  boundary, not zero-i18n)
- **Self-serve refund / dunning automation** — Phase 4 (manual handling
  via Redsys merchant console is fine for Phase 2's customer count)

*Infra / GTM:*

- **CloudNativePG migration** — Phase 4
- **Documentation site (`docs.arko.earth`)** — Phase 4
- **Marketing landing (`arko.earth`)** — Phase 4 *(per Arko GTM
  early-start memory: domains/handles/blog work happens on weekend
  writing time, not Phase 2 dev time)*

---

## 7. Phase 1 → Phase 2 hand-off checklist

| Item | Status at v0.2.0 |
|---|---|
| Engine API stable enough to compile to WASM | Untested; week-11 spike |
| `MethodRegistry::standard()` covers EN 15804+A2 core for first EPD render | ✅ (EF 3.1 + CML-IA + ReCiPe + AR6) |
| Reader compartment shapes consistent across ILCD and openLCA | Untested; week-11 spike |
| `FactoredSolver` available for the in-browser sensitivity-sweep path | ✅ (`fe30079`) |
| LCAx writer available for one of two Phase-2 EPD output paths | ✅ (`6cc5c03`) |
| ILCD+EPD writer available for the program-operator submission path | ❌ (V2, `D-0018`) — Phase 2 ships Environdec via LCAx + custom Word/PDF templating |
| Determinism contract intact (BLAKE3 canonical hash, seeded MC) | ✅ (per v0.0.1 release notes; Phase 1 didn't perturb) |
| Real-data parity smokes green | ✅ (carpet bit-exact, beef ulp-scale) |
| At least one preset registered for every category the Phase-2 EPD template needs | ✅ (CC, OD, POCP, AC, EU-fw, EU-m, EU-t covered by EF 3.1) |
| A user can go import → study → calc → result with current engine + CLI | ✅ (week-10 integration deliverable per guide line 145) |

*Account / billing layer (added per `D-0020`):*

| Item | Status at v0.2.0 |
|---|---|
| Auth provider chosen (Supabase vs Keycloak) | ❌ — week-11 `D-00xx` |
| Org/role schema designed | ✅ (§3 above) |
| Redsys sandbox credentials provisioned | ❌ — week-11 ask (BBVA lead time) |
| Tier feature gating decided | ✅ ([`D-0022`](../DECISIONS.md#d-0022)) |
| Tier prices decided | ✅ ([`D-0024`](../DECISIONS.md#d-0024): Studio €4.9k/yr, Team €13.9k/yr) |
| i18n library chosen | ✅ ([`D-0023`](../DECISIONS.md#d-0023): `next-intl`) |
| ES translator workflow set up | ❌ — week-11 ask (Imanol-network candidate?) |

Two engine-side items become week-11 spikes. Account/billing layer is
all greenfield — week 11 land architecture decisions, week 12+ build.

---

## 8. Phase 2 → Phase 3 hinge

Phase 3 ("EPD/PEF Production-Ready", weeks 25–34 under timeline shape A,
shifted under shape B per `D-0020`) starts when:

- Phase 2 screens exit-criterion green — **5 screens under shape A**, or
  **4 screens (Scenario Comparison deferred to Phase 3)** under shape B
- Imanol's week-24 (or shifted) score ≥ 6 (else: Phase 3 starts with
  UI rework)
- One Environdec EPD template renders end-to-end
- Engine WASM build is part of CI
- Postgres schema covers studies + versions + runs (verifier workflow
  in Phase 3 builds on `studies_versions`)
- **Billing live** — at least one paying org through the Redsys
  sandbox→production cutover; trial-to-paid conversion path exercised
- **Org model live** — at least one multi-user org with a non-owner
  member (editor or viewer) using a real study
- **EN + ES UI parity** — every Phase 2 screen ships both locales

Phase 3 entry conditions land in the Phase 2 closeout doc when v0.3.0
tags.
