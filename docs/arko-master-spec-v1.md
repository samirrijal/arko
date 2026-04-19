# Arko — Master Specification v1.0

**The open, AI-native life cycle assessment platform for modern product decisions.**

---

**Document status:** Draft v1.0 — April 18, 2026
**Author:** Samir (Goibix S.L., Bilbao) with Claude
**Product:** Arko (sibling to KarbonGarbi under Goibix)
**Primary domain:** `arko.earth` (with `arko.eus` redirect)
**Source of truth:** This document; all subsequent iterations branch from here.

---

## 1. Executive Summary

### The opportunity in one paragraph

The global life cycle assessment (LCA) software market is anchored by a single 30-year-old desktop application — SimaPro — which was recently acquired by One Click LCA, creating a 12–18 month disruption window. SimaPro has roughly 3,000 paying users globally, but the market that *needs* LCA is at least 100× larger: EU manufacturers facing CSRD, CBAM, PEF, and the Digital Product Passport mandate, consultancies serving SMEs, product designers needing rapid feedback during development, retailers building sustainability scores, and the growing open-science and developing-world communities. Arko is a web-native, AI-assisted LCA platform built on open data and open methods, designed for the 99% of the market SimaPro cannot profitably serve, with an EPD-first workflow, first-class comparison, automated document generation, real-time collaboration, and a legal/licensing architecture that supports use cases ecoinvent's EULA actively prohibits.

### Positioning

**Not "SimaPro killer." Arko is what modern LCA becomes when you rebuild the tool for the people SimaPro left behind.**

- Where SimaPro serves PhDs, Arko serves product teams.
- Where SimaPro optimizes for scientific rigor, Arko optimizes for decision clarity — without sacrificing rigor.
- Where SimaPro sells a desktop file format, Arko sells a collaborative workspace.
- Where SimaPro locks you into ecoinvent, Arko is database-neutral with an open-first default.
- Where SimaPro makes a new user watch a 5-minute orientation video, Arko has them generate their first result in 5 minutes.

### What we ship

Arko ships as a web application with:

1. **Study-first workflow** — opinionated data model collapses SimaPro's 15-object hierarchy into three first-class concepts: Study, Model, Library.
2. **EPD Mode** — first-class support for Environmental Product Declarations across all major program operators, with PCR-aware method configuration, auto-scaffolded lifecycle modules, and native document generation.
3. **Interactive graph canvas + structured table** — two equal-citizen views of the same model: Figma-style visual supply chain editing and Airtable-style bulk data editing.
4. **Live calculation** — no "Update expression results" button; incremental re-computation on every edit.
5. **Comparison as first-class object** — comparisons are studies with variants, not a calculation mode.
6. **AI interpretation layer** — every result gets auto-generated narrative, hotspot detection, anomaly flagging, and natural-language querying.
7. **Collaboration** — real-time multi-user editing, shareable links with permissions, threaded comments on any object, version history and branching.
8. **Open-first data & methods** — Agribalyse, USDA LCA Commons, ELCD, USEEIO, PEFAPs, Basque industrial factors; ReCiPe, EF 3.1, IPCC AR6, USEtox, CML-IA, TRACI, AWARE — all free at launch. Ecoinvent as optional paid tier with full EULA compliance architecture.
9. **Data provenance and license enforcement as code** — every number carries its license tag; publishing actions are gated by the most restrictive license in the calculation graph.

### Commercial shape

- **Sibling product under Goibix**, alongside KarbonGarbi. Shared SSO, shared billing infrastructure, shared tenant management. Separate brand, separate pricing, separate sales motion.
- **Buyer** is different from KarbonGarbi's: product managers, LCA consultants, R&D engineers, sustainability analysts at mid-market manufacturers, EPD project leads. Not the SME sustainability manager.
- **Pricing** tiers (indicative, validate with market research):
  - **Free** forever for students, open-source, small projects (max N processes, 1 study at a time). Converts the SimaPro pirate/academic population.
  - **Studio** (~€49–99/mo) for individual consultants and small design teams.
  - **Team** (~€299–499/mo) for small consultancies and internal sustainability teams, includes collaboration.
  - **Enterprise** (custom, ~€2,000–8,000/mo) for large manufacturers, includes ecoinvent reseller pass-through, SSO, audit, SLAs.

### Why now

Five tailwinds converge in 2026:

1. **One Click LCA's acquisition of SimaPro** creates customer uncertainty and competitive vacuum.
2. **EU regulatory cascade** — CSRD reporting in force, CBAM operational, PEFCR expanding, Digital Product Passport mandatory from 2027 for many categories, Green Claims Directive requiring substantiation.
3. **AI makes LCA accessible** at a level not previously possible — we can ship AI-assisted modeling, natural-language interpretation, and gap-filling that simply wasn't feasible two years ago.
4. **Ecoinvent pricing pressure** from open alternatives (Agribalyse, PEFAPs, ELCD) is creating room for database-neutral products.
5. **KarbonGarbi customer base** gives us a warm initial audience: SMEs who will need product-level LCAs once their corporate carbon accounting matures.

### The constraint that shapes everything

Ecoinvent's End User License Agreement (v3, 2022) fundamentally restricts what any ecoinvent-integrated tool can do. Clauses 7.1(e) and 7.1(f) prohibit using ecoinvent data in software tools for third-party viewers, in consumer-facing comparisons, in e-shop integrations, and in regulated declarations (like EN 15804 construction EPDs) without separate licensing. These restrictions do *not* apply to open data. **This is a legal moat for us, not a constraint.** Every growing LCA use case — Digital Product Passports, retailer sustainability scores, consumer-facing comparisons, real-time design feedback — is an ecoinvent-EULA-restricted use case and therefore natively served by open-data-first tools. Arko's open-data-first posture isn't a compromise; it's a legal and strategic advantage.

### The 18-month roadmap (aligned with Samir's reality)

