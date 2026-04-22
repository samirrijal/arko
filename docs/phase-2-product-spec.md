# Phase 2 Product Spec — Web UI MVP

**Drafted:** 2026-04-22
**Status:** Canonical scope source for Phase 2
**Sibling docs:**
[`phase-2-boundary-memo.md`](phase-2-boundary-memo.md) (architectural decisions, week-1 spikes, hand-off),
[`phase-1-closeout.md`](phase-1-closeout.md) (engine state at Phase 2 entry),
[`arko-execution-guide.md`](arko-execution-guide.md) (canonical roadmap; Phase 2 scope here supersedes the guide's `## Phase 2` section where they conflict)

This document defines *what* Phase 2 builds. The boundary memo defines
*how* it gets built (architecture, decisions, week-1 surface). Where
this spec expands beyond the Execution Guide (billing, orgs, i18n
pulled forward from Phase 4), the rationale lives in
[`DECISIONS.md` `D-0020`](../DECISIONS.md).

---

Five screens plus EPD output. That's the Phase 2 product. Here's what each one actually contains.

## Process Browser

**Purpose:** Import LCA databases, search across them, select processes to use in a study.

**Core UI:**

- Import area — drag-drop ILCD zip bundles or openLCA JSON-LD archives, or connect to a pre-loaded database (USDA LCA Commons, ÖKOBAUDAT, JRC EF reference)
- Search bar with instant filtering — by process name, by geography, by reference product
- Filter sidebar — by database source, by geography (country/region), by ISIC/NAICS sector, by year
- Results list — process name, reference product, geography, database source, date
- Process detail pane — opens when a process is clicked, shows: reference flow, inputs (with scrollable list), outputs, metadata (geography, time period, technology description), link back to source database
- "Add to study" button on each process

**What it's not:** A materials database you browse like a catalog. It's a search-and-select interface over databases the user imports or accesses.

## Study Builder

**Purpose:** Define what you're calculating — the goal, the scope, the functional unit, which processes contribute.

**Core UI:**

- Study metadata header — name, client, date, status (draft/complete), linked program operator (Environdec etc.)
- Goal & scope section — declared unit (e.g., "1 kg of cement"), reference service life, system boundary description (text), geography, time period
- Foreground system — the processes the user is directly modeling. Table with: process name, amount, unit, source (imported or custom), edit/delete actions. "Add process" button opens the Process Browser in modal mode.
- Background system — processes drawn from databases. Similar table structure, distinguished visually from foreground.
- Parameters section — user-defined parameters with values and descriptions, used for scenario analysis later
- LCIA method selector — dropdown: AR6 GWP100, EF 3.1 V1, CML-IA baseline v4.8, ReCiPe 2016 Midpoint H. Default: EF 3.1 V1 (most relevant for EPD output)
- Save/calculate actions in sticky footer

**What it's not:** A 3D modeling tool or a drawing interface. It's a structured form for specifying an LCA study.

## Calculation Runner

**Purpose:** Execute the calculation, show progress, display results.

**Core UI:**

- Pre-flight check — validates the study has all required fields before calculation (declared unit set, at least one process, method selected). Warnings shown inline.
- Run button — triggers calculation via API call to Arko engine
- Progress indicator — calculation time in seconds, with status ("factoring matrix," "computing inventory," "applying impact factors")
- Results display — impact categories as cards/rows: GWP100 in kg CO2-eq, ODP in kg CFC-11-eq, etc. Each card shows the value, the unit, the method, and a sparkline or small visualization hint
- Methodology transparency — expandable "How was this calculated?" section showing: method version, characterization factors source file, process count, elementary flow count
- Actions — "View contribution analysis," "Compare to another study," "Generate EPD," "Export raw results (JSON/CSV)"

**What it's not:** A dashboard with dozens of competing widgets. It's focused on getting from "calculate" to "results I can trust and understand."

## Contribution Analysis

**Purpose:** Understand which processes and flows drive the impact — the answer to "why is this number what it is."

**Core UI:**

- Header with impact category selector (switch between GWP100, AP, EP, etc.) and total value
- Sankey diagram — primary visualization. Flows from functional unit through processes to elementary flows, with width proportional to contribution. Click a band to filter everything else.
- Top contributors table — alongside or below the Sankey. Process name, contribution value, contribution percentage, link to drill into that process
- Drilldown panel — when a contributor is clicked, shows: the specific flows contributing, the CFs applied, the upstream processes driving it
- Impact category tabs — let user switch between categories without losing drilldown state
- Export options — PNG/SVG of current view, CSV of contribution data

**What it's not:** A general-purpose visualization tool. It's specifically about explaining "where does this impact come from" for one study at a time.

## Scenario Comparison

**Purpose:** Pin two (or more) studies side-by-side to see how changes affect impact.

**Core UI:**

- Study picker — select studies to compare (up to 3 in V1, with "add another" disabled after 3)
- Comparison table — impact categories as rows, studies as columns. Cells show the impact value with percentage diff from baseline. Color-coded (red for worse, green for better)
- Synchronized detail view — when a category is clicked, all studies expand to show contribution breakdowns side by side
- Difference drill-down — which specific processes differ between scenarios, and by how much
- Scenario metadata — shows what's different between scenarios: different processes used, different parameters, different databases
- Export — comparison table as CSV/PDF for client reports

**What it's not:** A Git-like diff tool with branching and merging. It's a practical "here's study A vs study B, what changed" view.

## EPD Output

**Purpose:** Generate an EPD document from a completed study.

**Core UI:**

- Prerequisites check — study must be complete, include required metadata (verifier, PCR reference, declared unit, reference service life, etc.)
- EPD metadata form — fills in EPD-specific fields the study doesn't carry: publication date, valid until, EPD version, subtype, PCR identifier. Reasonable defaults where possible.
- Program operator selection — dropdown with Environdec as default, other operators listed as "coming soon" or available if implemented
- Preview — shows the EPD document as it will be exported (LCAx structure rendered as readable HTML)
- Export — LCAx JSON file (ready for operator submission), plus human-readable PDF version
- Submission guidance — text explaining how to submit to the selected operator (Arko doesn't submit directly in V1)

**What it's not:** An automatic EPD publisher. It generates the document; the user submits it to the operator.

## The shared infrastructure

Beyond the five screens plus EPD, you need invisible infrastructure:

**Authentication** — Supabase Auth, email-based login, password reset. Organization/workspace concept so consultancies can have multiple users.

**Billing** — Redsys integration *(see `D-0020` and [`feedback_redsys_billing`](../../C--Users-hical-Desktop-karbongarbi/memory/feedback_redsys_billing.md) — Redsys for Spanish/Basque market; Arko initial market = Basque via Imanol's network; if non-Spanish customer base materializes, that's a future `D-00xx` revisit)*, three tiers (Studio, Team, Enterprise), monthly/annual subscription management, free 14-day trial for new signups. In-app plan management (Redsys has no hosted customer portal, unlike Stripe).

**Organization/team management** — users belong to organizations. Organizations have projects. Projects contain studies. Basic role management (owner, editor, viewer).

**Project structure** — projects are folders that contain multiple studies. A consultancy working with a client creates a project for that client, studies for individual products/buildings/scenarios within the project.

**Navigation** — sidebar with Projects (list), Databases (imported sources), Studies (across all projects), Account. Breadcrumbs everywhere because LCA workflows are nested.

**Data persistence** — studies save automatically as users edit. Version history (V2, not V1) so users can revert.

**Basic help** — tooltips on LCA-specific terms (ecoinvent newcomers may not know "functional unit" means). Link to documentation.

## What Phase 2 does *not* include

Being explicit about this because feature-creep is the risk:

- No 3D modeling, no building design, no geometry input
- No materials database Arko maintains (users bring their own databases)
- No in-app academy, bootcamp, training videos
- No Monte Carlo simulation (Phase 3 uses FactoredSolver)
- No sensitivity analysis UI (Phase 3)
- No custom LCIA method builder (users pick from the registered presets)
- No report-generator beyond EPD output (PDF reports with charts and narratives = Phase 3)
- No ecoinvent integration (Phase 4)
- No multi-language UI beyond English + Spanish (V1 ships EN + ES only; more later)
- No mobile app (web responsive only)
- No real-time collaboration beyond basic shared editing (Phase 3)
- No AI-generated LCA recommendations (not planned; against Arko's expert-elevation positioning)

## The minimum working flow

If someone asked "what's the smallest thing Arko does," the answer is this flow:

1. User signs up, creates organization, creates project
2. User imports an ILCD bundle or selects a pre-loaded database
3. User creates a study, defines declared unit, adds processes from the database
4. User runs calculation, sees impact results
5. User views contribution analysis to understand what drives impact
6. User generates an EPD document, exports LCAx, submits to Environdec

That flow — end to end — is Phase 2's deliverable. Every screen and piece of infrastructure serves that flow. Anything that doesn't serve that flow is out of scope for Phase 2.
