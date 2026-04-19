# Arko

> **Open, AI-native, web-native Life Cycle Assessment.**
> A SimaPro alternative built on published specifications, verifiable math,
> and sovereign infrastructure.

Arko is the LCA platform the industry should have had a decade ago:
spec-first, auditable end-to-end, real-time collaborative, and honest about
data provenance. No 1999-era desktop software. No vendor lock-in. No opaque
calculation engine you have to take on faith.

---

## Status

**Pre-alpha.** Specifications first, code second. Do not expect anything to
work yet. Do not expect anything to be stable before the calc engine v1.0
specification is ratified and the differential test suite passes against
Brightway and OpenLCA on ≥10,000 published studies.

If you are looking for a production LCA tool today, use OpenLCA or SimaPro.
Come back when v1.0 ships.

---

## The 12 commitments

Arko is built under 12 non-negotiable architectural commitments. These are
the contract we make with ourselves, and in time, with users.

| #   | Commitment                        | What it means in practice                                                   |
| --- | --------------------------------- | --------------------------------------------------------------------------- |
| C1  | Sovereignty                       | Self-hosted on bare metal. No dependency on US cloud providers.             |
| C2  | Boring infra, brilliant IP        | Off-the-shelf databases. Custom calc engine. Differentiation lives in math. |
| C3  | Specification-first               | Every subsystem has a public spec before it has code.                       |
| C4  | Differential + property testing   | Every result is cross-checked against ≥2 independent implementations.       |
| C5  | Multi-tenant by default           | No "we'll add tenancy later." It is in the schema from commit zero.         |
| C6  | API-first                         | Every UI action is a documented API call. The UI has no private endpoints. |
| C7  | Provenance on every mutation      | Every change is signed, timestamped, attributed, and reversible.            |
| C8  | Deterministic builds              | Same inputs → same bytes. Reproducible from source at any commit.           |
| C9  | OSS generic, closed commercial    | Primitives open. Product closed. See `LICENSE`.                             |
| C10 | No vendor hostage                 | Open export formats as a permanent commitment. Users leave with their data. |
| C11 | Progressive ops decentralization  | Start monolith-on-one-node. End as federated sovereign deployments.         |
| C12 | Monitored from day one            | Metrics, logs, traces before feature #1. You cannot fix what you cannot see. |

---

## Repository layout

```
arko/
├── specs/              # Public, versioned specifications (CC-BY-4.0)
│   ├── calc/           # Calculation engine spec — the core math contract
│   ├── sync/           # Real-time collaboration (Yjs + TLA+ state machine)
│   ├── license/        # License-tier propagation and enforcement rules
│   └── api/            # Public HTTP/JSON API
│
├── engine/             # Rust calc engine workspace (Apache-2.0) — see engine/README.md
│   ├── core/           # A·s=f, g=B·s, h=C·g pipeline + determinism harness
│   ├── solvers-dense/  # Dense LU/QR paths for small systems (faer)
│   ├── solvers-sparse/ # Sparse LU for mid-size systems (faer); UMFPACK path planned
│   ├── parameters/     # Expression language, DAG evaluation, forward-mode AD
│   ├── uncertainty/    # MT Monte Carlo and distributions (Sobol' planned v0.2)
│   ├── sensitivity/    # One-at-a-time + finite-difference sensitivity
│   ├── methods/        # Impact-method presets (IPCC AR6 GWP100, etc.)
│   ├── units/          # Unit system with dimensional checks
│   ├── license/        # License-tier policy engine (impl of specs/license/v0.1.md)
│   ├── validation/     # Cross-crate invariants and fixture validation
│   ├── differential/   # Parity harness vs Brightway + OpenLCA
│   ├── io-ecospold2/   # ecospold2 XML reader (ecoinvent format)
│   └── io-ilcd/        # ILCD XML reader (EU JRC format for PEF/EPDs)
│
├── api/                # Rust Axum API server (Apache-2.0 scaffold)
│
├── web/                # Next.js 15 UI (Proprietary)
│
├── docs/               # Internal + public documentation
│   ├── arko-master-spec-v1.md       # Product vision & 154 design principles
│   ├── arko-tech-spec-v1.md         # Brightway-wrapper architecture (superseded)
│   └── arko-tech-spec-v2.md         # Current architecture — "Excellence Edition"
│
└── .github/            # CI, issue templates, workflows
```

---

## Roadmap (approximate)

| Phase                           | Deliverable                                                 | Estimate |
| ------------------------------- | ----------------------------------------------------------- | -------- |
| **0 — Specifications**          | `specs/calc/v0.1.md` ratified; test vectors published.      | 6 weeks  |
| **1 — Engine core**             | Rust solver; differential parity on 1,000 studies.          | 16 weeks |
| **2 — Engine I/O**              | ecospold2, ILCD, EPDX, OpenLCA JSON-LD round-trip.          | 8 weeks  |
| **3 — API + persistence**       | Axum + Postgres + RLS; auth; license-tier enforcement.      | 10 weeks |
| **4 — Web UI (alpha)**          | Study editor, result viewer, collaboration.                 | 16 weeks |
| **5 — Hosted infra**            | Hetzner + k0s + Talos; CloudNativePG; observability stack.  | 6 weeks  |
| **6 — Private beta**            | 5 design partners. Close the feedback loop before public.   | 12 weeks |

That is ~**74 weeks to private beta** for a single engineer. The ordering is
deliberate: specs before code, engine before API, API before UI. Every phase
is independently useful as OSS even if the next one never ships.

---

## Non-goals (v1.0)

- Not a desktop application.
- Not a drop-in SimaPro migration (export paths yes; wire-compatibility no).
- Not a consultancy tool — it is a platform; the consultancy is someone else's business.
- Not free of charge for commercial use — the hosted product is paid.
- Not dependent on any hyperscaler — no AWS, no Azure, no GCP, no Vercel, no Supabase.

---

## License

Dual-licensed monorepo. See `LICENSE` for the full policy.

- `specs/`, `docs/` — **CC-BY-4.0**
- `engine/`, `api/` (scaffold) — **Apache-2.0**
- `web/`, `services/` — **Proprietary**

---

## Contact

**Goibix S.L.** · Bilbao, Basque Country · [arko.earth](https://arko.earth) *(not live yet)*

Built alongside [KarbonGarbi](https://karbongarbi.earth) — carbon accounting
for Basque industry. Same company. Different product. Different scope.