- **Now → Summer 2026:** Focus on KarbonGarbi. Close Imanol Bollegui. Ship the marketing site. Get KarbonGarbi to 3–5 paying customers. Do not build Arko yet. This Master Spec is banked for when we return.
- **Summer 2026 (Month 0):** Arko project begins. Two-month architecture and data model sprint. Shared Goibix backbone extension. Ingest open data bundle.
- **Autumn 2026 (Months 3–5):** Core modeling interface — canvas + table — and calculation engine. First closed alpha with 10 invited users (likely from Samir's network + KarbonGarbi prospects with product interest).
- **Winter 2026–27 (Months 6–8):** EPD Mode, document generation, comparison workflows. First 3 paying customers.
- **Spring 2027 (Months 9–12):** AI layer (interpretation, narrative, gap-filling, natural-language queries), collaboration, open beta.
- **Summer 2027 (Months 13–18):** Ecoinvent reseller negotiation, Enterprise tier launch, first European consultancy partnerships.

---

## 2. Product Vision

### The world we want to build toward

LCA today is a profession. Our vision is a world where LCA is a *capability* — something any product team can do, the way any team today can do analytics, A/B tests, or SEO research without needing a PhD. The environmental consequences of a product decision should be as visible during design as its cost or manufacturability.

For that to happen, the tools have to change. The scientific foundation of LCA (ISO 14040/14044, the LCIA methods, the inventory databases) is sound and should be preserved. But the software layer — how people build models, run calculations, interpret results, collaborate, and communicate — is frozen in 1998 and needs to be rebuilt from scratch.

Arko is that rebuild.

### The three user archetypes we're building for

1. **The LCA consultant** — currently tortured by SimaPro, charges clients €10–30k per EPD, wants to deliver faster and win more clients with better deliverables. Pays full price.
2. **The product team sustainability lead** — at a manufacturer, maybe a chemical engineer or designer, suddenly responsible for carbon footprint and EPDs, has no time for a two-week SimaPro training course. Pays Team tier.
3. **The curious / constrained user** — student, academic in a developing country, freelance designer, NGO worker, open-source maker. Currently uses cracked SimaPro or gives up. Pays nothing but generates word-of-mouth and converts to paying as their career advances.

### Non-users we're explicitly not chasing (yet)

- **Senior academic LCA researchers** publishing in peer-reviewed journals who must cite ecoinvent and use established methods in SimaPro to satisfy reviewers. They'll keep SimaPro for the foreseeable future; that's fine.
- **Large industrial consultancies** (ERM, Quantis, Ramboll) who have standardized on SimaPro with hundreds of licenses and trained staff. Their switching cost is too high in V1; we'll come back to them in V3.
- **Regulated EPD programs requiring specific tool certification** (some program operators formally accept only a short list of tools). We'll pursue this certification post-product-market-fit, not before.

### The wedge order

Markets to win in sequence, not simultaneously:

1. **Basque and Spanish industrial SMEs** with EPD/CSRD needs (via Samir's network, KarbonGarbi ecosystem, Imanol-adjacent sustainability technicians in city economic agencies)
2. **European LCA consultancies serving SMEs** — mid-sized firms who need modern tools but don't want to pay SimaPro enterprise pricing
3. **EU product design teams** in consumer goods, packaging, textiles, furniture needing iterative design-phase LCA
4. **Digital Product Passport-preparing manufacturers** as the 2027 mandate lands
5. **Global open-science and developing-world users** via the free tier

---

## 3. Competitive Landscape

### SimaPro (incumbent, acquired by One Click LCA in 2025)

- **Users:** ~3,000 paying globally
- **Price:** €3,000–5,000/yr SimaPro + €2,500/yr ecoinvent + €1,500 training, all seat-based
- **Strengths:** 30 years of trust, deep ecoinvent integration, scientific rigor, ISO compliance, regulated-program acceptance, matrix export for power users
- **Weaknesses (decoded from our exploration):**
  - Desktop-only, Windows-only, no real-time collaboration
  - 15-object data model exposing internal plumbing
  - No AI, no automation, no supply chain intelligence
  - No modern sharing — share = email a file
  - Documentation: 61 YouTube videos over 14 years, mostly webinars, barely 2.75k subscribers. Active education stopped a decade ago
  - Multi-tool architecture (Craft desktop + Report Maker Office add-in + Online Platform + Synergy API) each sold or bundled separately
  - Character-truncated UI labels visible in 2026
  - Hidden killer features (Pedigree, Compare Methods, Matrix Export) behind bad UX
  - Single-score weightings silently borrowed from Eco-Indicator 99 in a ReCiPe-branded tool
  - 1,117-substance method-coverage warnings buried in a tab
- **Status:** 12–18 month uncertainty window post-acquisition. Customer sentiment: wait-and-see. One Click LCA's roadmap for SimaPro is unclear to the market.

### One Click LCA (now parent of SimaPro)

- **Users:** Strong position in construction EPDs (EN 15804), buildings, materials
- **Strengths:** Web-based (partially), strong PCR coverage for construction, recent SimaPro acquisition consolidates market position
- **Weaknesses:** Historically narrow focus on buildings; how they integrate vs. maintain SimaPro is unclear; their UI is functional but not notable
- **Our posture:** Non-overlapping for V1. We avoid deep construction EPDs where they're entrenched; we target manufacturing, consumer goods, food, textiles, chemicals, electronics — categories where they're weaker and SimaPro is the default.

### OpenLCA (GreenDelta)

- **Model:** Free and open-source software, paid databases
- **Strengths:** Open-source credibility, academic user base, programmatic access via olca-ipc
- **Weaknesses:** UX is scientist-oriented, documentation is technical, community is small, no meaningful commercial support layer, no EPD workflow
- **Our posture:** OpenLCA serves the hardcore technical user; we serve the practitioner and decision-maker. We can even interoperate — import OpenLCA projects as a migration path.

### Ecochain (Mobius)

- **Model:** Web-based LCA, newer (~2015+), VC-backed, positions as "LCA for non-experts"
- **Strengths:** Modern UX, good product-design messaging, well-funded
- **Weaknesses:** Limited database coverage relative to SimaPro; proprietary approach; pricing opaque; primarily NL/EU focus
- **Our posture:** Most direct competitor on positioning. Differentiation: Arko is more open (open data, open methods, community PCRs), more AI-native, EPD-first, and has the Goibix platform advantage (bundled with KarbonGarbi for corporate-and-product suites).

### Sphera (GaBi)

- **Model:** Enterprise LCA and EHS software, large industrial focus
- **Strengths:** Deep industrial database, enterprise sales motion, integration with ERP/PLM
- **Weaknesses:** Expensive, legacy UX, complex deployment
- **Our posture:** Non-target for V1. They own the enterprise chemicals/automotive segment; we'll serve that segment later via Enterprise tier.

### Emerging: Carbon/ESG platforms with LCA modules

- **Sweep, Watershed, Persefoni, Plan A, Greenly, Normative** — mostly corporate carbon accounting, some adding product-level features
- **Our posture:** These are KarbonGarbi's competitors in corporate scope, not Arko's. Some may emerge as product-LCA competitors; monitor for now.

### Competitive summary

| Dimension | SimaPro | One Click LCA | OpenLCA | Ecochain | **Arko (target)** |
|-----------|---------|---------------|---------|----------|-------------------|
| Deployment | Desktop Windows | Web | Desktop cross-platform | Web | Web, offline-capable |
| Data model | 15 objects | Moderate | Similar to SimaPro | Simpler | 3 concepts |
| Real-time collab | No | Partial | No | Limited | Yes, Figma-style |
| Live calculation | No | Partial | No | Partial | Yes, always |
| AI layer | None | None | None | Limited | Core |
| EPD native | No (manual tricks) | Yes (construction) | No | Partial | Yes, all programs |
| Open data | Requires paid | Mix | Yes | Mix | Yes, primary default |
| Ecoinvent integration | Yes (paid) | Yes | Yes (paid) | Yes | Yes, V2 (via reseller) |
| Multilingual education | No | Partial | Partial | EN-only | EN, ES, FR, PT, DE, EU day 1 |
| Free tier | No | No | Free software only | No | Yes, generous |
| API | Legacy COM + Synergy | Partial | olca-ipc (Python) | Limited | REST+GraphQL+MCP day 1 |
| Community | Partner network | Limited | Active (academic) | Limited | Discord + gallery + open PCRs |
| PCR discovery | Manual | Yes (construction) | No | Partial | Full library, community-maintained |

---

## 4. Core Data Model

SimaPro's data model fails because it exposes implementation structure as UI structure. Our data model succeeds by separating concepts the user thinks about (Study, Model, Library) from concepts the math requires (processes, flows, characterization factors, parameters) — the latter live beneath, accessible but not required.

### The three first-class user concepts

```
Workspace (tenant / team)
  └── Study (a piece of analytical work with a clear purpose)
        ├── Goal & Scope (functional unit, system boundary, method, PCR)
        ├── Variants (products being compared, or scenarios)
        │     └── Lifecycle (the pipeline: raw materials → manufacturing → use → end of life)
        │           └── Stages (A1, A2, A3, A4, A5, B1–B7, C1–C4, D — per EN 15804, or custom)
        │                 └── Models (the actual data: BOM, processes, emissions, parameters)
        │                       └── Parts / Sub-assemblies (optional; hierarchical structure)
        │                             └── Inputs & Outputs (flows, either linked processes or elementary substances)
        ├── Parameters (study-level variables)
        ├── Results (cached computations, auto-refreshed)
        └── Deliverables (EPD draft, report, summary, exports — generated from the study)

Libraries (reusable data bundles)
  ├── Process libraries (Agribalyse, ELCD, USDA, ecoinvent [paid], user-shared)
  ├── Method libraries (ReCiPe 2016, EF 3.1, IPCC AR6, USEtox, custom methods)
  ├── PCR libraries (International EPD System, IBU, EPD Norge, EPD Italy, etc.)
  ├── Template libraries (starter studies by category: coffee, packaging, textile...)
  └── Factor libraries (custom emission factors, primary data bundles)

Catalog (read-only, system-level)
  ├── Substances (CO2, CH4, N2O, elemental flows)
  ├── Units & Quantities
  ├── Geographies (with ISO codes, electricity mixes, regional factors)
  └── Impact categories (with semantic names, icons, explanations)
```

### Objects the user rarely touches directly

| Object | Where exposed | How accessed |
|--------|---------------|--------------|
| Process | Inside a Model's inputs | Inline search when linking; editable in a side panel |
| Elementary flow | Inside emissions section of a Model | Typeahead add; categorized by compartment |
| Characterization factor | Method details | Read-only unless defining custom method; provenance-linked |
| Unit | Auto-handled | Exposed only when ambiguous (e.g., volume-to-mass conversion) |
| Substance (CAS) | Searchable | Shown on hover, not as a primary column |
| Pedigree score | Inline on every input | Edit via small popover; visualized as badge |

### The Study object in detail

Every Study declares its purpose upfront. The Goal & Scope wizard (shown once, at study creation) captures:

- **Title and description** (free text, but AI can suggest based on product type)
- **Functional unit** ("1 coffee pot with 1L capacity", "1 m² of laminated flooring over 50 years", "1 kg of compound feed")
- **System boundary** (cradle-to-gate, cradle-to-grave, gate-to-gate, cradle-to-cradle) — selected, not free text
- **Intended use** (internal R&D, marketing claim, EPD publication, academic study, regulatory submission) — this drives default settings and warnings
- **Target PCR** (optional; if selected, auto-configures method, lifecycle modules, data quality requirements)
- **LCIA method** (defaults from PCR if set; otherwise guided selection)
- **Data quality requirements** (threshold for proxy data, pedigree expectations)
- **Geographic scope** (regions/countries relevant to the product)
- **Time horizon** (for temporal data quality assessment)

This replaces SimaPro's scattered "Description / Libraries / Waste types / Parameters / Methods / Calculation setups" fields with a single, coherent declaration.

### The Model object in detail

A Model is a node in the graph that transforms inputs into outputs with a known mass/energy balance. Models come in two flavors:

1. **Assembly Models** — user-created; describe how something is made (the coffee pot, the circuit board, the packaging unit). Composed of Parts (optional), each with linked upstream Models and Processes.
2. **Unit Process Models** — usually from Libraries (e.g., "electricity generation, Spanish grid, 2023"); leaf nodes in the graph, usually with explicit elementary flows.

Every Model carries:
- **Reference flow** (what it produces; always explicit — never implicit as in SimaPro)
- **Inputs** (from supply chain = linked Models; from nature = elementary flows; energy broken out)
- **Emissions** (to air, water, soil, with explicit subcompartments)
- **Waste outputs** (to treatment, final disposal)
- **Co-products** (multi-output; trigger allocation prompts)
- **Mass/energy balance** (computed live; imbalances flagged with warnings, not hidden as in SimaPro)
- **Pedigree scores** (per input, required for EPD-bound studies)
- **License and provenance** (for every linked external value)

### The Variant object (why comparisons are first-class)

Most LCA studies compare two or more options: "Product A vs Product B", "Current supplier vs alternative supplier", "2023 formulation vs 2024 formulation". SimaPro forces users to create separate Product Stages and then configure a Calculation Setup in Compare mode, treating comparison as an afterthought.

In Arko, a Study has one or more **Variants**:

```yaml
Study: "Coffee maker redesign — Glass vs thermos"
  Variants:
    - Sima:
        description: "Current glass-pot design"
        lifecycle: [materials, manufacturing, distribution, use, end-of-life]
    - Pro:
        description: "Proposed thermos design"
        lifecycle: [materials, manufacturing, distribution, use, end-of-life]
```

Comparison is the default view; single-variant studies are a special case. This inverts the SimaPro assumption and matches how practitioners actually think.

### Licenses and provenance as data-model fields, not metadata

Every Flow, Process, and Method reference carries:

```yaml
Flow:
  id: uuid
  amount: 0.302
  unit: "MJ"
  linked_to: "process:ecoinvent/market_for_electricity_low_voltage_JP/3.9.1"
  license: "ecoinvent-3.0"
  provenance:
    source: "ecoinvent 3.9.1"
    accessed_via: "user_tenant_license_2026"
    data_year: 2020
    geography: "JP"
    allocation: "cut-off"
  pedigree:
    reliability: 2
    completeness: 3
    temporal_correlation: 2
    geographical_correlation: 1
    technological_correlation: 3
  mutable: false
```

This single schema change — license as a *required field*, provenance as *structured data* — enables everything downstream: compliance enforcement, audit trails, reproducibility, uncertainty auto-computation, data-quality scoring.

### Data model principles summary

- **Three concepts exposed** (Study, Model, Library); fifteen objects hidden beneath.
- **Every Model declares its reference flow explicitly.** No implicit "1 unit" assumptions.
- **Comparison is the default**; single-variant is the special case.
- **License and provenance are data fields, not metadata.** Enforced by the system.
- **Pedigree is inline and visible**, not buried in an Edit menu.
- **Mass/energy balance is live and surfaced**, with warnings for imbalances.
- **Empty compartments collapse** with affordances to add. Screen real estate follows data density.
- **Lifecycle modules are declarative** (A1–D), not manually configured with dummy processes.
- **Methods are versioned open resources.** Pinning a study to a method version gives bit-identical results forever.

---

## 5. The 154 Design Principles, Organized

The principles below are anchored in our decoded evidence from SimaPro, ecoinvent, the tutorial archive, and the EPD webinar. Each principle is numbered per its original emergence during exploration, but grouped here thematically for readability.

### 5.1 Information architecture and core UX (Principles 1–15, 41–44)

**P1. Hide the methodology, surface the task.**
When a user opens Arko, they see recent studies, a "new study" button, and a clear hero action ("What do you want to analyze today?"). ISO 14044's four phases are not the navigation.

**P2. One window, one workspace, one flow.**
No stacked dialogs. No modal-blocks-modal-blocks-modal. Progressive disclosure inside a single canvas.

**P3. Studies are first-class, not buried.**
The dashboard shows recent studies like Linear/Figma/Notion: cards, status, last activity, collaborators. Not a Projects modal inside a Database window.

**P4. Sidebar is about your work, not a 1997 standard.**
Dashboard · Studies · Library · Methods · Templates · Team. Not Goal and scope → Inventory → Impact assessment → Interpretation.

**P5. Numbers in the UI mean something to the user.**
"38,344 substances" is not a headline. "3 active studies, 2 shared with you" is. Reserve big numbers for things the user cares about.

**P6. Never expose plumbing as a default view.**
Substances, elementary flows, unit conversion tables, CAS numbers, process identifiers like AFPEconU000000000600851 — all hidden unless the user intentionally drills in.

**P7. Empty state is an opportunity, not a dump.**
New user with empty workspace → "Let's start your first study" with a 3-question wizard (what product? what functional unit? what scope?) → they have a scaffolded study in 30 seconds.

**P8. Three concepts, not fifteen.**
Study, Model, Library. Everything else lives beneath. Users never need to learn "Product stages vs Processes vs System descriptions vs Waste types."

**P9. Progressive disclosure over feature density.**
Our sidebar has 5–6 items because our workflow is clear. SimaPro's has 15 because they're proud of feature count. We're confident in flow, not in footprint.

**P10. Search is the primary navigation.**
Cmd/Ctrl+K command palette accesses every feature and every object by typing intent. The tree is secondary, for casual browsing.

**P11. Structured data, structured columns.**
Process names have fields: `name`, `geography`, `allocation_method`, `system_boundary`, `data_year`, `data_source`. Displayed as columns and filter chips, never as a mashed single string like "Broiler parents >20 weeks compound feed, at processing {JP} Economic, U".

**P12. Data quality is a first-class visual citizen.**
Every process card shows: 🟢 2023 data · 🇪🇸 Spain · ✅ Peer-reviewed · 📊 Primary data. Pedigree badges visible at a glance.

**P13. Semantic search with AI.**
User types "coffee machine" → we understand they want household appliances → we show relevant processes even if the underlying name is "electric drip coffee maker, household, at plant {RER}". Computer learns user's language, not vice versa.

**P14. Transparent provenance, everywhere.**
Hover any number → see source, year, confidence. Click → full process sheet. No five-screen navigation to answer "where did this come from?"

**P15. Data first, documentation second.**
Process editor opens on inputs/outputs. Documentation is a sidebar panel. Real modeling before paperwork.

**P41. Product stages get a canvas with semantic part-relationships.**
Assembly editor has Parts; each Part contains Materials + Processes. Structural hierarchy: Coffee pot → Handle (PP + injection moulding) + Jug (glass + furnace). No flat Materials/Processes dichotomy.

**P42. Reference flow / functional unit is front and center.**
Every Model editor shows at top: "This produces: 1 coffee pot (1 unit, 0.6 kg)." Always visible, never implicit.

**P43. Lifecycle as a visual pipeline.**
Horizontal pipeline: Raw Materials → Manufacturing → Distribution → Use → End of Life. Click any stage to edit. Life cycle isn't a separate object — it's the pipeline itself.

**P44. Product photos + AI-assisted BOM decomposition.**
User uploads a product photo → Claude's vision identifies likely components → suggests BOM entries: "I see a glass jug (~500 ml), plastic handle, electrical cord. Start with these?" Nothing like this in any LCA tool today.

---

### 5.2 Data entry, process editing, inventory (Principles 16–40, 45–47)

**P16. ISO 14044 fields as structured, validated inputs.**
Collection method → dropdown. Allocation rules → dropdown with explanation. Data year → separate field from Last modified. Verification → enum. Data source → reference to a Literature object. No free-text graveyards.

**P17. A single Data Quality Score, computed and visible.**
Pedigree + vintage + verification + representativeness → A–E badge (like KarbonGarbi supplier scoring). Displayed in every list, chart, and report.

**P18. Hide identifiers, show names.**
Internal IDs (UUIDs) for URLs and APIs; never in UI. If a user truly needs the ID for a support ticket, it's one hover away.

**P19. Treat documentation as living product data.**
Empty documentation fields are flagged: ⚠️ "This process has no verification record." Sortable, filterable, actionable. Transform "everyone ignores this" culture into "we make the gaps visible."

**P20. Provenance graph, not provenance prose.**
Instead of a Comment field with 2010 data buried in a paragraph, show:
> 📅 Data year: 2010 · 🌍 Region: Netherlands → adapted to Japan · 📚 Source: Agrifirm · ✅ Method: Compound feed formulation · ⚠️ Age: 16 years old

Structured. Scannable. Filterable. Reportable.

**P21. LCA processes modeled as a graph, not a form.**
Two equal views: Canvas (visual node graph, ReactFlow or Cytoscape.js) and Table (structured data, Airtable-style). Same underlying model; toggle between them.

**P22. Two fundamentally different row types, rendered differently.**
Process link (to another Model, drilldownable) and Elementary flow (CO2, water, land — terminal). Different icons, different interactions. SimaPro renders them identically; we don't.

**P23. Three zones, collapsible compartments.**
Products (expanded by default), Inputs (expanded; From supply chain / From nature / Energy sub-groups), Emissions & waste (collapsed if empty; Air / Water / Soil / Waste sub-groups). Empty compartments show a ghost "+ Add emission to air" affordance.

**P24. Mass/energy balance as a live header.**
Every process shows:
> ⚖️ Mass balance: 1,008 g in → 960 g out (⚠️ 48 g unaccounted, 4.8%)
> ⚡ Energy balance: 0.302 MJ in → 0 MJ out (✓ consumed, no co-product)

Red/yellow/green. Users see data quality issues immediately.

**P25. Uncertainty is a mode, not a column.**
Default view: one Amount column. Toggle Uncertainty mode → expands to distribution, parameters, bounds. Cleaner screens, same power.

**P26. Smart linking with typeahead.**
Click "+ Add input" → inline search → type "wheat" → fuzzy-matched processes sorted by (semantic match × data quality × geographic relevance to parent). One keystroke to pick, Enter to confirm. No modal.

**P27. Unit intelligence.**
"0.302 MJ" recognized. "302 kJ" auto-normalized. "1 gallon" into a kg-expected field → warning: "Can't convert gallon (volume) to kg (mass) without density. Specify substance?"

**P28. Sub-compartments as proper dropdowns with explanation.**
Emissions to air sub-compartment: [Urban / Rural / High-population / Low-population / Stratosphere / Unspecified] with tooltips explaining how each affects characterization.

**P29. AI-assisted process building.**
New process "broiler chicken feed production" → before typing, we suggest: "I found 8 similar processes in your libraries. Want to start from 'Broiler feed {NL}' and adapt, or build from scratch?"

**P30. AI gap-filling.**
User enters only a BOM → AI: "You've added wheat, maize, soybean meal. Typical broiler feed formulations also include vitamin premix (~3 g/kg) and calcium carbonate (~10 g/kg). Add these from ecoinvent defaults?"

**P31. Natural-language editing.**
Command bar: "Swap all Japanese electricity inputs for Spanish electricity" → renders a preview diff → user confirms → done. 45-minute SimaPro task becomes 15 seconds.

**P32. Live hotspot preview.**
Even before "running" the impact assessment, a sidebar shows: "Current estimated impact: 1.84 kg CO₂-eq / kg feed. Top contributor: Wheat grain (42%)." Updated as you edit.

**P33. Every number is drillable.**
Click "Wheat grain, 340 g" → slide-over panel with full details: geography, data year, impact per kg, supply chain depth. No navigation away.

**P34. Search results pages replace modal pickers.**
Finding a process to link is a proper in-app search UI with filters (geography, year, allocation, quality score), not a modal tree picker.

**P35. Filters as chips, not free text.**
[Geography: Japan] [Year: >2015] [Source: ecoinvent] — removable chips. SimaPro offers one "Filter on" free-text field; we offer faceted filtering.

**P36. Every list view has a star/pin.**
Frequently used processes can be pinned to a personal favorites list, surfacing at the top of searches. Replaces SimaPro's tree-diving for every new link.

**P37. Keyboard shortcuts are first-class.**
Cmd+N new study, Cmd+K command palette, Cmd+/ focus search, Cmd+Enter add row, Cmd+Shift+E toggle edit. Power users never touch the mouse. Display the shortcut next to every button.

**P38. Templates for every common structure.**
New Model → "Start from template?" options: Raw material extraction, Manufacturing step, Transport leg, Electricity generation, Waste treatment, etc. Pre-filled scaffolds with the right compartments and fields enabled.

**P39. Bulk operations.**
Select 12 rows → "Replace geography in all" or "Update data year" or "Scale by factor." SimaPro requires row-by-row edits; we treat inventory like a spreadsheet when that's the right mental model.

**P40. Undo-redo everywhere, with history panel.**
Cmd+Z works everywhere. A history panel shows all changes to a study with author, timestamp, and one-click revert. SimaPro has none of this.

**P45. Parameters as a first-class spreadsheet, not a buried tab.**
Dedicated Variables panel (right sidebar, always visible). Click any number in the inventory → "Make this a variable" → auto-creates a parameter. Formulas with Notion-style @variable_name autocomplete. Live preview of computed value.

**P46. Parameter scope is visualized.**
Scope badges: 🔒 Process-local / 📦 Study-level / 🌍 Workspace-global. Visible at a glance. No hidden collisions.

**P47. Comparison is a first-class object.**
A Study has one or more Variants. Variants can be: distinct products, the same product with different parameters, or the same product sourced differently. Comparison is the default; single-variant is the special case.

---

### 5.3 Calculation engine and results (Principles 48–72, 107–125)

**P48. One unified analysis workspace, not seven tabs.**
Summary → Breakdown → Drill-down → Audit. Four progressive levels. Not 8 parallel tabs.

**P49. The Summary is the product.**
First thing after running: *one number* (carbon footprint, or single score), *one sentence* ("Wheat grain is 58% of your climate impact"), *one recommendation* ("Consider local Spanish wheat to reduce transport"). Everything expandable below. SimaPro gives 18 stacked bars; we give the *answer*.

**P50. Network diagram as modern interactive graph.**
Built on ReactFlow/Cytoscape. Pan/zoom/fit. Node sizes ∝ contribution. Edge thickness ∝ flow (Sankey-style). Semantic grouping ("collapse all agriculture"). Click any node → side panel. Filter by impact category, by data quality. Export as interactive HTML, not just PNG.

**P51. Smart cutoff, not dumb threshold.**
Instead of "show processes ≥10% contribution" (SimaPro's manual slider), Arko offers: "show me everything contributing to 95% of impact" — finds the smallest node set covering 95%. Decision-oriented, not display-oriented.

**P52. Impact categories have human-readable names and icons.**
Not "Terrestri al acidifi" (truncated). Instead: 🌧️ **Acid rain potential** (terrestrial acidification) — with a tooltip explaining it. Each category: icon, plain-English name, one-line explanation.

**P53. Hotspots auto-highlighted.**
Results auto-compute: "Top 3 hotspots contributing to 80% of climate impact." Callout cards at the top. Decision-makers get decisions, not homework.

**P54. Method uncertainty surfaced, not hidden.**
The 1,117 missing-characterization warnings we saw in SimaPro get a **headline indicator** on summary: ⚠️ "1,117 substances couldn't be assessed by ReCiPe 2016. Impact may be underestimated by ~X%." No more burying data quality in a tab.

**P55. Single-score meters with confidence intervals.**
"0.045 Pt ± 0.008 (95% CI)" with a visible confidence ring. Users see precision AND uncertainty. SimaPro shows only precision and pretends uncertainty doesn't exist.

**P56. Comparisons are the default.**
Our results always show your value vs benchmark (industry average, previous version, scenario alternative). Never just one number alone. Context is half the insight.

**P57. Export to audience.**
- 📊 *Executive summary PDF* — one page, big number, top 3 hotspots, recommendation
- 📄 *Technical report PDF* — full ISO 14044 compliant, 50+ pages, appendices
- 📈 *Auditor pack* — Excel/CSV with every flow, factor, calculation step, hash-signed
- 🔗 *Interactive web link* — shareable URL with live network diagram (license-gated)
- 📋 *EPD draft* — pre-filled program operator template
- 🎨 *Design feedback card* — small summary for product designers, with "biggest lever to pull"

**P58. Traceability every direction.**
Click any number → "Where does this come from?" → provenance chain: Substance → Process → Library → Method → CF → Source literature. One click, not five screens.

**P59. Natural-language querying of results.**
Chat bar on results: "Which would reduce my climate impact more — switching to Spanish wheat or using organic fertilizer?" → AI runs what-ifs, returns comparison.

**P60. Automatic anomaly detection.**
"Your water consumption for category X is 4× the industry average. This may indicate a data error in Process Y." Proactive, not reactive.

**P61. AI-generated insight narrative.**
Beside every chart, auto-generated paragraph:
> "Climate impact is dominated by wheat grain cultivation (58%), primarily driven by nitrogen fertilizer application in Japanese farming practices. This pattern is typical for compound feed formulations. A 20% reduction in fertilizer intensity could reduce total climate impact by approximately 9.4%."

Claude-powered, grounded in actual data, with uncertainty disclaimers.

**P62. Complete scientific provenance per factor.**
Every characterization factor: substance, CAS, compartment, method, version, source paper, uncertainty, last updated, contested-or-not.

**P63. Every factor has full provenance, explorable in UI.**
Hover/click any factor → modal with full chain down to the peer-reviewed paper.

**P64. Factor uncertainty is a first-class field.**
Every factor has value + distribution. Monte Carlo uses these automatically.

**P65. Honest single-score handling.**
ReCiPe 2016 + single score → visible warning: "ReCiPe 2016 does not endorse single-score weighting. You're using borrowed Eco-Indicator 99 weights. Consider reporting category-level results instead." We don't silently stitch in unauthorized weights.

**P66. Plain-English method picker with "good fit" scoring.**
Not "ReCiPe 2016 Endpoint (H) V1.12 / World (2010) H/A" as a dropdown line. A proper dialog:
> 📏 LCIA Method Selection — Recommended for: [Corporate reporting ▾]
> ✅ EF 3.1 (PEF) — EU's recommended method. ⭐⭐⭐⭐⭐ for your use case
> ReCiPe 2016 — Widely cited in academia. ⚠️ Single-score not author-endorsed.
> IPCC AR6 GWP100 — Climate-only. ⭐⭐⭐⭐⭐ for CDP/SBTi.
> ...

**P67. Normalization baselines are explicit and current.**
World (2020), EU-27 (2020), user-supplied. Never silently use 2010 in a 2026 tool.

**P68. Method linting at study time.**
Before running: "Your inventory includes 1,117 substances ReCiPe 2016 cannot characterize. Of these, 4 are likely significant contributors: diuron, abamectin, acenaphthylene, glyphosate. Consider supplementing with USEtox 2.1."

**P69. Methods are versioned open resources.**
YAML/JSON, Git-versioned, with source papers, changelogs, diffs between versions.

**P70. Plain-English damage mapping.**
Damage assessment tab explains *why* the "1 × DALY/DALY" conversion is trivial and how DALYs are derived. Not a silent table of 1s.

**P71. Compare methods as a headline feature.**
"Run sensitivity analysis" button on results: one click runs inventory through ReCiPe + EF 3.1 + CML + IPCC → shows which conclusions are robust vs method-dependent. SimaPro's best hidden feature becomes our loudest one.

**P72. Method version pinning for reproducibility.**
Studies can be pinned to "ReCiPe 2016 v1.12 + ecoinvent 3.9.1" → bit-identical results forever, even as methods update.

**P107. Impact category colors are semantic, not decorative.**
Climate = red/orange (heat), water = blue, toxicity = purple, resources = brown, land use = green, ozone = violet. Users build intuition across studies. Hover → category name + explanation.

**P108. No truncated labels, ever.**
Rotate, wrap cleanly at word boundaries, or use abbreviations with tooltips. Never "Global warmin" on a boardroom deliverable.

**P109. Result granularity as a slider, not separate tabs.**
Single results view with granularity control: Full detail (18 categories) ↔ Damage areas (3) ↔ Single number. Chart morphs. Same data, chosen altitude. One concept, not five tabs.

**P110. Auto-detect scale mismatches; switch to log scale with a warning.**
Values spanning >3 orders of magnitude → "Linear scale makes the small one invisible. Switch to log?" Prevents unintentional deception.

**P111. Normalized results need units and explanation.**
Not "3.4e-4." Instead: "3.4 × 10⁻⁴ person-years — your coffee pot causes 1/2,940th of one average person's annual health damage." Human context.

**P112. Disclaimers travel with the numbers.**
Every chart using author-unendorsed weightings carries a visible footer: "⚠ Weightings from Eco-Indicator 99; ReCiPe 2016 authors do not endorse single-score weighting."

**P113. Drill-down is a side panel, never a modal.**
Click chart bar → side panel slides in with number, context, drill-down options. Chart stays visible. Panel dismissible, non-blocking, supports stacked navigation.

**P114. The three universal result lenses: substance / process / assumption.**
Every number offers:
- 🧪 What flows caused this? (substance decomposition)
- 🏭 Which processes contributed? (supply chain decomposition)
- 📐 What assumptions drove this? (parameter/method decomposition — novel to Arko)

**P115. Sensitivity as a standard lens.**
The third drill-down: "What if methane GWP100 were 28 instead of 34? What if electricity were Spanish not Japanese?" Novel feature; not in SimaPro.

**P116. Comparison charts show magnitude AND ratio.**
Default: dual-axis bars showing absolute values with ratio annotations ("NL: 80% of JP"). Toggle to pure ratio. Never hide the raw numbers.

**P117. Auto-generated comparison narrative.**
Above the chart: Claude-authored plain-English summary of trade-offs, grounded in the numbers, with actionable framing.

**P118. Variant color palette — high contrast, accessible.**
Clearly distinguishable colors (e.g., blue vs orange) with patterns/hatching for colorblind accessibility. Never two shades of the same hue.

**P119. "Winner" and "loser" badges per category.**
Green checkmark on lower-impact bars, red "⚠" on higher. Immediately scannable.

**P120. Trade-off analysis as a first-class view.**
Dedicated "Trade-off" tab: for each product pair, categories favoring A vs B, ranked by magnitude. Turns "no single winner" reality into a structured decision tool.

**P121. Comparison tables auto-highlight divergence.**
Sortable by largest absolute and relative difference. Cells color-coded by magnitude. "Show only divergent rows" toggle hides near-identical substances, focusing on what matters.

**P122. Headline numbers get headline treatment.**
Total score is a hero stat: huge typography, delta shown prominently. Contribution breakdown below, sorted by biggest contributor first.

**P123. Chart customization is inline, not nested in a Setup tab.**
Click a chart → edit-mode overlay with inline controls. No three-level-deep settings tabs.

**P124. Comparison linting specific to the comparison.**
"Among the 1,117 uncharacterized substances, 43 have asymmetric distributions between products — could shift the comparison if characterization were available. Significant: Glyphosate (40× higher in NL), Abamectin (1.6× higher in JP)."

**P125. Computation provenance as a signed, exportable manifest.**
Every result generates a manifest: "Computed using X processes from Library Y v3.9.1, Z methods from W v1.12, with parameter set P v2. SHA-256: ..." Auditable. Reproducible. Signed.

---

### 5.4 Methods, science transparency, and scientific rigor (Principles 73–79)

**P73. Methods as versioned open data.**
Every LCIA method stored as YAML/JSON with versioning, source papers, changelogs. Auditors see exactly what changed v1.11 → v1.12 and which factors updated.

**P74. Every factor has full provenance UI.**
Hover → source paper, uncertainty, last-updated, contested status.

**P75. Factor uncertainty is a first-class field.**
Not just a value; always a distribution. Monte Carlo uses these automatically. Users see "±σ" next to every number.

**P76. Honest single-score handling.**
Don't silently stitch in unauthorized weights. Surface the controversy.

**P77. Plain-English method picker.**
Not a raw dropdown. A guided selection with "good fit" scoring per use case.

**P78. Normalization baselines are explicit and current.**
World (2020) or later, EU-27 (2020), user-supplied. No silent 2010.

**P79. Method linting at study time.**
Pre-flight warnings about coverage gaps, with actionable suggestions for supplementary methods.

---

### 5.5 Control surface, discoverability, and efficiency (Principles 80–99)

**P80. Command palette as primary action surface.**
Cmd/Ctrl+K. Every action — create, compare, switch method, edit pedigree, export, compare methods — accessible by typing intent.

**P81. Contextual actions beat menu hierarchy.**
In a process editor, relevant actions appear as buttons/affordances on the screen. No digging through 7 menus.

**P82. No 1:1 duplication of toolbar and menus.**
Toolbars show curated frequent actions; menus show the complete set. Never the same 5 items twice.

**P83. Live calculation, always.**
Every edit to inventory, parameters, or method auto-recalculates in the background. Small changes → incremental updates <1s. No "Update expression results" button ever.

**P84. Five calculation modes → one unified Results workspace.**
Network, Tree, Analyze, Compare, Uncertainty are *views of the same computation*. Switching tabs never re-runs anything.

**P85. "Compare methods" promoted to first-class.**
Headline button on results: "Check method sensitivity." SimaPro's best hidden feature becomes our loudest.

**P86. Pedigree scoring as a first-class visual column.**
Every inventory row: 🟢🟢🟢🟡🔴 (one dot per dimension). Hover → 5 scores with explanations. Click → edit panel. Never buried in Edit menu.

**P87. Auto-generated uncertainty from pedigree.**
If a process has no explicit uncertainty distribution, we auto-derive lognormal distributions from pedigree scores (Weidema method). Monte Carlo works by default.

**P88. Data quality rollup to study level.**
Top-level "Data Quality Score" weighted by contribution. Displayed on results summary. Like a credit score for study credibility.

**P89. In-app contextual help, not PDFs.**
Every technical field has a "?" with a 2-sentence explanation + "Learn more" link to the deeper help in the right sidebar. No external PDF manuals.

**P90. Guided-tour onboarding for first-time users.**
Interactive first-run: "Let's build your first LCA together." Uses a real sample. By the end, user has a working study, a computed result, and has touched every major concept.

**P91. Tooltips with depth tiers.**
Hover → 1 sentence. Click → 1 paragraph. "Learn more" → full docs. Three tiers; never more than the user asked for.

**P92. Multi-user presence.**
See who else is in the study. Real-time cursors (Figma-style). Locks on objects being edited.

**P93. Comments inline.**
Any object (process, parameter, result) has threaded comments. "Why did you use 2010 data here?" — resolved inline, not in email.

**P94. Version history & branching.**
Every save creates a version. Diff versions ("what changed since Friday?"), revert, or branch for scenario exploration.

**P95. Sharing with permissions.**
Read-only link for clients (no software install). Commentable link for peer review. Full-edit for team members. SaaS default.

**P96. Web-first, with native-feeling performance.**
No installer, no COM server, no "Unregister COM server." Browser app that feels as fast as desktop (WASM for heavy math, incremental recalculation, local caching, offline mode for critical flows).

**P97. Methods and data as versioned open resources.**
Git-versioned. Pinnable. Reproducible.

**P98. Scriptable via modern API, not COM.**
REST + GraphQL + WebSocket + MCP. Python and Node SDKs. Webhooks for ERP/PLM. Day one, not decade thirty.

**P99. Matrix export as a standard endpoint.**
`GET /studies/{id}/matrix` → A-matrix + B-matrix + characterization factors as NumPy/CSV/MessagePack. Power users rejoice; no special clicks.

---

### 5.6 Data and license architecture (Principles 100–106)

**P100. Database-neutral architecture.**
Data layer treats ecoinvent, Agribalyse, user-uploaded, AI-generated, and KarbonGarbi-sourced data identically via an internal schema. No code paths assume ecoinvent.

**P101. License compliance as first-class architecture.**
Every process has a `license` field: `open`, `ecoinvent`, `agri-footprint`, `user-private`, `user-shared`. UI actions (publish, share, public link, embed) are gated by the most restrictive license in the calculation graph. Users *physically cannot* accidentally breach a license.

**P102. Open-data-first positioning.**
V1 markets openly: "Your data. Your license. Your reports. No vendor lock-in to a paywalled database." A feature, not a limitation.

**P103. Primary-data-first workflow.**
Users nudged to supply their own data (from KarbonGarbi, supplier questionnaires, measurements) before falling back to database defaults. Database factors become the fallback, not the default.

**P104. Provenance tier badges.**
Every number shows its tier:
- 🟢 Primary — user-measured or supplier-verified
- 🔵 Open database — free, redistributable
- 🟡 Paid database — ecoinvent etc., requires customer license
- 🟠 AI-estimated — generated by our models, flagged
- 🔴 Assumption — user-entered without source

Results show the tier mix: "78% of your climate impact is backed by primary + open data. 22% uses assumptions."

**P105. Static-report-safe mode.**
When a study includes restricted data, certain exports auto-switch to compliant modes: PDFs render static charts without underlying values in metadata, shareable links become view-only images, APIs redact licensed values. Architecture *enforces* license terms.

**P106. Compare mode validates it has something to compare.**
Running Compare on a single product → blocking warning: "Add a second variant to run a comparison." No more flat 100% bars that look meaningful but aren't.

---

### 5.7 Onboarding, pedagogy, and community (Principles 126–137)

**P126. Onboarding is inline and task-first, not video-and-orientation.**
First-time users are dropped into a guided "build your first LCA in 5 minutes" flow using a real sample or their own product. By minute 5, they have a result on screen. By minute 10, they've run a comparison.

**P127. Teach LCA by doing LCA, not before.**
In-context tooltips teach methodology when it matters: defining a functional unit → one-sentence explanation + "learn more." Picking allocation → same. Methodology learning is woven into use, never a prerequisite.

**P128. Generous free tier targeting SimaPro's pirate population.**
Free forever for: students (.edu verification), academic use, open-source projects, small projects (max N processes, M studies). Converts pirates and students into future paying professionals.

**P129. Community as a first-class product surface.**
Day-one Discord server. Project gallery (users publish studies with permission). In-app "ask the community" button. AI for common questions, escalating to community for hard ones. Community-authored template library.

**P130. In-product AI tutor replaces external videos.**
Persistent AI assistant grounded in the user's actual study: "Where should I add the transportation step?" → AI shows it and explains why in that context. Tutorials become unnecessary.

**P131. Content generated by frequent meaningful improvement.**
Every meaningful release gets a release note, a short demo video, and a community post. Continuous learning > annual webinar theater.

**P132. Ship with a worked-example gallery, not a tutorial series.**
30+ curated example studies (coffee, packaging, textiles, electronics, food, buildings, chemicals, cosmetics). One click to open as read-only template. Users learn by exploring real models, then fork.

**P133. The SimaPro tutorial view-count ranking is our feature backlog.**
Top tutorials = top user pain. Our priority:
1. Worked-example comparison → Comparison as first-class object (P47)
2. Parameters/scenarios → AI-assisted parameter authoring (P45)
3. Exchange projects with other users → Share-link-first collaboration (P95)
4. Importing from Excel → Native Excel drag-drop with AI column mapping
5. Copy process across projects → Universal clipboard, templating as primitive

**P134. Own the category thought leadership SimaPro abandoned.**
Publish weekly on: DPP implementation, CSRD/CBAM practicalities, Scope 3 best practices, AI-assisted LCA, French Affichage, Green Claims Directive. Every missed topic in their 14-year archive is our content gap to fill.

**P135. Developer content + API + integrations as a first-class pillar.**
Full docs for open API (REST, GraphQL, MCP). SDKs (Python, JS). Postman collections. Webhook guides. Importers (CSV, Excel, ILCD, EcoSpold, Brightway) with tutorials. Build the developer ecosystem SimaPro ignored.

**P136. Go-to-market window is NOW, narrowly.**
The One Click LCA / SimaPro acquisition creates a 12–18 month disruption window. Late 2026 to late 2027 is the moment. Miss it and the disruption settles.

**P137. Multilingual from day one.**
Spanish (home market + LatAm), then French (Affichage Environnemental), Portuguese (Brazil), German (industrial Europe). Not just UI strings — tutorials, AI responses, docs, community moderation. Spanish is the anchor. Massively underserved opportunity SimaPro structurally cannot pursue.

---

### 5.8 EPD workflow and document generation (Principles 138–152)

EPDs — Environmental Product Declarations — are the single highest-value real-world LCA use case. Construction EPDs alone have thousands published per program operator. CSRD, CBAM, Green Public Procurement, Digital Product Passport, and France's Affichage Environnemental all drive explosive EPD demand. SimaPro treats EPD creation as a general-purpose LCA workflow plus "tricks" (custom methods, group analysis, copy-paste to Word). Arko treats EPD creation as a first-class product.

**P138. EPDs as a first-class product, not a bolt-on report format.**
"EPD Mode" is an entire workflow. User selects target program operator (International EPD System, EPD Italy, EPD Norge, IBU, Environdec, etc.) and PCR → the study auto-structures per that PCR's requirements → impact assessment method pre-configured with exactly the required indicators → lifecycle stages pre-grouped per that PCR's rules → final EPD document auto-generated in the program operator's exact template.

**P139. PCR-aware method auto-configuration.**
Select a PCR from our library → auto-generate the LCIA method with exactly the required indicators, correctly configured, including non-LCA counter indicators (recycling rate, renewable energy content) wired through the model automatically. Zero manual method construction.

**P140. PCR library as a community-maintained versioned resource.**
Every published PCR from every major program operator, ingested and maintained. Versioned. Community contributions from consultants. Free access. This is a public good — and our moat.

**P141. Lifecycle module architecture is declarative.**
With EN 15804 selected, product stages auto-scaffold: A1 (raw materials), A2 (transport to manufacturer), A3 (manufacturing), A4 (transport to site), A5 (installation), B1–B7 (use), C1–C4 (end of life), D (benefits beyond system boundary). No dummy processes. No manual grouping. Data model reflects the standard.

**P142. Switching PCR reorganizes the model automatically.**
International EPD System (upstream/core/downstream) ↔ EN 15804 (A1–D) → same underlying model, re-projected into the new architecture. User doesn't rebuild anything.

**P143. Data quality is tracked as metadata, not reconstructed per-check.**
Every input has `data_source` field (`primary`, `secondary_generic`, `proxy`, `estimated`, `unknown`) and `pedigree` score. Compliance checks ("proxy <10%") are live dashboards. No manual group analysis required.

**P144. Pedigree scoring mandatory for EPD-bound studies.**
If EPD Mode is active, every input must have a pedigree score before finalization. Empty pedigree = blocking error. Forces the quality culture SimaPro admits is culturally absent.

**P145. Document generation is native and bidirectional.**
EPD Mode generates the complete EPD document (Word and PDF) automatically from the model, pre-filled with program-operator-correct templates. No copy-paste, no descriptor placeholders. When the model changes, the document changes. No Office add-in required.

**P146. Document review is collaborative and in-browser.**
Draft EPD reviewed and commented on in the browser (Google Docs model). Verification auditor gets a read-only shareable link, leaves inline comments, verifies sections independently. No "email Word file, wait for red-lines, merge manually."

**P147. Expert-scaffold, user-operate split.**
Expert users build **Study Templates** (scaffolded models with data-collection gaps). Business users fill in data via guided forms. On completion → auto-generated, verified-style EPD draft. Once-a-year process certification covers the template; individual EPDs produced continuously. SimaPro's 2021 future vision is our day-one product.

**P148. Native supplier data collection.**
Expert marks a parameter as "supplier-sourced" → generates a shareable link → supplier receives email with a tiny focused form (no Arko knowledge required) → submits → data flows live into the expert's model → expert validates before use. This is our "Collect" module, built native.

**P149. One product, one pricing tier includes everything.**
No add-ons. No separate "Report Maker" SKUs. Desktop-equivalent sync, supplier data collection, document generation, API access, verified EPD workflows — all included in each tier. One web app.

**P150. API-first architecture.**
Every UI feature is also a first-class API endpoint. Same EPD generation runs from the web UI and via REST. Automation isn't a separate product; it's the same product.

**P151. Program operator + PCR transparency built-in.**
Live-updated knowledge base: for every program operator, which product categories, which PCRs, whether they support process certification, fees, typical verification times. Users never have to ask "does X support Y?" — one search.

**P152. Community PCR drafting as a platform feature.**
If a PCR doesn't exist, users can kick off community PCR drafting within our platform: invite stakeholders, host open-consultation comments, draft structured rules, submit to program operators. We become the tool for PCR development, not just consumption. Positions us at the upstream of the ecosystem.

---

### 5.9 Strategic and architectural principles (Principles 153–154)

**P153. Our addressable market is the 100× pool blocked by current tooling supply.**
We don't steal SimaPro's 3,000 users. We enable the 300,000+ companies that need LCAs/EPDs but can't afford current tooling + consultant + timeline. Even 1% capture of the blocked market is 30× SimaPro's entire user base.

**P154. Execution is the moat, not ideas.**
Everyone in the LCA software industry knows what the ideal product should be — they've been describing it for years. The moat is shipping it. Speed, discipline, and modern tooling are our advantages. SimaPro is a 30-year codebase. We're a 2026 greenfield. This is the right moment.

---

## 6. MVP Scope — Phase 1 (Months 0–6)

The principles above describe the mature product. Phase 1 ships a narrow, deep slice that delivers genuine value to a specific use case while establishing the foundations for later expansion.

### What Phase 1 ships

**Core workflow:**
- Create a Study with clear Goal & Scope (wizard)
- Build a Model with Parts, inputs, outputs, emissions
- Canvas + Table views of the same model (editable in both)
- Link upstream processes via typeahead search (from open-data bundle)
- Live calculation (incremental recompute on edit)
- One Variant per Study (comparison in Phase 2)
- Results: Summary + Breakdown + Network + Sanity checks
- Export to PDF (executive + technical) and CSV (auditor pack)

**EPD Mode (single program operator to start):**
- International EPD System + EN 15804 (the two most requested)
- PCR library with ~20 of the most-used PCRs
- Auto-method configuration from PCR
- Auto-scaffolded lifecycle stages
- Draft EPD document generation in the program operator's template
- Pedigree scoring (required in EPD mode)
- Data quality compliance dashboard

**Data bundle (free tier + included in paid):**
- Agribalyse 3.1.1 (food and agriculture)
- USDA LCA Commons (US industry)
- ELCD 3.2 (European reference)
- USEEIO v2 (US environmentally-extended IO)
- PEFAPs (EU Product Environmental Footprint defaults)
- Basque industrial factors (primary data from KarbonGarbi ecosystem)
- Plus user-uploaded custom factors

**Method bundle (all open):**
- ReCiPe 2016 (H)
- EF 3.1 (PEF)
- IPCC AR6 GWP100
- USEtox 2.13
- CML-IA baseline
- TRACI 2.1
- AWARE (water scarcity)

**Collaboration (minimal):**
- Shareable links with permissions (view / comment / edit)
- Basic inline comments
- Version history (read-only initially)

**Platform:**
- Web-native, Chrome/Safari/Firefox/Edge supported
- Shared Goibix SSO (reuse KarbonGarbi auth)
- Spanish + English UI day 1

### What Phase 1 explicitly does NOT ship

- Ecoinvent integration (V2, after reseller negotiation)
- AI layer (V2 — comes online in Months 9–12)
- Real-time multi-cursor collaboration (V2)
- Full Monte Carlo uncertainty UI (inputs present; full UX in V2)
- Multiple variants in a single study (V2)
- Custom PCR drafting workflow (V3)
- Mobile app (web-responsive but not native)
- French / Portuguese / German UI (Phase 2; just Spanish + English at launch)
- Third-party app marketplace (V3)
- Offline mode (V3; PWA caching in V2)

### Phase 1 success criteria

- 3 paying customers by Month 6
- At least 1 completed EPD submitted via Arko to a program operator
- 50+ free-tier users active
- Imanol Bollegui uses Arko for one of his sustainability-technician clients at Bilbao Ekintza
- Net Promoter Score ≥30 among early users
- Average study-to-first-result time ≤15 minutes for a returning user

---

## 7. Technical Architecture

### Shared Goibix backbone (reuse, don't rebuild)

Arko reuses from KarbonGarbi:

- **Authentication and SSO** (Supabase Auth, existing tenant model)
- **Tenant management** (KarbonGarbi's Phase B.2 infrastructure)
- **Billing and subscriptions** (KarbonGarbi's Phase C / C.1, Redsys for EU payment)
- **Super Admin / support panel** (Phase B, Phase J)
- **Security hardening** (HSTS, CSP, rate limiting, RLS — Phase H)
- **Legal pages framework** (Phase I, under Spanish law / GDPR)
- **Feature flags** (Phase G)
- **Lifecycle cron engine** (Phase E)
- **Emission factors infrastructure** — extended for LCA factors with immutable versioning (building on KarbonGarbi's Phase B.1)
- **Supplier data collection flow** — adapted for LCA parameters (builds on KarbonGarbi's supplier A–E scoring)

### Arko-specific additions

- **LCA calculation engine** — likely Brightway 2.5+ (Python, open source, battle-tested) wrapped as a service, or a custom Rust/WASM engine for browser-side computation on small studies
- **Graph canvas UI** — ReactFlow for the visual supply chain editor
- **Data bundle management** — ingestion pipelines for Agribalyse, USDA, ELCD, etc. into a unified internal schema
- **Method library** — YAML/JSON versioned repository of LCIA methods
- **PCR library** — structured PCR documents with required-indicator declarations
- **Document generation** — Word/PDF rendering via docx-js (Word) and Puppeteer/Weasyprint (PDF)
- **License enforcement layer** — middleware that evaluates the license-mix of every publish/share/export action

### Technology stack (continuing the Goibix pattern)

- **Frontend:** Next.js on Vercel (same as KarbonGarbi and the marketing site)
- **Backend:** FastAPI on Hetzner (same as KarbonGarbi)
- **Database:** Supabase PostgreSQL EU West (same as KarbonGarbi)
- **Calculation engine:** Brightway 2.5 Python library, wrapped as a FastAPI microservice; for small studies (<50 nodes), consider WASM port for client-side computation
- **Graph rendering:** ReactFlow (Canvas view), AG Grid or TanStack Table (Table view)
- **Document generation:** docx-js for Word, Puppeteer or Weasyprint for PDF, custom renderer for EPD program-operator templates
- **Edge:** Cloudflare WAF/CDN (same as KarbonGarbi)

### Data model at the schema level (indicative)

```sql
-- Simplified schema sketch
create table workspace ( ... );  -- tenant
create table study ( ... );  -- the central object
create table variant ( ... );  -- 1..n per study
create table lifecycle_stage ( ... );  -- A1..D or custom
create table model ( ... );  -- graph node
create table flow ( ... );  -- edge: inputs/outputs, with license/provenance fields
create table parameter ( ... );  -- with scope enum
create table library ( ... );  -- process/method/PCR libraries
create table process ( ... );  -- library entries
create table method ( ... );  -- versioned LCIA methods
create table pcr ( ... );  -- product category rules
create table result ( ... );  -- cached computations with manifest
create table deliverable ( ... );  -- generated documents, exports
create table pedigree ( ... );  -- quality scores per flow
create table comment ( ... );  -- threaded, attached to any object
create table version ( ... );  -- history per study
```

Every mutable object gets a `license` tag; every reference to a library entry carries its provenance.

### Deployment topology (continuing Goibix pattern)

```
Cloudflare (DNS, WAF, CDN)
  │
  ├── arko.earth (primary) → Vercel (Next.js frontend)
  │     └── same app as KarbonGarbi's deployment, separate project
  │
  └── api.arko.earth → Hetzner (FastAPI backend)
        ├── /studies, /models, /libraries, /users → PostgreSQL (Supabase)
        ├── /calculate → Brightway microservice (Python)
        ├── /documents → document-generation microservice (Node)
        └── /enforce → license-compliance middleware
```

---

## 8. Legal and Licensing Architecture

### The ecoinvent problem, solved structurally

Ecoinvent EULA v3 (April 2022) imposes restrictions that cannot be navigated by goodwill alone. They must be enforced by architecture. Specifically:

- **Clause 7.1(e)** — cannot publish/disclose ecoinvent data through software tools to third parties
- **Clause 7.1(f)** — cannot use for consumer-facing comparisons, e-shops, or regulated declarations without separate licensing
- **Clause 7.1(b)** — cannot resell, rent, or act as intermediary
- **Clause 9** — CHF 100,000 penalty per breach
- **Clause 6.1 / 6.2** — reports must be static, graphical, not reverse-engineerable

### How Arko handles this

1. **License as a required field** — every Process, Flow, and Method reference has a `license` field. Arko's internal schema enforces it.
2. **License gating of actions** — publish, share-link, public-export, API-read-for-third-party are all gated at the middleware level by the most restrictive license in the study's calculation graph.
3. **Tenant-held licenses, pass-through** — each customer who wants ecoinvent signs an ecoinvent agreement directly (or via Arko as an authorized reseller in V2). Arko doesn't wholesale ecoinvent data.
4. **Compliant export modes** — PDFs render static charts without underlying values in metadata when licensed data is present; shareable links become view-only images; APIs redact licensed values.
5. **Static-report-safe mode as default** for any document containing licensed data.
6. **Audit log** — every license-sensitive action is logged: who accessed what ecoinvent data, when, for which study, in which tenant.

### What this enables

**V1 (open data only):** full freedom — Arko's own open-data results can go anywhere: public APIs, consumer-facing comparisons, e-shop widgets, Digital Product Passports. This is a feature unique to open-data-first architectures.

**V2 (ecoinvent as premium tier):** customers with their own ecoinvent licenses can use ecoinvent data inside Arko. Our architecture ensures their compliance. Premium pricing includes the compliance guarantee.

**V3 (Arko as authorized ecoinvent reseller):** we negotiate reseller status with ecoinvent Association. Customers can buy ecoinvent access through Arko as a single bill. Architecture is already ready.

### Trademark and corporate

- **Goibix S.L.** (parent company) files Arko trademark in EU (EUIPO) in Nice classes 9 (software) and 42 (scientific services). Budget: ~€900 for two classes, plus ~€900 for optional class 35 (if we offer consulting). Not urgent; before major marketing spend.
- **Domain strategy:** `arko.earth` primary; `arko.eus` redirect (cultural heritage, Basque SEO); register `arko.app` and `arko.eu` defensively if available.
- **No conflict** with ARKO Corp (Nasdaq: ARKO, US gas stations) — different trademark classes, different geography of primary trading.

---

## 9. Go-to-Market Hypothesis

### The wedge sequence

**Wave 1 — Warm network (Months 0–6).** Samir's existing contacts in the Basque Country: Imanol Bollegui at Bilbao Ekintza, KarbonGarbi warm prospects who mention product-level questions, Samir's network of Basque industrial SMEs. Goal: 3 paying customers, 1 published EPD.

**Wave 2 — Spanish-speaking LCA consultancies (Months 6–12).** Small-to-medium consultancies in Spain and LatAm currently torturing SimaPro or using spreadsheets. Pitch: "SimaPro pro deliverables, Notion UX, in Spanish, at 1/5 the price." Goal: 20 paying customers, 10+ EPDs published.

**Wave 3 — European SME manufacturers (Months 12–18).** Mid-market manufacturers needing CSRD / EPD / DPP compliance. Pitch: "Your first EPD in 2 weeks, not 6 months. Your CSRD product-level disclosures, ready." Goal: 100 paying customers.

**Wave 4 — DPP-preparing manufacturers (Months 18–24).** As the EU Digital Product Passport mandate lands in 2027 for textiles, batteries, and electronics, mass demand spikes. Pitch: "DPP-ready LCAs, API-accessible, multilingual." Goal: 500 customers, Enterprise tier launches.

**Wave 5 — Global consultancies, partnerships, ecoinvent reseller (Month 24+).** Large consultancies, enterprise deployments, strategic partnerships with program operators and industry associations.

### Key distribution channels

- **Environmental consultancies as the multiplier** (KarbonGarbi's identified GTM insight, equally applicable to Arko). One consultant adopting Arko brings 10–20 client engagements.
- **Content / thought leadership** — weekly writing on the abandoned topics: DPP, CSRD, CBAM, Green Claims, Affichage Environnemental, AI-assisted LCA. Own the SEO that SimaPro abandoned.
- **Conferences** — SETAC Europe (the LCA academic and practitioner conference) is the single best venue. Early presence (sponsored workshop, not a booth) in 2027.
- **Open-source / academic goodwill** — generous free tier for universities and students creates a pipeline of future professional users who learned on Arko.
- **Community-authored templates and PCRs** — viral-adjacent growth as consultants share templates and attribute them to Arko.

### Pricing hypothesis (to validate)

| Tier | Price | Target | Includes |
|------|-------|--------|----------|
| Free | €0 | Students, open-source, curious | 1 study at a time, max 20 models, open data only, watermarked exports |
| Studio | €49/mo | Individual consultants, designers | 10 active studies, full open-data bundle, unwatermarked exports, EPD Mode (1 program) |
| Team | €299/mo | Small consultancies, internal sustainability teams (up to 5 users) | Unlimited studies, collaboration, comments, version history, all EPD program operators |
| Enterprise | Custom (€2,000–8,000/mo) | Mid-to-large manufacturers, large consultancies | SSO, audit trails, SLA, ecoinvent pass-through (V2), API at scale, dedicated support |

Validate: 10 conversations with target buyers in each tier before locking prices.

### Key competitive moves we expect

- **One Click LCA integrates SimaPro's tech stack** — likely takes 2–3 years. During this period their combined offering is confusing; we exploit.
- **One Click LCA launches a cloud SimaPro** — probable outcome. When it lands, our differentiation shifts from "we're web-native, they aren't" to "we're AI-native, they aren't" plus "we're open-data-first, they aren't."
- **Ecochain Mobius raises more funding, expands** — stays focused on manufacturing, doesn't move toward open data. We differentiate on openness and community.
- **OpenLCA launches a hosted version** — possible. They'll target the technical user; we target the practitioner. Different markets.
- **A new AI-native entrant appears** — likely within 12–18 months. Our advantage is first-mover + Goibix platform + Samir's category fluency.

---

## 10. Brand, Name, and Voice

### Name: Arko

- **Basque:** "arc" / "bow" — the lifecycle arc of a product
- **Nepali:** "another" / "alternative" — Arko is the other way to do LCA (Samir's native language)
- **Romance languages (ES/IT/PT):** "arco" = arc, universally recognized
- **English:** pronounces identically to "arko" in every target language
- **Trademark:** clear in Nice classes 9 and 42 (software, scientific services); no conflict with US-based ARKO Corp

### Domain strategy

- **Primary:** `arko.earth` — climate-tech TLD signals positioning immediately
- **Cultural:** `arko.eus` redirect — honors Basque roots, SEO bonus for Basque / Spanish search
- **Defensive (register if available):** `arko.app`, `arko.eu`, `getarko.com`, `@arko` handles on X, LinkedIn, Bluesky, GitHub

### Tagline candidates

- "Life cycle assessment, built for modern product decisions."
- "The open alternative for life cycle assessment."
- "LCA without the learning curve."
- "Life cycle assessment, reimagined."

Lead candidate: **"LCA without the learning curve."** — direct, lands the positioning in one line, contrasts with SimaPro's reputation.

### Voice and tone

- **Clear over clever.** Never a cute joke where a sentence of explanation would help.
- **Respectful of expertise.** We know LCA is real science. We never talk down to practitioners. We make things simpler without making them simplistic.
- **Honest about uncertainty.** When a number has ±20% error bars, we say so. This builds the trust SimaPro squanders.
- **Global in reach, European in character.** Not Valley-bro. Not corporate-speak. Confident, grounded, slightly understated.
- **Multilingual from day one in product and marketing.** Spanish native, not translated.

---

## 11. Risks and Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Samir distracted from KarbonGarbi by Arko excitement | High | High | Commitment: do not start Arko until KarbonGarbi has 3–5 paying customers. Spec is banked. |
| Ecoinvent refuses reseller status in V3 | Medium | Medium | Open-data-first V1 means we don't depend on it. Many alternatives are improving. We can stay database-neutral indefinitely. |
| One Click LCA ships a cloud SimaPro faster than expected | Medium | High | Ship Arko Phase 1 by Q1 2027. Differentiation shifts to AI-native + open-data + EPD-first. |
| Open data quality is insufficient for serious EPDs | Medium | High | Pilot 2 EPDs in Phase 1 using open data; if critical gaps, negotiate early ecoinvent reseller for V2. |
| AI hallucinations in the interpretation layer | Medium | High | Ground every AI statement in specific calculated values, show provenance, never generate factors. Conservative defaults; visible "AI-assisted" badge. |
| Consultancies don't adopt / act as gatekeepers | Medium | Medium | Direct SME sales motion alongside consultancy channel. Free tier creates bottom-up demand consultancies have to respond to. |
| Regulatory requirements demand specific tool certification (e.g., PEF Representative Product methods) | Medium | Medium | Design from day one for audit, provenance, reproducibility. Pursue certification when product-market fit justifies the spend. |
| Goibix backbone limits what Arko can do | Low | Medium | Design explicit extension points; accept some duplication where KarbonGarbi patterns don't fit. |
| Funding runway (solo founder, full-time day job) | High | Medium | Start only after KarbonGarbi has revenue; target profitability quickly via the Studio/Team tiers; consider friends-and-family or grants (SPRI, CDTI Neotec, EIC Accelerator) only if needed. |
| Technical co-founder gap (Samir is solo) | High | Medium | Consider bringing in a technical partner for Arko specifically once KarbonGarbi is shipping revenue. Use this Master Spec as the shared vision document. |

---

## 12. 18-Month Roadmap

### Right now (April–July 2026) — DO NOT BUILD ARKO

- Close Imanol Bollegui for KarbonGarbi
- Ship KarbonGarbi marketing site
- Onboard 3–5 paying KarbonGarbi customers
- Begin KarbonGarbi's environmental-consultancy channel
- Validate pricing, positioning, adoption for KarbonGarbi
- Register `arko.earth` and `arko.eus` (cheap insurance; ~€30/yr)
- File EUIPO trademark for Arko in classes 9 + 42 (~€1,800)

### Arko Month 0 (August 2026)

- Decision gate: "Do we have evidence KarbonGarbi is working?" If yes, proceed. If no, pause Arko another quarter.
- Architecture sprint: shared Goibix backbone extension for LCA data model
- Data bundle ingestion begins (Agribalyse, ELCD, USDA)
- Method library: ingest ReCiPe 2016 open CSV, EF 3.1 open Excel
- Pick calculation engine: likely Brightway 2.5 wrapped as FastAPI service
- First UI prototypes of the three-concept data model (Study, Model, Library)

### Arko Months 1–3 (September–November 2026)

- Core modeling interface: Canvas + Table views
- Graph calculation engine hooked up
- Basic results views: Summary + Breakdown
- First closed alpha: 5–10 invited users (including Imanol)
- Spanish and English UI from day one
- Marketing site launch (not live until ready for external signups)

### Arko Months 4–6 (December 2026 – February 2027)

- EPD Mode v1 (International EPD System + EN 15804)
- PCR library with ~20 PCRs
- Document generation (Word and PDF) for draft EPDs
- Pedigree scoring, data quality dashboard
- Comparison between Variants
- First paying customer target
- Aim for 1 published EPD via Arko

### Arko Months 7–9 (March–May 2027)

- AI layer v1: interpretation narrative, hotspot detection, natural-language query
- Collaboration: shareable links, inline comments, version history
- Excel import with AI column mapping
- Community Discord launch + first worked-example gallery (20 studies)
- Open beta
- 20–50 paying customers target

### Arko Months 10–12 (June–August 2027)

- Monte Carlo uncertainty UI
- Compare Methods as first-class feature
- Supplier data collection workflow (port from KarbonGarbi's adjacent muscle)
- French UI launch
- 100 paying customers target
- First consultancy partnership formalized

### Arko Months 13–18 (September 2027 – February 2028)

- Ecoinvent reseller negotiations (begin)
- Enterprise tier launch (SSO, audit, SLA, API at scale)
- DPP export formats as EU mandate takes effect
- Portuguese and German UI
- 300+ paying customers target
- First €50k MRR milestone

---

## 13. What success looks like

### Six months after Arko launch

- 50+ paying customers, mostly Spanish-speaking consultancies and Basque industrial SMEs
- At least 5 published EPDs generated via Arko
- Imanol Bollegui using Arko at Bilbao Ekintza for at least one client
- Discord community with 200+ members and active weekly discussion
- 30+ worked-example studies in the public gallery
- A "we built this in 6 months" story credible enough to publish

### Eighteen months after Arko launch

- 300+ paying customers
- €50k+ MRR combined across Studio, Team, Enterprise tiers
- First Fortune-500-adjacent Enterprise customer (likely a Basque champion like Sidenor or a mid-sized Spanish industrial group)
- First consultancy partner bringing 20+ clients
- Arko named in at least one European sustainability publication / conference as a credible modern alternative to SimaPro
- A measurable flow of SimaPro customers switching — the disruption window exploited
- Ecoinvent reseller conversations advanced

### Three years after Arko launch

- Market-defining product for modern LCA in Europe
- 2,000+ paying customers (roughly parity with SimaPro's entire paid base, in 1/10 the time)
- Multilingual (EN, ES, FR, PT, DE, IT) with local partner networks in each
- Goibix platform emerges as a recognized family of sustainability products
- Samir is no longer solo; Goibix has 10–15 people across both products
- Arko + KarbonGarbi combined ARR above €5M
- Acquisition interest from larger sustainability platforms OR credible path to Series A

---

## 14. Appendix — The Evidence Base

This specification is grounded in the following decoded evidence:

- **14+ SimaPro screens decoded** in detail (Home, LCA Explorer, Substances, Processes tree, Process editor Documentation tab, Process editor Input/output tab, Product stages, Assembly editor, Parameters, Calculation setups list, New calculation setup dialog, Network diagram, Impact assessment characterization chart, Inventory, Process contribution, Checks, Product overview, Setup chart options, Methods General tab, Methods Characterization tab, Methods Damage assessment tab, Methods Normalization and Weighting tab, Compare single-product, Compare two-product, Compare Inventory, Compare Process contribution, Compare Setup, Compare Checks, Compare Product overview).
- **Complete menu + toolbar map** extracted across all 5 top-level menus (File, Edit, Calculate, Tools, Window, Help) and the full icon toolbar.
- **Ecoinvent EULA v3 (April 2022)** — full legal analysis of 14 clauses.
- **Tutorial #1 (two versions: Spanish community tutorial + English "LCA with SimaPro 8: Tutorial 1")** decoded for pedagogy signals.
- **Complete SimaPro YouTube channel content archive (61 videos, 14 years)** — view counts, dates, titles, and content type distribution analyzed.
- **Full transcript of the "Creating EPDs with SimaPro" webinar** (9,600 views, 6th most-viewed on the channel), featuring PRé's own Customer Service lead, their Italian partner consultant of 15 years, and their SimaPro Product Owner.

Every principle in this spec traces to specific observations in that evidence base. The document is falsifiable: if new evidence invalidates a principle, the principle updates.

---

*End of Arko Master Specification v1.0.*
*Banked for return after KarbonGarbi reaches 3–5 paying customers.*
*Goibix S.L., Bilbao, April 2026.*
