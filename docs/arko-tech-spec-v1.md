# Arko — Technical Architecture Specification v1.0

**The complete technical blueprint for Arko — a web-native, offline-capable, AI-assisted life cycle assessment platform.**

---

**Document status:** Draft v1.0 — April 18, 2026
**Companion to:** Arko Master Specification v1.0
**Author:** Samir (Goibix S.L., Bilbao) with Claude
**Audience:** Future technical co-founder, CTO, senior engineers, external auditors, investors performing technical due diligence
**Scope:** Every technical decision, stack choice, and architectural pattern needed to build Arko from Month 0 to Month 36. Intentionally rigorous and intentionally opinionated.

---

## Table of Contents

1. Architectural Philosophy and Principles
2. System Overview and Component Map
3. Frontend Architecture
4. Backend Architecture
5. The Calculation Engine (the hardest part)
6. Data Model and Database Schema
7. Real-time Collaboration and Sync
8. Offline-First and PWA Architecture
9. AI Integration Layer
10. Document Generation Pipeline
11. Security Architecture
12. Data Licensing Enforcement
13. Observability, Monitoring, and SLOs
14. DevOps, CI/CD, and Deployment
15. Development Workflow and Tooling
16. Third-party Dependencies (full vendor list)
17. Cost Modeling Across Scale Tiers
18. Scaling Plan and Capacity Thresholds
19. Migration Paths and Technical Debt Strategy
20. Risk Register and Mitigations
21. Open Technical Questions for Future Resolution
22. Appendix: Quick-reference cheat sheets

---

## 1. Architectural Philosophy and Principles

Before the specific choices, the meta-decisions that shape everything else.

### 1.1 The ten architectural commitments

**A1. Web-native, not desktop.** Arko is a browser application. There is no desktop installer, no COM server, no `.arko` file format. Users work in the browser from day one. Tauri-wrapped desktop distribution is a possibility for specific Enterprise customers in V2+, but the source of truth is always the web app.

**A2. Offline-capable, not offline-primary.** Users can work offline for hours. The app must never require a round-trip to function for a solo editor on a single study. But the default mental model is "cloud-first with local cache," not "local-first with cloud sync." This matters for UX copy, for conflict resolution defaults, and for how we market the product.

**A3. Calculation at the edge of the client when possible.** Small studies (<100 models, <5,000 flows) compute in the browser via WebAssembly. Users get sub-100ms recalculation on every edit. Only large studies round-trip to the Python calc service.

**A4. Multi-tenant by default.** Every row in every table carries a `workspace_id`. Row-level security enforces isolation at the database layer. Escape hatches (super admin access) are explicit, logged, and rare.

**A5. API-first internally.** The web app has no "private" API. Every feature the UI calls is available to Enterprise customers via documented REST/GraphQL endpoints. This forces good boundaries and prevents the UI-coupled backend spaghetti that plagues legacy software.

**A6. Python for science, TypeScript for product, Rust optional for hot paths.** Don't fight the ecosystem. Brightway and the scientific LCA stack are Python; we use Python. React and the product UX stack are TypeScript; we use TypeScript. If a specific hot path (in-browser matrix solver) benefits from Rust, we write Rust for that one thing. We don't rewrite Brightway in Rust.

**A7. Open-source where it exists, boring where it doesn't.** We pick mature, boring tech for infrastructure (Postgres, Redis, FastAPI, Next.js). We pick open-source where a mature option exists (Brightway, ReactFlow, Yjs). We avoid novel frameworks, unreleased tools, and anything that requires us to be "early adopters." Arko is a product company, not a framework showcase.

**A8. One database, one cache, one queue.** No polyglot persistence in V1. Postgres for everything structured, Redis for everything ephemeral, S3-compatible storage (Cloudflare R2) for blobs. Adding a graph database or time-series DB before we have customers would be premature.

**A9. EU data residency, always.** All customer data, all backups, all processing happens in the EU. This is non-negotiable for GDPR, CSRD customers, and our brand positioning. Hetzner (Finland, Germany) and Supabase (EU-West) comply. We never use a US-region service for customer data, even for a nightly backup.

**A10. Every number has provenance; every mutation has a user.** No anonymous data transformations. No orphan calculations. The audit log is a first-class feature, not an afterthought bolted on for SOC 2.

### 1.2 The anti-principles (what we explicitly reject)

**NOT microservices from day one.** We start as a modular monolith with three services: the main API, the calc service, and the doc generation service. We split further only when load or team size demands it. Fifty microservices on a three-person team is suicide.

**NOT event-sourced architecture.** We considered it for the audit log. We're rejecting it for V1. Traditional CRUD + append-only audit log gets us 95% of the benefit at 20% of the complexity. Reconsider at Wave 3.

**NOT Kubernetes.** Docker Compose on Hetzner dedicated servers via Coolify gets us to 1000 tenants. Kubernetes is overhead we don't need yet. Reconsider when we have a dedicated platform engineer.

**NOT serverless functions for the core API.** Cold starts kill UX on a calculation-heavy app. We run warm Python processes on Hetzner. Cloudflare Workers at the edge only, for auth/routing.

**NOT a proprietary binary file format.** All exports are open formats (JSON, CSV, ECOSPOLD, ILCD, docx, pdf). Users must never feel locked into Arko.

**NOT a forked version of Brightway.** We use Brightway upstream and contribute back. If we need custom behavior, we contribute a feature flag or monkey-patch at our service boundary.

**NOT on-premise installations before we have revenue.** The ops burden of supporting self-hosted customers before Year 3 would consume a founding engineer. Cloud-only for V1 and V2.

### 1.3 Performance budgets

Concrete targets that every architectural choice must serve:

| Operation | Target p50 | Target p95 | Non-goal |
|---|---|---|---|
| First page load (cold cache) | 1.5 s | 3.0 s | — |
| First page load (warm cache) | 400 ms | 800 ms | — |
| Canvas interaction (pan/zoom) | 16 ms (60 fps) | 33 ms (30 fps) | — |
| Typeahead search | 50 ms | 150 ms | — |
| Live re-calculation (small study) | 100 ms | 300 ms | full sensitivity |
| Live re-calculation (medium study) | 500 ms | 1.5 s | — |
| Full Monte Carlo (1,000 iterations, medium study) | 20 s | 60 s | real-time |
| EPD document generation | 5 s | 15 s | — |
| Offline → online sync (typical) | 2 s | 10 s | — |

Any proposed feature or library choice that violates these budgets goes back to the drawing board.

---

## 2. System Overview and Component Map

### 2.1 High-level topology

```
                    ┌─────────────────────────────────┐
                    │         User's browser          │
                    │  (desktop, tablet, mobile PWA)  │
                    └──────────────┬──────────────────┘
                                   │ HTTPS, WSS
                                   ▼
        ┌──────────────────────────────────────────────────┐
        │            Cloudflare (global edge)              │
        │  WAF · DDoS · CDN · DNS · Workers · R2 storage   │
        └──────────────────────┬───────────────────────────┘
                               │
        ┌──────────────────────┼───────────────────────────┐
        ▼                      ▼                           ▼
 ┌──────────────┐    ┌─────────────────────┐    ┌──────────────────┐
 │  Vercel      │    │  Hetzner (Finland)  │    │  Supabase EU-W   │
 │  Next.js     │◄───┤  API · Calc · Docs  │◄───┤  Postgres + Auth │
 │  frontend    │    │  Redis · Celery     │    │  Storage · Edge  │
 └──────────────┘    └──────────┬──────────┘    └──────────────────┘
                                │
                                ▼
                    ┌──────────────────────┐
                    │  Anthropic API       │
                    │  (Claude, EU region  │
                    │   when available)    │
                    └──────────────────────┘
```

### 2.2 Component inventory

**Frontend apps:**
- `arko-web` — Next.js 15 App Router, deployed on Vercel
- `arko-mobile` — React Native Expo (V2, for field data collection and study viewing)
- `arko-desktop` — Tauri wrapper around arko-web (V3, only if specific Enterprise customers require it)

**Backend services:**
- `arko-api` — FastAPI main API, handles CRUD, auth delegation, orchestration
- `arko-calc` — FastAPI + Brightway, handles heavy LCA calculations, Monte Carlo, sensitivity
- `arko-docs` — Node.js + docx-js + Puppeteer, generates EPD documents, PDFs, Excel exports
- `arko-sync` — Node.js + y-websocket, handles real-time collaboration CRDT sync
- `arko-ingest` — Python workers, scheduled jobs for library updates (Agribalyse releases, method version bumps)

**Data stores:**
- **Postgres 16** (Supabase EU-West) — primary database, all structured data
- **Redis 7** (Hetzner) — pub/sub for real-time, cache for hot reads, Celery broker
- **Meilisearch 1.x** (Hetzner) — full-text and faceted search for processes
- **Cloudflare R2** — S3-compatible object storage for document exports, study snapshots, user uploads
- **pgvector** (extension of Postgres) — embedding search for semantic process matching

**Supporting infrastructure:**
- **Cloudflare Workers** — edge auth, rate limiting, geo-routing
- **Sentry** — error tracking (self-hosted on Hetzner for EU residency)
- **Grafana stack (Prometheus + Loki + Tempo)** — metrics, logs, traces (self-hosted)
- **Doppler** — secrets management
- **GitHub Actions** — CI/CD

**Third-party APIs:**
- **Anthropic API** — Claude for interpretation and narrative
- **Stripe or Redsys** — billing (Redsys for EU SEPA, Stripe for international; shared with KarbonGarbi)
- **Postmark or Resend** — transactional email
- **Cloudflare Turnstile** — CAPTCHA alternative for public forms

### 2.3 Request flow examples

**User loads a study:**
1. Browser → Cloudflare (cached shell) → IndexedDB has cached study → render immediately
2. In parallel: browser → `api.arko.earth/v1/studies/:id` → Cloudflare edge → `arko-api` (Hetzner) → Postgres → response
3. If data changed on server: React Query invalidates cache, UI updates. If not: no-op.
4. WebSocket connects to `arko-sync` for real-time updates on this study.

**User edits an input quantity:**
1. Frontend: optimistic UI update (Zustand + React Query mutation)
2. Frontend: queue the mutation in a local outbox (IndexedDB) in case offline
3. Frontend: if online, POST to `arko-api`; on success clear outbox entry
4. Backend: validates, persists to Postgres, emits a Yjs update to `arko-sync`
5. `arko-sync`: broadcasts to other connected users of this study
6. Frontend (this user): WASM calc engine recomputes impact in-browser, updates summary
7. If the study is too large for WASM: fire-and-forget request to `arko-calc`, show stale result with a loading pip, swap in fresh result when it arrives

**User clicks "Generate EPD":**
1. Frontend → `arko-api` → create a `deliverable` record, enqueue a Celery job
2. Celery worker → fetch study → call `arko-calc` for fresh results → call `arko-docs` to render docx + pdf
3. Rendered files → uploaded to R2 → URL stored in `deliverable` record → status "ready"
4. Frontend polls or receives push via WebSocket → download button becomes active
5. User downloads; R2 serves with a signed URL that expires in 1 hour

**User asks Claude "What's driving my climate impact?":**
1. Frontend → `arko-api/ai/query` with study_id and question
2. Backend: fetch study results, metadata, provenance
3. Backend: construct a prompt with tool definitions (`get_contribution_breakdown`, `query_similar_processes`, etc.)
4. Backend → Anthropic API → streamed response
5. Backend streams through to frontend via Server-Sent Events
6. Frontend renders token by token

---

## 3. Frontend Architecture

### 3.1 Framework and core stack

**Next.js 15 App Router** is the foundation. Reasons:
- **Consistent with KarbonGarbi** — Samir already knows it, Goibix pattern established
- **React Server Components** — the dashboard, study list, public example gallery, marketing site render on the server for fast first paint
- **Streaming SSR** — large study pages stream their shell and fill in expensive sections
- **Route-level code splitting** — the canvas editor is heavy; only loaded when needed
- **Built-in i18n** — we need Spanish, English, French, German, Portuguese support
- **Vercel integration** — preview deployments per PR, edge network for European users, zero infra for the frontend

**React 19** — concurrent rendering, transitions, improved suspense. The canvas specifically benefits from `useTransition` for smooth interactions.

**TypeScript 5.x strict mode** — every file typed. No `any` except behind a commented ESLint escape. Types are generated from the OpenAPI schema so the frontend always matches the backend.

### 3.2 Routing structure

