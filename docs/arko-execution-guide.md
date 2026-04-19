# Arko — The Execution Guide

**From current state (engine 70% complete, UI not started) to first paying customer.**

Written for Samir, solo founder at Goibix S.L., Bilbao. Working evenings and weekends alongside a full-time day job and KarbonGarbi.

---

## How to use this document

Open it Monday morning. Find the current week. Do the tasks. Cross them off. Look at next week briefly to prepare. Don't read ahead to month 10 — it'll only make you anxious and the details will be wrong by the time you get there.

Every phase has:
- A clear exit criterion (you're done with Phase 1 when X is true)
- Weekly task breakdown
- What to show Imanol (and what not to)
- Red flags (things that mean "stop and reassess")

---

## The master timeline

| Phase | Weeks | Calendar | Outcome |
|---|---|---|---|
| Phase 0 | 1-2 | ~May 2026 | Engine tagged v0.0.1, CI green, license chosen |
| Phase 1 | 3-10 | ~May-July 2026 | Engine breadth complete (ILCD linker, EPDX, methods, databases) |
| Phase 2 | 11-24 | ~Jul-Oct 2026 | Web UI MVP + first EPD renderer working end-to-end |
| Phase 3 | 25-34 | ~Oct-Dec 2026 | EPD/PEF generation production-ready, verifier workflow |
| Phase 4 | 35-48 | ~Jan-Apr 2027 | Polish, docs, Monte Carlo UI, sensitivity UI |
| Phase 5 | 49-60 | ~Apr-Jul 2027 | First design-partner conversations, first paying customer |

**Total: ~14 months from today to first paid customer.** This is realistic for solo + evenings/weekends + KarbonGarbi on the side. If you're faster, great. If you're slower, adjust without panicking.

---

## Non-negotiable discipline commitments

Write these down. Sign them. Put them somewhere you'll see them.

1. **KarbonGarbi keeps shipping.** If by September 30, 2026, KarbonGarbi has fewer than 2 paying customers, Arko development pauses completely until KarbonGarbi has 3. This is the pause trigger. Date it, commit it.

2. **Time-box weekly, not daily.** Arko work Monday-Wednesday evenings. KarbonGarbi work Thursday-Friday evenings. Weekends flex based on momentum. Do not oscillate within a day.

3. **Imanol time is sacred.** Ask for feedback in batches, not in drips. Prepare specific questions and specific artifacts. Never "hey quick question" — always "I've prepared three options for X, when you have 20 minutes can you react?" Respect his time and he'll stay engaged.

4. **No scope expansion without deletion.** Before adding a feature, find one to cut. The spec is the spec.

5. **Weekly review, every Sunday, 30 minutes.** Did I do this week's tasks? What blocked me? What's next week's? Write it down. Review monthly.

---

## Phase 0: Foundations (Weeks 1-2)

**Exit criteria:** 
- `cargo test --workspace` passes on Linux CI
- `arko-engine v0.0.1` tagged on GitHub
- LICENSE file committed
- Repository structure documented in README

### Week 1

**Monday-Wednesday evenings:**
- [ ] Install Rust toolchain on Windows development machine (`rustup`)
- [ ] Clone the engine repo, run `cargo build --workspace`
- [ ] Fix faer 0.20 type-name changes — most likely `Mat<f64>` → `Mat::<f64>` or similar; check faer changelog
- [ ] Run `cargo test --workspace`; fix any test breakages
- [ ] Document every fix as a commit with a clear message

**Thursday-Friday evenings:** KarbonGarbi work only. Follow up with Imanol on the KarbonGarbi pilot.

**Weekend:**
- [ ] Decide licensing. My recommendation: Apache-2.0 for `arko-engine`, commercial (all rights reserved) for `arko-app` (future UI repo)
- [ ] Add `LICENSE` and `NOTICE` files to engine repo
- [ ] Write a one-paragraph README explaining what arko-engine is and what status it's at

### Week 2

**Monday-Wednesday evenings:**
- [ ] Set up GitHub Actions CI with a Linux runner:
  - `cargo fmt --check`
  - `cargo clippy -- -D warnings`
  - `cargo test --workspace`
  - Run on every push to main and every PR
- [ ] Add a CI badge to the README
- [ ] Tag `arko-engine v0.0.1` once green
- [ ] Publish the repo (private on GitHub under `goibix/arko-engine`, visibility stays private for now)

**Thursday-Friday evenings:** KarbonGarbi.

**Weekend:**
- [ ] Set up a separate `goibix/arko-specs` private repo. Commit the Master Spec v1.0 and Tech Spec v2.0 from earlier sessions. Add a `DECISIONS.md` that lists the pause trigger and the license call.
- [ ] Write one short personal note to Imanol: "Hey, in a few months I might show you something technical I've been working on outside of KarbonGarbi. Not urgent, just giving you a heads up that your feedback will be valuable when the time comes." This does two things: warms him up for the future ask, and signals that Arko is separate from your KarbonGarbi relationship.

### Red flags in Phase 0

- Cargo test failures that take more than 3 evenings to fix → the faer migration is deeper than expected. Consider pinning to faer 0.19 temporarily and budgeting 2 more weeks to migrate at Phase 1.
- Windows-specific build issues → just use WSL2 with Ubuntu. Don't fight it.

---

## Phase 1: Engine Breadth (Weeks 3-10)

**Exit criteria:**
- ILCD bundles (multi-file) can be loaded into Arko Study
- EPDX read/write works (at least the core schema)
- OpenLCA JSON-LD import works
- Four method presets available: IPCC AR6 GWP100, ReCiPe 2016 Midpoint, EF 3.1, CML 2001
- At least three free databases importable: ÖKOBAUDAT, Agribalyse, EF reference packages
- FactoredSolver trait implemented
- All above covered by unit tests

### Weekly breakdown

**Week 3-4: ILCD linker (`arko-io-ilcd-linker`)**

The critical unblocker. Without this, none of the free EU databases work.

- [ ] Read the ILCD specification carefully (it's available from EU JRC); note the bundle file structure (processes, flows, unit groups, sources, contacts all in separate XML files with UUID refs)
- [ ] Design the linker to resolve refs lazily — don't force-load the entire bundle
- [ ] Write the `LinkResolver` trait; implement for ILCD bundles
- [ ] Add tests against a small synthetic ILCD bundle you construct manually
- [ ] Add tests against a real ÖKOBAUDAT export (free download)

**Week 5-6: EPDX and OpenLCA JSON-LD importers**

- [ ] EPDX — the standard digital-EPD format; importing means you can ingest competitor EPDs as reference
- [ ] OpenLCA JSON-LD — importing means OpenLCA users can migrate to Arko later
- [ ] Both go through the same `StudyImporter` trait you already have for ILCD

**Week 7-8: Method presets**

- [ ] ReCiPe 2016 Midpoint (18 categories) — data from the RIVM publication
- [ ] EF 3.1 (16 categories) — data from EU JRC, free
- [ ] CML 2001 — still widely used by older studies; keep for compatibility
- [ ] Structure: each method is a YAML or JSON file with characterization factors, checked into `arko-methods/methods/<method-id>/v<version>/`
- [ ] Version pinning: studies reference methods by `(method_id, version)` tuple

**Week 9: FactoredSolver + database packaging**

- [ ] `FactoredSolver` trait: cache the LU factorization of A across sensitivity iterations. Big perf win for Monte Carlo.
- [ ] Package at least one database as importable bundle. Propose: Agribalyse first (cleanest data, French agriculture, well-documented)

**Week 10: Integration week + first Imanol show-and-tell**

- [ ] Wire all imports into a single CLI workflow: `arko import agribalyse.zip && arko study new && arko calc --method recipe2016`
- [ ] Generate a text-based result report (no UI yet, just terminal output)
- [ ] **Imanol session (30-45 minutes):** Show him the terminal workflow. Ask him three specific questions:
  1. "Does this data look right to you? Are the numbers in the ballpark you'd expect?"
  2. "What are the 3-5 things you do most often when you open SimaPro?"
  3. "If you could wave a magic wand and change one thing about SimaPro's UI, what would it be?"
- [ ] Write down his answers verbatim. These become the north star for Phase 2.

### What to show Imanol, what not to

**Show:**
- Terminal output with a real calculation
- A printed-out table of results
- Two or three methods side-by-side on the same study
- An imported ÖKOBAUDAT or Agribalyse process

**Don't show:**
- Code
- Architecture diagrams
- Tech stack slides
- Cost projections
- Anything that'll make him feel like he's being recruited

Imanol is a domain advisor, not an investor. Keep it domain-flavored.

### Red flags in Phase 1

- ILCD linker taking >3 weeks → the spec is genuinely gnarly; consider finding an existing open-source ILCD parser to adapt
- Method data is wrong somewhere and you can't find the discrepancy → this is why you needed domain review early. Ask Imanol to sanity-check specific values
- You find yourself building UI mockups "just to think through it" → resist. Phase 1 is engine-only. Write the UI specs as words in a markdown file, not Figma frames.

---

## Phase 2: Web UI MVP (Weeks 11-24)

**Exit criteria:**
- Three core screens functional: Process Browser, Study Builder, Calculation Runner
- Contribution analysis view with Sankey diagram working
- Scenario comparison view working
- First EPD renderer produces a valid Environdec-style PDF from an Arko study
- Entire flow is usable end-to-end: import data → build study → run calculation → see contribution → export EPD
- Imanol has used it for one real small study and given feedback

### Architecture reminder

- Next.js 15 + React 19 + Tailwind (matches KarbonGarbi, reuse design system)
- Arko engine compiled to WASM runs in the browser for small studies
- Larger studies hit a Rust API server (Axum) that wraps the same engine
- PostgreSQL for persistence (CloudNativePG via Supabase for V1 pragmatism; migrate to self-hosted when you have revenue)
- Start with Supabase-as-Postgres (not Supabase-as-backend) to keep sovereignty path open

**Pragmatic compromise from the Tech Spec v2.0:** for Phase 2, use Supabase as managed Postgres only. This saves you 30-50 hours of DevOps time during the UI push. Migrate to self-hosted CloudNativePG during Phase 4 when you have breathing room.

### Week 11-12: Project setup and design system

- [ ] Create `goibix/arko-app` repo (Next.js + TypeScript + Tailwind)
- [ ] Copy the design tokens from KarbonGarbi (colors, spacing, typography)
- [ ] Set up shadcn/ui components
- [ ] Set up authentication via Supabase Auth (matches KarbonGarbi pattern; we'll migrate to Keycloak later)
- [ ] Create the basic app shell: sidebar, topbar, workspace switcher
- [ ] Deploy to Vercel for now (sovereignty compromise for development speed; self-host in Phase 4)
- [ ] Compile arko-engine to WASM, integrate into the app

### Week 13-14: Process Browser

This is screen 1 of 5. It's the user's entry point.

- [ ] List view of imported processes with filters: database, geography, year, flow name
- [ ] Full-text search via Postgres
- [ ] Detail panel shows: reference product, inputs/outputs table, pedigree info, source citation
- [ ] "Add to study" button
- [ ] Persist user's recently-viewed list

**Imanol check at end of week 14:** "Can you browse the ÖKOBAUDAT processes? Find one you'd typically use. Anything missing from this view?"

### Week 15-17: Study Builder

This is screen 2 of 5. Where most of the work happens.

- [ ] Create a new study with goal & scope (functional unit, system boundary, geographic scope, time horizon)
- [ ] Add processes from the library
- [ ] Define parameters (numeric values that can drive multiple inputs)
- [ ] Tree view: study → variant → lifecycle stage → model → flows
- [ ] Canvas view (ReactFlow): same data as a visual graph
- [ ] Table view (TanStack Table): same data as a spreadsheet
- [ ] All three views sync in real-time
- [ ] Save/load studies to/from Postgres

**Imanol check at end of week 17:** Give him a 30-minute walkthrough. Ask him to build a small study with you pairing. Note every friction point.

### Week 18-19: Calculation Runner

This is screen 3 of 5.

- [ ] Pick method (dropdown of registered methods with versions)
- [ ] "Run Calculation" button
- [ ] For small studies: run in-browser via WASM, results in <100ms
- [ ] For larger studies: API call to Axum server
- [ ] Results table: impact category, unit, total, per-stage breakdown
- [ ] Download results as JSON/CSV
- [ ] Display the signed manifest (hash of inputs + engine version) for reproducibility

### Week 20-21: Contribution analysis view

This is screen 4 of 5. This is the screen consultants spend 80% of their time on.

- [ ] Pick an impact category (e.g., Climate Change)
- [ ] Tree table: each process contributing to the impact, % of total, drill-down
- [ ] Sankey diagram: visual flow from root to leaves
- [ ] Hotspot highlighting: top 3 contributors emphasized
- [ ] Filter: hide contributions below X%
- [ ] Export as PNG or SVG

**Technical note on the Sankey:** don't try to build it from scratch. Use `visx` Sankey or `@nivo/sankey`. Customize styling. The interactivity is where your time goes.

### Week 22: Scenario comparison

This is screen 5 of 5. The killer feature SimaPro does poorly.

- [ ] Select 2-3 variants from a study (or across studies)
- [ ] Side-by-side results table
- [ ] Bar chart showing each impact category, colored by variant
- [ ] "Difference" column: % change from baseline
- [ ] Statistical significance indicator if Monte Carlo data is available (V2 feature, skip if tight)

### Week 23-24: First EPD renderer end-to-end

This runs **in parallel with Phase 2** (started in week 13, intensifying now). You should have had one EPD renderer prototype working since week 15-16.

By week 24:
- [ ] One Environdec-style EPD template working end-to-end
- [ ] Takes an Arko study + calculated results → produces a valid PDF
- [ ] Uses `docx-js` for Word version, Puppeteer for PDF
- [ ] Imanol reviews the output against a real EPD he's seen before; you fix gaps

**Imanol session at end of Phase 2 (week 24):**
- Give him 90 minutes
- Let him build a small real study from scratch (maybe a simple packaging product or a concrete mix — something he knows)
- Let him generate an EPD
- Let him compare two variants
- Record his screen if he'll allow (you watch for friction; he doesn't need to narrate)
- Ask at the end: "On a scale of 1-10, how close is this to being something you'd actually use?" Under 6 means Phase 3 includes a significant UI rework.

### Red flags in Phase 2

- Week 15 and Study Builder isn't started → you're behind; cut scope, not quality. Drop the Sankey to Phase 3.
- Imanol's feedback at week 17 includes "this is confusing" more than twice → pause, rework the information architecture before continuing
- You find yourself implementing features Imanol didn't ask for → stop. The spec is the spec. Add them to a backlog file; revisit at Phase 4.
- KarbonGarbi customer work is getting deprioritized for more than 2 weeks → the pause trigger gets closer. Reset time-boxing.

---

## Phase 3: EPD/PEF Production-Ready (Weeks 25-34)

**Exit criteria:**
- Three EPD templates working: Environdec (international), IBU (Germany), EPD Norge (Nordic)
- PEF report generator functional (all 16 EF categories)
- Verifier review workflow: lock results, audit trail, digital signature
- License tier propagation working (studies with ecoinvent data get flagged; studies with only open data don't)
- All EPD outputs validated by Imanol against real examples he's seen

### Week 25-27: Second and third EPD templates

- [ ] IBU template (German construction products; common in Basque industrial context)
- [ ] EPD Norge template (Nordic; broader European applicability)
- [ ] Each template has its own test suite: given known inputs, produce expected output
- [ ] Factor out common rendering primitives into `arko-doc-primitives` library

### Week 28-29: PEF report generator

- [ ] EU Product Environmental Footprint methodology
- [ ] All 16 EF 3.1 impact categories
- [ ] PEF-specific rules: normalization, weighting, single score
- [ ] Template matches EU's published requirements

### Week 30-31: Verifier workflow

- [ ] Lock study state: once locked, no edits allowed without creating a new version
- [ ] Audit trail: every state change logged with user, timestamp, reason
- [ ] Digital signature on the final report (Ed25519, included in PDF metadata)
- [ ] Export signed manifest separately for third-party verification

### Week 32: License tier propagation

- [ ] Every flow carries a `license_tier` field (from Tech Spec v2.0)
- [ ] Studies' effective license = union of all flow licenses
- [ ] UI indicators: green (open), yellow (restricted), red (proprietary)
- [ ] EPD generation checks license tier; adds watermarks if restricted data present
- [ ] For now, all databases we ship are open; ecoinvent integration is Phase 4+

### Week 33-34: Integration testing and Imanol review

- [ ] End-to-end test: import ÖKOBAUDAT → build a small concrete-product study → run EF 3.1 → generate Environdec + IBU EPDs → lock and sign
- [ ] **Imanol full session (2 hours):** Hand him the tool; let him produce an EPD he would stand behind professionally. Fix anything he flags as "wouldn't pass verification."

### Red flags in Phase 3

- Imanol flags something in the EPD as "wrong" and you don't understand why → this is exactly what you need him for. Do not ship until you understand the fix. Pay for an hour of a proper LCA consultant's time if needed.
- PEF methodology turns out to have subtleties the spec didn't cover → standard; every PEF implementer hits this. Lean on JRC documentation; ask in PEF practitioner forums; ask Imanol.
- You want to add ecoinvent integration "just for testing" → don't. V1 ships with open data only. Ecoinvent is V2.

---

## Phase 4: Polish + Full MVP (Weeks 35-48)

**Exit criteria:**
- Monte Carlo UI working (upload study, set iterations, get uncertainty bands)
- Sensitivity analysis UI working (which parameters drive which outputs)
- Documentation site live at docs.arko.earth
- Landing page live at arko.earth (marketing site)
- Migration from Supabase to self-hosted CloudNativePG complete
- All of KarbonGarbi's hard-won infrastructure patterns applied: GDPR, legal pages, billing, admin panel
- Ready for first design-partner conversations

### Week 35-37: Monte Carlo and sensitivity UIs

Your engine already supports these. Just expose them in the UI.

- [ ] Monte Carlo run dialog: number of iterations (default 1000), distributions to use, seed for reproducibility
- [ ] Results show median, 2.5/97.5 percentiles, histogram per impact category
- [ ] Sensitivity analysis: bar chart of parameters ranked by influence
- [ ] Both runs are expensive; run on server, show progress via WebSocket

### Week 38-40: Documentation

Don't skip this. LCA is regulated; practitioners check documentation before they trust a tool.

- [ ] Docusaurus or similar static site at `docs.arko.earth`
- [ ] "Getting Started" tutorial: install, import data, build first study, generate EPD
- [ ] Method descriptions: what does ReCiPe measure, what does EF cover
- [ ] API reference (auto-generated from OpenAPI)
- [ ] FAQ: how does Arko compare to SimaPro / OpenLCA
- [ ] Verification guide: here's how you'd verify Arko's calculations

### Week 41-43: Marketing site and brand

- [ ] Landing page at `arko.earth`: clear value prop, three hero screens, CTA to waitlist
- [ ] Pricing page: Free / Studio €149 / Team €349 / Enterprise Contact Us
- [ ] "Compare to SimaPro" page: honest feature comparison
- [ ] "Open math spec" page: showcase the published calculation specification (your moat)
- [ ] Blog: 3 seed posts (e.g., "Why we built our own LCA engine," "Understanding EF 3.1 in 10 minutes," "Case study: EPD for Basque cement manufacturer")

### Week 44-46: Infrastructure hardening

Migrate from development infrastructure to production-grade.

- [ ] Move Postgres from Supabase to self-hosted CloudNativePG on Hetzner (see Tech Spec v2.0 §14.1)
- [ ] Deploy all services to your own Kubernetes cluster (Talos + k0s, per Tech Spec)
- [ ] Set up observability stack (VictoriaMetrics, Loki, Grafana)
- [ ] Set up Keycloak for auth, migrate user accounts from Supabase Auth
- [ ] Enable backups: continuous WAL to MinIO, nightly base backups, weekly cross-provider
- [ ] Test disaster recovery: intentionally lose a node, verify recovery

**Pragmatic note:** if infrastructure migration threatens the launch timeline, delay it to Phase 5. Self-hosted infrastructure is not a day-one customer requirement; it's a Year-2 sovereignty commitment. Ship on Supabase if you must.

### Week 47-48: Legal and compliance

- [ ] Terms of service (reuse KarbonGarbi's legal pages as base)
- [ ] Privacy policy
- [ ] Data Processing Agreement (DPA) template for Enterprise customers
- [ ] GDPR compliance review
- [ ] Cookie banner (honest minimal one, not dark-pattern)
- [ ] Impressum / legal entity disclosure

### Red flags in Phase 4

- You're 3+ weeks behind and tempted to skip documentation → don't. Ship a minimal-but-complete docs site; expand later. Zero docs = zero trust.
- Infrastructure migration is eating weeks → stop, ship on Supabase, migrate after first paying customer
- You're tempted to add "one more feature" before launching → refer to discipline commitment #4

---

## Phase 5: First Paying Customer (Weeks 49-60)

**Exit criteria:**
- 3-5 design-partner conversations happened
- 1 design partner active (free or discounted in exchange for real-world validation + testimonial)
- 1 paying customer with money in the bank
- At least one case study published

### Week 49-52: Design partner outreach

**Not Imanol.** Imanol remains your informal advisor. Design partners are external.

Target list (do this during Phase 4 in spare moments, so it's ready by week 49):

- [ ] 5 Spanish environmental consultancies (EcoIntelligent Growth, ITENE, Inèdit, Simbiosy, Zicla)
- [ ] 5 Basque industrial sustainability teams (CAF, Gestamp, Sidenor, ITP Aero, CIE Automotive)
- [ ] 5 European EPD consultancies (2.-0 LCA in Denmark, Systain in Germany, Solinnen in France, Tauw, Ramboll)
- [ ] 5 academic labs that might use Arko for research (universities in Bilbao, Madrid, Barcelona with LCA programs)

Outreach template (personalize, short):

> Hi [name], I'm Samir, solo founder based in Bilbao. I've been building Arko — an open-math, web-based LCA tool meant for practitioners tired of SimaPro's Delphi-era limitations. It's ready for real-world use; I'm looking for 3-5 design partners who'd get free access in exchange for using it on a real project and giving me honest feedback. Would you or someone on your team want to take a 30-minute look? I can come to your office in Bilbao / Madrid / Barcelona / or we do it remote.

Expect: 20% reply rate, 5% meeting rate. So 20 outreaches = 4 meetings = 1-2 design partners.

### Week 53-56: Design-partner onboarding

For each design partner:
- [ ] 30-minute first meeting: understand their workflow, their SimaPro pain
- [ ] Offer them a free workspace with full features for 6 months
- [ ] Agree on a specific real study they'll do in Arko (a project they're already working on)
- [ ] Weekly check-in call (30 min) to capture friction
- [ ] Fix critical bugs within 48 hours
- [ ] At month 3: case study interview, permission to publish

### Week 57-60: First paying customer

The design partner will turn into a paying customer, or introduce you to one. This is the conversion moment.

- [ ] Price firmly. Studio €149/user/mo, Team €349/user/mo. Don't discount below 50% even for design partners converting.
- [ ] Send a formal proposal, not a handshake. Include scope, SLA, support commitment.
- [ ] Use Redsys for billing (EU-sovereign, shared with KarbonGarbi).
- [ ] Publish the case study (with permission) on arko.earth/customers.
- [ ] **This is the milestone:** first money. Celebrate. Then get back to work.

### Red flags in Phase 5

- Zero meetings from first 20 outreaches → your value prop isn't landing; rework the message, A/B test subject lines
- Design partners are not finishing their pilot studies → friction is too high; triage and fix
- You're being asked for features that are 2+ months of work → politely decline, add to roadmap, focus on shipping to existing users

---

## Alongside all of this: KarbonGarbi

Non-negotiable. The pause trigger still applies.

### Minimum viable KarbonGarbi effort (parallel track)

**Every Thursday evening:**
- [ ] Review the KarbonGarbi pipeline
- [ ] Reach out to 5 new prospects
- [ ] Follow up on previous-week conversations
- [ ] Prepare the next week's outreach

**Every Friday evening:**
- [ ] Customer success: check in with active customers or pilots
- [ ] Fix any critical KarbonGarbi bugs reported
- [ ] Update metrics spreadsheet: MRR, active users, pipeline

**Monthly:**
- [ ] Review KarbonGarbi metrics against pause trigger
- [ ] If pause trigger approaches: reallocate Arko time back to KarbonGarbi

**Hard stop:**
- If by September 30, 2026 you have fewer than 2 paying KarbonGarbi customers: **pause Arko completely** until KarbonGarbi has 3. Full-time KarbonGarbi focus until that's true.

This isn't pessimism. This is the rule that keeps you alive if Arko takes longer than expected — which it probably will.

---

## How to use Imanol well (the informal-friend model)

Imanol is an unpaid domain advisor. Treat his time like it's expensive even though it's free.

**Do:**
- Ask for feedback in batches (once per phase major milestone)
- Prepare specific artifacts: screenshots, PDFs, short videos
- Prepare specific questions: "Does this number look right?" "Would you sign this EPD?" "Is this workflow closer or further from SimaPro?"
- Respond quickly to his feedback; show you value it
- Send him something nice periodically (bottle of wine, thank-you card) — this isn't manipulation, it's respect
- Credit him in the case study when Arko launches ("with domain expertise from Imanol Bollegui")

**Don't:**
- Drip-feed small questions every few days — batch them
- Ask him to review code or tech choices — he's an LCA practitioner, not a developer
- Use him as a substitute for professional validation when stakes are high (actual EPD submission to a verifier, regulatory interpretation) — pay a proper consultant for those moments
- Mention Arko to KarbonGarbi customers or prospects via Imanol — keep the worlds separate
- Assume his availability; always check and offer to reschedule

**When to escalate to paid professional help:**
- First EPD template: pay 2-4 hours of a proper LCA consultant to review template against real verifier checklists (~€200-400)
- Before any customer-paid EPD is generated: legal/methodological review (~€500-1000)
- Before SOC 2 audit or enterprise contract: formal LCA methodology audit (~€2000-5000)

Imanol as friend: free, limited bandwidth, informal.
Professional consultant: paid, bounded scope, formal sign-off.

Don't confuse the two.

---

## The decision log you'll maintain

Create `/arko/DECISIONS.md` in your repo. Append every significant decision with date and reasoning. Example entries:

```
## 2026-05-03 — License: Apache 2.0 for engine
Decided engine is Apache 2.0, app is commercial. 
Alternatives considered: AGPL (rejected — kills consultancy adoption), 
source-available (rejected — dishonest marketing).

## 2026-06-15 — UI stack: Next.js 15 + shadcn/ui + Tailwind
Reuse KarbonGarbi design system. Consistency + velocity.

## 2026-09-10 — Decided against ecoinvent integration in V1
Per Master Spec discipline. V1 ships with open EU databases only.
Revisit post first paying customer.
```

This becomes invaluable at month 12 when you're questioning decisions made at month 2.

---

## When things go wrong (they will)

**"I'm 3 weeks behind on Phase 2 already and it's only week 14."**
This is normal. Cut scope before cutting quality. Drop the Sankey to Phase 3. Ship the comparison view as a simple table. You're not behind; the estimate was optimistic. Adjust and continue.

**"Imanol hasn't responded to my message in 2 weeks."**
He's busy. Don't take it personally. Send one polite follow-up ("no rush, happy to wait"). If another week passes, move on and get professional paid help for that specific question. Resume contact later with something different.

**"A KarbonGarbi customer is asking for a feature that would take me a month."**
Classic founder trap. Ask: is this one customer or three? Is this pulling them from "interested" to "paying," or from "paying" to "paying more"? If it's one customer and they're already paying, politely decline. If it's the blocker between free-pilot and actual-revenue, and three customers want it, consider it.

**"I'm burning out."**
Stop. Take a week off both projects. Seriously. A solo founder with a day job who burns out loses everything. One week of rest > four weeks of degraded output. Your specs, your engine, your KarbonGarbi code will all still be there next week. Non-negotiable.

**"A competitor launched something similar."**
Good. Validates the market. They probably built it worse in the specific ways you care about (open math, modern stack, AI-native, license transparency). Focus on your moat — the published specification, the open engine, the verifiable reproducibility — not on feature parity. Write a blog post comparing you both honestly.

**"I'm tempted to accept VC money."**
Don't. Not yet. A solo founder with revenue and a working product commands 10× better terms than a solo founder with a spec and hope. If you wait until after first paying customer, your negotiating position is qualitatively different. And honestly — you might not need VC. KarbonGarbi + Arko as a bootstrapped portfolio might be a better life than venture-scale pursuit.

---

## The sanity check questions

Ask yourself every Sunday evening:

1. **Am I ahead, on track, or behind the 14-month plan?** If behind, by how much?
2. **Did I do KarbonGarbi work this week?** (If no two weeks running, something's wrong.)
3. **Did Imanol hear from me this week?** (If no 4+ weeks running, reconnect.)
4. **Am I building features Imanol asked for, or features I want?**
5. **Is my energy level green/yellow/red?** (Red means rest.)
6. **Is there anything I'm avoiding?** (Usually: hard conversations with customers, unglamorous infrastructure work, or rewriting something I got wrong.)
7. **What's the single most important thing to do next week?**

Write the answers in a journal. Review monthly.

---

## The definition of done

You'll know you've succeeded when all of these are true:

- Arko has at least 1 paying customer (€149+/mo MRR)
- KarbonGarbi has at least 3 paying customers (€200+/mo MRR each)
- Combined Goibix MRR: €750+/mo (enough to convince a reasonable person this is a real business)
- The published calc engine specification has been cited or referenced by at least one external LCA practitioner or academic
- You're not working 60+ hour weeks
- Imanol is still a friend

This takes 14 months. It's a lot. It's also finite. At the end of it, you're no longer a solo founder with a spec — you're a solo founder with a business.

Then you decide what comes next.

---

## Final words

Don't read this guide daily. It'll become either anxiety fuel or aspiration porn. Look at it weekly on Sunday. Look at next week's tasks Monday morning. That's it.

The real work is execution, not planning. This document exists so you don't spend mental energy on "what's next" — it's already decided. Your mental energy goes to the work itself.

Go build. Trust the plan. Adjust when reality disagrees. Ship.

Good luck.

---

*Document version 1.0 — April 2026*  
*Written for Samir, solo founder at Goibix S.L., Bilbao*  
*Companion to: Arko Master Specification v1.0 and Arko Technical Architecture Specification v2.0 (Excellence Edition)*
