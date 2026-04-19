# Arko — Technical Architecture Specification v2.0

## Excellence Edition

**A technically rigorous, sovereignty-first, self-hosted, open-source-foundation architecture for a modern life cycle assessment platform.**

---

**Document status:** Final draft v2.0 — April 18, 2026
**Supersedes:** Arko Technical Architecture Specification v1.0
**Companion to:** Arko Master Specification v1.0
**Author:** Samir (Goibix S.L., Bilbao) with Claude
**Scope:** The complete technical blueprint for building Arko at a standard of engineering excellence that senior engineers respect, while remaining buildable by a solo founder growing a small team over 12–24 months.

---

## Document philosophy

This is not an aspirational document. Every choice here is designed to be executed, by a real team, on a real schedule, with real money. It prioritizes durability over novelty, published specifications over proprietary cleverness, and data sovereignty over operational convenience. It rejects both "move fast and break things" and "research-grade perfectionism." It is the work of someone who has seen what is genuinely hard in production, cares about getting it right, and refuses both corners and ivory towers.

Wherever a choice is non-obvious, rationale is given. Wherever an alternative was considered, it is named. Wherever a decision depends on something we don't yet know, an Open Question is explicitly recorded.

This document is opinionated. It is not the only defensible architecture for this product, but it is *an* excellent one, chosen from a space of alternatives with care. When a future contributor disagrees, the process is: write an ADR, prototype the alternative, measure against stated criteria, update the spec. Nothing here is sacred except the principles.

---

## Table of Contents

1. Architectural Philosophy and Commitments
2. System Overview and Component Map
3. Frontend Architecture
4. Backend Services Architecture
5. The Calculation Engine (our core IP)
6. The Real-Time Sync Engine (our second core IP)
7. Data Model and Database Design
8. License Enforcement Architecture
9. AI Orchestration Layer
10. Document Generation Pipeline
11. Security Architecture
12. Observability, SLOs, and Incident Response
13. Infrastructure as Code
14. Self-Hosted Platform Services
15. CI/CD and Release Management
16. Testing Strategy
17. Disaster Recovery and Business Continuity
18. Development Workflow and Tooling
19. Vendor Map (self-hosted, owned, leased)
20. Cost Modeling Across Scale Tiers
21. Scaling Plan and Capacity Thresholds
22. Migration and Technical Debt Strategy
23. Risk Register
24. Open Questions

---

## 1. Architectural Philosophy and Commitments

### 1.1 The core principle

Own what makes Arko *Arko*. Run open-source software on your own infrastructure for everything else. Lease only where no open equivalent meets the bar, and only with an abstraction that makes replacement a configuration change. Publish specifications, open-source the generic, keep the commercial differentiation in product and operations, not in proprietary code hoarding.

This principle is not a compromise. It is what technically excellent SaaS looks like in 2026.

### 1.2 The twelve architectural commitments

**C1 — Sovereignty over convenience.** All customer data lives on hardware we control, in EU jurisdictions, with encryption keys we hold. No managed service touches production customer data in V1. Convenience wins for development tooling; sovereignty wins for customer data.

**C2 — Boring infrastructure, brilliant IP.** Postgres, Redis, Kubernetes, HTTP, TCP, POSIX. These are not where we innovate. Our calculation engine, our CRDT sync, our license enforcement, our AI orchestration, our data model — these are where we innovate. Innovating in the wrong place is a tax.

**C3 — Specification-first, not implementation-first.** The calc engine has a published mathematical specification. The CRDT has a published state machine specification (in TLA+). The license enforcement has a published policy language specification. Specifications outlive implementations. They're what we're proud of.

**C4 — Differential and property-based testing as the correctness regime.** Unit tests verify that a function does what the author thought. Differential testing verifies we agree with battle-tested reference implementations (Brightway, OpenLCA) across thousands of cases. Property-based testing verifies invariants hold across generated inputs. Together they give us correctness by evidence, at a fraction of the cost of formal verification, with coverage formal verification rarely achieves.

**C5 — Multi-tenant by default, at the schema level.** Every row of every table carries `workspace_id`. Row-level security is enforced at the database engine, not at the application layer. Escape hatches for super admin are explicit, audited, and rare. Tenant isolation is a correctness property, not a best-effort convention.

**C6 — API-first internally.** The web app has no private API. Every feature the UI uses is also available to Enterprise customers through documented, versioned REST + GraphQL + WebSocket APIs. This forces clean boundaries and prevents UI-coupled backend rot.

**C7 — Every mutation has a user, every number has provenance.** No anonymous writes. No orphan values. The audit log and the provenance graph are first-class features, not afterthought compliance checkboxes.

**C8 — Deterministic builds, reproducible results, signed artifacts.** Given a study, a method version, a library version, and a calc engine version, we produce bit-identical results forever. Every result ships with a SHA-256 signed manifest that a third party can independently verify. This is table stakes for regulatory-grade software.

**C9 — Open-source what's generic, close-source what's commercial.** The calc engine is Apache 2.0. The PCR library is CC-BY-SA. The sync protocol is documented publicly. The license enforcement pattern is published as a reference design. The application itself — the UI, the orchestration, the business logic, the EPD templates — is our commercial code.

**C10 — No vendor can hold us hostage.** Every lease relationship has an abstraction layer and a migration path. We can swap Claude for self-hosted Llama in a weekend. We can move from Hetzner to OVH in a week. We can replace Cloudflare with HAProxy in a day. Lock-in is a design failure.

**C11 — Progressive decentralization of ops.** V1 ships with a single-operator runbook. V2 splits responsibilities as the team grows. V3 has a formal on-call rotation. Don't build for the team you don't have; do build so that the team you will have can take over cleanly.

**C12 — If it's worth shipping, it's worth monitoring.** Every service has metrics, logs, traces, and SLOs from day one. Features ship with observability, not post-hoc. Incidents are documented, postmortems are public (internally), fixes are codified as tests that prevent recurrence.

### 1.3 The things we explicitly say no to

- **No microservices beyond what's justified.** V1 is a modular monolith plus four purpose-built services (calc, sync, docs, ingest). No service mesh. No event sourcing. No CQRS. When any of these is warranted, we introduce it deliberately with an ADR.
- **No Kubernetes complexity theater.** We use k0s on Talos Linux — the minimal, declarative, immutable choice — not a full CNCF stack. No Istio, no Argo CD complexity that outruns our team size. Helm charts, some custom manifests, done.
- **No proprietary binary file formats.** All exports are open formats. Users can leave with their data in any of twelve formats.
- **No Brightway fork.** We contribute to Brightway upstream when we use it for validation and reference. We do not fork. We write our own engine from first principles and keep Brightway as a reference implementation for differential testing.
- **No on-premise installations in V1 or V2.** Accepting self-hosted customer deployments while we're a small team is an ops dead weight that kills us. V3, for named Enterprise customers, with dedicated support pricing.
- **No formal verification (Coq, Lean) as a V1 blocker.** TLA+ for the sync state machine is the right amount of formal methods. Coq for the calc engine is V4 marketing, if ever.
- **No blockchain, no AI gimmicks, no Web3.** LCA is serious science. Our credibility depends on looking like serious software.
- **No pager-storm culture.** Alerts that don't require human action get fixed or deleted. If an alert has fired more than twice without leading to a human decision, it's broken and must be re-engineered.

### 1.4 Performance budgets

Every architectural choice must serve these:

| Operation | Target p50 | Target p95 | Absolute ceiling |
|---|---|---|---|
| First page load (cold cache) | 1.2s | 2.5s | 4s |
| First page load (warm cache) | 300ms | 600ms | 1.2s |
| Canvas interaction (60fps pan/zoom) | 12ms | 20ms | 33ms |
| Typeahead search (library of 100k processes) | 40ms | 120ms | 300ms |
| Live recalculation (small study, in-browser WASM) | 30ms | 100ms | 250ms |
| Live recalculation (medium study, server) | 400ms | 1.2s | 3s |
| Full Monte Carlo (1,000 iters, medium study) | 15s | 45s | 120s |
| EPD document generation (PDF + docx) | 4s | 12s | 30s |
| Offline-to-online sync (typical edit session) | 1.5s | 6s | 20s |
| API p95 (read) | — | 180ms | 500ms |
| API p95 (write) | — | 500ms | 1.5s |

A proposed feature or library choice that violates these is rejected or explicitly justified with an ADR.

---

## 2. System Overview and Component Map

### 2.1 Physical and logical topology

```
                    ┌───────────────────────────────────┐
                    │       User devices                │
                    │  Browser (primary), mobile PWA    │
                    └──────────────┬────────────────────┘
                                   │ HTTPS, WSS, HTTP/3
                                   ▼
        ┌──────────────────────────────────────────────────┐
        │     Cloudflare (edge, WAF, DDoS, CDN, DNS)       │
        │     Fungible: HAProxy-based fallback ready       │
        └──────────────────────┬───────────────────────────┘
                               │
                               ▼
        ┌──────────────────────────────────────────────────┐
        │     Arko Platform (Hetzner bare-metal, EU)       │
        │     Talos Linux → k0s Kubernetes cluster         │
        │                                                  │
        │   ┌─────────────┐    ┌─────────────────────┐    │
        │   │  Ingress    │    │   Applications       │    │
        │   │  Traefik    │◄───┤   - arko-web (Next)  │    │
        │   │             │    │   - arko-api (Rust)  │    │
        │   └─────────────┘    │   - arko-calc (Rust) │    │
        │                      │   - arko-sync (Rust) │    │
        │                      │   - arko-docs (Node) │    │
        │                      │   - arko-ingest (Py) │    │
        │                      └──────────┬───────────┘    │
        │                                 │                │
        │   ┌─────────────────────────────▼───────────┐    │
        │   │   Platform services (self-hosted)        │    │
        │   │   CloudNativePG (Postgres)               │    │
        │   │   Keycloak (auth)                        │    │
        │   │   MinIO (S3-compatible storage)          │    │
        │   │   Meilisearch (search)                   │    │
        │   │   Redis Cluster (cache)                  │    │
        │   │   NATS JetStream (messaging)             │    │
        │   │   HashiCorp Vault (secrets)              │    │
        │   │   VictoriaMetrics + Loki + Tempo         │    │
        │   │   Grafana (observability)                │    │
        │   │   GlitchTip (errors)                     │    │
        │   │   Postal (transactional email)           │    │
        │   │   Forgejo + Woodpecker (code + CI)       │    │
        │   └──────────────────────────────────────────┘    │
        └──────────────────────────────────────────────────┘
                               │
                               ▼ (controlled egress only)
        ┌──────────────────────────────────────────────────┐
        │     External services (leased, abstracted)       │
        │     - Anthropic API (Claude), via abstraction   │
        │     - Redsys (EU billing), via abstraction       │
        │     - Optional Postmark (email fallback)         │
        └──────────────────────────────────────────────────┘
```

### 2.2 Cluster layout (V1 → V2)

**Production cluster, V1 launch (0–500 customers):**

- 3× Hetzner EX101 (AMD Ryzen 9 7950X3D, 128 GB RAM, 2× 1.9TB NVMe) — Kubernetes control plane + general workloads
- 2× Hetzner EX130 (Intel Xeon, 256 GB RAM, 2× 3.84TB NVMe) — Postgres CloudNativePG primary + replica
- 1× Hetzner EX44 — MinIO dedicated storage node (additional MinIO replicas co-locate on main nodes)
- 1× Hetzner CX52 (cloud VM) — observability cluster (VictoriaMetrics, Loki, Grafana), isolated from production plane for blast-radius containment
- 1× Hetzner CX22 — bastion host, WireGuard VPN endpoint, emergency access

All on private network (Hetzner vSwitch). Public egress only through Cloudflare. Monthly baseline: ~€550.

**Scaled cluster, V2 (1,000–5,000 customers):** add 3× more EX101 for compute, second Postgres replica pair (read-only), MinIO erasure-coded to 4 nodes, dedicated calc-engine node with higher CPU.

### 2.3 Networking and trust boundaries

- **Public ingress** through Cloudflare only. Hetzner origin IPs are firewalled to accept traffic only from Cloudflare IP ranges (bypassed only for health checks from our monitoring).
- **Cluster internal network** on Hetzner vSwitch (Layer 2 private). All service-to-service traffic encrypted via mTLS (Cilium service mesh at the CNI layer, no separate mesh complexity).
- **Admin access** only via WireGuard VPN into the bastion. SSH keys issued via HashiCorp Vault with short-lived certificates (not static keys). Every session logged.
- **Database access** impossible from public internet. PGBouncer accessible only from application pods. Admin queries via port-forward over WireGuard + time-boxed Vault credentials.

### 2.4 Request flow examples

**A user loads a study page:**

1. Browser → Cloudflare (cached static assets, HTML shell) → rendered
2. Service worker checks IndexedDB for the study → render immediately with cached data
3. Parallel fetch: `GET /v1/studies/:id` → Cloudflare → Traefik → arko-api (Rust) → PGBouncer → Postgres → response
4. Response validated against cached version; if newer, React Query invalidates and re-renders
5. WebSocket connects to arko-sync → loads Y.Doc state → presence broadcast

**A user edits a flow amount:**

1. UI applies optimistic update to Y.Doc (local CRDT state)
2. Yjs update dispatched to arko-sync via WebSocket
3. arko-sync broadcasts to all other connected clients for this study
4. arko-sync persists Y.Doc snapshot to Postgres every 30 seconds (or on user disconnect)
5. arko-sync also translates the Yjs update to a relational mutation, applied to the structured tables via transactional outbox pattern
6. Client-side: WASM calc engine re-computes locally (small study) or arko-api enqueues a calc request via NATS (large study)
7. Result renders; user sees update within 100ms for small study, within 1.5s for large

**A user generates an EPD:**

1. UI → `POST /v1/deliverables` → arko-api creates deliverable record (status: pending)
2. arko-api publishes event to NATS JetStream `doc-gen.epd.requested`
3. arko-docs worker consumes event, fetches study + results from arko-api
4. arko-docs renders docx (via docx-js) and PDF (via headless Chromium)
5. Generated files uploaded to MinIO with a signed URL (1-hour expiry)
6. arko-docs publishes `doc-gen.epd.ready` event
7. arko-api updates deliverable record, emits WebSocket event to user
8. UI's download button becomes active

---

## 3. Frontend Architecture

### 3.1 Framework and core stack

**Next.js 15 App Router** is the frontend framework.

Rationale: mature, excellent streaming SSR, proven at scale, shared skill set with KarbonGarbi, excellent TypeScript integration, route-level code splitting by default. React Server Components for the dashboard and static content (marketing, examples, docs). Client components for interactive work (canvas, table, results).

We self-host the Next.js app on our Kubernetes cluster in production, not Vercel. Rationale: sovereignty commitment C1. The cost: we give up Vercel's edge network (acceptable; Cloudflare CDN covers static assets), preview-per-PR (we build our own via Woodpecker CI creating ephemeral Kubernetes namespaces), and zero-config deployment (we have Kubernetes).

For local development and PR previews we use a Next.js standalone server in a Docker container deployed as ephemeral Kubernetes deployments with Traefik-generated URLs.

**React 19** with concurrent rendering, transitions, improved Suspense.

**TypeScript 5.x in strict mode.** No `any` outside generated types from OpenAPI schema. Types are generated deterministically from the backend's OpenAPI spec on every build.

### 3.2 Routing and structural layout

```
app/
  (marketing)/                      # public, unauthenticated
    page.tsx                        # landing
    pricing/
    examples/                       # worked-example gallery (public)
    blog/
    docs/                           # product documentation
  (auth)/
    login/
    signup/
    invite/[token]/
    forgot-password/
  (app)/                            # authenticated, workspace-scoped
    layout.tsx                      # topbar, sidebar, workspace switcher
    dashboard/
    studies/
      page.tsx                      # list with filters, search
      new/                          # goal & scope wizard
      [studyId]/
        page.tsx                    # study overview
        model/                      # canvas + table (the heavy lift)
        results/                    # results workspace
        deliverables/               # exports, EPDs, reports
        history/                    # version history, diff view
        audit/                      # audit log (Enterprise)
        settings/
    library/
      processes/
      methods/
      pcrs/
      templates/
      custom-factors/               # workspace's own factors
    team/
    billing/
    settings/
  (share)/                          # public share links, unauthenticated
    s/[token]/                      # view a shared study
    p/[token]/                      # view a shared EPD draft
  api/                              # thin BFF layer only
    auth/[...nextauth]/             # Keycloak callback handler
    ai/stream/                      # SSE proxy to arko-api
    upload/                         # multipart proxy with size checks
```

The `api/` routes are a **thin BFF** only. They handle session cookies, SSE proxying, file upload streaming. No business logic. Business logic lives in arko-api (Rust). This keeps the backend reusable for SDK, mobile, automation.