```
app/
  (marketing)/                  # public marketing site, no auth
    page.tsx                    # landing
    pricing/
    examples/                   # public worked-example gallery
    blog/
    docs/                       # in-product docs
  (auth)/
    login/
    signup/
    invite/[token]/
  (app)/                        # authenticated, tenant-scoped
    layout.tsx                  # sidebar, topbar, workspace picker
    dashboard/                  # home
    studies/
      page.tsx                  # list
      new/                      # goal & scope wizard
      [studyId]/
        page.tsx                # study overview
        model/                  # model editor (canvas + table)
        results/                # results workspace
        deliverables/           # exports, EPDs, reports
        history/                # version history, diff
        settings/
    library/
      processes/
      methods/
      pcrs/
      templates/
    team/
    billing/
  (share)/                      # public share links, read-only or commentable
    s/[token]/                  # view a shared study
  api/                          # Next.js route handlers (thin BFF layer)
    auth/[...nextauth]/         # auth callback
    ai/stream/                  # SSE proxy for Claude
```

**Note:** the `api/` routes in Next.js are **thin BFF** — they handle auth session, SSE streaming, cookie management. They do not implement business logic. Business logic lives in `arko-api` (FastAPI). This keeps the backend reusable for SDK and mobile app.

### 3.3 State management

Three categories of state, handled by three tools:

**Server state: TanStack Query v5.**
- Every API call goes through a typed query hook generated from OpenAPI
- Automatic caching, refetching, optimistic updates, offline support
- Integrates with IndexedDB via `@tanstack/query-persist-client-core` for offline persistence
- Query invalidation on mutations is explicit

**Client state: Zustand.**
- One global store with slices per feature (canvas state, modal state, user preferences)
- Persisted slices go to localStorage (UI preferences) or IndexedDB (canvas viewport, selection)
- No Redux. Zustand's simpler API and smaller footprint win for a solo-to-small team

**Real-time collaborative state: Yjs.**
- The study model itself is a Y.Doc
- Changes made locally apply to the Y.Doc first, then propagate to other users
- Also the source of truth for conflict-free merging when coming back online
- Integrates with the Zustand store via an adapter

### 3.4 The canvas editor (the centerpiece)

The graph canvas is Arko's most technically demanding UI component. Specs:

**Library: ReactFlow 12.**
- Handles 1,000+ nodes with virtualization
- Pan, zoom, fit-to-screen, minimap out of the box
- Custom node types via React components
- Hooks for programmatic control

**Layout: ELK.js** for auto-layout when a model is first opened, Dagre for incremental layouts when adding a node.

**Custom node types:**
- `AssemblyNode` — a user-built model with Parts
- `ProcessNode` — a linked library process (read-only except for amount)
- `ElementaryFlowNode` — a leaf (CO2 to air, water from river) — rendered differently from processes
- `StageNode` — a lifecycle stage container (A1, A3, etc.)
- `VariantNode` — a variant container for comparisons
- `ParameterNode` — floating nodes that represent parameters for visual formula building (V2)

**Custom edge types:**
- `MassFlowEdge` — thickness proportional to mass flow, label with amount + unit
- `EnergyFlowEdge` — dashed, styled differently
- `CoproductEdge` — fan-out from a multi-output process with allocation percentages
- `AvoidedProductEdge` — different color, indicates substitution

**Interactions:**
- Drag from a node's output port → creates a new edge; dropping on empty space opens a search for what to connect to
- Shift+click → multi-select; bulk operations
- Right-click → context menu with node-specific actions
- Cmd+A select all in a stage; Cmd+D duplicate; Del delete
- Keyboard shortcuts for every common operation

**Performance optimizations:**
- `React.memo` on every node component
- Node positions stored in Yjs, not React state, to avoid re-renders on pan/zoom
- Virtualization via ReactFlow's native support; only visible nodes render
- Canvas re-renders throttled to 60 fps via `useTransition`

### 3.5 The table editor (the other centerpiece)

When a user switches from Canvas view to Table view, same data, different UI.

**Library: TanStack Table v8 + TanStack Virtual.**
- Column virtualization for 50+ columns
- Row virtualization for 100,000+ rows
- Cell-level editing with optimistic updates
- Keyboard navigation (Excel-like: arrow keys, Tab, Enter)

**Features:**
- Columns: Part, Flow type (link/elementary), Name, Amount, Unit, Distribution, SD², Min, Max, Pedigree (5 dots), License tier, Data year, Comments
- Column show/hide with persisted preference
- Faceted filters as chips above the table
- Bulk edit: select rows, click "Edit in bulk", change any column
- Paste from Excel: detects CSV/TSV in clipboard, maps columns intelligently (AI-assisted), previews, confirms
- Copy to Excel: formatted output that round-trips

### 3.6 Component library

