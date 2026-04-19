# Arko · Decision Log

Append-only log of significant architectural, strategic, and licensing decisions.
Every entry carries a date, the decision, and the reasoning behind it. When a
decision is later reversed, add a new entry with a back-reference; never delete
the old one.

Format: newest-first. Dates are `YYYY-MM-DD`, local to the author.

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