### 3.3 State management

Three disciplined state categories with three tools:

**Server state → TanStack Query v5 + generated typed hooks.**

Every API call goes through a typed hook generated from the OpenAPI schema. The generator (`openapi-typescript-codegen` configured per our conventions) produces hooks like `useGetStudy(id)`, `useCreateModel()`, etc. with full type safety, optimistic updates, and automatic cache invalidation.

TanStack Query persists its cache to IndexedDB via `@tanstack/query-persist-client-core` for offline support.

**Client state → Zustand with persistence middleware.**

One global store with feature slices: canvas state (selection, viewport, mode), UI state (open panels, modal stacks), user preferences (table column visibility, keyboard shortcut profile), onboarding state.

Persisted slices go to localStorage (preferences) or IndexedDB (canvas layout). Zustand over Redux: simpler API, smaller footprint, better TypeScript inference.

**Collaborative state → Yjs.**

The study model (models, flows, parameters, canvas positions, presence) is a Y.Doc. This is the *real-time source of truth* during active collaboration. Local edits apply to the Y.Doc first (instant), propagate to arko-sync, broadcast to collaborators.

Integration bridge: a Zustand slice subscribes to Y.Doc events and exposes a synchronous React-friendly API. Components read from Zustand; writes go to Yjs.

### 3.4 The canvas editor — our primary UI surface

The most demanding UI. Specifications:

**Library: ReactFlow 12**, with significant customization.

Why ReactFlow: handles 1,000+ nodes efficiently via virtualization, pan/zoom/minimap built in, custom node components via React, programmatic control via hooks, proven in production at n8n, Zapier, Retool.

**Auto-layout: ELK.js** for initial layout of loaded studies, **Dagre** for incremental re-layout on node additions. Both run in a Web Worker to keep the main thread at 60fps.

**Custom node types:**
- `AssemblyNode` — user-built models composed of Parts
- `PartNode` — sub-component of an assembly
- `UnitProcessNode` — leaf process (library-linked)
- `ElementaryFlowNode` — nature/emissions terminal (styled distinctly from processes)
- `StageNode` — lifecycle stage container (A1, A2, A3, B1, C1–C4, D per EN 15804, or custom)
- `VariantNode` — variant container for comparisons (top-level group)
- `ParameterNode` — floating parameter nodes for visual formula construction (V2)

**Custom edge types:**
- `MassFlowEdge` — thickness proportional to mass flow, label shows amount + unit, color by material type
- `EnergyFlowEdge` — dashed, distinct color
- `CoproductEdge` — fan-out from multi-output process with inline allocation percentages
- `AvoidedProductEdge` — distinct color (green), indicates substitution modeling
- `WasteFlowEdge` — distinct styling for end-of-life routing

**Interactions (documented, non-obvious, powerful):**
- Drag from a node's output port → creates a new edge; dropping on empty space opens an inline search for what to connect to (semantic search via embeddings, not just string match)
- Shift+click → multi-select; bulk operations (change geography, scale amounts, update data source)
- Alt+drag → duplicate a node with all its connections
- Cmd+click on a node → drills into its sub-model (recursive navigation)
- Cmd+Shift+F → fit selection to viewport
- Keyboard: arrow keys pan, `+`/`-` zoom, `0` reset, `F` fit all, `C` center on selection
- Right-click → context menu (node-specific actions, inline documentation links)

**Performance:**
- `React.memo` on every node component with explicit prop comparison
- Node positions stored in Y.Doc (not React state) to avoid re-renders on pan/zoom
- ReactFlow's native virtualization — only visible nodes render
- Canvas re-renders throttled via `useDeferredValue` for smooth interaction
- Target: 60fps pan/zoom on a 500-node graph on a 2020 MacBook Air

### 3.5 The table editor — our second primary UI surface

When users switch from Canvas to Table, the same data presented as a structured grid.

**Library: TanStack Table v8 + TanStack Virtual.**

Column virtualization for 50+ columns (all flow attributes visible when expanded). Row virtualization for 100,000+ rows. Cell-level editing with optimistic updates bound to the same Y.Doc.

**Column model:**
Part · Flow type · Direction · Name · Linked process (with library + geography) · Amount · Unit · Distribution · GSD² or range · Pedigree (5 dots, inline editor) · License tier · Data year · Source citation · Comments · (more, hidden by default)

Columns show/hide via a user-preference panel, persisted per user.

**Faceted filters above the table** — removable chips: `[Geography: Japan]` `[Year: ≥2015]` `[License: open]` `[Pedigree: ≥3 reliability]`. Filters apply to both canvas and table views (shared state).

**Bulk operations via selection + action bar:**
- "Replace geography in selected" (with unit conversion confirmation)
- "Update data year"
- "Scale amounts by factor"
- "Change linked library"
- "Assign pedigree scores"
- "Add comments" (with @mentions)