**shadcn/ui** as the base component library — the same choice as KarbonGarbi. Headless Radix UI primitives styled with Tailwind. We own the code (it's copied into our repo, not imported as a dependency) so we can customize freely.

**Tailwind CSS 4.x** with a custom design token system (color palette, spacing scale, typography scale) defined in `tailwind.config.ts`. Design tokens are also exported to Figma via Figma Tokens for design-dev alignment.

**Icons: Lucide React** (via shadcn). Consistent across all Goibix products.

**Charts: a layered approach:**
- **Recharts** for standard charts (bars, lines, stacked bars) — quick wins for 80% of result displays
- **D3.js directly** for custom visualizations (the network diagram alternative to ReactFlow, Sankey diagrams for flows, custom trade-off matrices)
- **Visx (Airbnb)** for complex hybrids that need D3 power with React ergonomics

**Fonts:**
- Inter Variable for UI (same as KarbonGarbi)
- JetBrains Mono for code/values/numbers
- Self-hosted via Vercel's font optimization, no Google Fonts external request

### 3.7 Progressive Web App

The frontend installs as a PWA. Specifics:

**Manifest** — `manifest.json` with icons for all platforms, display mode `standalone`, theme color matching our brand.

**Service Worker** — `sw.js` generated by Workbox. Strategies:
- Static assets (JS, CSS, fonts): Cache-First, 1 year
- API GET requests: Stale-While-Revalidate with 5-minute TTL
- API POST/PUT/DELETE: Network-Only, fails gracefully to offline queue
- HTML shell: Network-First with cache fallback

**Offline outbox** — mutations made while offline queue in IndexedDB. On reconnect, replayed in order with conflict detection.

**Install prompt** — after the third visit, a dismissible banner offers to install. Respects Do Not Track.

---

## 4. Backend Architecture

### 4.1 The main API service (`arko-api`)

**Framework: FastAPI 0.110+ on Python 3.12.**
- Async throughout (no sync endpoints in V1)
- Pydantic v2 for validation and serialization
- Automatic OpenAPI 3.1 schema generation
- Consistent with KarbonGarbi; Samir knows it

**Structure:**

```
arko-api/
  app/
    main.py                   # FastAPI app factory
    config.py                 # settings via pydantic-settings
    dependencies.py           # common deps: get_current_user, get_workspace
    middleware/
      auth.py                 # JWT verification, workspace scoping
      rate_limit.py           # per-endpoint Redis rate limits
      audit.py                # mutations → audit log
      license_enforcement.py  # the critical one; see §12
    routers/
      studies.py
      models.py
      library.py
      methods.py
      pcrs.py
      deliverables.py
      team.py
      billing.py
      ai.py
      webhooks.py
    services/
      study_service.py        # business logic
      calc_service.py         # client for arko-calc
      docs_service.py         # client for arko-docs
      ai_service.py           # client for Anthropic
      billing_service.py
      license_service.py
    models/                   # SQLAlchemy models
    schemas/                  # Pydantic schemas
    db/
      session.py
      migrations/             # Alembic
    tasks/                    # Celery tasks
    workers.py                # Celery worker entry point
```

**API conventions:**
- All endpoints under `/v1/` — we version from day one
- `/v2/` for breaking changes, with `/v1/` supported for 12 months after deprecation
- Resource-oriented URLs: `GET /v1/studies/:id`, `POST /v1/studies/:id/variants`
- Cursor-based pagination (not offset) for stable results: `?cursor=abc&limit=50`
- Consistent error envelope: `{ "error": { "code": "FORBIDDEN", "message": "...", "details": {...} } }`
- Every endpoint documents its permission requirements in the OpenAPI spec

**Auth pattern:**
- Supabase Auth issues JWTs with standard claims + custom `workspace_id`, `role`, `licenses[]`
- `arko-api` verifies JWTs locally using Supabase's public keys (no call to Supabase for each request)
- A middleware extracts `workspace_id` and injects it into every query (defense in depth atop RLS)

**Per-endpoint rate limits** via `slowapi` backed by Redis:
- Read endpoints: 100 req/min per user
- Write endpoints: 30 req/min per user
- AI endpoints: 10 req/min per user (also cost-metered)
- Export/doc generation: 5 req/min per workspace
- Public share endpoints: 20 req/min per IP (tighter, DDoS-prone)

### 4.2 Background jobs

**Celery 5.x with Redis as broker and result backend.**

**Task categories:**
- **Calc jobs** — heavy calculations, Monte Carlo. Queue: `calc-queue`, separate workers on CPU-rich nodes
- **Document jobs** — EPD/PDF generation. Queue: `docs-queue`
- **Ingest jobs** — library updates (nightly Agribalyse sync, method version bumps). Queue: `ingest-queue`, separate workers
- **Mail jobs** — transactional email via Postmark. Queue: `mail-queue`
- **Billing jobs** — invoice generation, subscription lifecycle. Queue: `billing-queue`
- **AI jobs** — batch embedding generation for new processes, offline ML tasks. Queue: `ai-queue`

**Why separate queues:** a stuck doc-generation job should not block a user's subscription renewal. Isolation matters.

**Task properties:**
- All tasks are idempotent (retry-safe)
- `task_id` returned to the client for status polling or WebSocket push
- Progress emitted via Redis pub/sub for real-time UI updates
- Failed tasks after 3 retries go to a dead-letter queue for manual review
- Soft time limits (warn, cleanup) and hard time limits (kill)

### 4.3 The realtime sync service (`arko-sync`)

**Stack: Node.js 22 + y-websocket + Redis pub/sub.**

This is a thin service. Its only job:
1. Hold a WebSocket connection per user per study
2. Relay Yjs updates from one client to all other clients viewing the same study
3. Periodically snapshot the Y.Doc state to Postgres (every 30 seconds or on user disconnect)
4. Handle presence (who's online, cursor positions)

**Why Node, not Python:** y-websocket is a JavaScript library, has the most battle-tested server implementation, and WebSocket-heavy workloads are Node's sweet spot. Different stack from arko-api is fine — this service is small and changes rarely.

**Scaling:** Redis pub/sub broadcasts across multiple `arko-sync` instances, so we horizontally scale by adding nodes behind a WebSocket-aware load balancer (Cloudflare supports WS, HAProxy sticky sessions for intra-cluster routing).

### 4.4 The document generation service (`arko-docs`)

**Stack: Node.js 22 + docx-js + Puppeteer + custom template engine.**

Responsibilities:
1. Render Word documents from an EPD template + study data
2. Render PDF versions via Puppeteer (headless Chromium) from HTML templates
3. Generate Excel exports using ExcelJS
4. Generate CSV and JSON exports natively

**Why Node:**
- docx-js is a Node library (the best in class for programmatic Word generation)
- Puppeteer's best integration is Node-native
- Templates are easier to express in TSX + CSS than Python equivalents

**Template system:**
- Each EPD program operator has one or more templates stored in `templates/epd/{operator}/{version}/template.tsx`
- Templates are TSX files that compile to HTML + CSS
- `docx-js` generates Word by walking the same AST
- New program operators require a new template but no core code changes

### 4.5 The ingest and library service

**Stack: Python 3.12 + Celery + custom ETL pipelines.**

Responsibilities:
1. Nightly sync of open data bundles (Agribalyse, USDA LCA Commons, ELCD, PEFAPs)
2. Monthly check for method updates (RIVM's ReCiPe, JRC's EF, IPCC)
3. Ingest PCR documents submitted by community PRs
4. Re-index Meilisearch after library changes
5. Re-compute embeddings for semantic search

**Why a separate service:** ingest jobs are long-running (hours) and resource-intensive. Running them on the main API cluster would interfere with user requests. They also have different scaling needs (1 worker at a time for schema-migration-type jobs, parallel workers for bulk processing).

---

## 5. The Calculation Engine (the hardest part)

This is where the product lives or dies. Get this right and everything else follows; get it wrong and we're building on sand. Let me spend more time here.

### 5.1 The problem, precisely

An LCA calculation, mathematically, is a linear algebra problem:

Given:
- **A-matrix** (technosphere): `n × n` square matrix where `A[i,j]` is the amount of product `j` required to produce one unit of product `i`. Mostly sparse.
- **B-matrix** (biosphere): `m × n` matrix where `B[k,j]` is the amount of elementary flow `k` emitted per unit of product `j`
- **f-vector** (functional unit): `n × 1` vector specifying the product demand; usually 0s with a 1 in the position of the study's reference product
- **C-matrix** (characterization): `p × m` matrix where `C[l,k]` is the characterization factor for flow `k` in impact category `l`

The calculation:
1. Solve `A · s = f` for `s` (the scaling vector) — this is a sparse linear solve
2. Compute `g = B · s` — the life cycle inventory (elementary flows)
3. Compute `h = C · g` — the impact results per category

For ecoinvent-scale problems, `n ≈ 20,000` and `m ≈ 2,000`. The `A` matrix is ~99.9% sparse.

For our V1 open-data bundle, `n ≈ 5,000` and `m ≈ 2,000`.

For a typical single-study calculation, we solve for one `f` but the supply chain recursion lives inside the `A⁻¹ f` computation.

### 5.2 Implementation choice: Brightway 2.5 (or Brightway 25 / bw25)

**Why Brightway:**
- Open source, active maintenance, community of contributors
- NumPy/SciPy native, uses SuperLU or UMFPACK for sparse solves
- Correct implementation of LCA math with decades of academic scrutiny
- Handles edge cases (circular references in the supply chain, allocation, avoided products) correctly
- Used internally by ecoinvent for their own validation
- Has a proper data format (Brightway's `Database`, `Activity`, `Exchange` model) that we can adapt
- The most widely-taught calculation engine in academic LCA

**Why NOT roll our own:**
- A correct sparse solver handling the subtle edge cases of LCA (negative production, circular exchanges, zero-demand products) is 6+ months of work
- The gains from a custom implementation would be marginal (maybe 2-3x speedup) and come at the cost of correctness bugs
- We need to focus our engineering effort on product, not on reinventing battle-tested math

**Why NOT olca-ipc (OpenLCA's):**
- Tied to OpenLCA's process model, harder to decouple
- Less Pythonic, more verbose
- Smaller community than Brightway

### 5.3 The calc service architecture (`arko-calc`)

```
arko-calc/
  app/
    main.py                   # FastAPI app
    routers/
      calculate.py            # POST /calculate
      monte_carlo.py          # POST /monte-carlo
      sensitivity.py          # POST /sensitivity
      compare.py              # POST /compare
      matrix_export.py        # GET /matrix (power users)
    services/
      brightway_adapter.py    # wraps Brightway calls
      arko_to_bw.py           # converts Arko study → Brightway project
      bw_to_arko.py           # converts Brightway results → Arko schema
      cache.py                # memoization of expensive solves
    models/                   # Pydantic request/response
  data/
    libraries/                # Brightway projects for each open-data bundle
      agribalyse_3_1_1/
      uslci_2_0/
      elcd_3_2/
      pefaps_latest/
      methods/
```

**Request flow for a standard calculation:**
1. `arko-api` collects the full study model (a JSON structure)
2. `arko-api` POST to `arko-calc/calculate` with `{study_json, method, options}`
3. `arko-calc` converts the study JSON into a transient Brightway project (in-memory, uses Brightway's IO-minimal "functional unit + matrix overrides" pattern)
4. `arko-calc` runs the calc, extracts results, converts back to our schema
5. Response includes: results per impact category, contribution tree, top hotspots, uncertainty estimates (from pedigree)
6. Response cached in Redis for 10 minutes keyed by `hash(study_json)` + method version

**Request flow for Monte Carlo:**
1. Same as above, but `calculate` → `monte_carlo` with iteration count
2. Celery task spawned; returns `task_id` immediately
3. Worker runs N iterations in parallel using Ray or multiprocessing
4. Progress events emitted every 5% to Redis pub/sub → WebSocket → UI progress bar
5. Result object stored in Redis; client polls or receives push

### 5.4 In-browser calculation for small studies

The real differentiator. For studies under 100 models, we compute in the browser.

**How:**
- A trimmed-down calc engine in Rust, compiled to WebAssembly
- Exposes a simple API: `wasm.calculate(study_json, method_id, library_hash)`
- The current study's linked library processes are pre-loaded into browser memory (compact binary format, ~5 MB for open-data bundle subset relevant to the study)
- Characterization factors for the selected method also pre-loaded
- Actual solve takes <50 ms for typical small studies on mid-range hardware

**Why Rust + WASM and not Python via Pyodide:**
- Pyodide is ~10 MB compressed and slow to initialize
- A focused Rust crate does just the matrix solve, ~500 KB compressed, initializes in <100 ms
- We still validate correctness by comparing Rust-WASM results against Brightway on a test suite of 1000+ studies

**Fallback logic:**
- If the study has more than 100 models OR uses features not supported in WASM (advanced Monte Carlo, certain allocation methods), the UI shows "Running calculation" and hits `arko-calc`
- User doesn't have to know which path — it's transparent

### 5.5 Incremental recalculation

The user changes one number. We don't recompute the whole supply chain from scratch; we update the affected parts.

**Technique: pre-computed inverse and Sherman-Morrison updates.**
- On first calculation of a study, compute `A⁻¹` and cache it
- For a single-cell change `δ` in `A`, we can compute the updated inverse in O(n²) via the Sherman-Morrison formula instead of O(n³) full resolve
- For small studies in WASM: this turns 50 ms into 2 ms on subsequent edits

**Cache invalidation:**
- Cache keyed on `(study_version, method_version, library_version)`
- On any structural change (adding/removing a model or flow), cache invalidates and a full resolve runs
- On amount-only changes (the common case — users tweak numbers), incremental updates apply

**User-visible behavior:**
- A small pulse animation on the result summary number when it updates
- Result number stays showing the old value for <50 ms to avoid flicker on rapid edits
- If recalc takes >500 ms (large study or network round-trip), a subtle progress pip appears

### 5.6 Monte Carlo and uncertainty

**Pedigree-derived uncertainty (automatic):**
- Every input to every process has pedigree scores (5 dimensions, 1-5 scale)
- We apply the Weidema matrix to convert pedigree to a lognormal GSD² (geometric standard deviation squared)
- During Monte Carlo, each input is sampled from its distribution

**User-specified uncertainty (explicit):**
- Users can override the default distribution on any input
- Supported distributions: Normal, Lognormal, Triangular, Uniform, Discrete
- Exposed in the "Uncertainty mode" of the table/canvas editor

**Monte Carlo runs:**
- Default 1,000 iterations (sufficient for 5% confidence on most results)
- Advanced: 10,000 iterations for publication-grade uncertainty
- Parallel execution via Ray actors; linear scaling up to CPU core count
- Output: mean, median, 2.5th and 97.5th percentiles, histogram data, correlation matrix between inputs and outputs

**Sensitivity analysis (complementary to MC):**
- Local sensitivity: compute ∂(result)/∂(input) for each input via automatic differentiation; rank inputs by absolute sensitivity
- Global sensitivity via Sobol indices for high-dimensional studies (V2)

### 5.7 Matrix export for power users

Per Principle P99, we expose the underlying matrices via API:

```
GET /v1/studies/:id/matrix?format=numpy
GET /v1/studies/:id/matrix?format=csv
GET /v1/studies/:id/matrix?format=mpk  (MessagePack, for Python clients)
```

Returns:
- `A.npz` — technosphere matrix in scipy sparse format
- `B.npz` — biosphere matrix
- `labels.json` — index-to-name mapping for both matrices
- `method.json` — characterization vectors per impact category
- `study.json` — functional unit, parameters

Academic users and power users can do their own analysis in MATLAB, R, or Python without our UI. SimaPro hides this in a File menu; we make it a documented API endpoint.

### 5.8 Validation and correctness

This is LCA — a single wrong number in a client's EPD could cost them reputation or a regulatory fine. Correctness discipline:

- **Reference test suite:** 1,000+ studies with known correct results from SimaPro and OpenLCA. Our engine must match to 6 significant digits.
- **Property-based testing** via Hypothesis (Python) on the Brightway adapter — random studies, invariants like "result is non-negative for non-negative inputs"
- **Differential testing:** on every CI run, compare our Rust-WASM engine output against Brightway's Python output on the test suite; any divergence >1e-9 fails the build
- **Numerical tolerance handling:** document explicitly what tolerances are acceptable and surface them in the UI (e.g., "results accurate to ±0.01%")
- **Bug bounty** for result correctness in V2 — pay users who find mathematical errors

### 5.9 What if Brightway breaks or we outgrow it

**Backup plan:** olca-ipc (OpenLCA's calculation engine, Python bindings). Similar enough that the adapter pattern lets us swap.

**Eventual custom engine:** if at Wave 3 we consistently hit Brightway's performance ceiling (likely at ecoinvent scale with Monte Carlo), we write a custom Rust engine for the hot path, keep Brightway as a reference implementation for correctness. Not V1.


---

## 6. Data Model and Database Schema

### 6.1 Schema overview

All structured data lives in Postgres 16 (Supabase EU-West). We use PostgreSQL-native features heavily: JSONB for schemaless per-entity extensions, row-level security for tenancy, pgvector for embeddings, generated columns for common computed values, triggers for audit.

### 6.2 Core tables

```sql
-- ================================
-- Identity and tenancy
-- ================================

create table workspace (
  id uuid primary key default gen_random_uuid(),
  slug text unique not null,                    -- arko.earth/ws/{slug}
  name text not null,
  plan text not null default 'free',            -- free, studio, team, enterprise
  subscription_id text,                         -- billing reference
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now(),
  settings jsonb not null default '{}'::jsonb,
  -- feature flags, locale preferences, default method, etc.
  deleted_at timestamptz
);

create table workspace_member (
  workspace_id uuid not null references workspace(id) on delete cascade,
  user_id uuid not null references auth.users(id) on delete cascade,
  role text not null,                           -- owner, admin, editor, viewer
  joined_at timestamptz not null default now(),
  invited_by uuid references auth.users(id),
  primary key (workspace_id, user_id)
);

create table workspace_license (
  -- tracks which paid libraries this workspace has access to
  workspace_id uuid not null references workspace(id) on delete cascade,
  library_id uuid not null references library(id),
  license_proof_url text,                       -- link to uploaded license doc
  expires_at timestamptz,
  added_at timestamptz not null default now(),
  primary key (workspace_id, library_id)
);

-- ================================
-- Studies (the central object)
-- ================================

create table study (
  id uuid primary key default gen_random_uuid(),
  workspace_id uuid not null references workspace(id) on delete cascade,
  
  title text not null,
  description text,
  status text not null default 'draft',         -- draft, in_review, finalized, archived
  intended_use text,                             -- internal, epd, marketing, academic, regulatory
  
  -- goal & scope
  functional_unit text,
  functional_unit_amount numeric,
  functional_unit_unit text,
  system_boundary text,                         -- cradle_to_gate, cradle_to_grave, etc.
  geographic_scope text[],                      -- ISO 3166 codes
  time_horizon_start date,
  time_horizon_end date,
  
  -- methodology
  method_id uuid references method(id),
  method_version text,                          -- pinned for reproducibility
  pcr_id uuid references pcr(id),
  pcr_version text,
  
  -- epd-specific
  epd_mode boolean not null default false,
  epd_program_operator text,                    -- 'environdec', 'ibu', 'epd_norge', etc.
  
  -- data quality requirements
  proxy_data_threshold numeric,                 -- e.g., 0.10 for 10%
  pedigree_required boolean not null default false,
  
  -- tracking
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now(),
  created_by uuid not null references auth.users(id),
  last_edited_by uuid references auth.users(id),
  deleted_at timestamptz,
  
  -- search
  search_vector tsvector generated always as (
    to_tsvector('simple', coalesce(title, '') || ' ' || coalesce(description, ''))
  ) stored,
  
  -- enforce tenancy
  constraint study_workspace_fk check (workspace_id is not null)
);

create index study_workspace_idx on study(workspace_id) where deleted_at is null;
create index study_search_idx on study using gin(search_vector);
create index study_updated_idx on study(updated_at desc);

-- row-level security
alter table study enable row level security;
create policy study_tenant_isolation on study
  using (workspace_id = current_setting('app.current_workspace_id')::uuid);

-- ================================
-- Variants
-- ================================

create table variant (
  id uuid primary key default gen_random_uuid(),
  study_id uuid not null references study(id) on delete cascade,
  name text not null,                           -- "Sima", "Pro", "2024 formulation"
  description text,
  order_index int not null default 0,
  is_baseline boolean not null default false,   -- one per study can be marked baseline
  created_at timestamptz not null default now()
);

create index variant_study_idx on variant(study_id);

-- ================================
-- Lifecycle stages
-- ================================

create table lifecycle_stage (
  id uuid primary key default gen_random_uuid(),
  variant_id uuid not null references variant(id) on delete cascade,
  code text not null,                           -- 'A1', 'A3', 'B1', 'C4', 'D', or custom
  name text not null,                           -- human-readable
  framework text not null default 'en15804',    -- 'en15804', 'iso_upstream_core_downstream', 'custom'
  order_index int not null default 0,
  description text
);

create index lifecycle_stage_variant_idx on lifecycle_stage(variant_id);

-- ================================
-- Models (the graph nodes)
-- ================================

create table model (
  id uuid primary key default gen_random_uuid(),
  variant_id uuid references variant(id) on delete cascade,
  lifecycle_stage_id uuid references lifecycle_stage(id),
  parent_model_id uuid references model(id),    -- for sub-assemblies / parts
  
  kind text not null,                            -- 'assembly', 'part', 'unit_process', 'reference'
  name text not null,
  description text,
  
  -- reference flow
  reference_flow_name text,
  reference_flow_amount numeric,
  reference_flow_unit text,
  
  -- canvas position (Yjs syncs this; db snapshots)
  canvas_x numeric,
  canvas_y numeric,
  
  -- library link (for linked processes from external libraries)
  linked_process_id uuid references process(id),
  linked_process_version text,
  
  -- provenance
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now(),
  created_by uuid references auth.users(id),
  
  -- extensions
  metadata jsonb not null default '{}'::jsonb
);

create index model_variant_idx on model(variant_id);
create index model_parent_idx on model(parent_model_id);

-- ================================
-- Flows (inputs and outputs of a model)
-- ================================

create table flow (
  id uuid primary key default gen_random_uuid(),
  model_id uuid not null references model(id) on delete cascade,
  
  direction text not null,                      -- 'input', 'output', 'emission', 'waste'
  category text not null,                       -- 'technosphere', 'nature', 'air', 'water', 'soil', 'non_material'
  subcategory text,                             -- 'urban', 'rural', 'stratosphere', etc.
  
  -- what the flow is
  flow_kind text not null,                      -- 'linked_process', 'elementary', 'custom'
  linked_model_id uuid references model(id),    -- for internal links within the study
  linked_process_id uuid references process(id), -- for library links
  elementary_flow_id uuid references elementary_flow(id),
  custom_name text,                             -- for user-defined without library link
  
  -- amount
  amount numeric not null,
  unit text not null,
  
  -- uncertainty
  distribution text,                            -- null, 'lognormal', 'normal', 'triangular', 'uniform'
  distribution_params jsonb,                    -- {'gsd_squared': 1.2} for lognormal, etc.
  
  -- pedigree
  pedigree_reliability int check (pedigree_reliability between 1 and 5),
  pedigree_completeness int check (pedigree_completeness between 1 and 5),
  pedigree_temporal int check (pedigree_temporal between 1 and 5),
  pedigree_geographic int check (pedigree_geographic between 1 and 5),
  pedigree_technological int check (pedigree_technological between 1 and 5),
  
  -- provenance & license (see §12)
  data_source text,                             -- 'primary', 'secondary_generic', 'proxy', 'estimated', 'unknown'
  license_tier text not null default 'open',    -- 'open', 'ecoinvent', 'agri_footprint', 'user_private', 'user_shared'
  data_year int,
  data_region text,
  
  -- allocation (for multi-output processes)
  allocation_method text,                       -- 'physical', 'economic', 'energy', 'mass', 'none'
  allocation_factor numeric,                    -- 0.0 to 1.0
  
  -- tracking
  comment text,
  order_index int not null default 0,
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);

create index flow_model_idx on flow(model_id);
create index flow_linked_process_idx on flow(linked_process_id);

-- ================================
-- Parameters (variables for scenarios)
-- ================================

create table parameter (
  id uuid primary key default gen_random_uuid(),
  scope_type text not null,                     -- 'workspace', 'study', 'variant', 'model'
  scope_id uuid not null,                       -- polymorphic; the scope's id
  
  name text not null,                           -- unique within scope
  value numeric,
  unit text,
  formula text,                                 -- if computed, e.g., 'hours_per_day * days_per_year'
  description text,
  
  -- uncertainty (same as flow)
  distribution text,
  distribution_params jsonb,
  
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now(),
  
  unique (scope_type, scope_id, name)
);

create index parameter_scope_idx on parameter(scope_type, scope_id);

-- ================================
-- Libraries (external data)
-- ================================

create table library (
  id uuid primary key default gen_random_uuid(),
  code text unique not null,                    -- 'agribalyse_3.1.1', 'ecoinvent_3.9.1_cutoff'
  name text not null,
  description text,
  version text not null,
  license_tier text not null,                   -- 'open', 'ecoinvent', 'agri_footprint', 'custom'
  source_url text,                              -- where this library came from
  license_terms_url text,                       -- URL to license document
  process_count int not null default 0,
  elementary_flow_count int not null default 0,
  ingested_at timestamptz not null default now(),
  active boolean not null default true
);

-- ================================
-- Processes (library entries)
-- ================================

create table process (
  id uuid primary key default gen_random_uuid(),
  library_id uuid not null references library(id) on delete cascade,
  external_id text not null,                    -- id within the source library
  
  name text not null,
  reference_product text,
  geography text,                               -- ISO 3166 or UN M49 region
  allocation_method text,
  system_model text,                            -- 'cutoff', 'apos', 'consequential'
  
  -- full data
  unit text,
  reference_flow_amount numeric,
  
  -- search
  search_vector tsvector generated always as (
    to_tsvector('simple', coalesce(name, '') || ' ' || coalesce(reference_product, '') || ' ' || coalesce(geography, ''))
  ) stored,
  embedding vector(1024),                        -- for semantic search via pgvector
  
  -- metadata (full process data as JSON; queryable subsets above)
  data jsonb not null,
  
  -- provenance
  data_year int,
  source_citation text,
  
  unique (library_id, external_id)
);

create index process_library_idx on process(library_id);
create index process_search_idx on process using gin(search_vector);
create index process_embedding_idx on process using hnsw(embedding vector_cosine_ops);
create index process_geography_idx on process(geography);
create index process_name_trgm_idx on process using gin(name gin_trgm_ops);

-- ================================
-- Elementary flows (substances)
-- ================================

create table elementary_flow (
  id uuid primary key default gen_random_uuid(),
  name text not null,
  cas_number text,
  compartment text not null,                    -- 'air', 'water', 'soil', 'natural_resource'
  subcompartment text,                          -- 'urban', 'freshwater', etc.
  unit text not null,                           -- 'kg' typically
  formula text,
  synonyms text[],
  
  search_vector tsvector generated always as (
    to_tsvector('simple', coalesce(name, '') || ' ' || coalesce(cas_number, '') || ' ' || array_to_string(coalesce(synonyms, '{}'::text[]), ' '))
  ) stored,
  
  unique (name, compartment, subcompartment)
);

create index elementary_flow_search_idx on elementary_flow using gin(search_vector);
create index elementary_flow_cas_idx on elementary_flow(cas_number);

-- ================================
-- Methods (LCIA)
-- ================================

create table method (
  id uuid primary key default gen_random_uuid(),
  code text unique not null,                    -- 'recipe_2016_endpoint_h', 'ef_3_1', 'ipcc_ar6_gwp100'
  name text not null,
  description text,
  version text not null,
  perspective text,                             -- 'hierarchist', 'individualist', 'egalitarian', 'n/a'
  normalization_region text,                    -- 'world_2010', 'eu_27_2020', etc.
  author text,                                  -- 'RIVM', 'EU JRC', 'IPCC'
  source_url text,
  source_publication text,
  changelog_url text,
  license text not null default 'open',
  ingested_at timestamptz not null default now()
);

create table impact_category (
  id uuid primary key default gen_random_uuid(),
  method_id uuid not null references method(id) on delete cascade,
  code text not null,                           -- 'global_warming', 'water_consumption'
  name text not null,                           -- 'Global warming'
  plain_english_name text,                      -- 'Climate change contribution'
  icon text,                                    -- lucide icon name
  unit text not null,                           -- 'kg CO2 eq'
  damage_category text,                         -- 'human_health', 'ecosystems', 'resources'
  order_index int not null default 0,
  unique (method_id, code)
);

create table characterization_factor (
  id uuid primary key default gen_random_uuid(),
  method_id uuid not null references method(id) on delete cascade,
  impact_category_id uuid not null references impact_category(id) on delete cascade,
  elementary_flow_id uuid not null references elementary_flow(id) on delete cascade,
  factor numeric not null,
  uncertainty_distribution text,
  uncertainty_params jsonb,
  source_citation text,
  unique (method_id, impact_category_id, elementary_flow_id)
);

create index cf_lookup_idx on characterization_factor(method_id, elementary_flow_id);

-- ================================
-- PCRs (Product Category Rules)
-- ================================

create table pcr (
  id uuid primary key default gen_random_uuid(),
  code text unique not null,                    -- 'pcr_2019_14_seats', 'pcr_en15804_concrete'
  program_operator text not null,               -- 'environdec', 'ibu', 'epd_norge'
  category text not null,                       -- 'seats', 'concrete', 'chairs'
  title text not null,
  version text not null,
  effective_date date,
  expires_date date,
  
  -- required impact categories (as codes from a method)
  required_impact_codes text[] not null,        -- ['global_warming', 'ozone_depletion', ...]
  
  -- lifecycle stage framework
  stage_framework text not null,                -- 'en15804_a1_d', 'iso_upstream_core_downstream'
  
  -- data quality rules
  data_quality_rules jsonb,                     -- {proxy_threshold: 0.10, primary_required: true, ...}
  
  -- full PCR document
  document_url text,
  structured_rules jsonb,                       -- the PCR content in structured form
  
  active boolean not null default true
);

create index pcr_category_idx on pcr(category);
create index pcr_program_idx on pcr(program_operator);

-- ================================
-- Results (cached computations)
-- ================================

create table result (
  id uuid primary key default gen_random_uuid(),
  study_id uuid not null references study(id) on delete cascade,
  variant_id uuid references variant(id),
  
  method_id uuid not null references method(id),
  method_version text not null,
  
  -- what was computed
  calculation_type text not null,               -- 'deterministic', 'monte_carlo', 'sensitivity'
  
  -- results as structured JSON
  -- { impact_results: {...}, contribution_tree: [...], hotspots: [...], uncertainty: {...}, manifest: {...} }
  payload jsonb not null,
  
  -- reproducibility
  manifest_hash text not null,                  -- sha256 of inputs for cache lookup
  computed_at timestamptz not null default now(),
  computation_duration_ms int,
  
  -- TTL (results expire as underlying data changes)
  valid_until timestamptz
);

create index result_study_idx on result(study_id);
create index result_manifest_idx on result(manifest_hash);

-- ================================
-- Deliverables (exports and documents)
-- ================================

create table deliverable (
  id uuid primary key default gen_random_uuid(),
  study_id uuid not null references study(id) on delete cascade,
  workspace_id uuid not null references workspace(id),
  
  kind text not null,                           -- 'epd_draft', 'technical_report', 'exec_summary', 'audit_pack', 'interactive_link'
  format text not null,                         -- 'docx', 'pdf', 'xlsx', 'csv', 'json', 'web'
  program_operator text,                        -- if epd
  
  status text not null default 'pending',       -- 'pending', 'generating', 'ready', 'failed', 'expired'
  file_url text,                                -- R2 URL (signed, expires)
  file_size_bytes bigint,
  
  -- for interactive links
  share_token text unique,                      -- url-safe random token
  share_permissions text,                       -- 'view', 'comment', 'edit'
  expires_at timestamptz,
  
  -- license enforcement (see §12)
  restricted_data_present boolean not null default false,
  restriction_details jsonb,
  
  created_at timestamptz not null default now(),
  created_by uuid references auth.users(id),
  ready_at timestamptz,
  error_message text
);

create index deliverable_study_idx on deliverable(study_id);
create index deliverable_share_token_idx on deliverable(share_token) where share_token is not null;

-- ================================
-- Comments (on any object)
-- ================================

create table comment_thread (
  id uuid primary key default gen_random_uuid(),
  workspace_id uuid not null references workspace(id) on delete cascade,
  target_type text not null,                    -- 'study', 'model', 'flow', 'parameter', 'result'
  target_id uuid not null,
  resolved boolean not null default false,
  created_at timestamptz not null default now(),
  created_by uuid not null references auth.users(id)
);

create table comment (
  id uuid primary key default gen_random_uuid(),
  thread_id uuid not null references comment_thread(id) on delete cascade,
  author_id uuid not null references auth.users(id),
  body_markdown text not null,
  created_at timestamptz not null default now(),
  edited_at timestamptz
);

create index comment_thread_target_idx on comment_thread(target_type, target_id);

-- ================================
-- Versions (history)
-- ================================

create table study_version (
  id uuid primary key default gen_random_uuid(),
  study_id uuid not null references study(id) on delete cascade,
  version_number int not null,
  snapshot jsonb not null,                      -- full study state as of this version
  created_at timestamptz not null default now(),
  created_by uuid references auth.users(id),
  commit_message text,
  unique (study_id, version_number)
);

create index study_version_study_idx on study_version(study_id, version_number desc);

-- ================================
-- Audit log
-- ================================

create table audit_log (
  id bigserial primary key,
  workspace_id uuid references workspace(id),
  user_id uuid references auth.users(id),
  
  occurred_at timestamptz not null default now(),
  action text not null,                         -- 'create', 'update', 'delete', 'read_sensitive', 'share', 'export'
  resource_type text not null,
  resource_id uuid,
  
  -- the change
  old_value jsonb,
  new_value jsonb,
  
  -- context
  ip_address inet,
  user_agent text,
  request_id text,
  
  -- for license enforcement violations
  alert_level text                              -- null, 'info', 'warning', 'violation'
);

create index audit_workspace_time_idx on audit_log(workspace_id, occurred_at desc);
create index audit_resource_idx on audit_log(resource_type, resource_id);
-- partition by month for scale
```

### 6.3 Row-level security (critical)

Every query scoped to a workspace. Set the workspace context per request:

```sql
-- middleware sets this per-request based on JWT
set local app.current_workspace_id = '...';

-- RLS policies use this setting
create policy study_isolation on study
  using (workspace_id = current_setting('app.current_workspace_id')::uuid);
```

This is defense in depth atop application-level checks. Even a buggy endpoint that forgets a `WHERE workspace_id = ?` clause cannot leak data.

### 6.4 The Yjs document schema

For real-time collaboration, the study model is also represented as a Y.Doc. This doc is persisted to Postgres periodically as a blob in `study.yjs_state` (BYTEA column; simplified from above):

```typescript
// Yjs doc structure
const ydoc = new Y.Doc()

const yStudy = ydoc.getMap('study')        // top-level metadata
const yVariants = ydoc.getArray('variants') // variants
const yModels = ydoc.getMap('models')       // keyed by model id
const yFlows = ydoc.getMap('flows')         // keyed by flow id
const yParameters = ydoc.getMap('parameters')
const yCanvas = ydoc.getMap('canvas')       // canvas positions, zoom
const yPresence = ydoc.getMap('presence')   // cursors, selections per user
```

Mutations go through Yjs first (optimistic, fast), then Yjs updates flow to the server where they're also applied to the Postgres relational model. On reconnect after offline, Yjs handles merging automatically.

### 6.5 Embedding generation

For semantic process search:

- Every process gets a vector embedding via Claude Haiku (fast, cheap) or BGE (self-hosted)
- Embedding input: `name + reference_product + geography + category + synonyms`
- 1024-dim vectors stored via pgvector HNSW index
- Query: `select * from process order by embedding <-> query_embedding limit 20`
- Takes <20ms for 70,000 processes

---

## 7. Real-time Collaboration and Sync

### 7.1 The CRDT approach

Yjs is a CRDT library — Conflict-free Replicated Data Type. The properties we need:
- **Eventual consistency** — if all clients see the same operations, they converge to the same state regardless of order
- **Intention preservation** — when Alice adds a flow and Bob edits a different flow at the same time, both changes survive
- **Offline-friendly** — operations made while disconnected merge cleanly on reconnect
- **No central coordinator** — peer-to-peer possible, though we use a server for routing

### 7.2 The server's role

`arko-sync` runs on Node.js. Its responsibilities:

```typescript
// Pseudocode
websocket.on('connection', (ws, req) => {
  const { studyId, userId } = authenticate(req)
  const authz = checkAccess(userId, studyId)
  if (!authz) return ws.close()
  
  const ydoc = loadYDoc(studyId)  // from Postgres cache or cold load
  
  // send current state to new client
  ws.send(Y.encodeStateAsUpdate(ydoc))
  
  // relay future updates
  ws.on('message', (msg) => {
    const update = Y.parseUpdate(msg)
    Y.applyUpdate(ydoc, update)
    
    // broadcast to all other clients viewing this study
    broadcastToStudy(studyId, msg, exceptClient: ws)
    
    // persist to Postgres every 30s or on significant change
    schedulePersist(studyId, ydoc)
    
    // also write to the relational model (async)
    applyUpdateToRelationalModel(studyId, update)
  })
  
  // presence
  ws.on('presence', (presence) => {
    broadcastToStudy(studyId, { userId, presence })
  })
})
```

### 7.3 Yjs to relational sync

The Y.Doc is the real-time source of truth during collaboration, but our relational tables (`model`, `flow`, `parameter`) need to stay in sync for queries and API reads.

Strategy:
- Yjs updates are applied to both the Y.Doc (for broadcast) and a relational-model diff
- Diff is applied to Postgres via a SQL transaction
- Both operations happen async, but the client sees the Yjs update immediately (optimistic)
- If the relational write fails, we retry; in extreme cases, the Y.Doc snapshot eventually wins

### 7.4 Conflict scenarios

**Simultaneous edits to the same field:** Yjs resolves via last-write-wins with Lamport timestamps. The UI shows both users' cursors during the edit.

**Alice deletes a flow while Bob edits it:** Bob's edit on a deleted object is orphaned. The UI shows Bob a toast: "This flow was deleted by Alice. Your edit was discarded." Bob can undo to restore.

**Offline edits that conflict on reconnect:** Yjs merges cleanly for most cases. Structural conflicts (Alice and Bob both added a flow with the same `order_index`) are resolved deterministically. The UI may show a merge notification if significant.

### 7.5 Scaling the sync service

- Single `arko-sync` instance handles ~2000 concurrent WebSocket connections comfortably on a modest Hetzner CPX node
- Horizontal scale via multiple instances fronted by a WebSocket-aware load balancer (Cloudflare supports this natively)
- All instances subscribe to a Redis channel per study; messages relay across instances
- At 100,000 concurrent collaborative editors we'd migrate to a sharded approach, but that's a Year-5 problem

---

## 8. Offline-First and PWA Architecture

### 8.1 What "offline-capable" means, precisely

**Works offline:**
- Reading any study cached locally
- Editing the current study (additions, deletions, amount changes)
- Running calculations for small studies (WASM engine)
- Browsing recently-accessed library processes
- Drafting new studies
- Commenting (queued, synced on reconnect)

**Doesn't work offline (graceful degradation):**
- Searching the full library (only cached subset is searchable)
- Generating EPD documents (requires server rendering)
- Running Monte Carlo on large studies
- Collaborating in real-time (obvious)
- AI queries (Claude requires internet)

### 8.2 The caching strategy

**Three-tier cache:**

1. **Service Worker cache (HTTP cache)**
   - Static app assets (JS, CSS, fonts, icons): cache-first, 1 year
   - HTML shell: network-first with cache fallback

2. **IndexedDB (structured data cache)**
   - Open studies: full Y.Doc snapshot + relational projection
   - Recently-used library processes: up to 5,000 processes (~50 MB)
   - User preferences, UI state
   - Offline mutation outbox
   - Total budget: 500 MB per user per browser (configurable, can increase for power users)

3. **WASM-loaded memory (calc engine)**
   - Active library matrices loaded once, reused across calculations
   - Characterization factor tables for selected method
   - Cleared on tab close

### 8.3 The mutation outbox

When a user makes a change offline:

```typescript
// Pseudocode for the outbox pattern
async function mutate(mutation: Mutation) {
  // 1. Apply optimistically to local state
  applyToYDoc(mutation)
  updateUI()
  
  // 2. Queue to outbox
  await idb.outbox.add({
    id: uuid(),
    mutation,
    attemptedAt: null,
    retryCount: 0,
    createdAt: Date.now()
  })
  
  // 3. Attempt network sync
  if (navigator.onLine) {
    await trySyncOutbox()
  }
}

// On reconnect
window.addEventListener('online', trySyncOutbox)

async function trySyncOutbox() {
  const pending = await idb.outbox.getAll()
  for (const entry of pending) {
    try {
      await api.post('/mutations', entry.mutation)
      await idb.outbox.delete(entry.id)
    } catch (err) {
      if (isConflict(err)) {
        await handleConflict(entry)
      } else if (entry.retryCount > 5) {
        await idb.outboxFailed.add(entry)
        await idb.outbox.delete(entry.id)
      } else {
        entry.retryCount++
        entry.attemptedAt = Date.now()
        await idb.outbox.put(entry)
      }
    }
  }
}
```

### 8.4 UI patterns for offline

- Discreet status indicator in the topbar: green dot online, amber dot syncing, grey dot offline with "N changes pending"
- Blocking operations (like "Generate EPD") that require the server show a clear message: "Internet required for document generation. You can still edit."
- Sync notifications: "3 changes synced" toast on reconnect
- Conflict notifications: "Alice edited this while you were offline. [View changes]"

---

## 9. AI Integration Layer

### 9.1 Provider and deployment

**Primary: Anthropic API.**
- Claude Sonnet 4.5 for most tasks (interpretation, gap-filling, narrative)
- Claude Opus for complex reasoning (sensitivity analysis interpretation, comparison narratives)
- Claude Haiku for fast, cheap tasks (embeddings, classification, typeahead suggestions)

**EU data residency:** Anthropic is expanding EU region support. When available, we use it exclusively for customer study data. Until then, we have DPA-level commitments with Anthropic for GDPR-compliant processing.

**Fallback (Enterprise, V2):** customers can choose to run Llama 4 70B or Qwen 3 on-premise / in their own cloud for data sovereignty. Our AI service abstracts the provider so this is a configuration swap, not a rewrite.

### 9.2 The grounding architecture

**Never generate numeric values.** Claude interprets, summarizes, recommends — it never invents a characterization factor, a process amount, or an emission quantity. Every number in AI output must trace to an actual value in the study or library.

**Tool-calling pattern:**

```python
# Tools exposed to Claude
tools = [
    {
        "name": "get_study_results",
        "description": "Get impact assessment results for the current study",
        "input_schema": {"type": "object", "properties": {"study_id": {"type": "string"}}}
    },
    {
        "name": "get_contribution_breakdown",
        "description": "Get the contribution of each upstream process to a given impact category",
        "input_schema": {...}
    },
    {
        "name": "query_similar_processes",
        "description": "Find processes in the library similar to a description",
        "input_schema": {"type": "object", "properties": {"description": {"type": "string"}, "geography": {"type": "string"}}}
    },
    {
        "name": "compute_what_if",
        "description": "Run a scenario calculation with a parameter change",
        "input_schema": {...}
    },
    {
        "name": "lookup_pcr",
        "description": "Look up Product Category Rules by product type",
        "input_schema": {...}
    },
    {
        "name": "search_literature",
        "description": "Search LCA publications for context on a method or factor",
        "input_schema": {...}
    }
]
```

**Response post-processing:**
- Parse Claude's response for numbers
- Verify every number cited appears in the tool-call results it had access to
- If a number is unverifiable, flag the response with a "⚠ Unverified" badge
- Every AI response shows the tool calls it made (collapsed by default, expandable for trust)

### 9.3 Prompt engineering and system prompts

System prompts live in version control (`prompts/` directory) and are loaded at runtime. Key prompts:

- **Interpretation narrative** — "Given these LCA results, write a 2-paragraph plain-English summary for a product manager..."
- **Hotspot detection** — "Identify the top 3 hotspots by contribution to single score. For each, suggest a specific reduction strategy..."
- **Gap-filling suggestion** — "The user is building a process for X. They've added these inputs. What's typically missing from a complete model of this product?"
- **Anomaly detection** — "Compare this study's water consumption per functional unit against similar published studies. Flag if >3x the industry average."
- **Natural-language query** — "The user asks: '{query}'. Use the tools to find the answer. Explain your reasoning."

All prompts include:
- Explicit instruction not to fabricate numbers
- Required output format
- Style guidelines (confident but appropriately cautious; plain language; no jargon without explanation)
- Multilingual instructions (respond in user's locale)

### 9.4 Cost controls

AI is expensive. Guardrails:

- Per-user rate limits: 10 AI queries / minute, 500 / day on paid tiers; 50 / day on free
- Per-workspace monthly cost budgets with soft and hard caps
- Expensive operations (Opus for complex narratives) gated to Team+ tiers
- Caching: identical query + study state = cached response for 1 hour
- Batch embedding generation runs off-peak on Haiku

### 9.5 AI feature roadmap

**V1 (at launch):**
- Interpretation narrative on results page
- Hotspot callouts
- Semantic process search (embeddings)
- Basic natural-language query

**V2 (months 9-12):**
- AI gap-filling for new processes
- Comparison narrative
- Anomaly detection
- Draft EPD section generation (not numbers, just prose)

**V3 (year 2+):**
- Conversational "LCA copilot" that walks users through study construction
- Automated sensitivity analysis with narrative interpretation
- Local/on-premise LLM support for Enterprise

---

## 10. Document Generation Pipeline

### 10.1 The pipeline

User clicks "Generate EPD" →
1. `arko-api` creates `deliverable` record (status: pending), enqueues Celery task
2. Celery `docs-queue` worker picks up task
3. Worker fetches latest study results from `arko-calc` (forces recompute if stale)
4. Worker sends a job to `arko-docs`: `POST /render-epd { study_data, program_operator, template_version }`
5. `arko-docs` renders Word and PDF versions
6. Files uploaded to Cloudflare R2
7. Signed URL stored in `deliverable` record (status: ready), expires in 24 hours
8. User notified via WebSocket; download button activates

### 10.2 Template architecture

Each EPD program operator has a template directory:

```
templates/
  epd/
    environdec/                 # International EPD System
      v4.0/
        template.tsx            # main template
        sections/
          cover.tsx
          product_info.tsx
          lca_results.tsx
          certifications.tsx
        styles.css
        assets/
          logo.png
    ibu/                        # German IBU
      v2.5/
        ...
    epd_norge/                  # Norwegian
      ...
    epd_italy/
      ...
```

Templates are TSX files. At render time:
1. Template receives `{ study, results, variant, workspace }` as props
2. Returns a JSX tree of docx-js elements (for Word output) or HTML elements (for PDF output via Puppeteer)
3. Same template file can output both via a rendering abstraction

**Example template section:**

```tsx
// templates/epd/environdec/v4.0/sections/lca_results.tsx
import { Paragraph, TextRun, Table, TableRow, TableCell } from 'docx-js'
import { formatNumber } from '@/lib/format'

export function LcaResultsSection({ results, pcr, variant }) {
  const requiredCategories = pcr.requiredImpactCodes
  const data = requiredCategories.map(code => {
    const cat = results.impactCategories.find(c => c.code === code)
    return {
      name: cat.plainEnglishName,
      unit: cat.unit,
      total: cat.total,
      byStage: variant.stages.map(s => ({ stage: s.code, value: cat.byStage[s.code] }))
    }
  })
  
  return [
    new Paragraph({ heading: 'Heading2', children: [new TextRun('Environmental Impact Indicators')] }),
    new Table({
      rows: [
        new TableRow({ children: [
          new TableCell({ children: [new Paragraph('Indicator')] }),
          new TableCell({ children: [new Paragraph('Unit')] }),
          ...variant.stages.map(s => new TableCell({ children: [new Paragraph(s.code)] })),
          new TableCell({ children: [new Paragraph('Total')] })
        ]}),
        ...data.map(row => new TableRow({ children: [
          new TableCell({ children: [new Paragraph(row.name)] }),
          new TableCell({ children: [new Paragraph(row.unit)] }),
          ...row.byStage.map(bs => new TableCell({ children: [new Paragraph(formatNumber(bs.value))] })),
          new TableCell({ children: [new Paragraph(formatNumber(row.total))] })
        ] }))
      ]
    })
  ]
}
```

### 10.3 PDF rendering

For PDF, the same data renders through an HTML template, then Puppeteer converts to PDF:

```tsx
// templates/epd/environdec/v4.0/html/template.tsx
export function EpdDocument({ study, results, variant, workspace }) {
  return (
    <html>
      <head>
        <link rel="stylesheet" href="/templates/epd/environdec/v4.0/styles.css" />
      </head>
      <body>
        <CoverPage study={study} workspace={workspace} />
        <ProductInfo variant={variant} />
        <LcaResults results={results} pcr={pcr} variant={variant} />
        {/* ... */}
      </body>
    </html>
  )
}
```

Puppeteer renders this to PDF with print-specific CSS (page breaks, headers, footers, page numbers).

### 10.4 License-aware rendering

Before rendering, the `license_enforcement` middleware (see §12) inspects the study's flows. If any flow is tagged `license_tier = ecoinvent`, the rendered PDF:
- Has a watermark: "Contains ecoinvent data — see licensing"
- Renders impact values as aggregated totals only (not per-process breakdowns)
- Disables interactive drilldown in the interactive-link mode
- Adds a mandatory reference to the ecoinvent license in the footer

This is programmatic compliance with the ecoinvent EULA, not a legal afterthought.

---

## 11. Security Architecture

### 11.1 Defense in depth

Seven layers, each assuming the others might fail:

1. **Edge layer (Cloudflare):** WAF rules, DDoS mitigation, rate limiting, IP reputation
2. **Network layer (Hetzner + VPC):** private networks between services, only HTTPS exposed publicly
3. **Application layer (FastAPI middleware):** auth, authorization, input validation, rate limiting
4. **Data layer (Postgres RLS):** row-level security as a safety net if middleware fails
5. **Audit layer:** every mutation logged with full context
6. **Encryption layer:** TLS in transit, AES-256 at rest, application-level encryption for sensitive fields
7. **Operational layer:** secret management, patch management, incident response

### 11.2 Authentication

**Primary: Supabase Auth** (shared with KarbonGarbi).

**Supported methods:**
- Email + password (bcrypt-hashed, pepper stored in Doppler)
- Magic link (passwordless)
- OAuth: Google, Microsoft, GitHub
- SAML 2.0 SSO (Enterprise tier) — Okta, Entra ID, Google Workspace, OneLogin
- OIDC (Enterprise tier)

**Session management:**
- JWT access tokens, 1-hour expiry
- Refresh tokens, 30-day expiry, rotated on each use
- Refresh tokens revocable (for "log out everywhere")
- Tokens stored in httpOnly, Secure, SameSite=Lax cookies (never localStorage)

**Multi-factor authentication:**
- TOTP (Google Authenticator, 1Password, Authy) — always available
- WebAuthn / passkeys — default encouraged for new accounts
- SMS — not offered (phishing-susceptible, SIM swap risk)
- Required for Team+ workspace admins; optional for editors and viewers

### 11.3 Authorization

**Role-based access control within workspaces:**

| Role | Permissions |
|---|---|
| Owner | Everything, including workspace deletion, billing, license management |
| Admin | Everything except workspace deletion and billing |
| Editor | Create/edit studies, run calculations, generate deliverables |
| Viewer | Read studies, comment, export own-permitted deliverables |
| Auditor | Read-only, plus audit log access (Enterprise only) |

**Resource-level permissions:**
- Studies can be shared individually with non-members via shareable links with scoped permissions (view / comment / edit)
- Library contributions (user-uploaded custom factors) default to workspace-private; can be shared workspace-wide or published to community gallery (with license tag)

**Enforcement:**
- Application-level checks in every API endpoint (documented in OpenAPI spec)
- Postgres RLS as defense in depth
- Middleware logs any denied access attempt to `audit_log` with `alert_level = 'warning'`

### 11.4 Encryption

**In transit:**
- TLS 1.3 everywhere
- HSTS with preload (12-month policy)
- Certificate pinning in the mobile app (V2)
- No SSLv3, no TLS 1.0/1.1

**At rest:**
- Postgres: Supabase's native AES-256 encryption
- R2 storage: Cloudflare's server-side encryption
- Redis: TLS between services; Redis persistence disabled for ephemeral data

**Application-level (for high-sensitivity fields):**
- Enterprise tier option: customer-managed keys via AWS KMS or Hashicorp Vault
- Encrypted fields: `workspace.settings` (if containing API keys), `workspace_license.license_proof_url` content

### 11.5 Input validation and injection prevention

- **Pydantic everywhere** — every request body and query parameter validated
- **Parameterized queries only** — SQLAlchemy ORM or explicit parameterized raw SQL; no string concatenation
- **Output encoding** — React handles XSS by default; Markdown rendered via `@uiw/react-markdown-preview` with sanitizer
- **CSP headers** — strict Content Security Policy; no inline scripts, only trusted CDNs
- **File upload validation** — type checking, size limits, malware scanning (ClamAV on-premise or VirusTotal API)

### 11.6 Secrets management

**Doppler** as the secrets manager. All services pull secrets at startup from Doppler via injected environment variables. No secrets in git, ever.

**Secret categories:**
- Infrastructure: database passwords, Redis auth, API keys for third parties
- Signing keys: JWT secret, share-link token salt, webhook HMAC keys
- Service credentials: Anthropic API key, Stripe/Redsys keys, Postmark API key
- Rotation schedule: JWT and signing keys rotated quarterly; service credentials rotated annually or on staff departure

### 11.7 Compliance roadmap

| Compliance | Status at launch | Target |
|---|---|---|
| GDPR | Day 1 (inherited from KarbonGarbi pattern) | ✓ |
| SOC 2 Type I | Not at launch | Month 12-15 |
| SOC 2 Type II | Not at launch | Month 24 |
| ISO 27001 | Not planned V1 | Year 3-4 if enterprise demand |
| ISO 14001 (LCA context) | Not applicable to software | — |
| HIPAA | Not applicable | — |

---

## 12. Data Licensing Enforcement

This is the novel architectural contribution that the ecoinvent EULA analysis made us realize we need. It deserves its own section.

### 12.1 The problem restated

Ecoinvent's EULA restricts how their data can be published. If we let users put ecoinvent data into Arko and then publish a public share link showing ecoinvent-derived numbers, the user and potentially Arko as an intermediary violate the EULA (CHF 100,000 penalty per breach).

We cannot rely on users to read the EULA. The architecture must enforce it.

### 12.2 The architectural pattern

**Every flow carries a `license_tier` field.** When flows roll up into a study's calculation graph, the study's **effective license restriction** is the union of all input license restrictions.

```python
# Pseudocode for license enforcement
class LicenseTier(str, Enum):
    OPEN = "open"
    ECOINVENT = "ecoinvent"
    AGRI_FOOTPRINT = "agri_footprint"
    USER_PRIVATE = "user_private"
    USER_SHARED = "user_shared"
    PROPRIETARY = "proprietary"

RESTRICTIONS = {
    LicenseTier.OPEN: set(),
    LicenseTier.ECOINVENT: {
        "cannot_public_share_interactive",
        "cannot_expose_unit_values_via_api",
        "cannot_embed_in_consumer_product",
        "requires_static_pdf_export",
        "requires_license_footer",
        "requires_ecoinvent_attribution"
    },
    LicenseTier.AGRI_FOOTPRINT: {
        "cannot_public_share_interactive",
        "requires_attribution"
    },
    LicenseTier.USER_PRIVATE: {
        "cannot_share_outside_workspace"
    },
    LicenseTier.USER_SHARED: set()  # but must preserve attribution
}

def effective_restrictions(study: Study) -> set[str]:
    restrictions = set()
    for flow in study.all_flows():
        for r in RESTRICTIONS[flow.license_tier]:
            restrictions.add(r)
    return restrictions

def can_share_interactively(study: Study) -> bool:
    return "cannot_public_share_interactive" not in effective_restrictions(study)

def allowed_export_formats(study: Study) -> list[str]:
    restrictions = effective_restrictions(study)
    formats = ["pdf_static", "exec_summary", "auditor_pack"]
    if "cannot_expose_unit_values_via_api" not in restrictions:
        formats.append("json_api")
    if "cannot_public_share_interactive" not in restrictions:
        formats.append("interactive_link")
    return formats
```

### 12.3 UI consequences

When a user attempts an action restricted by license:

- "Share this study publicly" button shows a warning: "This study contains ecoinvent data. Public sharing would violate ecoinvent's license. Options: (a) Generate a static PDF instead. (b) Share with workspace members only. (c) Replace ecoinvent data with open alternatives."
- EPD document generation automatically applies the right watermarks and attributions
- API endpoints that would return licensed data to unauthorized parties return a `402 Payment Required` or `451 Unavailable For Legal Reasons`, not the data

### 12.4 Audit and compliance reporting

Every license-sensitive action is logged with sufficient detail for a compliance audit:

- Who accessed which licensed library
- What they computed from it
- What documents they exported
- Whether those documents applied the required compliance treatment

Monthly reports per workspace summarize: "You made N queries against ecoinvent data this month, generated M compliant documents, and zero non-compliant exports." Customer can provide this report to ecoinvent on request.

### 12.5 The reseller path (V2+)

When Arko becomes an authorized ecoinvent reseller (targeted Month 18-24):

- `workspace_license` table tracks which tenants have valid ecoinvent access via Arko
- Middleware checks this on every query involving ecoinvent data
- Tenants without a license see ecoinvent processes in search as "locked" with a "Request access" CTA
- Billing integrates ecoinvent license fees as a pass-through (with a reseller margin)

This architecture scales from V1 "your data, your license" to V3 "buy everything through Arko" without code changes.

---

## 13. Observability, Monitoring, and SLOs

### 13.1 SLIs and SLOs

Service Level Indicators we measure and the Service Level Objectives we commit to:

| Service | SLI | SLO (target) | Error budget / month |
|---|---|---|---|
| arko-web | Availability | 99.9% | 43 min |
| arko-api | Availability | 99.9% | 43 min |
| arko-api | p95 latency (read) | <300 ms | — |
| arko-api | p95 latency (write) | <800 ms | — |
| arko-calc | Availability | 99.5% | 3.6 hr |
| arko-calc | p95 calc duration (small) | <500 ms | — |
| arko-calc | p95 calc duration (medium) | <3 s | — |
| arko-docs | Availability | 99.5% | 3.6 hr |
| arko-docs | p95 EPD generation | <30 s | — |
| arko-sync | Availability | 99.9% | 43 min |
| arko-sync | p95 message delivery | <200 ms | — |

### 13.2 The observability stack

**Self-hosted Grafana stack on Hetzner:**

- **Prometheus** — metrics collection; scrapes each service every 15s
- **Loki** — log aggregation; all services ship structured JSON logs via Promtail
- **Tempo** — distributed tracing; OpenTelemetry instrumentation
- **Grafana** — unified dashboards and alerting
- **Alertmanager** — routes alerts to PagerDuty (or Opsgenie) + Slack

**Why self-hosted:** GDPR (logs may contain customer data), cost (Datadog at scale is expensive), and control. The ops burden is ~2 hours / week once stable.

**External uptime monitoring:** Better Uptime or Oh Dear, from 5+ global locations, checking public endpoints. This catches issues the internal stack can't detect (DNS, CDN, edge).

### 13.3 Key dashboards

Each service has a "golden signals" dashboard: traffic, errors, latency, saturation. We look at it daily.

Business dashboards: signups per day, active studies, calculations run, documents generated, AI queries per tier. These inform product decisions.

### 13.4 Alerting philosophy

Three severity tiers:

- **Page (wake someone up):** availability SLO breach for >5 min, data loss risk, security incident, billing pipeline failure
- **Ticket (handle next business day):** latency SLO breach, non-critical service degraded, cost anomaly
- **FYI (weekly review):** budget alerts, capacity warnings, expired certificates approaching

On-call rotation starts as Samir solo. When team grows to 3+ engineers, weekly rotation with an 8-5 coverage model (European hours).

### 13.5 Error tracking

**Sentry** (self-hosted EU instance):
- Frontend errors with source maps
- Backend exceptions with full context
- Alerting on new error types
- Issue assignment and resolution tracking

### 13.6 Audit log as observability

The `audit_log` table is both a compliance artifact and an observability signal. Unusual patterns (bulk deletions, off-hours admin logins, many failed auth attempts) fire alerts.

---

## 14. DevOps, CI/CD, and Deployment

### 14.1 Repo structure (Turborepo monorepo)

```
arko/
  apps/
    arko-web/               # Next.js frontend
    arko-api/               # FastAPI main API
    arko-calc/              # FastAPI calculation service
    arko-docs/              # Node.js doc generation service
    arko-sync/              # Node.js sync service
    arko-ingest/            # Python library ingestion workers
    arko-marketing/         # optional: separate Next.js for marketing if useful
  packages/
    shared-types/           # TS types generated from OpenAPI
    shared-python/          # shared Python utilities
    design-tokens/          # colors, spacing, typography (JSON)
    ui/                     # shared React component library (shadcn variants)
    lca-wasm/               # Rust crate compiled to WASM
    prompts/                # versioned Claude system prompts
  libraries/                # LCA data sources
    agribalyse/
    uslci/
    elcd/
    pefaps/
    methods/
    pcrs/
  infrastructure/
    terraform/              # declarative infra (Hetzner Cloud, Cloudflare, Supabase)
    ansible/                # server configuration
    docker/                 # Dockerfiles per service
    coolify/                # Coolify configs
  scripts/
    dev.sh                  # bootstrap local dev env
    test-all.sh             # run full test suite
  docs/
    architecture/
    api/                    # OpenAPI specs
    runbooks/               # operational playbooks
  .github/
    workflows/              # GitHub Actions CI/CD
  turbo.json
  package.json              # workspace root
  pyproject.toml            # shared Python tooling
```

### 14.2 CI/CD pipeline

**On every push:**
1. Lint (Biome for TS, Ruff for Python)
2. Type check (tsc for TS, mypy for Python)
3. Unit tests (Vitest for TS, Pytest for Python)
4. Build each changed app (Turborepo caches)
5. Security scan (Snyk, Bandit, npm audit)

**On PR to main:**
6. Integration tests (Playwright for E2E on preview env)
7. Visual regression tests (Chromatic or Percy)
8. Performance budget check (Lighthouse CI)
9. Reviewer approval required
10. Merge → auto-deploy to staging

**On merge to main:**
11. Deploy to staging automatically
12. Smoke tests on staging
13. Manual approval for production deploy
14. Production deploy (blue-green for API, rolling for frontend)

### 14.3 Deployment topology

**Frontend (arko-web):** Vercel with automatic preview per PR, production on main. Zero infra cost at low scale.

**Backend services:** Dockerized, deployed via Coolify on Hetzner dedicated servers. Coolify handles reverse proxy (Traefik), SSL via Let's Encrypt, and rolling deploys.

**Production cluster (Year 1):**
- 1x dedicated server (EX44): arko-api + arko-sync + Redis
- 1x dedicated server (EX44): arko-calc (Python + Brightway)
- 1x VPS (CX32): arko-docs + arko-ingest
- 1x VPS (CX22): observability stack (Prometheus, Loki, Grafana)

Total cost: ~€200/month for infrastructure. Supports several hundred paying customers.

**Scaling levers:**
- Vertical first: bigger servers as load grows
- Horizontal second: replicate calc workers, add API instances behind a load balancer
- Split database into read replicas when read:write > 10:1

### 14.4 Database migrations

**Alembic** (Python) for Postgres schema migrations. Migrations are:
- Reviewed like code (PR required)
- Backward-compatible for one version (add columns, don't drop them; deprecate then drop in a later release)
- Tested on a staging database that's a clone of production
- Rolled back only via forward-fix (new migration that reverses the change), never `downgrade`

### 14.5 Feature flags

**LaunchDarkly or a self-hosted alternative (Flagsmith, Unleash).** Features gated behind flags allow:
- Gradual rollout (1% → 10% → 100%)
- Instant rollback without deploy
- Per-workspace overrides (beta users, specific customers)
- A/B testing for UX variations

---

## 15. Development Workflow and Tooling

### 15.1 Local development environment

`scripts/dev.sh` starts everything:

```bash
#!/bin/bash
# Start Postgres, Redis, Meilisearch in Docker Compose
docker compose -f docker-compose.dev.yml up -d

# Seed test data
pnpm db:seed

# Start all services in parallel via Turborepo
pnpm turbo dev
```

Services started:
- arko-web on http://localhost:3000
- arko-api on http://localhost:8000 (OpenAPI at /docs)
- arko-calc on http://localhost:8001
- arko-docs on http://localhost:3001
- arko-sync on ws://localhost:3002
- Supabase local on http://localhost:54321

Boots in ~30 seconds on a modern laptop.

### 15.2 Code quality

**Linters and formatters:**
- **Biome** (TS/JS) — replaces ESLint + Prettier, single tool, faster
- **Ruff** (Python) — replaces Black + isort + pylint + flake8
- **cargo fmt + clippy** (Rust)
- Pre-commit hooks enforce on commit

**Type systems:**
- TypeScript strict mode
- mypy strict mode on Python services
- Pydantic v2 for runtime validation

**Testing pyramid:**
- Unit tests: 70%+ coverage on business logic (not UI glue)
- Integration tests: key flows end-to-end
- E2E: Playwright on critical paths (signup, create study, generate EPD)
- Property-based: Hypothesis for LCA math correctness
- Performance: k6 scripts for load on API endpoints

### 15.3 Git workflow

**Trunk-based development with short-lived feature branches.**
- main is always deployable
- Feature branches live for 1-3 days max, rebased before merge
- No long-running branches
- Release cadence: continuous (multiple deploys per day as we grow)

### 15.4 Documentation

Three categories:

**Product documentation** (`/docs` in the app, markdown rendered): user-facing help, tutorials, concept guides. Maintained alongside product code.

**Architecture documentation** (`docs/architecture/` in repo): ADRs (Architecture Decision Records), system diagrams, this spec. Updated on every significant change.

**Operational runbooks** (`docs/runbooks/`): "how to handle X incident", "how to rotate Y secret", "how to provision a new tenant". Used on-call.

**API documentation**: auto-generated from OpenAPI, published at docs.arko.earth with examples, SDK snippets, change log.

---

## 16. Third-party Dependencies (Vendor List)

Transparency for due diligence. Everything we depend on, categorized.

### 16.1 Infrastructure vendors

| Vendor | Purpose | EU data residency | Cost at scale | Alternative |
|---|---|---|---|---|
| Hetzner | Dedicated servers, VPS | Yes (Finland, Germany) | ~€200-2000/mo | OVH, Scaleway |
| Cloudflare | Edge, CDN, WAF, DNS, R2 | Yes (with EU settings) | Free tier → $200/mo | Bunny.net |
| Vercel | Frontend hosting | Yes (EU edge) | Free → $20/user/mo | Netlify, self-host |
| Supabase | Postgres, Auth, Storage | Yes (EU-West) | $25 → $599/mo | Neon + Auth0 |
| Doppler | Secrets management | US, but encrypted | $0 → $18/user/mo | HashiCorp Vault (self-host) |

### 16.2 Third-party APIs

| Vendor | Purpose | EU support | Cost model | Alternative |
|---|---|---|---|---|
| Anthropic | LLM (Claude) | EU regions expanding | Per-token | OpenAI, self-hosted Llama |
| Postmark | Transactional email | Yes | Per-email | Resend, SES |
| Stripe / Redsys | Billing | Yes (Ireland / Spain) | 1.4-2.9% | Paddle, Lemon Squeezy |
| Sentry (self-host) | Error tracking | Our servers | Self-host only | Rollbar, BugSnag |
| Better Uptime | Uptime monitoring | EU check locations | $20-100/mo | UptimeRobot |

### 16.3 Frontend libraries

| Library | Purpose | License |
|---|---|---|
| Next.js 15 | Framework | MIT |
| React 19 | UI | MIT |
| TypeScript | Types | Apache 2.0 |
| Tailwind CSS 4 | Styling | MIT |
| shadcn/ui | Components | MIT (we own the code) |
| ReactFlow 12 | Canvas | MIT |
| TanStack Query v5 | Server state | MIT |
| TanStack Table v8 | Grids | MIT |
| Zustand | Client state | MIT |
| Yjs | CRDT | MIT |
| y-websocket | Yjs server | MIT |
| Lucide React | Icons | ISC |
| Recharts | Charts | MIT |
| D3.js | Custom viz | BSD-3 |
| Visx | React + D3 | MIT |
| Workbox | Service worker | MIT |
| Biome | Linter | MIT |

### 16.4 Backend libraries (Python)

| Library | Purpose | License |
|---|---|---|
| FastAPI 0.110+ | API framework | MIT |
| Pydantic v2 | Validation | MIT |
| SQLAlchemy 2.x | ORM | MIT |
| Alembic | Migrations | MIT |
| Celery 5 | Jobs | BSD-3 |
| Brightway 2.5 | LCA engine | BSD-3 |
| NumPy / SciPy | Math | BSD |
| Pandas | Data | BSD-3 |
| Ruff | Linter | MIT |
| mypy | Types | MIT |
| Pytest + Hypothesis | Testing | MIT |
| pgvector | Vector search | PostgreSQL license |

### 16.5 Node.js (sync and docs services)

| Library | Purpose | License |
|---|---|---|
| Node.js 22 LTS | Runtime | MIT |
| y-websocket | Sync server | MIT |
| docx-js | Word generation | MIT |
| Puppeteer | PDF via Chromium | Apache 2.0 |
| ExcelJS | Excel export | MIT |
| Fastify or Hono | HTTP framework | MIT |

### 16.6 Rust (WASM calc engine)

| Library | Purpose | License |
|---|---|---|
| `nalgebra` or `sprs` | Sparse matrix | BSD / MIT |
| `wasm-bindgen` | JS interop | MIT / Apache 2.0 |
| `serde` | Serialization | MIT / Apache 2.0 |

### 16.7 License strategy

- All direct dependencies: MIT, Apache 2.0, BSD-3, ISC, or similarly permissive
- No GPL or AGPL in runtime dependencies (build tools fine)
- License compliance scan in CI (`licensee`, `fossa-cli`)
- SBOM generated per release

---

## 17. Cost Modeling Across Scale Tiers

Numbers are indicative, for sanity-checking and budget planning. Validate against actual bills.

### 17.1 Year 1 (0-200 paying customers, ~1,000 total users)

| Line item | Monthly cost |
|---|---|
| Vercel (Pro team) | €20 |
| Hetzner dedicated (2x EX44) | €150 |
| Hetzner VPS (2x CX32) | €30 |
| Supabase Pro | €25 |
| Cloudflare Pro | €20 |
| R2 storage (100 GB) | €1.50 |
| Anthropic API | €100-500 depending on usage |
| Postmark (10k emails) | €15 |
| Sentry self-hosted | (included in VPS cost) |
| Doppler | €18 |
| Domain, SSL, misc | €10 |
| **Total infrastructure** | **€400-800 / month** |

With ~150 paying customers at avg €200/mo MRR: €30,000 MRR vs €600 infra = 2% infra cost of revenue. Healthy.

### 17.2 Year 2 (500-2000 paying customers)

| Line item | Monthly cost |
|---|---|
| Vercel (Enterprise) | €500 |
| Hetzner dedicated (4x EX64) | €600 |
| Hetzner VPS (cluster) | €200 |
| Supabase Team | €599 |
| Cloudflare Business | €200 |
| R2 storage (1 TB) | €15 |
| Anthropic API | €2,000-5,000 |
| Postmark (100k emails) | €75 |
| Grafana Cloud (might move off self-host) | €200 |
| SOC 2 tooling (Drata/Vanta) | €1,500 |
| Pen testing (quarterly) | €2,000 amortized |
| **Total infrastructure** | **€6,000-10,000 / month** |

With 1000 customers at avg €300/mo MRR: €300,000 MRR vs €10,000 infra = 3%. Still healthy.

### 17.3 Year 3 (5000+ customers, Enterprise tier scaled)

Costs scale roughly linearly with load. Major line items now include:
- Dedicated enterprise support engineers
- On-premise/regional deployment options for select Enterprise customers
- Compliance audits (SOC 2 Type II, ISO 27001 optional)
- Ecoinvent reseller license fees (passed through + margin)

Infra as % of revenue should stay <5% if we design well.

---

## 18. Scaling Plan and Capacity Thresholds

### 18.1 Thresholds and triggers

What changes at what scale.

| Scale | Trigger | Architectural change |
|---|---|---|
| 100 customers | Current | Single-node per service, no changes needed |
| 1,000 customers | p95 API latency breaches SLO | Add API read replicas, add calc worker pool |
| 5,000 customers | Postgres write contention | Postgres connection pooling (PgBouncer), move audit log to separate DB |
| 10,000 customers | Full-text search slow | Dedicated Meilisearch cluster, shard by tenant |
| 50,000 customers | Monolithic API strain | Split arko-api into domain services (studies, library, billing) |
| 100,000 customers | Single region insufficient | Multi-region deployment (EU-Central + EU-West primary) |
| Enterprise customer with >1 million calcs/month | — | Dedicated calc cluster for that tenant |

### 18.2 The things we'd rewrite at scale

- **Calc engine**: at very large scale (ecoinvent + 10x Monte Carlo iterations concurrently), our Rust-WASM engine becomes the server-side engine too. The Python Brightway layer stays for correctness reference and niche operations.
- **Sync service**: at 100,000 concurrent WebSockets, we'd move from single-Node y-websocket to a sharded Erlang/Elixir implementation (which is what Figma did).
- **Database**: at 10+ TB, we shard Postgres by workspace (Citus extension) or consider moving analytics to a data warehouse (ClickHouse).
- **AI layer**: expensive at scale; batch non-real-time AI tasks, cache aggressively, potentially fine-tune our own smaller models for common tasks.

None of these are V1 problems. Don't build for scale you don't have.

---

## 19. Migration Paths and Technical Debt Strategy

### 19.1 Expected technical debt

Things we'd knowingly build "wrong" in V1 to ship faster, with a plan to fix:

| Debt | Why we accept | When to fix |
|---|---|---|
| Shared Postgres with KarbonGarbi | Saves ops overhead | When either product's data volume makes isolation cleaner |
| No separate analytics DB | Read replicas suffice early | When analytics queries slow OLTP |
| AI cost at-will (no careful optimization) | Early product-market fit matters more | When AI costs > 15% of revenue |
| No compliance automation tooling | Manual is fine under 100 customers | Before SOC 2 Type II audit |
| Single-tenant template storage | PCRs/templates are slow-changing | Fine long-term; promote to CDN if needed |
| Brightway's Python speed ceiling | Correctness > raw speed | When the Rust engine matures and passes cross-validation |

### 19.2 Migration paths we preserve

- From any tier to Enterprise: add features, don't rebuild
- Self-hosted option for Enterprise: our web-native architecture runs anywhere Docker runs
- SimaPro migration for customers: we import their ECOSPOLD and SimaPro CSV exports (day-1 feature)
- OpenLCA migration: read their project format via olca-ipc
- Export to SimaPro / OpenLCA: never locked in; users can always leave with their data

### 19.3 Deprecation policy

- API `/v1` supported for 12 months after `/v2` ships
- Data libraries: older versions remain available for studies pinned to them, indefinitely
- Breaking UX changes: feature-flagged, rolled out gradually, with migration UI

---

## 20. Risk Register and Mitigations

| # | Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|---|
| R1 | Brightway upstream changes break our calc | Low | High | Pin to major version; CI runs against pinned version; contribute upstream to reduce drift |
| R2 | Anthropic changes pricing or policies abusively | Medium | Medium | Abstract provider layer from day 1; test with Llama and Mistral in staging quarterly |
| R3 | Ecoinvent denies reseller status | Medium | Medium | V1 doesn't need it; V2 as enhancement; open data remains viable alternative indefinitely |
| R4 | Supabase discontinuation or major price hike | Low | High | Postgres + self-hosted Auth (Keycloak) as backup plan; monthly backup restore tested |
| R5 | WASM engine produces subtly wrong results | Medium | Critical | Differential testing against Brightway on every commit; 1000+ reference studies; bug bounty V2 |
| R6 | Yjs CRDT edge cases corrupt studies | Low | High | Yjs is battle-tested; frequent Postgres snapshots allow restore; conflict UI is explicit |
| R7 | GDPR audit finding | Low | High | KarbonGarbi's Phase I (legal pages, DPA) reused; annual DPIA review; EU-only processors |
| R8 | Regulatory requirement for specific tool certification | Medium | Medium | Design for audit from day 1; pursue certifications when customer demand justifies |
| R9 | Cloudflare outage | Low | High | Multi-region origin; Cloudflare bypass procedures documented; status page independent |
| R10 | Solo founder getting stuck on technical decisions | High (today) | Medium | This spec provides pre-made decisions; technical advisor network; "decide in 48 hours or punt" rule |
| R11 | Codebase outgrows solo founder | Medium | High | Technical co-founder or first senior hire by Month 6-9; monorepo + docs reduce onboarding friction |

---

## 21. Open Technical Questions for Future Resolution

Questions this spec doesn't answer, which need more investigation.

1. **Should the calc engine expose a gRPC interface in addition to REST?** For power users doing thousands of calculations (portfolio-level LCAs), gRPC would be faster. Not V1, but evaluate at V2.

2. **How do we version the open-data bundles?** Agribalyse releases annually; USDA less often. Our method library versioning is clean; data bundle versioning needs a policy. Propose: semver-ish, pinned per study, updates offered but never forced.

3. **What's the exact WASM engine's feature coverage?** TBD by a 2-week spike at Month 2. Expected: deterministic solves, simple uncertainty, standard allocation. Monte Carlo stays server-side.

4. **CRDT snapshot strategy for very large studies.** At what size (number of models? Y.Doc byte size?) do we split a study into sub-documents? Preliminary: at 10,000 models or 50 MB Y.Doc size.

5. **Mobile app scope.** When we build it (V2), is it native React Native or just a better-tuned PWA? Decision point at Month 12 based on customer asks.

6. **Fine-tuned models for specific LCA tasks.** When does it become worth fine-tuning a small model (Llama 3.1 8B) on LCA-specific tasks? Likely Year 2 when we have enough training data from user interactions.

7. **Formal verification of calc correctness.** Is there value in applying formal methods (TLA+ for the state machine, Coq for the math) to the calc engine? Interesting long-term; not V1.

8. **Desktop app via Tauri: realistic customer demand?** Validate via customer conversations before investing. If no customer specifically asks in Year 1, don't build.

---

## 22. Appendix: Quick-reference cheat sheets

### 22.1 Service ownership at steady-state

| Service | Primary responsibility | First response if on fire |
|---|---|---|
| arko-web | Frontend team | Frontend engineer |
| arko-api | Backend team | Backend engineer |
| arko-calc | LCA engineering team | LCA engineer or Samir |
| arko-docs | Backend team | Backend engineer |
| arko-sync | Backend team | Backend engineer |
| arko-ingest | Data team | Data engineer |

In year 1 with solo/small team, "team" = Samir + maybe one collaborator.

### 22.2 Key decisions log (ADRs to write first)

When development starts, write these ADRs (Architecture Decision Records) first:

- ADR-001: Choice of Brightway vs alternatives for calc engine
- ADR-002: Yjs CRDT for real-time collaboration
- ADR-003: Rust-WASM for in-browser calculation
- ADR-004: License-enforcement architecture
- ADR-005: Multi-tenancy via workspace_id + Postgres RLS
- ADR-006: AI tool-calling over RAG for grounded answers
- ADR-007: Monorepo with Turborepo
- ADR-008: Self-hosted observability (Grafana) vs SaaS
- ADR-009: Vercel for frontend, Hetzner for backend (hybrid rationale)
- ADR-010: Supabase as foundation (shared with KarbonGarbi)

### 22.3 Critical first-quarter development milestones

M1 (Month 0-1): Data model finalized, migrations in place, Goibix SSO integrated, empty study CRUD working end-to-end.

M2 (Month 2-3): Canvas editor renders and accepts input, table view renders, basic calc via Brightway returns results for a toy study.

M3 (Month 4-6): Results view with summary + breakdown, first open-data library imported and searchable, deterministic calculation matches SimaPro on reference study suite.

These are minimum viable milestones. Any of them delayed by more than 1 month is a yellow flag; delayed by 2+ is a red flag and we re-scope.

---

## Closing note

This specification is intentionally opinionated. Many choices are not the only defensible option; they're the defensible option that best fits Arko's specific constraints: solo founder, Goibix platform heritage, open-data positioning, EU market focus, and a 12-18 month window to ship before the disruption settles.

When a new contributor disagrees with a choice, the process is:

1. Write an ADR proposing the change
2. Build a prototype demonstrating the proposed alternative
3. Compare against the current approach on specific metrics
4. Decide — the spec updates if the new approach wins

The spec is a starting line, not a finish line. It exists to accelerate decisions, not to prevent them.

---

*End of Arko Technical Architecture Specification v1.0*
*Goibix S.L., Bilbao, April 2026*
*Companion to Arko Master Specification v1.0*
