# Phase 0 — compile-risk audit

Pre-cargo self-audit of code shipped without a local Rust toolchain. When
you run `cargo build --workspace` for the first time on Windows, expect
breakage in one or more of the items below. Each entry lists the file,
the risk, and the minimal fix if the default call doesn't compile.

**Date of audit:** 2026-04-19
**Audit scope:** code added through `arko-io-ilcd 0.0.1` and
`specs/license/v0.1.md`.

---

## 1. `engine/solvers-sparse/src/lib.rs` — faer 0.20 surface

All five of these are API-name bets made without cargo on hand. If any one
of them fails, the fix is local to this file; no callers depend on the
internal faer types.

### 1a. `SparseColMat::try_new_from_triplets` (line 77)

```rust
let mat = SparseColMat::<usize, f64>::try_new_from_triplets(rows, cols, &triplets)
```

In faer 0.20 the canonical signature takes `&[Triplet<I, E>]` rather than
`&[(I, I, E)]` tuples. If the tuple form is rejected, convert with:

```rust
let triplet_list: Vec<faer::sparse::Triplet<usize, f64>> = triplets
    .into_iter()
    .map(|(r, c, v)| faer::sparse::Triplet::new(r, c, v))
    .collect();
let mat = SparseColMat::<usize, f64>::try_new_from_triplets(rows, cols, &triplet_list)?;
```

### 1b. `mat.sp_lu()` (line 85)

Expected to return `Result<Lu<usize, f64>, LuError>`. In older faer the
method is `sparse_lu()`; in newer it is `sp_lu()`. If `sp_lu` is not a
method, try `sparse_lu()` and widen the `Singular` mapping:

```rust
let lu = mat.sp_lu().map_err(|_| EngineError::Singular)?;
// fallback:
let lu = mat.sparse_lu().map_err(|_| EngineError::Singular)?;
```

### 1c. `lu.solve_in_place(sol.as_mut())` (line 87)

`Col::as_mut` returns `ColMut`. In faer 0.20 `solve_in_place` may expect
a `MatMut` — `ColMut` has `as_2d_mut()` to widen:

```rust
lu.solve_in_place(sol.as_mut().as_2d_mut());
```

### 1d. `rhs[i] = v;` (line 82) and `sol[i]` read (line 89)

Col indexing via `IndexMut` is supported in recent faer but was added
over time. If either fails to compile:

```rust
// write:
rhs.as_mut().write(i, v);
// read:
sol.read(i)
```

### 1e. Return collect (line 89)

```rust
Ok((0..rows).map(|i| sol[i]).collect())
```

Depends on 1d. If `sol[i]` doesn't return `f64`, use `sol.read(i)`.

---

## 2. `engine/methods/src/standard.rs` — AR6 GWP100 factor values

**Resolved 2026-04-19 per `DECISIONS.md` D-0007.** Option D adopted: ship
both AR6 (default) and AR5 (legacy parity) presets with a new
`FlowOrigin` classifier on `FlowMeta` and a `FactorMatch::CasOrigin`
variant.

| Species | AR6 shipped | AR6 table | AR5 shipped | AR5 table | Status |
| --- | --- | --- | --- | --- | --- |
| CO2 | 1.0 | 1.0 | 1.0 | 1.0 | ✅ reference |
| CH4 fossil | **29.8** | 29.8 | — | — | ✅ fixed from `27.9` |
| CH4 non-fossil | **27.0** | 27.0 | — | — | ✅ added |
| CH4 (AR5, origin-agnostic) | — | — | **28** | 28 | ✅ added |
| N2O | 273 | 273 | **265** | 265 | ✅ both verified |
| SF6 | 25,200 | 25,200 | 23,500 | 23,500 | ✅ |
| NF3 | 17,400 | 17,400 | 16,100 | 16,100 | ✅ |
| HFC-134a | 1,530 | 1,530 | 1,300 | 1,300 | ✅ |
| HFC-23 | 14,600 | 14,600 | 12,400 | 12,400 | ✅ |
| HFC-32 | 771 | 771 | 677 | 677 | ✅ |
| CF4 (PFC-14) | 7,380 | 7,380 | 6,630 | 6,630 | ✅ |
| C2F6 (PFC-116) | 12,400 | 12,400 | 11,100 | 11,100 | ✅ |

Sources: AR6 WG1 Ch7 Table 7.15 (Forster et al. 2021) and AR5 WG1 Ch8
Table 8.A.1 without climate-carbon feedback column (Myhre et al. 2013).

### 2a/2b. Seed vector & end-to-end test updates — done.
- `l1_two_process_independent`: CH4 tagged fossil, `h=[2.98]`.
- `l1_coupled_two_process`: CH4 tagged fossil, `h=[35.84]`.
- New `l1_ch4_non_fossil_origin_split`: `h=[2.70]`.
- `end_to_end.rs::full_pipeline_...`: `h=2.98`.
- `end_to_end.rs::ar6_rejects_unspecified_origin_ch4_as_unmatched`:
  new regression guard.

---

## 3. `engine/io-ilcd/src/reader.rs` — roxmltree namespace behavior

**Risk:** roxmltree 0.20's `has_tag_name("foo")` matches local names
regardless of namespace. This is what we rely on to handle ILCD's dual
namespace (default + `common:` prefix) transparently. The claim is
documented in roxmltree 0.20, but the tests only exercise synthetic
fixtures we authored ourselves. If a real EU JRC ILCD file fails to
parse, suspect namespace handling first.

**Mitigation:** the v0.1 license spec conformance corpus (pending)
will include at least one real published ILCD file; a parse failure
there would surface this before external positioning.

---

## 4. Files/paths not yet wired

These are called out in the execution guide as Phase 1 work, but cargo
won't catch their absence. Track them in `CHANGELOG.md` Pending, not
here.

- `engine/io-ilcd-linker/` — cross-reference resolution across ILCD
  process datasets
- `engine/io-epdx/` — EPDX reader
- `engine/io-openlca-jsonld/` — OpenLCA JSON-LD round-trip
- `engine/methods/src/recipe_2016.rs`, `ef_31.rs`, `cml_2001.rs` —
  additional method presets

---

## v0.0.1 tag gate

Per the user's 2026-04-19 directive, **do not tag `arko-engine v0.0.1`
until CI is green on the corrected method library.** This means, at
minimum:

1. `cargo build --workspace` passes locally on Windows (rustup install
   still required — blocker).
2. `cargo test --workspace` passes — all tests touched by the AR5/AR6
   split (standard.rs, registry.rs, method.rs, builder.rs, end_to_end.rs,
   seed.rs, meta.rs, minimal_example.rs, authorize_tests.rs,
   validation/src/lib.rs) are green.
3. `.github/workflows/engine.yml` matrix (Ubuntu + macOS + Windows)
   reports success.
4. Any §1 faer-API fixes required by the real compiler are committed.

Only then: `git tag arko-engine-v0.0.1 -m "..."` and push.

## Exit criterion for this doc

When the v0.0.1 tag is cut, move this file to
`docs/phase-0-postmortem.md` with the resolution summary. Do not
delete it — the fix trace is useful for future version-drift audits.