**Spreadsheet-friendly features:**
- Paste from Excel: detect CSV/TSV in clipboard, map columns via AI assistance (Claude), preview the mapping, confirm, apply as a single bulk edit (undoable)
- Copy to Excel: formatted output that round-trips (with provenance preserved in a metadata sheet)
- Formula bar for parameter-linked cells (like Airtable formula fields, not Excel's full spreadsheet semantics)

### 3.6 Component library

**shadcn/ui** as the base — the KarbonGarbi pattern, same for Goibix consistency. Radix UI primitives styled with Tailwind, code copied into our repo (we own it, can modify freely).

**Tailwind CSS 4.x** with design tokens defined in `tailwind.config.ts`. Tokens are the source of truth for:
- Color system (brand, accent, neutral, semantic: success/warning/error/info, impact category colors)
- Spacing scale (4px base, consistent across Goibix)
- Typography scale (Inter Variable for UI, JetBrains Mono for numbers/code, Lora for long-form prose)
- Elevation, radii, shadows, transitions

Tokens exported to Figma Tokens for design-dev alignment. Tokens are part of the published design system; changes require review.

**Icons: Lucide React** (consistent with KarbonGarbi, the shadcn default).

**Charts: layered approach**
- **Recharts** for standard charts (bars, lines, stacked bars, pie) — covers 80% of result displays
- **Observable Plot** for statistical visualizations (violin plots, box plots, quantile ribbons for Monte Carlo results)
- **D3.js directly** for custom visualizations (Sankey for flow diagrams, network graph alternatives, comparison trade-off matrices)
- **Visx** for React-native complex charts that need D3 power

**Typography:**
- **Inter Variable** for UI — self-hosted (bunny.net mirror, not Google Fonts)
- **JetBrains Mono** for numeric values, code, technical data
- **Lora** for long-form prose (docs, blog, EPD rendered content)

### 3.7 Progressive Web App architecture

The frontend installs as a full PWA. Specifics:

**Manifest (`manifest.json`)** with icons for all platforms (192px, 512px, maskable variants), display mode `standalone`, theme color matching brand, start URL pointing to the dashboard.

**Service worker generated with Workbox**, with specific strategies:
- Static assets (JS, CSS, fonts, images): Cache-First, 1-year TTL, content-addressable filenames
- API GET responses: Stale-While-Revalidate, 5-minute background refresh
- API POST/PUT/PATCH/DELETE: Network-Only, fall through to offline queue on failure
- HTML shell: Network-First with cache fallback
- `/s/:token` share links: Cache-First (share content rarely changes)

**Offline outbox in IndexedDB** — mutations queued locally when offline, synced on reconnect with conflict detection.

**Install prompt** — shown after 3rd visit, dismissible, respects user preferences.

### 3.8 Accessibility

Not optional. Commitments:

- **WCAG 2.2 AA** baseline across the app
- All interactive elements keyboard-accessible with visible focus rings
- Screen-reader-friendly: proper ARIA labels, live regions for async updates, landmark roles
- Color is never the sole carrier of information (comparison charts use patterns + colors)
- Zoom to 200% without horizontal scroll
- Prefers-reduced-motion respected
- All charts have accessible data tables as a fallback
- Automated axe-core audits in CI, manual review quarterly

### 3.9 Internationalization

**Day-one languages:** English, Spanish. **Month-3:** Basque. **Month-6:** French. **Month-9:** Portuguese. **Month-12:** German, Italian.

Implementation: `next-intl` for runtime locale switching. Translation catalogs managed via Crowdin (community contributions welcome).

Critical standards:
- Not just UI strings — error messages, validation feedback, email templates, document templates, AI system prompts all localized
- Date, number, and currency formatting via `Intl` native APIs
- RTL support readied for future (V3+) Arabic, Hebrew
- Locale-aware units (metric always default; imperial optional per user preference)

---

## 4. Backend Services Architecture

### 4.1 Why Rust for the main API

The original v1.0 spec chose FastAPI (Python) for arko-api, following KarbonGarbi's pattern. The v2.0 Excellence Edition revisits this.

Rust for arko-api, arko-calc, and arko-sync. Python only where Python's ecosystem is essential (arko-ingest for scientific data pipelines). Node.js only for arko-docs because docx-js is JavaScript-native.

Rationale for Rust:
- **Predictable performance** — no GIL, no garbage collection pauses, consistent sub-millisecond handler latency
- **Memory safety without GC** — correctness by construction, no data races
- **Single binary deployment** — small, fast-starting, no runtime dependencies
- **Excellent type system** — aligns with our specification-first philosophy
- **Shared Rust ecosystem** with the calc engine and sync engine (code sharing of types and algorithms)
- **Axum framework** is production-proven, ergonomic, composable
- **SQLx for Postgres** with compile-time query checking — SQL validated at build time

The cost: Rust has a learning curve. Samir (primarily a designer) isn't a Rust expert. But:
- We invest in learning Rust properly before building production code
- We pair with a consultant for the first 2 months (budget: €20-30k for 2 months of senior Rust mentoring)
- We document rigorously so future contributors can ramp faster
- The alternative — Python or Node — would compound technical debt we'd pay for over years

This is a deliberate choice: accept higher upfront cost to get a better long-term foundation.

### 4.2 arko-api — the main API service

**Stack: Rust 1.80+, Axum 0.7+, SQLx, Tokio runtime.**

Structure:

```
arko-api/
  src/
    main.rs                     # app factory, tracing init, graceful shutdown
    config.rs                   # layered config: defaults → file → env
    error.rs                    # unified error types with conversion traits
    auth/
      mod.rs
      jwt.rs                    # JWT verification (Keycloak public keys)
      session.rs                # session management
      rbac.rs                   # role-based access control
    middleware/
      audit.rs                  # mutation logging
      license_enforcement.rs    # see §8
      rate_limit.rs             # per-endpoint Redis-backed
      tracing.rs                # OpenTelemetry instrumentation
      workspace_scope.rs        # RLS context setting
    routes/
      studies.rs
      models.rs
      library.rs
      methods.rs
      pcrs.rs
      deliverables.rs
      team.rs
      billing.rs
      ai.rs
      webhooks.rs
      share.rs
    services/                   # business logic
      study_service.rs
      calc_client.rs            # client for arko-calc
      docs_client.rs            # client for arko-docs
      ai_service.rs
      billing_service.rs
      license_service.rs
    db/
      models.rs                 # query result types
      migrations/               # SQL migrations (sqlx-migrate)
      queries.rs                # type-checked queries
    events/
      publisher.rs              # NATS publisher
      schemas.rs                # event payloads
    grpc/                       # gRPC endpoints for internal service communication
      calc.proto
      docs.proto
    openapi/                    # OpenAPI schema generation
  tests/
    integration/
    e2e/
  benches/                      # criterion benchmarks
```

**API conventions:**

- All public endpoints under `/v1/`. `/v2/` introduced for breaking changes, `/v1/` supported for 18 months after `/v2/` GA.
- Resource-oriented URLs: `GET /v1/studies/:id`, `POST /v1/studies/:id/variants`.
- Cursor-based pagination: `?cursor=abc123&limit=50`. Offset pagination explicitly not supported (stability, performance).
- Consistent error envelope:
  ```json
  {
    "error": {
      "code": "FORBIDDEN",
      "message": "You don't have permission to view this study.",
      "details": { "resource": "study:abc123", "required_role": "editor" },
      "trace_id": "0af7651916cd43dd8448eb211c80319c"
    }
  }
  ```
- Every endpoint documents required permissions in the OpenAPI schema.
- Idempotency keys for mutations: clients can send an `Idempotency-Key` header for exactly-once semantics on critical operations (billing, document generation).

**Rate limiting** via Redis-backed sliding window, tiered by plan:

| Plan | Read (req/min) | Write (req/min) | AI (req/min) | Export (req/min) |
|---|---|---|---|---|
| Free | 60 | 20 | 5 | 2 |
| Studio | 300 | 60 | 20 | 10 |
| Team | 600 | 200 | 60 | 30 |
| Enterprise | Negotiated | Negotiated | Negotiated | Negotiated |

Bursts allowed up to 2× steady rate for 30 seconds. Limits logged to audit.

**Per-request tracing** via OpenTelemetry. Trace ID in every log line, every external call, every error response. Full distributed tracing across the cluster.

### 4.3 arko-ingest — data ingestion and library management

**Stack: Python 3.12, Pydantic, Polars, Celery-compatible workers via Arq (Redis-backed async task queue).**

Rationale for Python here: the scientific data ecosystem is Python. Brightway is Python. ECOSPOLD and ILCD parsers are Python. Agribalyse distributes in formats Python handles natively. Fighting this would be perverse.

Responsibilities:
1. Nightly sync of open data bundles (Agribalyse, USDA LCA Commons, ELCD, PEFAPs)
2. Monthly check for method updates (RIVM's ReCiPe, JRC's EF, IPCC)
3. Ingest PCR documents via Git webhook (community PRs to our PCR repo)
4. Re-index Meilisearch after library changes
5. Re-compute embeddings for semantic search (via Claude Haiku or self-hosted BGE)
6. Validate library integrity (no dangling references, characterization completeness)
7. Cross-validate via Brightway: our ingested data → Brightway project → run reference calculations → confirm results match expected values

**Library versioning:** ingested libraries are immutable once published. Updates produce new versions. Studies pin specific versions for reproducibility. Users get notifications when new library versions are available, with a diff showing what changed.

### 4.4 arko-docs — document generation

**Stack: Node.js 22 LTS, Fastify, docx-js, Puppeteer (Chromium).**

Why Node: docx-js is the best-in-class programmatic Word generator, JavaScript-native. Puppeteer integrates best with Node. Templates in TSX share code with the frontend's React components.

This service is small and changes rarely. Different language from arko-api is fine.

Responsibilities:
1. Render Word (.docx) documents from templates + study data
2. Render PDF via Puppeteer (headless Chromium) from HTML templates
3. Generate Excel exports via ExcelJS
4. Generate CSV, JSON, ECOSPOLD v2, ILCD exports
5. Apply license-aware rendering (watermarks, static-only mode, attribution)

**Template structure:** per §10.

### 4.5 arko-sync — real-time collaboration server

**Stack: Rust 1.80+, custom CRDT server with Yjs wire protocol compatibility, Tokio, Redis Streams for distribution, Postgres for snapshots.**

Why Rust and custom rather than Node.js + y-websocket: the v1.0 spec chose y-websocket (Node) for speed of implementation. v2.0 Excellence Edition chooses a Rust implementation of the Yjs wire protocol for three reasons:

1. **Memory safety and predictable performance** at 10,000+ concurrent connections
2. **Formal state machine specification in TLA+** — the CRDT server is a distributed state machine, and TLA+ is the right tool for proving properties. Rust's type system aligns with this formal mindset.
3. **Shared types with arko-api** — flow validation, access control, license enforcement are shared code

**The CRDT specification is public** (published at `spec.arko.earth/crdt`), so customers and auditors can verify correctness.

Responsibilities:
1. Hold persistent WebSocket per user per study
2. Broadcast Yjs updates across all clients viewing a study
3. Snapshot Y.Doc state to Postgres on a schedule (every 30s, on disconnect, on explicit save)
4. Relay presence information (cursors, selections, who's online)
5. Translate Yjs updates to relational mutations via transactional outbox pattern
6. Enforce per-study access control (reject unauthorized clients)

**Scaling strategy:**
- Redis Streams distribute updates across arko-sync instances (each instance subscribes to per-study streams it has active clients for)
- Cloudflare supports WebSocket — clients connect to any arko-sync instance transparently
- Sticky sessions at the load balancer layer via cookie-based routing (Traefik)
- Target: 10,000 concurrent WebSocket connections per arko-sync instance on a Hetzner EX101

---

## 5. The Calculation Engine (our core IP)

This is where Arko earns its technical credibility. The v1.0 spec wrapped Brightway. The v2.0 Excellence Edition builds our own engine, with Brightway as a differential-testing reference.

### 5.1 The mathematical specification

An LCA calculation is a sparse linear system with characterization, where:

**Given:**
- **A** ∈ ℝⁿˣⁿ — technosphere matrix; A[i,j] is the amount of product j consumed to produce one unit of product i. Typically 99%+ sparse.
- **B** ∈ ℝᵐˣⁿ — biosphere matrix; B[k,j] is the amount of elementary flow k emitted/consumed per unit of product j.
- **f** ∈ ℝⁿ — demand vector; typically zeros with a 1 in the reference product's position.
- **C** ∈ ℝᵖˣᵐ — characterization matrix; C[l,k] is the characterization factor for flow k in impact category l.

**Compute:**
1. **s** = A⁻¹ f (the scaling vector) — product scaling to meet demand
2. **g** = B · s (life cycle inventory) — total elementary flows
3. **h** = C · g (impact results per category)

**Edge cases the mathematics must handle:**
- Circular exchanges in the technosphere (steel → car → … → steel)
- Multi-output processes requiring allocation (chemical plant producing product + heat)
- Avoided products (recycling credit, substitution)
- Zero-demand products (intermediate products not directly demanded)
- Negative exchanges (by convention, inputs are negative in some formulations)
- Parameterized amounts (formulas that resolve at calculation time)
- Non-invertible A matrices (structural errors in study; must produce actionable error messages)

### 5.2 Implementation: Rust, from first principles

**Language: Rust 1.80+.**

**Crates:**
- `nalgebra-sparse` for sparse matrix operations (CSR/CSC formats)
- `sprs` as an alternative for specific operations where it's faster
- `suitesparse-sys` bindings to UMFPACK (the gold-standard sparse LU solver used in MATLAB)
- `faer` as a pure-Rust alternative solver for comparison/fallback
- `serde` for serialization
- `proptest` for property-based testing
- `wasm-bindgen` for the WASM compilation target

**Architecture:**

```
arko-calc-core/
  src/
    lib.rs
    spec/                     # the published mathematical specification
      mod.rs
      matrix.rs               # formal definitions
      characterization.rs
    engine/
      solver.rs               # sparse LU via UMFPACK
      allocator.rs            # allocation logic (mass, economic, energy)
      parameterizer.rs        # formula resolution
    model/
      technosphere.rs         # A-matrix construction
      biosphere.rs            # B-matrix construction
      method.rs               # C-matrix construction
    uncertainty/
      pedigree.rs             # Weidema matrix, pedigree-to-distribution
      monte_carlo.rs
      sensitivity.rs          # local + global sensitivity
    contribution/
      tree.rs                 # supply chain contribution analysis
      hotspots.rs
    io/
      brightway_compat.rs     # read Brightway project format for migration
      ecospold2.rs
      ilcd.rs
      arko_native.rs          # our efficient native format
    audit/
      manifest.rs             # signed calculation manifest
  benches/
    against_brightway.rs      # differential benchmarks
  tests/
    reference_studies/        # 10,000+ reference cases
    properties/               # property-based tests
    numerical/                # numerical stability tests
```

### 5.3 The correctness regime

**Differential testing against Brightway + OpenLCA:**

A test suite of 10,000+ reference studies, drawn from:
- Brightway's own example projects (their published test suite, contributed back)
- OpenLCA's reference cases
- Published LCA studies with full inventory data (e.g., ecoinvent's system process validation cases)
- Synthetic cases generated to probe edge conditions (circular references, near-singular matrices, extreme scale differences)

Every CI run:
1. Loads each reference study
2. Computes results via our Rust engine
3. Computes results via Brightway (as a Python subprocess)
4. Computes results via OpenLCA (via olca-ipc where applicable)
5. Asserts results match to within 1e-9 relative tolerance
6. Any divergence beyond tolerance fails the build

This gives us empirical correctness evidence beyond what any single reference implementation provides.

**Property-based testing via proptest:**

Invariants encoded as properties:
- For any non-negative A, B, f: result is non-negative
- For any scaling c > 0: calc(c·f) = c·calc(f) (linearity)
- For any permutation of row/column ordering: results are invariant
- For empty biosphere: all elementary flow impacts are zero
- For disconnected subgraphs: combined result equals sum of isolated results

Proptest generates thousands of random cases per property, shrinking failures to minimal reproductions.

**Numerical stability analysis:**

Documented explicitly what tolerances we meet:
- Results accurate to 1e-6 relative error for well-conditioned A matrices (condition number <1e10)
- Results accurate to 1e-3 for moderately ill-conditioned (1e10–1e14)
- Actionable errors for ill-conditioned (>1e14) with suggested model fixes

Condition number reported with every result for transparency.

**Formal specification of the calculation:**

A published specification at `spec.arko.earth/calc` that documents:
- The mathematical definition
- Our implementation's compliance with the definition
- The test suite that verifies compliance
- Known limitations and their conditions
- Versioning and compatibility guarantees

This specification is the primary artifact. The implementation is one realization of it; future improvements (a Rust-native sparse solver replacing UMFPACK, GPU acceleration for Monte Carlo, etc.) must continue to satisfy it.

### 5.4 The WASM compilation target

The calc engine compiles to two targets:

1. **Native Rust binary** for server-side calculation (arko-calc service)
2. **WebAssembly** for in-browser calculation

Binary size budget for WASM: <1.5 MB gzipped. Instantiation time: <100ms on a 2020 MacBook Air.

**What WASM supports:**
- Deterministic calculation
- Pedigree-derived uncertainty
- Small Monte Carlo (up to 100 iterations)
- Sensitivity analysis
- Contribution tree analysis

**What WASM doesn't do (falls through to server):**
- Large Monte Carlo (1,000+ iterations)
- Complex methodology comparisons
- Ecoinvent-scale calculations (memory pressure on mobile)

Transparent fallback: the client library tries WASM first, falls through to arko-calc over HTTP if the study is too large or the operation unsupported.

### 5.5 Incremental recalculation

When a user changes a single input, we don't re-solve the entire system.

**Sherman-Morrison updates:**

For a rank-1 update to A (changing one matrix entry), we update A⁻¹ in O(n²) rather than recomputing in O(n³). For typical studies, this is the difference between 5ms and 500ms.

**When to incremental vs full re-solve:**
- Single flow amount change → Sherman-Morrison (always)
- Adding/removing a flow (structural change) → full re-solve (mandatory)
- Changing method → recompute only the C-matrix multiplication, reuse A⁻¹ f and B·s

**Result caching:**

Results cached in Postgres keyed by `hash(study_state, method_version, library_versions)`. Cache lookup is O(1). Hit rate in practice is high because most "calculations" are re-requests of already-computed results.

### 5.6 Monte Carlo and uncertainty

**Default Monte Carlo:**
- Pedigree-derived distributions computed automatically via the Weidema matrix for every input lacking explicit uncertainty
- 1,000 iterations for standard analysis (sufficient for 5% confidence on mean)
- 10,000 iterations for publication-grade (V2 feature, optional)
- Parallel execution across CPU cores via Rayon
- Sobol sequence option for faster convergence (V2)

**User-specified uncertainty:**
- Override defaults with explicit distributions: Normal, Lognormal, Triangular, Uniform, Discrete
- Distributions can be parameterized (mean, SD, or user-provided samples)

**Output:**
- Mean, median, 2.5th and 97.5th percentiles per impact category
- Full histogram data (binned for visualization)
- Correlation matrix: which inputs drive which outputs (Pearson for linear, Spearman for rank)
- Covariance matrix (for advanced users)

**Sensitivity analysis:**
- **Local:** ∂(result)/∂(input) via automatic differentiation (forward-mode via `ad` crate)
- **Global (V2):** Sobol indices for high-dimensional studies
- **Pareto frontier analysis (V3):** for multi-objective decisions

### 5.7 Matrix export for power users

Per Principle P99 of the Master Spec, matrix export is a first-class API:

```
GET /v1/studies/:id/matrix?format=numpy         # .npz, scipy sparse
GET /v1/studies/:id/matrix?format=csv           # three CSV files, zipped
GET /v1/studies/:id/matrix?format=messagepack   # .mpk, binary compact
GET /v1/studies/:id/matrix?format=brightway     # Brightway-compatible project export
GET /v1/studies/:id/matrix?format=ecospold2     # ECOSPOLD v2 XML
GET /v1/studies/:id/matrix?format=ilcd          # ILCD zipped XML
```

Returns:
- A.npz — technosphere sparse matrix
- B.npz — biosphere sparse matrix
- labels.json — index-to-name mapping
- method.json — characterization vectors
- study.json — functional unit, parameters, provenance
- manifest.json — signed manifest for reproducibility

Academic users and power users run their own analyses in MATLAB, R, Python. We don't gate their workflows behind our UI.

### 5.8 The published specification and benchmarks

Commitments:

- **Specification:** `spec.arko.earth/calc` — versioned, changelog, SemVer semantics
- **Open benchmark suite:** `github.com/goibix/arko-calc-bench` — 10,000+ studies with known correct results, anyone can run against any engine
- **Reproducibility manifests:** every calculation result includes a signed manifest (SHA-256 over study state, library versions, method version, calc engine version). Third parties can independently verify.
- **Open-source under Apache 2.0:** the calc engine itself is open-source. Moat is product, not algorithm.
- **Academic collaboration:** we publish at SETAC. We contribute to Brightway upstream. We're good citizens of the LCA community.

### 5.9 What this costs in time

Honest assessment of the calc engine's build cost:

- Foundations (Rust project, sparse matrix ops, basic solver): **6-8 weeks**
- Brightway-equivalent feature parity (allocation, parameters, substitution): **8-10 weeks**
- Differential testing infrastructure (10k reference studies, Brightway subprocess orchestration): **4-6 weeks**
- Property-based tests, numerical stability analysis: **3-4 weeks**
- WASM compilation and browser integration: **4-6 weeks**
- Monte Carlo and sensitivity: **4-5 weeks**
- Matrix export formats: **2-3 weeks**
- Documentation, published specification, open benchmarks: **4-5 weeks**

**Total: ~40 weeks of focused work.** Not calendar weeks — focused work. With a learning curve, code review, integration, this is realistically **12 months of the core engineering work.**

This is why we commit to it only for Arko V1 and don't try to do it alongside formal verification. One hard thing done well.

---

## 6. The Real-Time Sync Engine (our second core IP)

### 6.1 The problem

LCA studies are collaborative. Two consultants on a client project, a senior reviewing a junior's work, an expert filling gaps a business user started — all these are real workflows.

SimaPro has no collaboration. Google-Docs-style editing for LCA is genuinely novel.

The challenge: LCA studies are structured, not text. Concurrent edits must preserve model integrity. Offline edits must merge cleanly. Conflicts must be rare and resolvable.

### 6.2 The approach: Yjs protocol, Rust server, formal specification

**Client library: Yjs** — battle-tested, extensively studied CRDT library. Fighting the reference implementation here would be a mistake.

**Server: Rust implementation of the Yjs protocol.** Why not use `y-crdt` (the official Rust port) directly? We do, as a dependency, but we wrap it in our own server that adds:
- Authentication and authorization (which users can read/write which studies)
- Persistence (Y.Doc snapshots to Postgres at scheduled intervals)
- Relational mutation derivation (Yjs updates → structured mutations applied to Postgres tables)
- Access control validation on every update
- Rate limiting per user/study
- Audit logging

**State machine specification in TLA+:**

The sync server is a distributed state machine. We specify it formally in TLA+ and use TLC model checker to verify properties:
- **Safety:** two clients that apply the same updates converge to the same state
- **Liveness:** updates from a connected client reach all other connected clients within a bounded time
- **Authorization:** unauthorized clients cannot mutate a study's state
- **Durability:** acknowledged updates are persisted (won't be lost on server restart)
- **Isolation:** workspace A's updates never propagate to workspace B's clients

The TLA+ spec is public at `spec.arko.earth/sync`. Model checking runs in CI.

### 6.3 The CRDT structure for an Arko study

The Y.Doc contains:

```typescript
// Conceptual schema (TypeScript shown for readability; Rust equivalents in code)
const ydoc = new Y.Doc()

const yMeta = ydoc.getMap('meta')              // title, description, updated_at
const yGoalScope = ydoc.getMap('goal_scope')   // functional unit, method, PCR
const yVariants = ydoc.getArray('variants')    // variants (each is Y.Map)
const yModels = ydoc.getMap('models')          // keyed by model id (Y.Map per model)
const yFlows = ydoc.getMap('flows')            // keyed by flow id (Y.Map per flow)
const yParameters = ydoc.getMap('parameters')  // keyed by parameter id
const yCanvas = ydoc.getMap('canvas')          // canvas layout (positions, viewport)
const yPresence = ydoc.getMap('presence')      // per-user presence (cursors, selection)
const yComments = ydoc.getArray('comments')    // comment threads
```

**Structural integrity maintained by:**
- Type checking at insert time (client-side, validated server-side)
- Referential checks: flows reference existing models; parameters reference existing scopes
- Server rejects updates that would violate structural invariants

### 6.4 Conflict resolution semantics

**Same-field concurrent edits:** last-write-wins by Lamport timestamp. UI shows both users' cursors during the edit.

**Structural conflicts:**
- Alice deletes a flow while Bob edits its amount → Bob's edit is orphaned, UI surfaces: "This flow was deleted by Alice. Your edit was discarded."
- Alice and Bob both add flows with the same order index → CRDT resolves deterministically, one gets the index, the other shifts

**Offline reconciliation:**
- Client maintains Y.Doc updates in IndexedDB while offline
- On reconnect, sends all pending updates to server
- Server applies updates with concurrency semantics, broadcasts reconciled state
- Client converges to server state

**Merge policy for high-conflict scenarios:**
- If a merge results in >10% of the study being changed from the client's last-known state, show user a merge dialog with the option to review before accepting
- Always allow rollback to the pre-merge state (versioning)

### 6.5 Persistence and recovery

**Snapshot strategy:**
- Y.Doc state persisted to Postgres every 30 seconds of activity
- Also persisted on user disconnect
- Full snapshots (not deltas) for simplicity — Y.Doc state is compact
- Snapshot retention: rolling 30 days for point-in-time recovery

**Recovery:**
- Client requests a study → server loads latest snapshot → Y.Doc reconstructed
- Client applies its local updates on top → state converges
- If server has newer state than client's last-known, server sends update

**Disaster recovery:**
- Postgres snapshot includes full Y.Doc state
- Postgres backups are the source of truth
- Arko-sync instance loss is a 30-second inconvenience, not data loss

### 6.6 Scaling approach

**Single arko-sync instance V1 capacity:**
- 10,000 concurrent WebSocket connections on a Hetzner EX101 (128 GB RAM)
- 1,000 studies concurrently active with >1 editor each
- 100 Yjs updates per second aggregate

**Horizontal scaling:**
- Multiple arko-sync instances behind Traefik WebSocket-aware load balancer
- Redis Streams as the distribution channel: each update published to `study.:id`, all instances subscribed to streams for studies with active clients
- Cross-instance state consistency: each instance maintains its own Y.Doc replicas, Redis Streams ensure convergence
- At 100,000 concurrent connections (Year 3+), shard by study ID hash across instances

### 6.7 The published specification

`spec.arko.earth/sync` publishes:
- Wire protocol (Yjs-compatible with our extensions)
- Authentication flow
- Access control semantics
- State machine (TLA+ spec)
- Persistence guarantees
- Conflict resolution rules
- Versioning policy

This transparency serves two purposes:
1. Auditors and Enterprise customers can verify correctness
2. Third parties can build compatible clients (unlikely but possible)

---

## 7. Data Model and Database Design

### 7.1 Engine and deployment

**PostgreSQL 16** via **CloudNativePG** (CNPG) on our Kubernetes cluster.

Why CloudNativePG: it's the Postgres Operator that treats Postgres as cattle — declarative cluster config, automated failover, streaming replication, WAL archiving to MinIO, rolling upgrades. Developed by EDB (Postgres experts), actively maintained, mature.

**Cluster configuration:**
- 1 primary + 2 hot standby replicas (synchronous replication to at least 1 replica)
- Replication lag monitored; alert at >5 seconds
- Point-in-time recovery via WAL archiving to MinIO (retention: 30 days rolling)
- Base backups nightly, retained 90 days
- Failover target: <30 seconds RTO via Patroni (included in CNPG)
- Connection pooling via PGBouncer sidecar
- Automatic vacuum tuning via pg_repack extension

**Extensions enabled:**
- `pgvector` for embedding search (semantic process matching)
- `pg_trgm` for fuzzy text matching
- `pgcrypto` for hashing and encryption
- `uuid-ossp` for UUID generation
- `pg_stat_statements` for query performance analysis
- `pg_partman` for audit log partitioning

### 7.2 Schema design philosophy

- **Every table has `workspace_id`** (for tenancy) or is globally shared (libraries, methods, PCRs — read-only for tenants)
- **Every table has `id uuid primary key`** — UUIDs generated with `gen_random_uuid()` (not sequential; unpredictable; safe to expose in URLs)
- **Every table has `created_at timestamptz`** and `updated_at timestamptz`
- **Soft deletes** via `deleted_at timestamptz null` — hard deletes only for GDPR compliance and unused draft data
- **JSON escape hatches** sparingly — `metadata jsonb` on tables where extensibility is valuable, but structured columns for queried fields
- **Row-level security** via workspace_id policy on every tenant-scoped table
- **Foreign keys everywhere** — referential integrity is non-negotiable
- **Check constraints** for enum-like string fields (no random strings getting in)
- **Indexes designed for actual query patterns**, not speculative — documented per index why it exists

### 7.3 Core schema

```sql
-- =================================================================
-- EXTENSIONS
-- =================================================================
create extension if not exists pgcrypto;
create extension if not exists pg_trgm;
create extension if not exists vector;
create extension if not exists pg_stat_statements;

-- =================================================================
-- TENANCY
-- =================================================================

create table workspace (
  id uuid primary key default gen_random_uuid(),
  slug text unique not null,
  name text not null,
  plan text not null default 'free'
    check (plan in ('free', 'studio', 'team', 'enterprise')),
  billing_external_id text,
  
  settings jsonb not null default '{}'::jsonb,
  
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now(),
  deleted_at timestamptz
);

create index workspace_slug_idx on workspace(slug) where deleted_at is null;
create index workspace_plan_idx on workspace(plan) where deleted_at is null;

create table workspace_member (
  workspace_id uuid not null references workspace(id) on delete cascade,
  user_id uuid not null,                         -- Keycloak user id
  role text not null
    check (role in ('owner', 'admin', 'editor', 'viewer', 'auditor')),
  joined_at timestamptz not null default now(),
  invited_by uuid,
  primary key (workspace_id, user_id)
);

create index workspace_member_user_idx on workspace_member(user_id);

create table workspace_license (
  -- tracks which paid libraries this workspace has access to
  workspace_id uuid not null references workspace(id) on delete cascade,
  library_id uuid not null references library(id),
  proof_url text,                                -- signed MinIO URL to license doc
  activated_at timestamptz not null default now(),
  expires_at timestamptz,
  seat_count int,                                -- for per-seat licensed libraries
  primary key (workspace_id, library_id)
);

-- =================================================================
-- STUDIES
-- =================================================================

create table study (
  id uuid primary key default gen_random_uuid(),
  workspace_id uuid not null references workspace(id) on delete cascade,
  
  -- identity
  title text not null,
  description text,
  status text not null default 'draft'
    check (status in ('draft', 'in_progress', 'in_review', 'finalized', 'archived')),
  intended_use text
    check (intended_use in ('internal', 'epd', 'marketing_claim', 'academic', 'regulatory', 'r_and_d')),
  
  -- goal and scope
  functional_unit text,
  functional_unit_amount numeric,
  functional_unit_unit text,
  system_boundary text
    check (system_boundary in ('cradle_to_gate', 'cradle_to_grave', 'gate_to_gate', 'cradle_to_cradle', 'custom')),
  geographic_scope text[],
  time_horizon_start date,
  time_horizon_end date,
  
  -- methodology (pinned for reproducibility)
  method_id uuid references method(id),
  method_version text,
  pcr_id uuid references pcr(id),
  pcr_version text,
  
  -- EPD mode
  epd_mode boolean not null default false,
  epd_program_operator text,                     -- 'environdec', 'ibu', 'epd_norge', etc.
  epd_target_submission_date date,
  
  -- data quality expectations
  proxy_data_threshold numeric check (proxy_data_threshold >= 0 and proxy_data_threshold <= 1),
  pedigree_required boolean not null default false,
  
  -- Yjs state
  yjs_state bytea,                               -- latest snapshot for recovery
  yjs_snapshot_at timestamptz,
  
  -- audit
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now(),
  created_by uuid not null,
  last_edited_by uuid,
  deleted_at timestamptz,
  
  -- search
  search_vector tsvector generated always as (
    setweight(to_tsvector('simple', coalesce(title, '')), 'A') ||
    setweight(to_tsvector('simple', coalesce(description, '')), 'B')
  ) stored
);

create index study_workspace_idx on study(workspace_id) where deleted_at is null;
create index study_search_idx on study using gin(search_vector) where deleted_at is null;
create index study_updated_idx on study(workspace_id, updated_at desc) where deleted_at is null;
create index study_method_idx on study(method_id);
create index study_pcr_idx on study(pcr_id);

-- RLS
alter table study enable row level security;
create policy study_isolation on study
  using (workspace_id = current_setting('app.workspace_id', true)::uuid);
create policy study_admin_bypass on study
  using (current_setting('app.role', true) = 'super_admin');

-- =================================================================
-- VARIANTS (for comparison)
-- =================================================================

create table variant (
  id uuid primary key default gen_random_uuid(),
  study_id uuid not null references study(id) on delete cascade,
  name text not null,
  description text,
  order_index int not null default 0,
  is_baseline boolean not null default false,
  created_at timestamptz not null default now()
);

create index variant_study_idx on variant(study_id, order_index);

-- Only one baseline per study
create unique index variant_baseline_unique on variant(study_id) where is_baseline = true;

-- =================================================================
-- LIFECYCLE STAGES
-- =================================================================

create table lifecycle_stage (
  id uuid primary key default gen_random_uuid(),
  variant_id uuid not null references variant(id) on delete cascade,
  code text not null,                            -- 'A1', 'A3', 'B1', 'C4', 'D', or custom
  name text not null,
  framework text not null default 'en15804'
    check (framework in ('en15804', 'iso_upstream_core_downstream', 'iso_21930', 'custom')),
  order_index int not null default 0,
  description text,
  
  created_at timestamptz not null default now(),
  unique (variant_id, code)
);

create index lifecycle_stage_variant_idx on lifecycle_stage(variant_id, order_index);

-- =================================================================
-- MODELS (the graph nodes)
-- =================================================================

create table model (
  id uuid primary key default gen_random_uuid(),
  variant_id uuid references variant(id) on delete cascade,
  lifecycle_stage_id uuid references lifecycle_stage(id),
  parent_model_id uuid references model(id),     -- for sub-assemblies
  
  kind text not null
    check (kind in ('assembly', 'part', 'unit_process', 'reference_link')),
  name text not null,
  description text,
  
  -- reference flow
  reference_flow_name text,
  reference_flow_amount numeric,
  reference_flow_unit text,
  
  -- canvas position (snapshot; live state is in Y.Doc)
  canvas_x numeric,
  canvas_y numeric,
  
  -- for reference_link kind: the library process referenced
  linked_process_id uuid references process(id),
  linked_process_version text,
  
  -- metadata
  metadata jsonb not null default '{}'::jsonb,
  
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now(),
  created_by uuid
);

create index model_variant_idx on model(variant_id);
create index model_parent_idx on model(parent_model_id);
create index model_stage_idx on model(lifecycle_stage_id);
create index model_linked_process_idx on model(linked_process_id);

-- =================================================================
-- FLOWS (inputs and outputs of models)
-- =================================================================

create table flow (
  id uuid primary key default gen_random_uuid(),
  model_id uuid not null references model(id) on delete cascade,
  
  direction text not null
    check (direction in ('input', 'output', 'emission', 'waste')),
  category text not null
    check (category in (
      'technosphere_material', 'technosphere_energy',
      'nature_resource', 'nature_land_use', 'nature_water_use',
      'emission_air', 'emission_water', 'emission_soil',
      'non_material', 'product_output', 'coproduct_output', 'avoided_product'
    )),
  subcategory text,                              -- 'urban', 'rural', 'stratosphere', 'freshwater', etc.
  
  -- What the flow is
  flow_kind text not null
    check (flow_kind in ('linked_process', 'linked_model', 'elementary', 'custom')),
  linked_process_id uuid references process(id),
  linked_model_id uuid references model(id),     -- for internal intra-study links
  elementary_flow_id uuid references elementary_flow(id),
  custom_name text,
  
  -- Amount
  amount numeric not null,
  unit text not null,
  
  -- Uncertainty
  distribution text
    check (distribution is null or distribution in ('lognormal', 'normal', 'triangular', 'uniform', 'discrete')),
  distribution_params jsonb,
  
  -- Pedigree (Weidema matrix, 5 dimensions, 1-5 scale)
  pedigree_reliability int check (pedigree_reliability between 1 and 5),
  pedigree_completeness int check (pedigree_completeness between 1 and 5),
  pedigree_temporal int check (pedigree_temporal between 1 and 5),
  pedigree_geographic int check (pedigree_geographic between 1 and 5),
  pedigree_technological int check (pedigree_technological between 1 and 5),
  
  -- Provenance
  data_source text
    check (data_source in ('primary', 'secondary_generic', 'proxy', 'estimated', 'ai_suggested', 'unknown')),
  data_year int,
  data_region text,
  
  -- License tier (the critical field for §8)
  license_tier text not null default 'open'
    check (license_tier in (
      'open',                    -- open data (Agribalyse, USDA, ELCD, PEFAPs)
      'ecoinvent',               -- ecoinvent licensed
      'agri_footprint',          -- Agri-footprint licensed
      'industry_data',           -- Industry Data 2.0
      'user_private',            -- user's own, workspace-private
      'user_shared',             -- user's own, shared within workspace or publicly
      'proprietary'              -- other commercial library
    )),
  
  -- Allocation (for multi-output processes)
  allocation_method text
    check (allocation_method is null or allocation_method in ('mass', 'economic', 'energy', 'volume', 'none')),
  allocation_factor numeric check (allocation_factor is null or (allocation_factor >= 0 and allocation_factor <= 1)),
  
  -- Annotation
  comment text,
  order_index int not null default 0,
  
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now(),
  
  -- Constraint: exactly one of linked_process_id, linked_model_id, elementary_flow_id, custom_name must be set
  constraint flow_target_exclusive check (
    (linked_process_id is not null)::int +
    (linked_model_id is not null)::int +
    (elementary_flow_id is not null)::int +
    (custom_name is not null)::int = 1
  )
);

create index flow_model_idx on flow(model_id, order_index);
create index flow_linked_process_idx on flow(linked_process_id);
create index flow_linked_model_idx on flow(linked_model_id);
create index flow_elementary_idx on flow(elementary_flow_id);
create index flow_license_tier_idx on flow(license_tier) where license_tier != 'open';

-- =================================================================
-- PARAMETERS
-- =================================================================

create table parameter (
  id uuid primary key default gen_random_uuid(),
  scope_type text not null
    check (scope_type in ('workspace', 'study', 'variant', 'model')),
  scope_id uuid not null,
  
  name text not null,
  value numeric,
  unit text,
  formula text,                                   -- if computed
  description text,
  
  distribution text,
  distribution_params jsonb,
  
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now(),
  
  unique (scope_type, scope_id, name)
);

create index parameter_scope_idx on parameter(scope_type, scope_id);

-- =================================================================
-- LIBRARIES (external data sources)
-- =================================================================

create table library (
  id uuid primary key default gen_random_uuid(),
  code text unique not null,                      -- 'agribalyse_3.1.1', 'ecoinvent_3.9.1_cutoff'
  name text not null,
  description text,
  version text not null,
  
  license_tier text not null,
  license_terms_url text,
  source_organization text,                        -- 'ADEME', 'ecoinvent Association'
  source_url text,
  
  process_count int not null default 0,
  elementary_flow_count int not null default 0,
  
  ingested_at timestamptz not null default now(),
  last_validated_at timestamptz,
  active boolean not null default true,
  
  unique (code, version)
);

-- =================================================================
-- PROCESSES (library entries)
-- =================================================================

create table process (
  id uuid primary key default gen_random_uuid(),
  library_id uuid not null references library(id) on delete cascade,
  external_id text not null,
  
  name text not null,
  reference_product text,
  reference_amount numeric,
  reference_unit text,
  
  -- Classification
  category text,                                  -- Classified category tree path
  isic_code text,                                 -- ISIC industry code
  
  -- Geography and temporal scope
  geography text,                                 -- ISO 3166 or custom code
  data_year int,
  
  -- Modeling parameters
  allocation_method text,
  system_model text,                              -- 'cutoff', 'apos', 'consequential'
  activity_type text,                             -- 'market', 'production', 'transformation', 'treatment'
  
  -- Full process data as structured JSON (our internal format)
  data jsonb not null,
  
  -- Search
  search_vector tsvector generated always as (
    setweight(to_tsvector('simple', coalesce(name, '')), 'A') ||
    setweight(to_tsvector('simple', coalesce(reference_product, '')), 'A') ||
    setweight(to_tsvector('simple', coalesce(geography, '')), 'B') ||
    setweight(to_tsvector('simple', coalesce(category, '')), 'C')
  ) stored,
  embedding vector(1024),                         -- semantic embedding
  
  -- Provenance
  source_citation text,
  
  unique (library_id, external_id)
);

create index process_library_idx on process(library_id);
create index process_search_idx on process using gin(search_vector);
create index process_embedding_idx on process using hnsw (embedding vector_cosine_ops);
create index process_geography_idx on process(geography);
create index process_name_trgm_idx on process using gin(name gin_trgm_ops);

-- =================================================================
-- ELEMENTARY FLOWS
-- =================================================================

create table elementary_flow (
  id uuid primary key default gen_random_uuid(),
  name text not null,
  cas_number text,
  formula text,
  synonyms text[],
  
  compartment text not null
    check (compartment in ('air', 'water', 'soil', 'natural_resource', 'social', 'economic', 'non_material')),
  subcompartment text,
  
  unit text not null,
  
  search_vector tsvector generated always as (
    to_tsvector('simple',
      coalesce(name, '') || ' ' ||
      coalesce(cas_number, '') || ' ' ||
      array_to_string(coalesce(synonyms, '{}'::text[]), ' ')
    )
  ) stored,
  
  unique (name, compartment, subcompartment)
);

create index elementary_flow_search_idx on elementary_flow using gin(search_vector);
create index elementary_flow_cas_idx on elementary_flow(cas_number) where cas_number is not null;

-- =================================================================
-- METHODS (LCIA)
-- =================================================================

create table method (
  id uuid primary key default gen_random_uuid(),
  code text not null,                             -- 'recipe_2016_endpoint_h'
  name text not null,
  description text,
  version text not null,
  
  perspective text,                                -- 'hierarchist', 'individualist', 'egalitarian', 'n/a'
  normalization_region text,                       -- 'world_2010', 'eu_27_2020', etc.
  
  author text,                                     -- 'RIVM', 'EU JRC', 'IPCC'
  source_url text,
  source_publication text,
  changelog_url text,
  
  license text not null default 'open',
  ingested_at timestamptz not null default now(),
  
  unique (code, version)
);

create table impact_category (
  id uuid primary key default gen_random_uuid(),
  method_id uuid not null references method(id) on delete cascade,
  code text not null,
  name text not null,
  plain_english_name text,
  icon text,
  unit text not null,
  damage_category text,                            -- 'human_health', 'ecosystems', 'resources'
  description text,
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
create index cf_category_idx on characterization_factor(impact_category_id);

-- =================================================================
-- PCRs (Product Category Rules)
-- =================================================================

create table pcr (
  id uuid primary key default gen_random_uuid(),
  code text not null,
  program_operator text not null,
  category text not null,
  title text not null,
  version text not null,
  effective_date date,
  expires_date date,
  
  required_impact_codes text[] not null,
  stage_framework text not null,
  data_quality_rules jsonb,
  
  document_url text,
  structured_rules jsonb,
  
  active boolean not null default true,
  created_at timestamptz not null default now(),
  
  unique (code, version)
);

create index pcr_category_idx on pcr(category) where active = true;
create index pcr_program_idx on pcr(program_operator) where active = true;

-- =================================================================
-- RESULTS (cached computations)
-- =================================================================

create table result (
  id uuid primary key default gen_random_uuid(),
  study_id uuid not null references study(id) on delete cascade,
  variant_id uuid references variant(id),
  
  method_id uuid not null references method(id),
  method_version text not null,
  
  calculation_type text not null
    check (calculation_type in ('deterministic', 'monte_carlo', 'sensitivity_local', 'sensitivity_global', 'uncertainty')),
  
  payload jsonb not null,                          -- full result structure
  
  -- Reproducibility
  manifest_hash text not null,                     -- sha256 of inputs
  manifest jsonb not null,                         -- full reproducibility manifest
  signature text,                                  -- Ed25519 signature of manifest_hash
  
  calc_engine_version text not null,
  computed_at timestamptz not null default now(),
  computation_duration_ms int,
  
  valid_until timestamptz
);

create index result_study_idx on result(study_id, computed_at desc);
create index result_manifest_idx on result(manifest_hash);

-- =================================================================
-- DELIVERABLES
-- =================================================================

create table deliverable (
  id uuid primary key default gen_random_uuid(),
  study_id uuid not null references study(id) on delete cascade,
  workspace_id uuid not null references workspace(id),
  
  kind text not null
    check (kind in ('epd_draft', 'technical_report', 'executive_summary', 'auditor_pack', 'interactive_link', 'matrix_export', 'excel_export', 'csv_export', 'json_export', 'ecospold_export', 'ilcd_export')),
  format text not null,
  program_operator text,
  
  status text not null default 'pending'
    check (status in ('pending', 'generating', 'ready', 'failed', 'expired', 'revoked')),
  
  file_path text,                                  -- MinIO object key
  file_size_bytes bigint,
  content_type text,
  
  -- For interactive share links
  share_token text unique,
  share_permissions text
    check (share_permissions is null or share_permissions in ('view', 'comment', 'edit')),
  share_password_hash text,                        -- optional
  expires_at timestamptz,
  access_count int not null default 0,
  last_accessed_at timestamptz,
  
  -- License enforcement outcome
  restricted_data_present boolean not null default false,
  restriction_details jsonb,
  compliance_treatment_applied text[],             -- which compliance treatments were applied
  
  created_at timestamptz not null default now(),
  created_by uuid,
  ready_at timestamptz,
  error_message text,
  
  -- Signature for auditability
  signature text
);

create index deliverable_study_idx on deliverable(study_id);
create index deliverable_share_token_idx on deliverable(share_token) where share_token is not null;
create index deliverable_workspace_status_idx on deliverable(workspace_id, status);

-- =================================================================
-- COMMENTS
-- =================================================================

create table comment_thread (
  id uuid primary key default gen_random_uuid(),
  workspace_id uuid not null references workspace(id) on delete cascade,
  target_type text not null,
  target_id uuid not null,
  resolved boolean not null default false,
  resolved_at timestamptz,
  resolved_by uuid,
  created_at timestamptz not null default now(),
  created_by uuid not null
);

create index comment_thread_target_idx on comment_thread(target_type, target_id) where resolved = false;

create table comment (
  id uuid primary key default gen_random_uuid(),
  thread_id uuid not null references comment_thread(id) on delete cascade,
  author_id uuid not null,
  body_markdown text not null,
  mentions uuid[],                                 -- mentioned user ids
  created_at timestamptz not null default now(),
  edited_at timestamptz,
  deleted_at timestamptz
);

create index comment_thread_created_idx on comment(thread_id, created_at);

-- =================================================================
-- VERSIONS (history)
-- =================================================================

create table study_version (
  id uuid primary key default gen_random_uuid(),
  study_id uuid not null references study(id) on delete cascade,
  version_number int not null,
  snapshot jsonb not null,                         -- full study state
  yjs_snapshot bytea,                              -- Y.Doc state
  
  commit_message text,
  created_at timestamptz not null default now(),
  created_by uuid,
  
  unique (study_id, version_number)
);

create index study_version_study_idx on study_version(study_id, version_number desc);

-- =================================================================
-- AUDIT LOG (partitioned by month)
-- =================================================================

create table audit_log (
  id bigserial not null,
  workspace_id uuid references workspace(id),
  user_id uuid,
  
  occurred_at timestamptz not null default now(),
  action text not null
    check (action in ('create', 'update', 'delete', 'read_sensitive', 'share', 'export', 'login', 'logout', 'permission_change', 'license_violation_attempt')),
  resource_type text not null,
  resource_id uuid,
  
  old_value jsonb,
  new_value jsonb,
  
  ip_address inet,
  user_agent text,
  request_id text,
  session_id text,
  
  alert_level text
    check (alert_level is null or alert_level in ('info', 'warning', 'violation', 'critical')),
  
  primary key (id, occurred_at)
) partition by range (occurred_at);

-- Create monthly partitions (automated via pg_partman)
create index audit_workspace_time_idx on audit_log(workspace_id, occurred_at desc);
create index audit_resource_idx on audit_log(resource_type, resource_id);
create index audit_alerts_idx on audit_log(alert_level, occurred_at desc) where alert_level is not null;
```

### 7.4 Migrations

**Tool: sqlx-migrate.** Reasons:
- Rust-native (aligns with arko-api's language)
- Compile-time query checking works seamlessly with migrations
- SQL-first migrations (no ORM abstractions getting in the way)
- Clear forward migrations (no automatic down migrations encouraged)

Migration conventions:
- Every migration is reviewed like code (PR required)
- Migrations are backward-compatible for one release cycle (add columns, then migrate reads, then remove)
- Never rename columns directly — add new, copy, deprecate old, remove later
- Test migrations on a production-size staging database before production deploy
- Migration timeouts are set explicitly to prevent long locks (statement_timeout, lock_timeout)

### 7.5 Row-level security implementation

Every tenant-scoped table has RLS enabled. The application sets `app.workspace_id` per request via a middleware that extracts the workspace from the JWT:

```rust
// Pseudocode in Rust
async fn workspace_scope_middleware(req: Request, next: Next) -> Response {
    let workspace_id = extract_workspace_from_jwt(&req)?;
    let user_role = extract_role(&req)?;
    
    // Acquire connection and set session variables
    let mut conn = db_pool.acquire().await?;
    sqlx::query!("set local app.workspace_id = $1", workspace_id.to_string())
        .execute(&mut *conn).await?;
    sqlx::query!("set local app.role = $1", user_role)
        .execute(&mut *conn).await?;
    
    // Store in request extensions for handler use
    req.extensions_mut().insert(conn);
    
    next.run(req).await
}
```

RLS is defense in depth. Application-layer checks come first; RLS catches anything that slips through.

### 7.6 Backups and recovery

- **Continuous WAL archiving** to MinIO (30-day retention)
- **Daily base backups** to MinIO (90-day retention)
- **Weekly full dumps** encrypted with GPG and uploaded to off-cluster storage (an OVH bucket in a different datacenter for cross-provider redundancy)
- **Monthly restore drills** — actually restore a backup to a staging database and run integration tests against it
- **Point-in-time recovery** capability tested quarterly — can we recover to any moment in the last 30 days?

### 7.7 Performance budgets for the database

- p95 for typical read queries: <20ms
- p95 for typical write transactions: <50ms
- Connection pool: 100 connections via PGBouncer in transaction pooling mode
- Max concurrent long-running queries: 10 (higher tier, separated pool)
- Replication lag alert threshold: 5 seconds

---

## 8. License Enforcement Architecture

This is the novel architectural contribution from our ecoinvent EULA analysis. It deserves its own section because getting it wrong costs CHF 100,000 per violation and reputation that doesn't recover.

### 8.1 The problem restated

Ecoinvent's EULA (v3, April 2022) restricts publication and disclosure:
- §7.1(e): cannot publish/disclose ecoinvent data through software tools to third parties
- §7.1(f): cannot use for consumer-facing comparisons, e-shops, or regulated declarations without separate licensing
- §7.1(b): cannot resell, rent, or act as intermediary
- §6.1: reports must be static, graphical, not reverse-engineerable
- §9: CHF 100,000 penalty per breach

We cannot rely on users to read the EULA. Our architecture must enforce it, visibly, auditably.

### 8.2 The policy language

License restrictions are expressed as a declarative policy language, not scattered if-statements:

```rust
// policies/ecoinvent.yaml
name: ecoinvent
version: "3.0"
restrictions:
  - cannot_public_share_interactive
  - cannot_expose_unit_values_via_public_api
  - cannot_embed_in_consumer_product
  - cannot_use_for_en15804_epd_without_separate_license
  - requires_static_pdf_export
  - requires_license_footer_attribution
  - requires_ecoinvent_logo_where_applicable
  - requires_audit_log_on_access
  
export_transformations:
  pdf:
    apply: static_only
    watermark: "Contains ecoinvent data — see licensing"
    footer_text: "This report contains ecoinvent v{{version}} data licensed to {{workspace_name}}."
  
  interactive_share:
    apply: blocked
    user_message: "Interactive sharing of ecoinvent data violates the ecoinvent license. Options: (a) Share as static PDF. (b) Share within workspace members only. (c) Replace ecoinvent data with open alternatives."
  
  api_public:
    apply: redact_unit_values
    allow_aggregated_totals: true
    
audit_requirements:
  log_on_every_query: true
  log_on_every_export: true
  monthly_compliance_report: true
```

This policy-as-code is:
- Reviewable by the actual ecoinvent legal team if they ask
- Auditable (we can prove what we enforce)
- Updatable when licenses change (new file version, deploy, done)
- Extensible (new license tiers add new policy files)

### 8.3 The enforcement layer

Lives in arko-api as middleware. Every response and every export passes through it.

```rust
// Pseudocode for the enforcement flow
pub async fn license_enforcement_middleware(
    req: Request,
    next: Next,
) -> Result<Response, LicenseError> {
    // 1. Determine what's being requested
    let action = classify_action(&req);
    
    // 2. Determine what data will be touched
    let data_refs = predict_data_references(&req).await?;
    
    // 3. Determine applicable licenses
    let licenses = resolve_licenses(&data_refs).await?;
    
    // 4. Union all restrictions
    let restrictions = licenses.iter()
        .flat_map(|l| l.restrictions.iter())
        .collect::<HashSet<_>>();
    
    // 5. Check action against restrictions
    if action_is_blocked(&action, &restrictions) {
        audit_violation_attempt(&req, &action, &restrictions).await;
        return Err(LicenseError::ActionBlocked {
            restriction: first_blocking_restriction(&action, &restrictions),
            user_message: user_message_for(restriction),
            alternatives: suggest_alternatives(&action),
        });
    }
    
    // 6. Apply required transformations (e.g., redact values)
    let response = next.run(req).await;
    let response = apply_transformations(response, &restrictions).await?;
    
    // 7. Audit
    audit_license_sensitive_access(&req, &action, &licenses).await;
    
    Ok(response)
}
```

### 8.4 UI integration

When the enforcement layer blocks an action, the UI gets a structured error that it renders helpfully:

```tsx
<LicenseBlockedBanner
  restriction="cannot_public_share_interactive"
  message="This study contains ecoinvent data. Public sharing would violate ecoinvent's license."
  alternatives={[
    { action: "export_static_pdf", label: "Generate a static PDF instead" },
    { action: "share_within_workspace", label: "Share with workspace members only" },
    { action: "replace_with_open_alternatives", label: "Replace ecoinvent data with open alternatives" }
  ]}
/>
```

The user is never left wondering why something failed. They're guided to compliant alternatives.

### 8.5 Compliance reporting

Every workspace can generate a compliance report:

```
Workspace: Goibix Test Factory
License compliance report: March 2026

Ecoinvent queries: 847
Ecoinvent data exports: 12 (all compliant, static PDFs)
Interactive share attempts: 3 (all blocked as expected)
License violations: 0

[Sign and export as PDF]
```

This report can be provided to ecoinvent on request, closing the audit loop.

### 8.6 The reseller path

When Arko negotiates ecoinvent reseller status (V2 target, Month 18–24), this architecture extends naturally:

1. `workspace_license` table records which tenants have valid ecoinvent access (provisioned by our billing)
2. Middleware checks this on every query involving ecoinvent data
3. Tenants without a license see ecoinvent processes as "locked" in search with a "Request access" CTA
4. Billing integrates ecoinvent license fees as a pass-through with a reseller margin

The code changes from V1 to reseller-enabled V2 are small. The policy framework is designed for it.

---

## 9. AI Orchestration Layer

### 9.1 Provider strategy

**V1: Anthropic API (Claude).**

Rationale:
- Measurably higher quality on analytical and interpretive tasks (LCA narrative, hotspot detection, anomaly analysis)
- Strong tool-calling support (required for our grounding architecture)
- EU regions are expanding (we use them as soon as available)
- Reasonable pricing at our scale
- Enterprise agreements available for GDPR-DPA compliance

**V2+: Self-hosted LLM as secondary option for Enterprise customers requiring data sovereignty.**

Candidate models: Llama 4 (Meta's flagship, when released), Qwen 3 (Alibaba, strong on European languages), Mistral Large 2 (European-origin, aligned preferences). Evaluated via our own eval harness against production use cases. Deployment: vLLM on a dedicated GPU cluster (4+ H100s or equivalent), served behind the same abstraction as Anthropic.

**The provider abstraction:**

```rust
// Pseudocode
#[async_trait]
pub trait LlmProvider: Send + Sync {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse>;
    async fn stream(&self, request: CompletionRequest) -> impl Stream<Item = Result<Chunk>>;
    async fn with_tools(&self, request: CompletionRequest, tools: Vec<Tool>) -> Result<CompletionResponse>;
    fn capabilities(&self) -> ProviderCapabilities;
}

pub struct AnthropicProvider { /* ... */ }
pub struct SelfHostedLlmProvider { /* ... */ }

impl LlmProvider for AnthropicProvider { /* ... */ }
impl LlmProvider for SelfHostedLlmProvider { /* ... */ }
```

The AI service accepts a provider via configuration. Enterprise customers who require self-hosted can run Arko with a `SelfHostedLlmProvider` pointed at their own infrastructure.

### 9.2 The grounding architecture

**Core rule: Claude never generates numbers.** 

Claude interprets, summarizes, recommends, narrates. Every numeric value in output must trace to a specific tool call response or known input.

**Tool-calling schema:**

```rust
pub enum AiTool {
    GetStudyResults { study_id: Uuid },
    GetContributionBreakdown { study_id: Uuid, impact_code: String },
    QuerySimilarProcesses { description: String, geography: Option<String>, top_k: u32 },
    ComputeWhatIf { study_id: Uuid, parameter_changes: HashMap<String, f64> },
    LookupPcr { product_category: String, program_operator: Option<String> },
    SearchLiterature { query: String, filter: LiteratureFilter },
    GetPedigreeDistribution { study_id: Uuid, model_id: Uuid },
    CompareWithIndustryBenchmark { study_id: Uuid, benchmark_source: String },
}
```

The AI service constructs prompts that:
1. Describe the user's question
2. Attach relevant study context (scoped to what the user has access to)
3. Expose the above tools
4. Instruct Claude to use tools before answering
5. Require citations for every number in the response

**Response verification:**

Every AI response passes through a verification step:
1. Parse the response for numeric tokens
2. Check each number against the tool call results and study data
3. Unverifiable numbers trigger a warning badge: "⚠ Some values in this response couldn't be verified"
4. Severe unverifiable claims (numbers not appearing anywhere in the grounding context) block the response and retry with more explicit grounding instructions

### 9.3 Prompt management

Prompts are versioned, tested, and reviewable:

```
prompts/
  v1/
    system/
      base.md                              # shared system message
      interpretation.md                    # narrative interpretation
      hotspot_detection.md
      gap_filling.md
      anomaly_detection.md
      nl_query.md
      epd_section_draft.md
    evaluation/
      golden_outputs/                      # reference completions for eval
      eval_harness.py                      # runs prompts against golden cases
  v2/
    ...
```

Every prompt change triggers:
1. Eval harness runs against golden cases
2. Diff in outputs flagged for review
3. Regression tests for hallucination (specific hallucination cases we've seen in the past)
4. Sign-off before deploy

### 9.4 Cost governance

AI is expensive. Controls:

**Per-user quotas by tier:**

| Tier | Queries/min | Queries/day | Opus allowed |
|---|---|---|---|
| Free | 5 | 20 | No (Sonnet only) |
| Studio | 20 | 200 | Limited (10/day) |
| Team | 60 | 1,000 | Yes |
| Enterprise | Negotiated | Negotiated | Yes |

**Per-workspace monthly budgets:** soft cap at 80% of budget (warning email), hard cap at 100% (AI features disabled until next cycle or overage payment).

**Caching:** identical prompt + identical study state = cached response for 1 hour. Hit rate expected ~40% in steady-state use.

**Model routing:** automatic selection of model tier by task complexity. Simple tasks (typeahead suggestions, classification) use Haiku. Standard tasks (interpretation, hotspot detection) use Sonnet. Complex tasks (comparison narrative, sensitivity interpretation) use Opus only when explicitly invoked by user.

### 9.5 AI feature roadmap

**V1 at launch:**
- Interpretation narrative on results page
- Hotspot callouts on results
- Semantic process search via embeddings
- Natural-language query ("what's driving my climate impact?")

**V2 (months 9–15):**
- AI-assisted gap filling for new processes
- Comparison narrative auto-generation
- Anomaly detection against industry benchmarks
- AI-drafted EPD prose sections (never numbers)
- Excel column mapping for paste operations
- Supplier data collection form generation

**V3 (year 2+):**
- Conversational LCA copilot (multi-turn guided study construction)
- Automated sensitivity analysis with interpretation
- On-premise LLM for Enterprise sovereignty
- Model fine-tuning on LCA interpretation tasks
- Multi-agent workflows (research agent, validation agent, drafting agent)

---

## 10. Document Generation Pipeline

### 10.1 Architecture

```
User clicks "Generate EPD" in UI
    │
    ▼
POST /v1/deliverables
    │
    ▼
arko-api:
  - Validates permissions
  - Creates deliverable row (status: pending)
  - Publishes NATS event: doc-gen.epd.requested
    │
    ▼
NATS JetStream (durable queue)
    │
    ▼
arko-docs worker (Node.js):
  - Fetches study state from arko-api
  - Fetches fresh results from arko-calc
  - Loads template for (program_operator, version)
  - Renders docx via docx-js
  - Renders HTML, converts to PDF via Puppeteer
  - Uploads files to MinIO with signed URLs
  - Publishes event: doc-gen.epd.ready
    │
    ▼
arko-api:
  - Updates deliverable row (status: ready)
  - Notifies user via WebSocket
    │
    ▼
UI: "Download EPD" button activates
```

### 10.2 Template architecture

Each program operator has its own template directory:

```
templates/
  epd/
    environdec/                    # International EPD System
      v4.0/
        index.tsx                  # main template
        sections/
          cover.tsx
          product_info.tsx
          system_diagram.tsx
          impact_results.tsx
          data_quality.tsx
          certifications.tsx
          references.tsx
        styles.css                 # PDF-specific styling
        assets/
          logo.png
        schema.json                # template requirements
    ibu/                           # German IBU
      v2.5/
        ...
    epd_norge/
      ...
    epd_italy/
      ...
    epd_denmark/
      ...
  technical_report/
    iso_14044_compliant/
      ...
  executive_summary/
    one_page/
      ...
```

Each template is a TSX component that renders docx-js elements (for Word) and HTML (for PDF). Puppeteer converts the HTML to PDF with print-specific CSS.

### 10.3 Rendering primitives

A shared library provides consistent primitives across templates:

```tsx
import { 
  Heading, Paragraph, Table, Chart, ImageFromMinIO, 
  ImpactResultsTable, ContributionNetworkDiagram,
  ComplianceFooter, LicenseWatermark,
  LocalizedText, Signature
} from '@arko/doc-primitives'

export function EpdDocument({ study, results, variant, workspace, license_context }) {
  return (
    <Document>
      {license_context.restrictions.includes('requires_license_footer_attribution') && 
        <LicenseWatermark text="Contains ecoinvent data" />}
      
      <CoverPage study={study} workspace={workspace} />
      <ProductInfo variant={variant} />
      <SystemDiagram variant={variant} />
      <ImpactResultsTable results={results} pcr={study.pcr} />
      <DataQualitySection pedigree={results.aggregatedPedigree} />
      <Certifications />
      <References study={study} />
      <ComplianceFooter license_context={license_context} />
    </Document>
  )
}
```

### 10.4 License-aware rendering

Before rendering, `license_enforcement` analyzes the study:

- If any flow has `license_tier = ecoinvent`:
  - Apply watermark
  - Use aggregated totals only in tables (no per-flow breakdowns)
  - Disable interactive drilldown in interactive-link mode
  - Require attribution footer
  - Log the export to audit

- If any flow has `license_tier = user_private`:
  - Restrict sharing to workspace members
  - Add private watermark if exported
  - Log who accessed the document

- If all flows are `license_tier = open`:
  - Full interactive features available
  - No watermarks required
  - Attribution to sources still required (good practice)

### 10.5 Output formats

Per Principle P57 of the Master Spec:

| Kind | Format | Typical size | Generation time |
|---|---|---|---|
| Executive summary | PDF | 1 page | 2-3s |
| Technical report | PDF | 20-80 pages | 6-12s |
| Auditor pack | Excel + JSON + signed manifest | 5-30 MB | 4-8s |
| EPD draft | Word + PDF | 15-25 pages | 8-15s |
| Interactive share | URL pointing to view-only web UI | N/A | <1s |
| Matrix export | Zipped NPZ/CSV/JSON | 100 KB - 50 MB | 2-10s |
| ECOSPOLD v2 | Zipped XML | 500 KB - 10 MB | 3-6s |
| ILCD | Zipped XML | 500 KB - 10 MB | 3-6s |

All formats reproducibly generated from the same source (study + results + manifest).

---

## 11. Security Architecture

### 11.1 Defense in depth: seven layers

Each layer assumes the others might fail:

1. **Edge (Cloudflare):** WAF rules (OWASP core ruleset, custom rules for our API patterns), DDoS mitigation, bot management, IP reputation filtering, rate limiting at edge.

2. **Ingress (Traefik):** TLS termination with mTLS to upstream pods, HTTP security headers enforcement, per-route authentication.

3. **Application (Axum middleware):** JWT verification, RBAC, input validation via `validator` crate, rate limiting per user and workspace.

4. **Data access (RLS policies):** workspace isolation at Postgres engine level, defense against application bugs.

5. **Audit (append-only log):** every sensitive action logged with user, workspace, timestamp, IP, old/new values.

6. **Encryption (TLS 1.3 + AES-256 at rest):** transit and rest encryption on all customer data.

7. **Operational (Vault + secrets rotation):** no secrets in code, secrets rotated on schedule and on staff departure.

### 11.2 Authentication

**Keycloak** as the identity provider, self-hosted on our Kubernetes cluster with its own Postgres instance.

**Keycloak realms:**
- `arko-users` — customer realm, public signup
- `arko-admin` — internal admin realm (for Goibix employees)

**Supported authentication methods:**
- Email + password (Argon2id hashing, per-user pepper)
- Magic link (passwordless, single-use 15-minute tokens)
- OAuth: Google, Microsoft, GitHub (for free/Studio tiers)
- SAML 2.0 SSO (Team+ tiers): Okta, Entra ID, Google Workspace, OneLogin
- OIDC (Team+ tiers)
- Passkeys (WebAuthn) — encouraged default

**MFA:**
- TOTP (Google Authenticator, 1Password, Authy, Aegis)
- WebAuthn / passkeys
- No SMS (SIM swap / phishing susceptibility)
- Required for admin roles
- Optional for editors/viewers, strongly encouraged

**Sessions:**
- Access tokens: JWT, 1-hour expiry, verified locally with Keycloak's public keys
- Refresh tokens: 30-day expiry, rotated on every use, revocable
- Tokens stored in httpOnly + Secure + SameSite=Strict cookies
- Never localStorage, never URL parameters

### 11.3 Authorization (RBAC)

Workspace-level roles:

| Role | Permissions |
|---|---|
| Owner | Everything, including workspace deletion, billing, license management, member management |
| Admin | Everything except workspace deletion, billing ownership transfer |
| Editor | Create/edit studies, run calculations, generate deliverables, invite viewers |
| Viewer | Read studies, comment, export own-permitted deliverables |
| Auditor | Read-only, plus full audit log access (Enterprise tier) |

Resource-level permissions:
- Studies can be individually shared with non-members via share tokens
- Libraries (workspace-owned) can be shared across workspaces or published
- Share permissions: view, comment, edit
- Share tokens revocable instantly

**Policy checks are explicit in every endpoint.** Pseudocode:

```rust
#[get("/v1/studies/{id}")]
async fn get_study(
    Path(id): Path<Uuid>,
    auth: AuthenticatedUser,
    ws: WorkspaceContext,
    state: State<AppState>,
) -> Result<Json<Study>, ApiError> {
    let study = state.db.get_study(id).await?;
    
    require_permission(&auth, &ws, &study, Permission::ViewStudy)?;
    
    Ok(Json(study))
}
```

RLS catches anything the explicit check misses.

### 11.4 Input validation

- **Pydantic-equivalent in Rust: `validator` + `serde`.** Every request body validated before business logic sees it.
- **Parameterized queries only** via SQLx. Compile-time query validation rejects dynamic SQL at build time.
- **Output encoding:** React escapes by default. Markdown rendered through `comrak` with sanitizer.
- **CSP headers:** strict policy. No inline scripts. Trusted CDN list.
- **File upload validation:** type sniffing (not MIME header trust), size limits, malware scanning via ClamAV in a sidecar pod.

### 11.5 Encryption

**In transit:**
- TLS 1.3 everywhere
- HSTS with preload
- Certificate pinning in mobile app (V2)
- mTLS between cluster services via Cilium
- No SSL, no TLS 1.0/1.1/1.2

**At rest:**
- LUKS full-disk encryption on all Hetzner servers (keys managed by HashiCorp Vault)
- Postgres at rest encryption via Postgres native + disk-level
- MinIO server-side encryption with per-object keys
- Redis and NATS: no persistence of sensitive data; encrypted in transit only

**Application-level (sensitive fields):**
- User PII in audit logs: encrypted with per-workspace keys (can be crypto-shredded for GDPR erasure)
- License proof documents: encrypted at rest
- Enterprise option: customer-managed keys via KMS integration (V2)

### 11.6 Secrets management

**HashiCorp Vault** self-hosted, with auto-unseal via Hetzner Cloud HSM.

Secret categories:
- **Infrastructure:** database passwords, Redis auth, TLS certificates
- **Signing keys:** JWT signing key, Ed25519 manifest-signing key, share-token salts
- **Service credentials:** Anthropic API key, Redsys credentials, Postmark key
- **Rotation schedule:**
  - JWT signing keys: quarterly
  - Database passwords: semi-annually
  - Service credentials: on staff departure or annually
  - Share-token salts: never rotated (would break existing share links)

All services fetch secrets from Vault on startup. No environment variables contain secrets in plaintext. No secrets in Kubernetes ConfigMaps. Vault tokens are pod-scoped via the Kubernetes auth method.

### 11.7 Compliance roadmap

| Compliance | V1 launch | Target |
|---|---|---|
| GDPR | Day 1 (reused from KarbonGarbi pattern) | ✓ |
| SOC 2 Type I | Not at launch | Month 15-18 |
| SOC 2 Type II | Not at launch | Month 24-27 |
| ISO 27001 | Not V1/V2 | Year 3 if Enterprise demand |
| NIS2 (if in scope) | Assessed | Assessed annually |

### 11.8 Security testing

- **Automated:** Dependabot (dependency updates), Trivy (container scanning), Semgrep (SAST), Bandit (Python), Cargo audit (Rust), npm audit.
- **DAST:** OWASP ZAP scans on staging environment, weekly.
- **Penetration testing:** external firm, annually once we have Enterprise customers; more frequent (quarterly) for SOC 2.
- **Bug bounty:** V2 feature, via Intigriti (EU-based platform).
- **Secrets scanning:** gitleaks on every commit, TruffleHog on repository history.

---

## 12. Observability, SLOs, and Incident Response

### 12.1 SLIs and SLOs

Service Level Indicators with Service Level Objectives and error budgets:

| Service | SLI | SLO | Error budget/mo |
|---|---|---|---|
| arko-web | Availability (success %, 4xx excluded) | 99.9% | 43 min |
| arko-api | Availability | 99.9% | 43 min |
| arko-api | p95 latency (read) | <200ms | — |
| arko-api | p95 latency (write) | <600ms | — |
| arko-calc | Availability | 99.5% | 3.6 hr |
| arko-calc | p95 calc duration (small) | <500ms | — |
| arko-calc | p95 calc duration (medium) | <3s | — |
| arko-docs | Availability | 99.5% | 3.6 hr |
| arko-docs | p95 EPD generation | <20s | — |
| arko-sync | Availability | 99.9% | 43 min |
| arko-sync | p95 message delivery | <150ms | — |
| Postgres | Availability | 99.95% | 22 min |
| Postgres | Replication lag | <5s p99 | — |

**Error budget policy:** when a service exceeds its error budget in a month, a formal freeze on feature deployment to that service takes effect until the budget resets. Only reliability fixes ship until then.

### 12.2 The observability stack

Self-hosted on a dedicated Kubernetes namespace, isolated from production workloads:

- **VictoriaMetrics** (Prometheus-compatible, faster at scale) for metrics
- **Loki** for log aggregation (structured JSON logs from all services via Promtail)
- **Tempo** for distributed tracing (OpenTelemetry)
- **Grafana** for dashboards and alerting
- **GlitchTip** for error tracking (Sentry-API-compatible, open-source)
- **Alertmanager** for alert routing

All components in a separate Kubernetes namespace, on a separate physical node from production workloads (blast radius isolation). Monitoring the monitors via a small external uptime checker (Uptime Kuma on a separate VPS).

### 12.3 Instrumentation standards

- Every service emits:
  - Prometheus metrics (RED: Rate, Errors, Duration)
  - Structured JSON logs with trace correlation IDs
  - OpenTelemetry spans for external calls
- Every metric has a defined SLI and alert
- Every log line has trace_id, span_id, user_id (if authenticated), workspace_id, request_id
- Critical business events emit both logs and metrics (user signup, study created, EPD generated, license violation attempted)

### 12.4 Dashboards

**Production dashboards** (Grafana), all team members have access:
- Cluster overview (Kubernetes, nodes, disk, network)
- Service health (per-service RED metrics, SLO burn rate)
- Database (CPU, memory, connections, replication lag, slow queries)
- Business metrics (signups, active studies, calculations, EPDs generated, AI queries, MRR)
- Security (audit log alert rate, failed logins, license violation attempts)

**Incident dashboards** for on-call:
- Current active incidents
- SLO burn rates
- Error budget status
- Recent deployments (correlate with incidents)

### 12.5 Alerting philosophy

**Three severity tiers:**

- **Page (immediate response):** availability SLO breach >5 min, data loss risk, active security incident, billing pipeline failure, error budget exhausted
- **Ticket (handle next business day):** latency SLO breach without outage, non-critical service degradation, cost anomaly, deployment failed
- **FYI (weekly review):** budget warnings, capacity approaching thresholds, certificates expiring soon, new CVEs in dependencies

**Alert hygiene:** alerts that fire >3 times without leading to a human decision get fixed or deleted. Alert fatigue is a security risk.

### 12.6 On-call rotation

**V1 (solo founder):** Samir is on-call 24/7 with a relaxed policy — paging only for genuine outages of paying services, not for every glitch.

**V2 (team of 3-5):** weekly rotation covering European hours (8am-11pm), with a secondary for wake-up incidents.

**V3 (team of 8+):** formal follow-the-sun rotation if we have international contributors, or paid on-call for nighttime coverage.

### 12.7 Incident response

**Documented runbooks** in `docs/runbooks/`:
- Database primary failure → Patroni failover verification → verify client reconnection
- Cluster node failure → Kubernetes eviction → verify workloads rescheduled
- DDoS attack → Cloudflare rate limiting tightening → traffic analysis
- Calc engine wrong result → isolate study → run via Brightway reference → file as P0 bug
- Data corruption → identify blast radius → point-in-time recovery → verify integrity

**Incident severity:**
- **SEV-1:** customer data loss or exposure, extended outage (>1 hr)
- **SEV-2:** service degradation affecting multiple customers
- **SEV-3:** single-customer impact
- **SEV-4:** potential issue, no customer impact yet

Every SEV-1/2 gets a public (internal) postmortem within 5 business days. Postmortems are blameless, focus on systems, and result in concrete action items tracked to completion.

---

## 13. Infrastructure as Code

### 13.1 The commitment

Every piece of production infrastructure is defined in code, version-controlled, reviewed, deployable by a new team member in an afternoon. Nothing is clicked in a UI and hoped to persist.

### 13.2 Tool selection

- **Terraform** for cloud resources (Hetzner Cloud, Cloudflare, DNS)
- **Talos Linux** for immutable OS configuration on bare-metal nodes
- **k0s** for Kubernetes cluster bootstrap (minimal, secure-by-default)
- **Helm** for Kubernetes application deployment (charts version-controlled in our repo)
- **Kustomize** for environment overlays (staging vs production)
- **Ansible** only for things that genuinely need imperative config (some OS-level tasks not covered by Talos)
- **Argo CD (V2)** for GitOps-style continuous deployment; V1 uses direct kubectl via CI

### 13.3 Repository structure

```
infrastructure/
  terraform/
    modules/                    # reusable modules
      hetzner-server/
      cloudflare-zone/
    environments/
      production/
      staging/
  talos/
    control-plane.yaml
    worker.yaml
    patches/
  k0s/
    cluster.yaml
  kubernetes/
    base/                        # base manifests
      arko-api/
      arko-calc/
      arko-sync/
      arko-docs/
      arko-ingest/
      arko-web/
    overlays/
      production/
      staging/
    platform/                    # platform services
      cloudnative-pg/
      keycloak/
      minio/
      redis/
      nats/
      meilisearch/
      vault/
      observability/
        victoriametrics/
        loki/
        tempo/
        grafana/
        glitchtip/
      ingress/
        traefik/
        cert-manager/
  ansible/
    playbooks/
      hardening.yml              # OS hardening
      backup-restore-drill.yml
```

### 13.4 GitOps workflow

All changes to production infrastructure flow through Git:

1. Developer edits Terraform or Kubernetes manifests in a feature branch
2. Opens a PR; CI runs `terraform plan` and `kubectl diff` to show impact
3. Reviewer approves (second person for production changes — enforced by branch protection)
4. Merge to main
5. CI applies changes to staging automatically
6. Automated smoke tests run
7. Manual approval for production apply
8. Apply completes; monitoring alerts on anomalies

### 13.5 Disaster recovery of infrastructure

The entire production cluster can be rebuilt from scratch in <4 hours:

1. `terraform apply` provisions new Hetzner servers and Cloudflare config (~20 min)
2. Talos Linux installs via PXE boot or Hetzner rescue (~10 min)
3. k0s cluster bootstraps (~10 min)
4. Helm charts deploy platform services (~30 min)
5. Latest Postgres backup restores from MinIO (~60 min)
6. Application services deploy (~20 min)
7. Smoke tests verify functionality (~30 min)
8. DNS cutover (~5 min, if different IPs)

Target RTO: 4 hours.
RPO: <5 minutes (streaming replication + WAL archiving).

**This is tested quarterly.** Not claimed — tested.

---

## 14. Self-Hosted Platform Services

Everything we self-host, with specific versions and configurations.

### 14.1 CloudNativePG (Postgres 16)

- **Cluster:** 1 primary + 2 hot standby replicas, sync replication to 1 replica
- **Connection pooling:** PGBouncer sidecars in transaction mode
- **Backup:** WAL to MinIO continuous, base backup nightly, retention 90 days
- **Monitoring:** pg_exporter → VictoriaMetrics
- **Upgrades:** rolling, tested on staging first, zero-downtime via failover
- **Extensions:** pgvector, pg_trgm, pgcrypto, pg_stat_statements, pg_partman

### 14.2 Keycloak

- **Deployment:** 2 replicas for HA, shared Postgres database (dedicated schema)
- **Realms:** `arko-users` (customer realm), `arko-admin` (internal)
- **Themes:** custom Arko branding, localized to all supported languages
- **Event logging:** to our observability stack
- **Upgrades:** quarterly minor versions, annual major versions
- **Realm config as code:** managed via Keycloak Operator + custom resources in Git

### 14.3 MinIO

- **Cluster:** 4-node erasure-coded (tolerates 2 node losses)
- **Buckets:**
  - `arko-user-uploads` — user-uploaded files (invoice images, product photos)
  - `arko-deliverables` — generated EPDs, reports, exports
  - `arko-wal` — Postgres WAL archiving
  - `arko-backups` — Postgres base backups
  - `arko-yjs-snapshots` — large Y.Doc snapshots
  - `arko-libraries-raw` — ingested library source files (Agribalyse XML, etc.)
- **Encryption:** server-side, per-object keys managed by Vault
- **Lifecycle rules:** delete expired share-link files, tier old backups to cold storage
- **Monitoring:** Prometheus metrics

### 14.4 Redis Cluster

- **Deployment:** 3-master + 3-replica Redis cluster
- **Use cases:** caching, session storage, rate limiting counters, pub/sub for arko-sync
- **Persistence:** AOF for recovery, RDB snapshots
- **Evaluated:** Dragonfly (modern Redis alternative). Chose Redis for ecosystem maturity.

### 14.5 NATS JetStream

- **Cluster:** 3-node
- **Streams:**
  - `doc-gen` — document generation events
  - `calc` — calculation requests
  - `audit` — audit log events (to offline storage)
  - `ai` — AI queries (for cost tracking)
  - `ingest` — library update events
- **Retention:** per stream, from 1 hour (realtime) to 7 days (audit)
- **Why NATS over Kafka or RabbitMQ:** lighter weight, simpler operations, modern Go codebase, JetStream gives durable queues with exactly-once semantics

### 14.6 Meilisearch

- **Cluster:** 2 nodes (HA via replication)
- **Indexes:** processes, elementary_flows, PCRs, documentation
- **Updates:** triggered by library ingest events via NATS
- **Evaluated:** Typesense (close second), OpenSearch (too heavy for our scale). Chose Meilisearch for excellent default relevance, low operational burden, typo tolerance.

### 14.7 HashiCorp Vault

- **Deployment:** 3-node Raft-based HA cluster
- **Auto-unseal:** Hetzner Cloud HSM (or transit auto-unseal via a secondary Vault in a different cluster)
- **Secrets engines:**
  - KV v2 for static secrets
  - Database for dynamic Postgres credentials
  - PKI for internal certificates
  - Transit for encryption-as-a-service (application-level crypto)
- **Access:** Kubernetes auth method for pods, OIDC for humans (via Keycloak)
- **Audit:** all access logged, shipped to Loki

### 14.8 Observability stack

- **VictoriaMetrics:** 3-node cluster, 90-day retention for metrics
- **Loki:** 2-node with S3 (MinIO) backend, 30-day hot retention, archived to cold storage indefinitely
- **Tempo:** 1-node (lower query volume), 7-day retention for traces
- **Grafana:** 1 instance with HA config (Postgres-backed, session storage in Redis)
- **GlitchTip:** 2-node, Postgres-backed error storage

### 14.9 Postal (transactional email)

- **Self-hosted email sending** for transactional messages (signup confirmations, password resets, notifications)
- **Fallback:** Postmark for critical deliverability (legal notices, billing emails) — configured as a secondary SMTP relay
- **DMARC, SPF, DKIM** properly configured for arko.earth domain
- **Bounce handling and suppression list** managed internally

### 14.10 Forgejo + Woodpecker CI

- **Forgejo (Gitea fork)** for source code hosting, issues, PRs, project management
- **Woodpecker CI** for CI/CD pipelines
- **Both self-hosted** on separate VPS (not in production cluster; dev tooling isolated)
- **Backup:** repositories mirrored to a secondary Forgejo instance

---

## 15. CI/CD and Release Management

### 15.1 Repository strategy

**Monorepo via Turborepo.** Rationale:
- Shared types (generated from OpenAPI) consumed by frontend and backend
- Unified dependency management
- Atomic cross-service changes (schema change + API change + UI change in one PR)
- Single place for developer tooling

### 15.2 The pipeline

**On every push:**
1. Lint (Biome for TS, Ruff for Python, Clippy for Rust)
2. Type check (tsc, mypy, cargo check)
3. Unit tests (Vitest, Pytest, cargo test)
4. Build changed services (Turborepo caches; only changed services rebuild)
5. Security scans (Trivy for containers, Semgrep for SAST, gitleaks for secrets)

**On PR to main:**
6. Integration tests (spin up test cluster, run cross-service tests)
7. E2E tests (Playwright against staging-equivalent ephemeral environment)
8. Visual regression (Playwright screenshot comparisons)
9. Performance budget check (Lighthouse CI on frontend)
10. Differential calc tests (our Rust engine vs Brightway, sample of 100 studies)
11. Full differential test (all 10,000 studies) — nightly, not per-PR
12. Reviewer approval required (minimum 1 for non-production, 2 for production changes)

**On merge to main:**
13. Auto-deploy to staging
14. Smoke tests on staging (~5 minutes)
15. Manual approval gate for production (via ChatOps bot in Slack)
16. Production deploy (canary: 10% of traffic for 30 minutes, automated rollback on error spike, then 100%)

### 15.3 Release cadence

- **Continuous deployment to staging** — every merge
- **Production deploys multiple times per day** (typical for mature phase)
- **Breaking changes** gated behind feature flags, rolled out gradually
- **Database migrations** deployed independently of application code (backward-compatible always)

### 15.4 Feature flags

**Self-hosted: Unleash.** Why Unleash over alternatives:
- Open source, MIT licensed
- Good UI for non-engineers to flip flags
- Per-workspace targeting (for beta rollouts to select customers)
- Gradual rollout (1% → 10% → 100%)
- Instant rollback without redeploy

Flag categories:
- **Release flags:** gate new features during rollout; removed after full launch
- **Experiment flags:** A/B tests
- **Ops flags:** circuit breakers (disable a feature if it's misbehaving)
- **Permission flags:** per-workspace access to beta features

### 15.5 Deployment artifacts

Every service produces:
- **Container image** (multi-arch: amd64, arm64; arm64 for dev Mac laptops)
- **SBOM** (Software Bill of Materials, SPDX format)
- **Signed image** (Cosign signatures via Sigstore)
- **Changelog entry** (auto-extracted from commit messages via Conventional Commits)

Container registry: self-hosted **Harbor** or **Zot**. Images pulled from our registry at deploy time; no external registry dependencies for production.

---

## 16. Testing Strategy

### 16.1 The testing pyramid, adapted for our reality

Standard pyramid (unit-heavy, fewer integration, fewest E2E) doesn't capture our needs. We have a **testing mountain**:

```
                          ╱╲  formal model checking (TLA+)
                         ╱  ╲    — sync state machine
                        ╱    ╲
                       ╱──────╲ chaos and property tests
                      ╱        ╲   — calc engine invariants
                     ╱──────────╲
                    ╱            ╲ differential tests
                   ╱              ╲  — 10k studies vs Brightway
                  ╱────────────────╲
                 ╱                  ╲ E2E tests (Playwright)
                ╱                    ╲  — critical user journeys
               ╱──────────────────────╲
              ╱                        ╲ integration tests
             ╱                          ╲  — API contract tests
            ╱────────────────────────────╲
           ╱                              ╲ unit tests
          ╱                                ╲  — every service
         ╱──────────────────────────────────╲
```

### 16.2 Unit tests

- **Rust services:** cargo test. Target 70%+ coverage on business logic (lower on UI-coupled glue).
- **Python services:** Pytest. Similar target.
- **Node services:** Vitest. Similar.
- **Frontend:** Vitest + React Testing Library. Focus on business logic, not styling.

### 16.3 Property-based tests

- **Rust: proptest.** Invariants on calc engine (non-negativity, linearity, permutation invariance), CRDT consistency properties.
- **Python: Hypothesis.** Invariants on data ingestion, transformations.
- **TypeScript: fast-check.** Invariants on formula evaluation, data transformations.

### 16.4 Differential tests (the calc engine's core correctness regime)

10,000+ reference studies with known correct results, drawn from:
- Brightway test suite (contributed back)
- OpenLCA reference cases
- Published LCA studies with full inventory
- Synthetic edge cases (circular refs, near-singular matrices, extreme scale)

**Per-PR:** 100 studies run in CI (fast smoke).
**Nightly:** full 10,000 studies run.
**On calc engine release:** full run, plus specific regression suites.

**Tolerance:** 1e-9 relative error against reference. Divergence blocks release.

### 16.5 Integration tests

Against a real test cluster (ephemeral, created per test run):
- API contract tests (OpenAPI schema compliance)
- Cross-service tests (arko-api calling arko-calc, arko-sync with arko-api)
- Database integration (actual Postgres, not mocks)
- Queue integration (actual NATS)

Run on every PR to main.

### 16.6 E2E tests

**Playwright** for critical user journeys:
- Signup → create workspace → first study → first calculation → EPD generation
- Team collaboration: invite colleague, concurrent edit, conflict resolution
- Import existing study from SimaPro CSV
- Generate and verify a compliant EPD
- License enforcement blocks an expected action
- Offline: go offline, edit, come online, verify sync

Run on every PR, full suite nightly. Parallelized across 10 workers.

### 16.7 Chaos and fault injection

- **Chaos Mesh** in staging environment
- Monthly exercises: kill a random pod, inject network latency, partition services, fill a disk
- Must recover within SLOs

### 16.8 Load testing

**k6** for HTTP load tests. Scenarios:
- 1,000 concurrent users browsing (baseline)
- 100 concurrent calc requests (peak)
- 1,000 Monte Carlo runs queued simultaneously (stress)
- EPD generation burst (50 in 1 minute)

Run weekly against staging. Results captured as time series in VictoriaMetrics for trend analysis.

### 16.9 Security testing

- **SAST:** Semgrep with custom rules for our patterns
- **DAST:** OWASP ZAP weekly against staging
- **Dependency scanning:** Dependabot, cargo-audit, npm audit, pip-audit
- **Container scanning:** Trivy in CI, on registry pull
- **Secrets scanning:** gitleaks pre-commit + repo history scan
- **Annual pen test** by external firm
- **Bug bounty** post-SOC 2 Type II

### 16.10 Formal verification (what we commit to)

**TLA+ specification and model checking** for:
- arko-sync state machine (CRDT convergence, access control, durability)
- License enforcement policy (consistency of restrictions across operations)
- Audit log correctness (append-only, no loss)

**Not committing to Coq/Lean proofs** in V1 or V2. Possibly V3+ for the calc engine core if it becomes a marketing differentiator.

---

## 17. Disaster Recovery and Business Continuity

### 17.1 Recovery objectives

- **RTO (Recovery Time Objective):** 4 hours for full infrastructure reconstruction
- **RPO (Recovery Point Objective):** 5 minutes for customer data (WAL streaming)
- **Mean time to detect:** <5 minutes for critical incidents
- **Mean time to respond:** <15 minutes during business hours, <60 minutes outside

### 17.2 Backup strategy

**Tiered backups with cross-provider redundancy:**

- **Hot (in-cluster):** Postgres streaming replication, 2 sync replicas
- **Warm (same provider, different datacenter):** Continuous WAL + daily base to Hetzner's secondary DC
- **Cold (different provider):** Weekly full encrypted dump to OVH bucket (different legal entity, different DC, different country)
- **Offline (quarterly):** Encrypted archive to a physical encrypted disk, stored off-site

### 17.3 Tested recovery scenarios

Quarterly DR exercises with documented playbooks:

**Scenario A: Primary database crash**
- Patroni failover to standby (automated, <30s)
- Verify client reconnection
- Verify no data loss (compare WAL positions)

**Scenario B: Entire cluster loss**
- Terraform apply creates new cluster
- Restore from latest base backup + WAL replay
- Verify data integrity
- DNS cutover

**Scenario C: Regional outage (Hetzner Finland down)**
- Cluster standup in Hetzner Germany
- Restore from cross-provider backup
- Customers notified of extended outage, expected duration

**Scenario D: Ransomware / cluster compromise**
- Pre-compromise clean backup from offline storage
- New infrastructure provisioning
- Customer data notification procedures per GDPR
- Legal/PR response coordination

### 17.4 Business continuity for the founder

This is often ignored but matters: what happens to Arko if Samir is hit by a bus?

**Bus factor mitigations:**
- All critical knowledge in docs/runbooks (not in Samir's head)
- Goibix S.L. is a proper legal entity with succession planning
- Second key-holder for critical systems (identified, in sealed envelope)
- Cold-chain escrow of infrastructure access (lawyer holds keys for agreed scenarios)
- Customer data migration plan if the business is wound down

Not pleasant to think about, but non-negotiable for customers who depend on us.

---

## 18. Development Workflow and Tooling

### 18.1 Local development

**`scripts/dev.sh`** starts everything:

```bash
#!/bin/bash
# Start local infrastructure
docker compose -f docker-compose.dev.yml up -d
# Postgres, Redis, NATS, Meilisearch, MinIO local

# Apply migrations
pnpm db:migrate

# Seed test data
pnpm db:seed

# Start all services in parallel
pnpm turbo dev
```

After ~30 seconds:
- arko-web: http://localhost:3000
- arko-api: http://localhost:8000 (OpenAPI at /docs)
- arko-calc: http://localhost:8001
- arko-sync: ws://localhost:8002
- arko-docs: http://localhost:8003
- Keycloak local: http://localhost:8080
- Grafana: http://localhost:3030
- Mailhog (email testing): http://localhost:8025

### 18.2 Code quality tools

**Linters (all set to reject, not warn):**
- **Biome** (TypeScript/JavaScript) — replaces ESLint + Prettier, faster, unified config
- **Ruff** (Python) — replaces Black + isort + flake8 + pylint
- **Clippy** (Rust) — stricter than default
- **cargo fmt** (Rust formatting)
- **shellcheck** (shell scripts)
- **yamllint** (YAML files)

**Pre-commit hooks:**
- Format check
- Lint
- Type check
- Secrets scan
- Test changed files

**Commit convention:** Conventional Commits (enforced by commitlint). Enables automated changelog generation.

### 18.3 IDE and developer experience

- **Cursor** or **VSCode** with extensions: rust-analyzer, biome, pylance, tailwind, prisma
- **Shared devcontainer config** for consistent environments
- **`.env.example`** files for every service; never commit `.env` files
- **Local k3d cluster** available for testing Kubernetes changes before staging

### 18.4 Documentation

Three tiers:

- **Product docs** (`/docs` in the app) — user-facing help, tutorials, concept guides. Maintained alongside product code.
- **Architecture docs** (`docs/architecture/` in repo) — ADRs, system diagrams, this spec, API documentation. Updated on significant changes.
- **Operational runbooks** (`docs/runbooks/`) — "handle incident X", "rotate secret Y", "onboard new customer Z". Referenced during on-call.

**API documentation** auto-generated from OpenAPI schema, published at `docs.arko.earth` with interactive examples, SDK code snippets, changelog.

**ADR (Architecture Decision Record) template:**
```
# ADR-XXX: [Title]

## Status
[Proposed, Accepted, Deprecated, Superseded by ADR-YYY]

## Context
What is the issue we're addressing?

## Decision
What we decided to do.

## Consequences
What becomes easier or harder because of this decision?

## Alternatives considered
What else we thought about and why we rejected them.

## References
Links to relevant material.
```

### 18.5 Coding standards

**Rust:**
- Rustfmt default config
- Clippy warnings-as-errors
- No `unwrap()` or `expect()` outside tests and obvious-infallible contexts
- Every public function has doc comments
- Errors use `thiserror` for internal, convert to API errors at boundary

**TypeScript:**
- Strict mode, no `any` (outside generated types)
- No default exports (named exports for better refactoring)
- Components are functional, not classes
- Props typed explicitly, not inferred

**Python:**
- Type annotations required (mypy strict)
- Pydantic for all data models crossing boundaries
- No circular imports
- No mutable default arguments

**SQL:**
- Lowercase SQL (readability)
- Migrations reviewed for lock impact
- `EXPLAIN ANALYZE` included in PR for non-trivial queries
- Index additions justified with query patterns

---

## 19. Vendor Map

### 19.1 What we own (the code and the IP)

- Arko calc engine (Rust, Apache 2.0 open source)
- Arko sync engine (Rust, published spec)
- Arko license enforcement layer
- Arko data model and schemas
- Arko PCR library (community-maintained, CC-BY-SA)
- Arko method library (curated, open methods)
- All application code (commercial license, proprietary)
- All brand assets, templates, prompts

### 19.2 What we self-host (OSS we operate)

| Service | Version | Purpose |
|---|---|---|
| Postgres | 16 | Primary database |
| CloudNativePG | Latest stable | Postgres operator |
| Keycloak | 25.x | Authentication |
| MinIO | Latest | Object storage |
| Redis | 7.x | Cache, queues, rate limit |
| NATS JetStream | 2.10.x | Message queue |
| Meilisearch | 1.x | Full-text search |
| HashiCorp Vault | 1.15+ | Secrets |
| VictoriaMetrics | Latest | Metrics |
| Loki | Latest | Logs |
| Tempo | Latest | Traces |
| Grafana | 10+ | Observability UI |
| GlitchTip | Latest | Error tracking |
| Postal | Latest | Email sending |
| Forgejo | Latest | Code hosting |
| Woodpecker CI | Latest | CI/CD |
| Talos Linux | Latest stable | OS |
| k0s | Latest stable | Kubernetes |
| Traefik | 3.x | Ingress |
| Cilium | Latest | CNI + service mesh |
| cert-manager | Latest | TLS automation |
| Unleash | Latest | Feature flags |
| Argo CD (V2) | Latest | GitOps |

### 19.3 What we lease (external services with abstraction layers)

| Vendor | Purpose | Abstraction | Replacement path |
|---|---|---|---|
| Hetzner | Bare-metal, VPS, backup storage | Terraform | OVH, self-colo in 1 week |
| Cloudflare | Edge, CDN, WAF, DNS | Config | Bunny.net + HAProxy in 1 week |
| Anthropic | Claude API | `LlmProvider` trait | Self-hosted Llama in 1 day (quality loss) |
| Redsys | Billing (EU SEPA) | `BillingProvider` trait | Stripe in 1 week (shared with KarbonGarbi) |
| Postmark (fallback) | Email deliverability | SMTP | SES, self-hosted Postal primary |

### 19.4 What we explicitly reject

- Vercel (self-host our Next.js)
- Supabase (self-host Postgres and Keycloak)
- Datadog / New Relic (self-host Grafana stack)
- GitHub (self-host Forgejo)
- Stripe-only (Redsys for EU sovereignty)
- Any SaaS that stores customer data without our encryption keys
- Any US-region service for customer data

### 19.5 License compatibility scan

- All direct dependencies: MIT, Apache 2.0, BSD, ISC, PostgreSQL, MPL 2.0
- No AGPL in runtime (GPL/AGPL fine for dev tools)
- License compliance in CI via `cargo-license`, `license-checker` (npm), `pip-licenses`
- SBOM generated per release (SPDX format)

---

## 20. Cost Modeling Across Scale Tiers

Indicative costs for budgeting. Validate against actual bills; adjust quarterly.

### 20.1 Year 1 (0–200 paying customers, ~1,000 total users)

| Line item | Monthly cost (€) |
|---|---|
| Hetzner — 3× EX101 | 180 |
| Hetzner — 2× EX130 (Postgres) | 180 |
| Hetzner — 1× EX44 (MinIO) | 45 |
| Hetzner — 2× CX VPS | 25 |
| Hetzner bandwidth | 20 |
| Cloudflare Pro | 20 |
| Cloudflare R2 (minimal, we primarily use MinIO) | 5 |
| Anthropic API | 200–800 (usage-dependent) |
| Postmark fallback | 15 |
| Domain + SSL + misc | 15 |
| OVH cold backup bucket | 10 |
| **Total infrastructure** | **€720–1,320/mo** |
| Rust consulting (months 0–2 only) | — |
| SOC 2 prep (months 12+) | — |

At 150 paying customers avg €200/mo MRR = €30,000 MRR. Infra at 2.5-4% of revenue. Healthy.

### 20.2 Year 2 (500–2,000 customers)

| Line item | Monthly cost (€) |
|---|---|
| Hetzner expansion: +3 EX101, +1 Postgres replica pair | 350 |
| Additional Hetzner servers | 200 |
| Cloudflare Business | 200 |
| R2/MinIO storage scale | 30 |
| Anthropic API | 2,000–5,000 |
| Postmark, misc SaaS | 50 |
| SOC 2 Type I audit | 8,000 amortized (one-time) |
| Pen testing (annual) | 1,000 amortized |
| **Total infrastructure** | **€3,800–7,000/mo ongoing** |

At 1,000 customers avg €300/mo MRR = €300,000 MRR. Infra at 1-3% of revenue. Excellent.

### 20.3 Year 3+ (5,000+ customers)

Scales roughly linearly. Major additions:
- Dedicated Enterprise support (1-2 engineers dedicated): €15-25k/mo people cost
- Compliance audits (SOC 2 Type II): €20-40k/year
- Ecoinvent reseller license fees: pass-through + margin
- Potential GPU cluster for self-hosted Llama (Enterprise): €14-30k/mo if activated
- Multi-region deployment: effectively doubles infra cost but improves SLA

**Target:** infrastructure stays below 5% of revenue.

---

## 21. Scaling Plan and Capacity Thresholds

What changes at what scale. Concrete triggers, not vague plans.

| Scale | Trigger | Architectural change |
|---|---|---|
| 100 customers | Baseline | Current topology; no changes |
| 500 customers | p95 API latency breach | Add API replica pods, tune Postgres connection pool |
| 1,000 customers | Postgres write contention | Read replicas for heavy read queries, query rerouting |
| 2,000 customers | Calc queue backlog | Dedicated calc-engine nodes with more CPU |
| 5,000 customers | Single-region risk | Second-region standby (warm), DNS-based failover |
| 10,000 customers | Observability storage | Tiered storage for metrics (hot 7d, warm 90d, cold archive) |
| 25,000 customers | Monolith arko-api strain | Split into domain services: studies, library, billing |
| 50,000 customers | Cross-region demand | Active-active multi-region with regional data residency |
| 100,000 customers | WebSocket saturation | Sharded sync cluster by study ID hash |
| 500,000+ customers | Database partitioning | Partition Postgres by workspace_id (Citus or native) |

### 21.1 The things we'd replace at scale

- **Brightway → our Rust engine** for large-scale Monte Carlo (the calc-engine path already goes through our engine primarily; Brightway stays as reference)
- **Single-node Meilisearch → sharded** if search QPS > 1000/sec
- **Keycloak → custom auth service** only if Keycloak becomes a bottleneck (unlikely at our anticipated scale)
- **y-websocket protocol → custom protocol** only if Yjs becomes limiting (not expected)

### 21.2 Things we would NOT change at scale

- Postgres as primary DB (we'd scale up, not switch engines)
- Rust as the language (won't outgrow it)
- Our overall architecture (it's designed for 100× current scale)

---

## 22. Migration and Technical Debt Strategy

### 22.1 Accepted technical debt (deliberately taken, with payoff plan)

| Debt | Why we accept | When to pay off |
|---|---|---|
| Shared calc-engine core with generic features | Faster initial delivery | Refactor when we need specialized engines per use case (likely never) |
| No fine-tuned LLM for LCA tasks | Claude API is excellent | When cost > €5k/mo consistently (Year 2+) |
| Manual customer onboarding for Enterprise | Small volume | When signups > 5/mo Enterprise |
| No mobile app | Web PWA suffices | If field data collection demand is validated (Year 2) |
| Single-tenant template storage | PCRs rarely change | Never; might cache in CDN |

### 22.2 Migration paths preserved

- **From V1 → V2 calc engine improvements:** API-stable, underlying changes transparent
- **From Claude → self-hosted Llama:** provider abstraction, configuration swap
- **From single-region → multi-region:** data-model fields prepared (region_id on workspace), enablement via config
- **From Arko Cloud → on-premise Enterprise deployment:** all services containerized, K8s manifests portable
- **Customer migration IN:** importers for SimaPro CSV, OpenLCA, ECOSPOLD, ILCD day-1
- **Customer migration OUT:** exporters to same formats; users can always leave with data

### 22.3 Deprecation policy

- **APIs:** 18-month deprecation window. `/v1` stays alive 18 months after `/v2` GA.
- **Data libraries:** older versions remain available indefinitely for studies pinned to them.
- **Calc engine versions:** older versions deployable on demand for reproducibility audits (we keep container images indefinitely).
- **Breaking UX changes:** feature-flagged, rolled out gradually, with migration UI for affected workflows.

---

## 23. Risk Register

| # | Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|---|
| R1 | Self-hosted infrastructure ops burden exceeds solo founder bandwidth | **High** | **High** | Runbooks for all common ops tasks, automation for routine maintenance, budget for DevOps consultant months 3-9, prepare first hire to be DevOps-capable |
| R2 | Calc engine produces subtly wrong results | Low | Critical | 10k-study differential suite, property-based tests, published spec, bug bounty V2 |
| R3 | Yjs CRDT edge cases corrupt studies | Low | High | Postgres snapshots every 30s, TLA+ model checking, incident drills |
| R4 | Self-hosted Postgres catastrophic failure | Low | Critical | 3-node replication, WAL archiving, quarterly restore drills, cross-provider backup |
| R5 | Keycloak CVE requires emergency patch | Medium | Medium | Subscribed to Keycloak security announcements, monthly patch window, automated CVE alerts |
| R6 | Anthropic API policy change / price hike | Medium | Medium | Provider abstraction, regular testing with Llama alternatives, cost governance |
| R7 | Hetzner outage | Low | High | Cross-provider backup, documented failover to OVH in 24h |
| R8 | Cloudflare outage | Low | High | HAProxy origin bypass, runbook tested quarterly |
| R9 | Solo founder burnout | **High** | **High** | Strict boundaries (no Arko development while KarbonGarbi at <3 customers), advisor relationships, mental health priority |
| R10 | Rust learning curve slows delivery | Medium | Medium | 2 months paired with senior Rust consultant upfront, documentation-first culture, no production code until foundational modules reviewed |
| R11 | Ecoinvent denies reseller status | Medium | Medium | V1 doesn't need it; V2 enhancement; open alternatives remain viable |
| R12 | Customer requires specific tool certification we lack | Medium | Medium | Design for audit from day one; pursue certifications when customer demand justifies |
| R13 | Competitor ships first in the disruption window | Medium | High | Focus and ship; don't over-scope V1; the One Click LCA window extends to ~mid-2027 |
| R14 | GDPR audit finding | Low | High | KarbonGarbi's Phase I legal foundation reused; annual DPIA; EU-only everything |
| R15 | Bus factor (Samir unavailable) | Low | Catastrophic | Legal succession for Goibix S.L., infrastructure access escrow, runbook completeness, identified second key-holder |
| R16 | Open-source dependency vulnerability (Log4Shell-class) | Medium | High | Dependabot, automated CVE monitoring, documented emergency patching process |
| R17 | TLA+ formal model diverges from implementation | Medium | Medium | TLA+ spec updated with every sync protocol change; model checking in CI |
| R18 | Customer data breach | Very Low | Catastrophic | Defense-in-depth security architecture, annual pen tests, incident response plan, cyber insurance |

---

## 24. Open Questions

Questions this spec deliberately doesn't answer, which need resolution before or during implementation.

1. **The exact WASM engine feature coverage.** TBD by a 2-week spike at Month 2. Expected scope: deterministic calc, pedigree uncertainty, simple Monte Carlo, sensitivity. Large studies server-side.

2. **Library versioning policy for open data bundles.** Agribalyse releases annually; USDA less often. How often do we force users to upgrade vs. support old versions indefinitely? Propose: indefinite support with clear "this version is N years old" warnings; force upgrade only if a critical correction is issued.

3. **CRDT snapshot strategy for very large studies.** At what size do we split a study into sub-documents? Preliminary: 10,000 models or 50 MB Y.Doc size.

4. **Native mobile app scope.** When we build it (V2), React Native + Expo or pure-web PWA? Decision at Month 12 based on actual customer asks for field data collection.

5. **Fine-tuned LLM for LCA tasks.** When is it worth fine-tuning? Likely Year 2 when we have 1,000+ user-validated interpretations as training data.

6. **Formal verification of calc core (Coq/Lean).** Interesting as V3+ marketing differentiator. Decision at Year 2 based on customer feedback on trust/rigor.

7. **On-premise installation option.** When do we offer it? Proposed: earliest at €100k ARR from any single Enterprise customer willing to pay premium for dedicated ops.

8. **GPU-accelerated Monte Carlo.** When does Monte Carlo speed become limiting? Likely Year 3 with many Enterprise customers running 10,000-iteration studies. Cost/benefit analysis then.

9. **Multi-region deployment for data residency.** Which second region first? Candidates: Germany (AWS/Hetzner FFM), France (Scaleway Paris), Switzerland (Exoscale Zurich). Tied to first Enterprise customer request.

10. **Decentralized / P2P calculation for extreme-privacy use cases.** Relevant for defense or pharma. Not a V1-V2 concern. Research topic for V3+.

11. **Hardware security module (HSM) integration for Enterprise-managed keys.** Tied to first customer request; expected year 2-3.

12. **Formal specification language for PCRs.** Currently PCRs are YAML. Should they be a DSL with formal semantics (like Rego for OPA)? Attractive but defer.

---

## Closing

This specification is the work of someone who looked hard at what it would take to build the best possible life cycle assessment platform — and then held themselves accountable to actually shipping it. Every choice here is defensible under review. Every compromise is named. Every aspiration has a deadline.

It is uncompromising on what matters: the calc engine is ours, formally specified, open-sourced, differentially tested at a scale no LCA tool has publicly demonstrated. The sync engine is formally modeled in TLA+. The license enforcement is a novel architectural contribution. Every customer's data lives on hardware we control. No vendor can hold Arko hostage.

It is pragmatic about what doesn't matter: we don't reinvent Postgres. We use Claude until we have budget for our own GPUs. We use Cloudflare at the edge because it's excellent and replaceable. We use OSS instead of writing our own auth server, search engine, or secrets manager — each of those is a full-time role at a mature company, and we have better places to spend our limited attention.

It is honest about cost: ~12-14 months of focused engineering to ship V1 with this standard of quality. ~10-15 hours/week of ops in steady state. ~€800-1,500/month infrastructure in Year 1. A Rust consultant budget in the first months. These are the real numbers.

It is honest about risk: the calc engine is genuinely hard. The Rust learning curve is real. The ops burden of self-hosted infrastructure requires discipline. The One Click LCA disruption window is finite. We are not pretending any of this away.

And it is honest about what matters most: execution. Everyone in the LCA software industry knows what the ideal product looks like. The moat is shipping. This spec exists to make shipping possible — by reducing the number of decisions a builder has to make in the moment, by encoding the discipline that protects quality under deadline pressure, and by being honest about trade-offs that will need to be made.

The specification is a starting line, not a finish line. When a future contributor disagrees with a choice, the process is: write an ADR, prototype the alternative, measure, update the spec. Nothing here is sacred except the principles.

Go make it real.

---

*End of Arko Technical Architecture Specification v2.0 — Excellence Edition*

*Supersedes Arko Technical Architecture Specification v1.0*

*Companion to Arko Master Specification v1.0*

*Goibix S.L., Bilbao — April 2026*
