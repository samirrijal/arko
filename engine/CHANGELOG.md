# Engine changelog

All notable changes to the Arko calculation engine are recorded here.
Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
Versioning follows the calc specification (`specs/calc/`) — engine
releases track the spec version they implement.

## [Unreleased]

### Added — `arko-io-ilcd-linker` 0.0.1 (Phase 1, Week 3)

First crate of Phase 1. Resolves the cross-document refs that
`arko-io-ilcd` deliberately leaves dangling: `flowDataSet`,
`flowPropertyDataSet`, `unitGroupDataSet` — just enough of each to walk
the chain **flow → reference flow property → unit group → reference
unit**, which is the question every `A`-matrix column-builder needs to
answer ("given a flow UUID on an ILCD exchange, what unit is its
amount reported in?").

- `LinkResolver` trait — UUID-keyed lookup over Flow / FlowProperty /
  UnitGroup. The extension point for future `ZipBundle`, HTTP-backed,
  or cached resolvers; callers (e.g. future column-builders) depend on
  the trait, not the concrete backend.
- `DirectoryBundle` — the v0.1 reference implementation, backed by
  the standard EU JRC on-disk layout
  (`processes/<UUID>.xml`, `flows/<UUID>.xml`, `flowproperties/`,
  `unitgroups/`). Lazy by design: opening a bundle is free, every
  `resolve_*` re-reads and re-parses the one file it needs. ÖKOBAUDAT
  has ~1k flows and a calc touches ~50 of them, so caching is deferred
  to first profiling-driven need rather than designed in.
- `resolve_reference_unit(resolver, flow_uuid) -> ReferenceUnit` —
  the one-call chain walker. Returns UUID + name for every link in
  the chain so callers can attribute a unit without re-walking
  (useful for error messages and provenance).
- Typed model: `Flow { uuid, base_name, flow_type, cas,
  reference_flow_property_id, flow_properties }` with
  `FlowType::{Elementary, Product, Waste, Other}`;
  `FlowProperty { uuid, base_name, reference_unit_group_uuid }`;
  `UnitGroup { uuid, base_name, reference_unit_id, units }`;
  `Unit { internal_id, name, mean_value }`. `Flow::reference_flow_property()`
  and `UnitGroup::reference_unit()` are the dangling-ref-safe accessors
  (return `Option`; the chain walker converts `None` to
  `LinkError::MissingInternalId`).
- `LinkError` — `thiserror` enum carrying path context on every
  variant: `Io`, `Xml`, `UnexpectedRoot`, `MissingElement`,
  `MissingAttribute`, `InvalidText`, `MissingInternalId`. Diagnostics
  stay useful across bundles with thousands of datasets.
- Strict on missing cross-refs; permissive on unknown elements
  (same posture as `arko-io-ilcd`). `roxmltree`-based, pure-Rust,
  zero unsafe, default-namespace-transparent (ILCD's dual
  `default + common:` namespaces parse without per-element prefix
  tracking).
- Shared `crate::xml` module — `first_child`, `node_text`, `parse_int`,
  `parse_f64` (with non-finite rejection). Three parsers, one place for
  the boilerplate.
- **Integration tests against a synthetic minimal bundle**
  (`tests/fixtures/minimal_bundle/`): hand-crafted fossil-CO2 flow →
  Mass property → Units-of-mass group with `kg` as reference. Five
  tests cover per-dataset parse, `resolve_reference_unit` end-to-end
  chain walk, and `Io` error on missing flow file. Every link in the
  chain exercised with predictable values — the first Week 3 checkpoint
  from the Execution Guide ("tests against a small synthetic ILCD
  bundle you construct manually").

### Added — `arko-io-ilcd-linker` bridge (Phase 1, Week 3-4)

Closes the gap between `arko-io-ilcd` (single-process XML reader) and
the matrix-assembly code downstream: one call turns a `ProcessDataset`
into a column where every exchange carries its flow's resolved
reference unit and flow type.

- `build_typed_column(&ProcessDataset, &impl LinkResolver)
  -> Result<TypedColumn, LinkError>` — walks every exchange, resolves
  each flow + its reference-unit chain, and packs the result.
- `TypedExchange { data_set_internal_id, direction, flow_uuid,
  flow_name, flow_type, amount, reference_unit, is_reference_flow }`
  — one row per exchange, with everything a column-builder needs to
  decide A-vs-B placement and keep units dimensionally honest.
- `TypedColumn { process_uuid, process_name,
  reference_exchange_internal_id, exchanges }` — the column-builder-
  ready shape.
- `resolve_reference_unit_from_flow(resolver, &Flow)` — extracted
  helper so the bridge can reuse an already-loaded `Flow` without a
  duplicate file read. `resolve_reference_unit(resolver, uuid)` now
  delegates to it.
- `arko-io-ilcd` added as a dependency of `arko-io-ilcd-linker`. The
  bridge is the intended coupling point; future caller crates depend
  on `arko-io-ilcd-linker` and inherit `arko-io-ilcd` transitively.
- Amount semantics at v0.1: `resultingAmount` is passed through
  unchanged and labelled with the resolved reference unit. Multi-
  flow-property unit math (mass ↔ energy for a fuel) is deferred —
  lives in `arko-units`, hooks in later.
- Fail-fast: first unresolvable exchange surfaces the underlying
  `LinkError`. Per-exchange error collection for whole-bundle scans
  is a caller concern.
- **Tests**: two new integration tests against the minimal bundle
  plus a hand-built `processDataSet` fixture
  (`processes/00000000-0000-0000-0000-000000000500.xml`) — one happy
  path (CO2 output, direction + flow type + amount + unit assertions,
  `is_reference_flow == true`), one dangling-flow-ref path (confirms
  `LinkError::Io` bubble-up when the exchange's flow isn't in the
  bundle).

### Deferred from this crate (→ v0.2)

- Source and Contact datasets (pure provenance; off the unit-
  resolution path).
- LCIA Method datasets (belong in `arko-methods`, not here).
- Compliance / modelling metadata blocks.
- ZIP-packaged bundles — directory-backed only at v0.1. The
  `LinkResolver` trait is the extension point; a future `ZipBundle`
  plugs in without disturbing callers.
- Caching — re-reads on every call at v0.1 per the lazy-load design.

## [0.0.1] - 2026-04-19

### Added — `arko-core` 0.0.1
- Workspace scaffolded, targeting calc spec **v0.1**.
- `Study`, `ProcessMeta`, `FlowMeta`, `ImpactMeta`, `Parameter`, `Expression`
  AST, `LicenseTier`, `DerivativeRule`, `Provenance`,
  `EffectiveRestriction`, `Warning`, `EngineError` — all types from spec
  §4, §5, §11, §12, §13.
- `Study::canonical_hash` via BLAKE3 for the determinism contract (§7).
- `Solver` trait — the backend contract of §6.3.
- `pipeline::compute` — the §4.3 three-equation pipeline, with license-
  tier join (§11.2) and contributing-process tracking (§12).

### Added — `arko-solvers-dense` 0.0.1
- `DenseLuSolver` — nalgebra-backed dense LU with partial pivoting.
- Intended for `n < 100` per spec §6.3; larger systems will use the
  future `arko-solvers-sparse` (faer) and `arko-solvers-umfpack`.
- Unit tests: identity, non-square rejection, dim-mismatch rejection,
  singular detection, config echoing for provenance.

### Added — `arko-uncertainty` 0.0.1
- `Distribution` enum with all 5 families from spec §9.1: LogNormal,
  Normal, Triangular, Uniform, Beta-PERT. `Distribution::point(v)` is
  the canonical degenerate-lognormal point value.
- Every variant carries `validate()` + `sample()` — validation is
  strict and explicit (negative sd, out-of-order triangular bounds,
  geometric_sd < 1, non-finite inputs all rejected) so malformed
  distributions can't ship silent NaNs through MC runs.
- `run_monte_carlo(config, closure)` — closure-driven runner seeded
  from a Mersenne-Twister stream (`rand_mt::Mt64`). Same seed ⇒
  bit-identical `MonteCarloResult` under the §7.1 + §9.2 determinism
  contract; verified in-test.
- `DimensionStats` — mean, sd, min/max, p05/p25/p50/p75/p95, standard
  error, full sorted samples vector for histogram reuse.
- `MonteCarloConfig.convergence_threshold` drives the `converged` flag
  that surfaces `W_MC_NONCONVERGENT` per spec §13.2.
- 17 tests across `distribution.rs`, `stats.rs`, and
  `tests/monte_carlo_tests.rs` cover validation, sampling bounds,
  determinism, multi-dim consistency, seed independence, dimension-
  mismatch detection, convergence flag behavior, JSON round-trip.

### Deferred from this crate (→ v0.2)
- Sobol' low-discrepancy sampling (spec §9.2 default path). MT-only
  in v0.1. Requires a unified inverse-CDF pipeline.
- Correlated draws (pedigree-based).
- Adjoint sensitivity analysis (§9.3) — belongs with AD in
  `arko-parameters`.

### Added — `arko-solvers-sparse` 0.0.1
- `SparseLuSolver` — faer-backed sparse LU with partial pivoting and a
  COLAMD-family fill-reducing ordering.
- Deterministic matrix construction: triplets sorted by `(col, row)`
  before handing to faer, so the fill-reducing ordering is
  bit-reproducible across runs (spec §7.2).
- Intended for `100 ≤ n < 10_000` per spec §6.3.
- Cross-solver parity tests against `DenseLuSolver` on the §16 example,
  a banded 5×5, and a 20×20 diagonally-dominant sparse system — within
  §8.1 reference-parity tolerance. First down-payment on the §14
  differential harness.
- Version pinning: targets **faer 0.20**. Method names on `SparseColMat`
  (`sp_lu`, `solve_in_place`) have churned across recent faer releases;
  re-verify on upgrade.

### Added — `arko-methods` 0.0.1
- `ImpactMethod`, `ImpactCategory`, `CharacterizationFactor`,
  `FactorMatch` — the data model for a named, versioned method and
  its characterization factor table. `FactorMatch` supports CAS
  (primary), flow-id (fallback), and name+compartment (fuzzy).
- `MethodRegistry` backed by `BTreeMap` for deterministic iteration
  order (spec §7.1). `MethodRegistry::standard()` returns a batteries-
  included instance populated with the default presets.
- `build_c_matrix(method, flows) -> CMatrixBuild` — produces the
  sparse `k × m` characterization matrix plus `ImpactMeta` labels and
  a list of `unmatched_flows` for UI surfacing. Enforces one-factor-
  per-(category, flow) cell: `CMatrixError::DuplicateMatch` surfaces
  author bugs rather than silently summing. Rejects non-finite
  factors.
- Deterministic triplet sort `(row, col)` before handing to sprs so
  the resulting `CsMat` is byte-stable (§7.2).
- **Preset: IPCC AR6 GWP100 (`ipcc-ar6-gwp100` v1)** — 10 canonical
  long-lived GHGs with AR6 Table 7.15 factors: CO2, CH4 (fossil &
  non-fossil — see the AR5/AR6 split entry above), N2O, SF6, NF3,
  HFC-134a, HFC-23, HFC-32, CF4, C2F6. Each factor carries an
  attribution note for audit-log use.
- End-to-end test drives a full `Study` through `pipeline::compute`
  against this method: `0.1 kg fossil CH4 × 29.8 = 2.98 kg CO2-eq`
  to `1e-12` reference-parity tolerance (§8.1). First non-trivial
  test in the repo for the complete methods → builder → pipeline →
  impact chain.
- 18 tests across `method.rs`, `registry.rs`, `builder.rs`,
  `standard.rs`, and `tests/end_to_end.rs`.

### Added — `arko-sensitivity` 0.0.1
- `FactoredSystem { a, f_dense, scaling, generation }` — caches a
  solved `(A, f, s)` triple plus the §10.2 staleness counter.
- `update_rank_1(u, v, solver)` — pure Sherman-Morrison with
  `m = 1 + v^T·A⁻¹·u`, `|m| < 1e-12` → `E_SINGULAR`.
- `update_rank_r(u, v, solver)` — Woodbury identity with nalgebra-LU
  inversion of the `r × r` capacitance matrix; `r = 0` no-ops,
  `r = 1` delegates to `update_rank_1`.
- Convenience entry points required by §10.1: `edit_entry(i, j, δ)`,
  `replace_column(j, new_col)`, `modify_edge(i, j_from, j_to, value)`
  — the last one is a rank-2 update per the spec's "Adding/removing
  an edge" clause.
- `refactor(a, f, solver)` — full recompute path that **resets**
  `generation` to 0 per §10.2.
- `SMW_EPS = 1e-12` — determinant / `m` threshold; tight enough to
  not false-positive on §8.1 problems, loose enough that a truly
  singular update surfaces as `E_SINGULAR` rather than NaN.
- Deterministic `A` rebuild: triplets sorted by `(row, col)` before
  handing to sprs so duplicate-entry accumulation is bit-stable
  (spec §7.2).
- Generation counter invariants verified: bumps on success, not on
  failure or on no-op paths (`δ = 0`, `r = 0`, zero-column replace).
- 17 tests split across `factored.rs`/`updates.rs` inline and
  `tests/smw_parity.rs`. Every update path is verified against a
  full-refactor ground truth within §8.1 tolerance; additional
  coverage for singular-detect, dimension-mismatch, JSON round-trip,
  near-singular pivots, and sequential composition.

### Added — `arko-license` 0.0.1
- `fire_rules(study, scaling)` — walks every process with `|s[j]| >
  EPS_PRESENCE`, evaluates each `DerivativeRule` trigger, and returns
  `Vec<FiredRule>` in byte-stable order (process index then rule
  declaration — matches pipeline's §11.2 join order for determinism).
- `authorize(intent, study, computed, now)` — combines three signals
  into one `Authorization`: base `EffectiveRestriction` flag for the
  intent, worst-outcome across fired derivative rules (`Blocked >
  Watermark > Warn > Allowed`), and expiry check against the caller-
  provided `now`. Fired rules are returned verbatim even on `Blocked`
  so callers can write the audit log required by the §11 spec.
- `Intent::{Publish, Share, Export}` and `Outcome::{Allowed, Warn,
  Watermark, Blocked}` — the `Ord` impl on `Outcome` is deliberate so
  downstream code can use `.max()` across a rule set.
- `presets::ecoinvent_v3`, `sphera_strict`, `open_cc_by`, `custom_user`
  — conservative std-library presets with doc-commented disclaimers
  that they are not legal advice. Seed content for the future
  `specs/license/` ratification.
- **Guiding principle preserved** from `specs/license/README.md`:
  the solve is always legal. This crate only runs at publish/share/
  export time, never during `pipeline::compute`.
- 22 tests across `fire.rs`, `authorize.rs`, `presets.rs`, and
  `tests/authorize_tests.rs` cover every branch: base-flag
  dispatch per intent, threshold boundary (fires at `==`, not below),
  sign-independence of `ScalingGe`, worst-outcome ordering, expiry
  (before/after caller-provided `now`), zero-contribution processes
  skipped, preset end-to-end against a real solve, JSON round-trip.

### Added — `arko-differential` 0.0.1
- **The conformance harness of spec §14.** Ship the *framework* — test-
  vector data model, runner, property checks, report — at v0.0.1. The
  ≥10,000-vector corpus is a populate-over-time operation; the loader,
  runner, and reporter do not need to change as it grows.
- `TestVector { id, level, description, study, expected_h,
  tolerance_class, notes }` — the §14.1 unit of conformance: a fully-
  specified `Study` paired with the expected impact vector and the
  tolerance class under which parity is checked.
- `ConformanceLevel::{L1Basic, L2Full, L3Elite}` with a total order so
  `highest_level_passed` can compare levels monotonically (§14.2).
- `ToleranceClass::{ReferenceParity, CrossImpl}` implementing §8.1's
  `max(ε_abs, ε_rel · |want|)` with the exact (1e-12, 1e-9) vs
  (1e-9, 1e-6) pairs the spec mandates.
- `run_conformance(vectors, solver, config) -> ConformanceReport` —
  drives a `Solver` through every vector, collecting pass / fail /
  engine-error / shape-mismatch verdicts, timings, and worst
  deviations per vector. Deterministic output order = input order.
- `ConformanceReport` is the §14.4 required output: `engine_version`,
  `spec_version`, `solver_name`, `started_at`, `total_ms`, summary
  counts, `highest_level_passed`, and `per_vector`. Round-trips
  through `serde_json` for on-disk `conformance-report.json`.
- `VectorVerdict::{Pass, Fail, ShapeMismatch, EngineError}` with
  `is_pass/is_fail/is_error` classifiers; `Pass` carries
  `max_abs_deviation` + `max_rel_deviation` for regression dashboards,
  `Fail` surfaces the first component that exceeded tolerance plus the
  tolerance actually applied (useful when triaging).
- **§14.3 property tests** — four of five shipped at v0.0.1:
  - `check_scaling_identity` — `A·s ≈ f` to §8.1 parity.
  - `check_idempotent_recompute` — two solves produce bit-identical
    `(s, g, h)` per §7.1 determinism.
  - `check_block_diagonal_independence` — stacking two disjoint
    studies yields the concatenation of their individual scaling
    vectors.
  - `check_sherman_morrison_parity` — the `arko-sensitivity` rank-1
    path matches a full refactor within §8.1.
  - Parameter-rewrite equivalence is deferred to v0.2 (needs
    parameter-expression evaluation wired into `pipeline::compute`).
- `load_vector_directory(dir)` — reads every `*.json` file from a
  directory in sorted filename order (reproducible iteration per §7.1).
  `VectorLoadError` includes the offending path so malformed vectors
  are easy to locate in a corpus of hundreds.
- `seed_vectors()` — five hand-crafted seed vectors covering L1
  (identity-single-impact, two-process-independent, coupled-upper-
  triangular forcing real back-substitution), L2 (allocation metadata
  round-trip), and L3 (determinism-tagged variant). Self-contained
  fixtures so the crate ships with working test material even before
  the on-disk corpus is populated.
- End-to-end conformance tests drive `run_conformance` against the
  seed corpus with `DenseLuSolver`: every vector passes,
  `highest_level_passed == L3Elite`, per-vector order preserved, and
  the report JSON contains every §14.4 required top-level key.
- Property integration tests exercise each `check_*` function against
  seed studies.

### Added — `arko-parameters` 0.0.1
- Expression evaluator for the restricted arithmetic of spec §5.1.
- Kahn's-algorithm topological sort with deterministic tie-breaking.
- Cycle detection → `E_PARAM_CYCLE`; unresolved refs → `E_PARAM_UNRESOLVED`;
  non-finite results (sqrt of negative, div-by-zero, log of non-positive)
  → `E_PARAM_NONFINITE`.
- Public `walk_deps` / `eval_expr` helpers for reuse by matrix-entry
  evaluators.

### Added — `arko-io-ecospold2` 0.0.1
- `parse_dataset(xml)` — reads a single ecospold2 XML document (both
  `<activityDataset>` and `<childActivityDataset>` envelopes accepted).
- Typed model: `ActivityDataset`, `Activity`, `Geography`,
  `IntermediateExchange`, `ElementaryExchange`, `Direction`. Everything
  round-trips through `serde_json` so parsed datasets can be cached.
- `roxmltree` backend — pure-Rust, zero-unsafe, default-namespace-aware.
- Permissive on unknown elements (silently ignored), strict on
  structural violations (missing attributes, ambiguous direction,
  non-finite amounts).
- Integration tests cover: reference-product detection, technosphere-
  input identification, elementary-exchange compartment parsing, JSON
  round-trip, error cases (bad root, missing envelope, both directions,
  child-dataset envelope acceptance).

### Added — `arko-units` 0.0.1
- **UCUM-subset parser** covering the unit strings LCA data actually
  uses: SI bases, SI-derived named units (N, Pa, J, W, Wh, Hz, L,
  bar), and LCA-common composites (`t.km`, `kg.m`, `MJ`, `kWh`,
  `GJ`). Grammar: `.`/`*` multiplication, `/` division
  (left-associative per UCUM 4.2.2), `^` explicit exponent, `m2`
  implicit-positive exponent. Unicode middle-dot `·` and `×`
  normalize to `.`.
- **Full SI prefix set** `Y..y` (1e24..1e-24) including unicode `μ`
  and the ASCII `µ`/`u` micro aliases. Two-character `da` (deka)
  works correctly under greedy longest-match.
- **Semantic tag carrier** — anything after the first ASCII space
  (trimmed and whitespace-collapsed) rides as `tag`. `kg` and
  `kg CO2-eq` parse to the same 7D dimension but are reported **not
  commensurable** because their tags differ. Conversion between
  them is the characterization-factor problem; it is *not* a unit
  problem and must not silently succeed.
- `Dimension` — a 7×`i8` tuple over L/M/T/N/Θ/I/J with `add`/`sub`/
  `scale` for compound-unit arithmetic. `Display` impl emits the
  canonical `L^a M^b T^c N^d Θ^e I^f J^g` form for diagnostics.
- `ParsedUnit { source, dimension, scale_to_si, tag }` — the
  public analyzed form; JSON-serializable for on-disk method
  definitions and round-trippable under `serde_json`.
- Public API: `parse`, `commensurable`, `conversion_factor`,
  `convert`, `convert_str` (string-level convenience), plus
  `check_compatibility(a: &arko_core::Unit, b: &arko_core::Unit)`
  — the hook `arko-validation` uses.
- **Mass atom is `g`, not `kg`.** `kg` parses as prefix `k` +
  atom `g`, net scale 1.0 kg. Tonne `t` = 1000 kg is a distinct
  atom. This mirrors the UCUM convention and avoids double-counting
  the prefix when users write `Mg` (megagram).
- End-to-end conversion identities verified in
  `tests/conversion_tests.rs`: `1 kWh = 3.6 MJ`, `1 t.km = 10^6 kg.m`,
  `1 yr = 365.25 d`, `1 L = 10^-3 m3`, plus every round-trip
  (`MJ → kWh → MJ ≈ identity within 1e-15`).
- **Not covered at v0.0.1 (→ v0.2)**: affine conversions (°C ↔ K),
  UCUM curly-brace annotations (`kg{dry}`), non-metric units (lb,
  ft, BTU), and dimensional analysis across parameterized matrix
  entries. Parser error messages surface the offending identifier
  so unknown units get routed to the right follow-up crate rather
  than silent acceptance.

### Added — `specs/license/v0.1.md`
- First formal draft of the **license-tier policy language**, the
  semantic layer on top of the storage types ratified in
  `specs/calc/v0.1.md` §11. The reference implementation
  (`engine/license/`) targets this draft.
- Sections: guiding principle (solve is always legal), `Intent`
  enumeration, three-signal authorization decision (base flag →
  derivative rules → expiry, most-severe-wins ordering on
  `Outcome`), trigger semantics (`ScalingGe { threshold }` with
  abs-value `≥` semantics, `Always`), action → outcome mapping,
  audit-log requirements (append-only, operator-attestation for
  `warn`/`watermark`, no overrides at the API layer), preset
  catalogue, conformance, determinism contract, error handling,
  forward-compatibility.
- `specs/license/README.md` and `specs/README.md` updated to point
  at the new draft and remove the "Not started" tag.
- v1.0 ratification gated on: (1) preset EULA verification by
  counsel, (2) one independent implementation passing the
  forthcoming conformance corpus, (3) audit-log validation against
  a real-world reporting workflow.

### Added — `arko-io-ilcd` 0.0.1
- ILCD `<processDataSet>` reader — parses EU JRC ILCD process datasets,
  the format underlying **PEF** (Product Environmental Footprint) and
  most European EPDs (EPD-Norge, GlobalEPD/AENOR, DAPcons, EPD Italy,
  ECO Platform members). The single most consequential format outside
  ecoinvent for KarbonGarbi's Spanish industrial market.
- Typed model: `ProcessDataset`, `ProcessInformation`,
  `QuantitativeReference`, `Exchange`, `Direction`. Round-trips via
  `serde_json` so parsed datasets can be cached.
- Captured fields per dataset: dataset UUID, base name, treatment-
  standards-routes qualifier, ISO/region location code, reference
  year, the `dataSetInternalID` of the declared reference flow, and
  per-exchange: internal ID, linked flow UUID/URI, English short
  description, direction, `meanAmount`, `resultingAmount`, parameter
  binding (`<referenceToVariable>`), data-derivation status.
- `roxmltree`-based parser; `has_tag_name` matches local name only,
  so the dual `default + common:` namespace declarations on every
  ILCD root parse without per-element prefix tracking.
- Permissive on unknown / EPD-extension elements (modelling metadata,
  validation, compliance declarations, bundled `<LCIAResults>`),
  strict on structural violations: missing UUID, dangling
  `<referenceToReferenceFlow>` (`MissingReferenceFlow { id }`),
  unknown `<exchangeDirection>`, non-finite amounts.
- Resulting-amount fallback: when `<resultingAmount>` is absent (very
  common in published EPDs) the reader copies `<meanAmount>` forward
  so downstream column-builders have one canonical amount field.
- 11 integration tests over a Spanish CEM I cement fixture: 6
  positive (envelope parse, reference-product resolution,
  parameterized input, elementary emission, JSON round-trip,
  mean-only fallback) and 5 negative (bad root, missing UUID,
  dangling `<referenceToReferenceFlow>`, unknown direction, `NaN`
  amount).

### Deferred from this crate (→ v0.2)
- Flow, FlowProperty, UnitGroup, Source, Contact, LCIA Method
  datasets — separate readers, future crates.
- Full modelling-and-validation / administrative-information
  metadata blocks (preserved-as-strings would bloat the model).
- `<mathematicalRelations>` parameter expressions — name binding
  only at v0.1; expression evaluation belongs in `arko-parameters`.
- Bundled `<LCIAResults>` blocks in published EPDs — they are
  *outputs*, not inputs, and we recompute from B and C.
- Cross-document linking against flow / unit-group catalogues —
  belongs in a future `arko-io-ilcd-linker`.

### Added — `arko-validation` 0.0.1
- `validate(study)` — enforces the §6.1 construction-order checks that
  are implementable in v0.1: parameter DAG well-formedness (delegated to
  `arko-parameters`), license-tier reference resolution, matrix shape
  consistency, user-allocation factor sum ≈ 1.
- Unit-consistency check (§6.1 step 3) now wired to `arko-units` —
  every `ProcessMeta::reference_unit`, `FlowMeta::unit`, and
  `ImpactMeta::unit` must parse under the UCUM-subset grammar;
  unparseable inputs surface as `E_UNIT_INCOMPATIBLE` naming the
  offending field. Cross-dimensional consistency (flows ↔ impacts via
  characterization) and the §6.1 step-6 well-conditioning heuristic
  remain deferred.

### Tests
- `engine/core/tests/minimal_example.rs` — the §16 worked example end
  to end: constructs the two-process steel/electricity study, solves
  `A·s=f`, computes `g = B·s`, `h = C·g`, and asserts
  `s = [1.0, 0.5]`, `g = [2.05]`, `h = [2.05]` within §8.1 tolerance.
- Also asserts the §11.2 effective-restriction join and the §12
  contributing-processes list.

### CI
- `.github/workflows/engine.yml`: `cargo fmt --check`, `cargo clippy
  -D warnings`, `cargo test` on Linux / macOS / Windows.
- `.github/workflows/specs.yml`: markdownlint + lychee link-check on
  `specs/**/*.md`.

### Added — AR5/AR6 GWP100 split (2026-04-19, pre-`v0.0.1` tag)

**Decision:** D-0007 (Option D) in `DECISIONS.md`. Ship both assessment
reports as first-class presets so new studies default to current
science (AR6) and legacy EPD re-verification stays bit-exact on AR5.

**Schema — `arko-core`:**
- `FlowOrigin { Unspecified, Fossil, NonFossil }` added to
  `core::meta`. `#[default]` is `Unspecified`, so existing JSON
  fixtures round-trip unchanged and legacy blobs with no `origin`
  key deserialize cleanly.
- `FlowMeta.origin` added with
  `#[serde(default, skip_serializing_if = "FlowOrigin::is_unspecified")]`
  so the wire format is backward-compatible.
- `FlowOrigin` re-exported from `arko_core::` prelude.
- Three new unit tests in `core/src/meta.rs`: default, JSON skip,
  legacy-blob round-trip.

**Schema — `arko-methods`:**
- `FactorMatch::CasOrigin { cas, origin }` variant added alongside the
  existing `Cas { cas }`. Matches CAS **and** exact `FlowOrigin`:
  a flow with `Unspecified` origin does **not** match a `CasOrigin`
  factor (regression guard against silently applying fossil factors
  to unknown-origin CH4).
- `FactorMatch::matches()` updated; `builder::matcher_label()` updated.
- Four new unit tests exercising `CasOrigin` match/reject, CAS
  mismatch rejection, plain `Cas` remaining origin-agnostic, and
  JSON round-trip including the new tag.

**Method library — `arko-methods::standard`:**
- `ipcc_ar6_gwp100()` — now splits CH4 into two `CasOrigin` factors:
  fossil `29.8` and non-fossil `27.0` per AR6 WG1 Ch7 T7.15.
  **N2O verified 273** (no drift vs table). All fluorinated-gas
  factors (SF6 `25_200`, NF3 `17_400`, HFC-134a `1_530`, HFC-23
  `14_600`, HFC-32 `771`, CF4 `7_380`, C2F6 `12_400`) reconfirmed
  against AR6 Table 7.15 — **no drift detected**.
- `ipcc_ar5_gwp100()` — new preset. CH4 single-valued `28`, N2O
  `265`, SF6 `23_500`, NF3 `16_100`, HFC-134a `1_300`, HFC-23
  `12_400`, HFC-32 `677`, CF4 `6_630`, C2F6 `11_100` per AR5 WG1
  Ch8 T8.A.1 (column without climate-carbon feedback). CH4 stays
  as a plain `Cas` match — AR5 did not differentiate fossil vs
  non-fossil.
- `MethodRegistry::standard()` now returns both presets; `len() == 2`.
- Nine new unit tests across `standard.rs` (AR6 CH4 split, AR5 CH4
  single-valued + origin-agnostic, AR6 N2O = 273, AR5 N2O = 265,
  distinct `(id, version)` keys).

**Seed corpus — `arko-differential::seed`:**
- `l1_two_process_independent`: CH4 flow tagged `FlowOrigin::Fossil`;
  CF updated `27.9 → 29.8`; `expected_h: [2.79] → [2.98]`.
- `l1_coupled_two_process`: CH4 flow tagged fossil; CF updated
  `27.9 → 29.8`; `expected_h: [34.32] → [35.84]` (re-derived from
  `s = [4, 4]`, `g = [12, 0.8]`, `h = 12·1 + 0.8·29.8`).
- **New vector** `l1_ch4_non_fossil_origin_split`: paired regression
  guard using `FlowOrigin::NonFossil` and `27.0`, `expected_h = [2.70]`.
  An engine that confuses the origin split will silently compute
  `2.98` here and fail.

**End-to-end test — `arko-methods::tests::end_to_end`:**
- `full_pipeline_with_real_method_and_real_numbers`: CH4 flow tagged
  fossil; `expected_h: 2.79 → 2.98`.
- `build_c_matrix_against_ipcc_flows`: CH4 flow tagged fossil so the
  `CasOrigin{fossil}` matcher fires; assertions unchanged structurally.
- **New test** `ar6_rejects_unspecified_origin_ch4_as_unmatched`:
  asserts that an `Unspecified`-origin CH4 flow surfaces in
  `unmatched_flows` rather than silently inheriting the fossil GWP.
- **New test** `ar5_and_ar6_agree_where_they_should_and_disagree_where_they_should`:
  cross-method consistency guard. A single study with 0.06 kg fossil
  CH4 + 0.04 kg non-fossil CH4 is computed under both presets —
  AR6 must return `2.868` (split-weighted, and **not** `2.98` or
  `2.70`), AR5 must return `2.8` (origin-agnostic), and the two
  results must differ. Catches the class of bug where the CH4 split
  silently collapses to a single origin in dispatch.
- `standard_registry_contains_ipcc_gwp100_and_it_resolves` updated
  for `reg.len() == 2`.

**Other construction sites updated for the new field** (each uses
`origin: Default::default()` since they test unrelated machinery):
- `engine/core/tests/minimal_example.rs`
- `engine/validation/src/lib.rs`
- `engine/license/tests/authorize_tests.rs`
- `engine/methods/src/builder.rs` test helper.

**Resolved from Pending:** CH4 GWP100 reconciliation (D-0007) and
N2O verification. The stale `docs/phase-0-compile-risks.md` §2a / §2b
items are now superseded — when cargo goes green the doc becomes a
postmortem per its own exit criterion.

### Fixed — Phase 0 first-green shakedown (2026-04-19)

First-ever `cargo build --workspace && cargo test --workspace` against
the live ecosystem surfaced the following issues. None changed the
engine's public contract; all are fixed in-place.

**Toolchain & resolver:**
- Bumped `rust-toolchain.toml` from `1.83` → `1.95` (D-0008). `1.83`
  predates edition 2024, which is now required by `blake3 1.8.4`
  (direct) and `constant_time_eq 0.4.3` (transitive via blake3).
  1.85 was tried as an interim step but `constant_time_eq` pushed the
  MSRV to 1.95. Rather than pin transitives down individually, sit
  on current stable.

**Code fixes:**
- `core::parameters::Expression` — dropped `#[serde(tag = "kind")]`.
  Internal-tag representation is incompatible with tuple variants;
  switched to default externally-tagged. No committed fixtures used
  the `{"kind": "..."}` shape.
- `units::dimension::Dimension::scale` — marked `const fn` so it can
  appear in the static `ATOMS` table entries for `Hz`, `L`, and `l`.
- `solvers-sparse` — added `use faer::prelude::SpSolver;` so
  `sp_lu().solve_in_place(...)` resolves. Confirmed §1b and §1c of
  `docs/phase-0-compile-risks.md` on the actual faer 0.20.2 surface:
  `sp_lu` works as-is, `solve_in_place` needs the trait imported.
- `sensitivity/Cargo.toml` — added missing `thiserror.workspace = true`
  dependency (was declared in source but not in the manifest).
- `validation/Cargo.toml` — added `sprs.workspace = true` as a
  dev-dependency; inline tests construct `TriMat`s.
- `uncertainty/Cargo.toml` — added `serde_json.workspace = true`
  dev-dep for the JSON-roundtrip tests.
- `license/Cargo.toml` — added `serde_json.workspace = true` dev-dep.
- `differential::vector::TestVector` — dropped unused `PartialEq`
  derive; `Study` isn't `PartialEq`, and the vector is only ever
  identified by its `id` field.
- `differential::runner::compute_highest_level` — fixed vacuous-pass
  bug. An L1+L2-only run was earning `L3Elite` because "no failures
  at level ≤ L3" is not the same as "at least one L3 vector passed."
  Now requires a vector to exist at the exact level being claimed.
- `differential::vector::tolerance_for_scales_with_large_values` —
  replaced bit-equality check with an `abs() < 1e-18` tolerance.
  `1e-9 * 1000.0 = 1.0000000000000002e-6` per IEEE-754 and the test
  was effectively asserting against rounding.
- `core/tests/minimal_example.rs` — transposed the technosphere
  triplet from `(0, 1, -0.5)` to `(1, 0, -0.5)`. The spec §16 narrative
  "1 kg steel needs 0.5 MJ electricity, so s = [1, 0.5]" only holds
  under Brightway convention (`A[product][activity]`), and the engine
  solves `A·s = f` literally without transposing. **Flagged as spec
  errata for v0.1.1** — §16's written `A = [[1.0, -0.5], [0.0, 1.0]]`
  is arithmetically inconsistent with its claimed `s`; it should read
  `A = [[1.0, 0.0], [-0.5, 1.0]]`. The seed vector
  `l1_coupled_two_process` uses the same `(0, 1, -0.5)` triplet but
  is self-consistent because its `f = [2, 4]` is chosen to make the
  literal `A·s = f` yield the expected `s = [4, 4]`.
- `units::parser::parse_unit_expr` — handle bare `"1"` as the canonical
  dimensionless literal. The tokenizer rejects leading digits (they
  aren't identifier starters), so special-cased before tokenizing.
- `methods/tests/end_to_end.rs` — removed trailing string-message
  arguments from `assert_relative_eq!` calls; the `approx` macro
  doesn't accept them (pre-existing typo, surfaced here for the first
  time). Behavior unchanged.

### Documentation — Phase 0 pre-cargo sweep
- `engine/README.md` rewritten to reflect the actual 13-crate workspace
  (was stuck on "Not started. No Cargo workspace yet").
- Root `README.md` repository-layout section updated to list the real
  crate directory names (`solvers-dense`, `solvers-sparse`, `io-ecospold2`,
  `io-ilcd`, `methods`, `sensitivity`, `units`, `validation`, `license`)
  instead of the outdated placeholder paths.
- `DECISIONS.md` seeded at repo root with six initial entries dated
  2026-04-19: dual-license policy, Rust 1.83 + faer 0.20 pin with a
  version-drift caveat, MT-only Monte Carlo in v0.1, solver thresholds
  (dense<100 / sparse 100-10k / UMFPACK 10k+), no ecoinvent in V1,
  strategic upgrade of Arko from side-quest to primary product.
- `docs/phase-0-compile-risks.md` added: pre-cargo self-audit flagging
  faer 0.20 API surface bets in `engine/solvers-sparse/src/lib.rs`
  (`try_new_from_triplets`, `sp_lu`, `solve_in_place`, `Col` index
  mutation) and an **AR6 GWP100 CH4 factor discrepancy** — shipped
  `27.9` does not match AR6 Table 7.15 (fossil `29.8` / non-fossil
  `27.0`). Downstream `engine/differential/src/seed.rs` uses 27.9 in
  `l1_two_process_independent` (`h=[2.79]`) and `l1_coupled_two_process`
  (`h=[34.32]`); both must be updated in lockstep when the CH4 factor
  is reconciled.

### Pending
- `arko-solvers-umfpack` — UMFPACK bindings for `n ≥ 10,000`.
- `arko-io-epdx`, `arko-io-openlca-jsonld`.
- `arko-io-ilcd-linker` — resolve flow / unit-group / LCIA-method
  cross-references across an ILCD bundle into a column of `A` and
  rows of `B`. The reader gives one process; the linker turns a
  zip of processes + flows + unit groups into a study.
- Populate the `arko-differential` corpus toward the §14 ≥10,000-vector
  requirement — reference dumps from Brightway 2.5, OpenLCA, SimaPro.
  The framework is in place; only the on-disk JSON fixtures need
  generating.
- Parameter-rewrite equivalence property (§14.3 fifth property) —
  blocked on parameter-expression evaluation inside `pipeline::compute`.
- Additional method presets: ReCiPe 2016 Midpoint, CML 2001 baseline,
  TRACI 2.1, EF 3.1 (16 categories).
- `FactoredSolver` trait — let `arko-sensitivity` reuse a cached
  factorization instead of re-factoring once per rank-r solve.
- License-spec **conformance corpus** (`specs/license/v0.1.md` §10).
  Generate study + computed + intent + now tuples that bracket every
  preset's threshold edge, expiry instant, and join-collapse case;
  freeze expected `Authorization` JSON for cross-implementation
  parity testing.
