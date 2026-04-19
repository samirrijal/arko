# License-tier specification

- **Current version:** [v0.1](v0.1.md) (Draft, 2026-04-19)
- **Status:** Reference implementation in `engine/license/` targets
  this draft. Pre-ratification — breaking changes may be introduced
  in any minor version until v1.0.

This sub-specification defines the **license-tier policy language**
of an Arko engine: how per-process restrictions propagate, how
derivative rules fire, and how the combined `Authorization`
decision for a publish/share/export attempt is computed.

The storage types themselves (`LicenseTier`, `DerivativeRule`,
`EffectiveRestriction`) are normative in
[`../calc/v0.1.md`](../calc/v0.1.md) §11; this document defines the
*semantics* layered on top.

## Guiding principle

**The solve is always legal.** License enforcement gates
*publication and export* of results, not computation. This keeps the
engine simple and places enforcement at the natural business-logic
boundary (the publish endpoint).

## Coverage at v0.1

- §3 — Guiding principle (separation of solve and publish).
- §4 — Intent enumeration (`publish`, `share`, `export`).
- §5 — Three-signal authorization decision (base flag,
  derivative rules, expiry).
- §6 — Derivative-rule trigger semantics
  (`ScalingGe { threshold }`, `Always`).
- §7 — Action → outcome mapping.
- §8 — Audit-log requirements.
- §9 — Standard library presets (ecoinvent, Sphera, CC-BY,
  custom-user).
- §10 — Conformance.
- §11 — Determinism contract.
- §12 — Error handling.
- §13 — Forward compatibility.

## Roadmap

Targeted for **v0.2**:

- Cryptographically-signed tiers (`LicenseTier.signature`).
- Per-impact-category restrictions.
- Time-windowed (becomes-restrictive) tiers.
- Operator-attestation persistence format.

Targeted for **v1.0** ratification (per §1):

1. The presets of §9 verified against actual EULA text by counsel.
2. At least one independent implementation passes the conformance
   vectors of §10.
3. Audit-log requirements validated against a real-world
   reporting workflow.
