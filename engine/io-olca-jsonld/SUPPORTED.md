# `arko-io-olca-jsonld` — what v0.1 supports, and what it deliberately doesn't

The v0.1 openLCA JSON-LD reader was written against the **USDA LCA
Commons beef cattle finishing bundle** (five-process cow-calf-finisher
subgraph). Discipline: parse what that bundle needs, leave every other
openLCA feature for the next bundle's pressure. This file enumerates
what is and isn't handled so that future readers of an unfamiliar
JSON-LD bundle can immediately distinguish "feature I punted" from
"regression."

If you hit a parse failure on a bundle listed as unsupported below,
that is expected behaviour for v0.1 — the path forward is either to
extend the reader or to carve that bundle out of the study, not to
patch around the error silently.

## Supported object kinds

- **`Process`** — `@id`, `name`, `processType` (`UNIT_PROCESS`,
  `LCI_RESULT` parses but is not semantically handled by the adapter),
  `defaultAllocationMethod` (pass-through only), `exchanges[]`.
- **`Exchange`** — `internalId`, `amount`, `input`, `avoidedProduct`
  (parsed flag only), `quantitativeReference`, `flow.@id`,
  `flow.flowType` (from embedded ref), `unit.@id`, `flowProperty.@id`,
  `defaultProvider.@id`.
- **`Flow`** — `@id`, `name`, `flowType`, `cas` (normalised — leading
  zeros trimmed), `formula`, `flowProperties[]` with the one
  `referenceFlowProperty: true` entry identified.
- **`FlowProperty`** — `@id`, `name`, `unitGroup.@id`.
- **`UnitGroup`** — `@id`, `name`, `units[]` with one
  `referenceUnit: true`, each carrying a `conversionFactor` back to
  that reference.

## On-disk layout

Directory-backed, standard Federal LCA Commons layout:

```text
<bundle-root>/
  processes/<UUID>.json
  flows/<UUID>.json
  flow_properties/<UUID>.json
  unit_groups/<UUID>.json
```

ZIP-packaged bundles: not read at v0.1. `OlcaBundle` is the natural
extension point; a future `OlcaZipBundle` plugs in without disturbing
callers.

## Deliberately unsupported (do not treat as regressions)

### Object kinds not read
- `actors/*.json` (provenance, non-calc)
- `sources/*.json` (bibliographic)
- `categories/*.json` (taxonomy, non-calc)
- `locations/*.json` (geography, carried through embedded refs only)
- `dq_systems/*.json` (data-quality metadata)

### Process semantics
- `allocation` factor blocks — the adapter carries
  `defaultAllocationMethod` as a string and applies no allocation
  logic. Beef bundle ships already-allocated exchanges.
- `parameters[]` + `mathematicalRelations` — v0.1 assumes every
  exchange amount is a literal numeric. Parameterised datasets will
  parse the amount as `0.0` or fail outright.
- `avoidedProduct: true` sign handling — the flag is parsed but the
  adapter does **not** sign-flip the amount. Downstream matrix
  assembly must honour the flag. Beef bundle has none.

### Flow semantics
- **Cross-property exchanges.** If an exchange declares a
  `flowProperty` that differs from its flow's
  `referenceFlowProperty`, the adapter errors with
  `FlowPropertyNotDeclaredOnFlow`. Beef bundle does not trip this.
  Cross-property conversion (mass ↔ energy for a fuel) belongs in a
  future dimensional-analysis pass (`arko-units`).
- **Alternate unit groups** reached via a flow property's
  non-reference unit groups. Out of scope — same reason.

### Process-type semantics
- `LCI_RESULT` processes (aggregated inventories published as if they
  were processes). The parser accepts them but the adapter currently
  treats them identically to `UNIT_PROCESS`. Beef bundle is entirely
  `UNIT_PROCESS`.

### Other
- `LCIA_METHOD` documents — a separate crate's concern
  (`arko-methods`).
- `RESULT` documents — openLCA's serialised study results. We
  recompute from A, B, and C; we don't read published results.

## Extension posture

Additions land when the next bundle's pressure demands them, not
speculatively. The `Exchange` → `TypedExchange` boundary in
`adapter.rs` is the single place to touch when adding a new semantic:
keep the parser (`reader.rs`) pure and the adapter as the only site
that assembles cross-document state.

## License posture

USDA LCA Commons data is CC0 1.0 Universal (public-domain dedication,
mandatory at submission per USDA-NAL policy). No attribution plumbing
is required and none is built in. See
`arko/docs/licenses/usda-lca-commons.md` for the full primary-source
analysis.
