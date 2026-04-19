# engine/

The Rust calculation engine. Implements [`specs/calc/v0.1.md`](../specs/calc/v0.1.md).

**Status:** Scaffolded. 13 crates in the workspace; `v0.0.1` pre-tag. Tracked
progress is in [`CHANGELOG.md`](CHANGELOG.md). Phase 0 exit criterion is a
green `cargo build --workspace` + `cargo test --workspace` on the Linux CI
matrix and a tagged `arko-engine v0.0.1`.

## Workspace layout

```
engine/
├── Cargo.toml            # workspace root — shared deps, lints, profiles
├── CHANGELOG.md          # per-crate additions, fixes, and pending items
├── core/                 # matrix types, three-equation pipeline, determinism harness
├── solvers-dense/        # dense LU/QR paths (faer) for small systems
├── solvers-sparse/       # sparse LU (faer) for mid-size; UMFPACK path planned for 10k+
├── parameters/           # expression language, DAG, forward-mode AD
├── uncertainty/          # MT Monte Carlo, distributions (Sobol' planned v0.2)
├── sensitivity/          # one-at-a-time + finite-difference sensitivity
├── methods/              # IPCC AR6 GWP100 and other impact-method presets
├── units/                # unit system + dimensional checks
├── license/              # license-tier policy engine (impl of specs/license/v0.1.md)
├── validation/           # cross-crate invariants, fixture validation
├── differential/         # parity harness vs Brightway / OpenLCA
├── io-ecospold2/         # ecospold2 XML reader (ecoinvent format)
└── io-ilcd/              # ILCD XML reader (EU JRC format for PEF/EPDs)
```

Planned for later phases (see [`docs/arko-execution-guide.md`](../docs/arko-execution-guide.md)):

- `io-ilcd-linker` — cross-reference resolution across ILCD process datasets
- `io-epdx` — EPDX digital-EPD format
- `io-openlca-jsonld` — OpenLCA JSON-LD import/export
- `wasm/` — WASM bindings for in-browser studies
- `cli/` — `arko-calc` command-line tool

## Toolchain

- Channel pinned in [`../rust-toolchain.toml`](../rust-toolchain.toml) to `1.83` with `rustfmt`, `clippy`, and the `wasm32-unknown-unknown` target.
- Workspace lints: `unsafe_code = "forbid"`, clippy `pedantic = "warn"`.
- CI: [`.github/workflows/engine.yml`](../.github/workflows/engine.yml) runs rustfmt + clippy `-D warnings` + tests on Ubuntu, macOS, and Windows.

## Why Rust

- **Memory safety** without GC pauses in the hot path of a matrix solve.
- **Deterministic behavior** — no JIT warmup, no hidden reflection.
- **WASM target** is first-class; same code runs in-browser for small studies.
- **Ecosystem:** `faer`, `nalgebra`, `sprs`, `ndarray` cover what we need.
- **License:** Apache-2.0 throughout; no GPL surprises.

## Non-goals

- Not a general-purpose sparse-linear-algebra library. We reuse `faer` and
  UMFPACK; we do not compete with them.
- Not a Python engine. Brightway already exists and is excellent. Arko's
  engine is Rust-native; Python users call it via bindings or via the API.
