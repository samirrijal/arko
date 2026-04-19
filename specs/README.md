# Arko Specifications

This directory holds the **public specifications** that govern Arko. Everything
here is versioned, reviewable, and licensed CC-BY-4.0. The rest of the codebase
is an implementation of these documents.

| Spec                             | Version | Status                       | Summary                                                    |
| -------------------------------- | ------- | ---------------------------- | ---------------------------------------------------------- |
| [calc/v0.1.md](calc/v0.1.md)     | v0.1    | Draft (pre-ratification)     | The math contract: matrices, solver, determinism, license. |
| [sync/](sync/)                   | —       | Not started                  | Real-time collaboration state machine (Yjs + TLA+).        |
| [license/v0.1.md](license/v0.1.md) | v0.1  | Draft (pre-ratification)     | License-tier propagation policy language.                  |
| [api/](api/)                     | —       | Not started                  | Public HTTP/JSON API surface.                              |

## Ratification process

A specification moves from **Draft** to **Ratified** only when:

1. At least two independent implementations exist and pass the conformance
   suite at the declared level.
2. The conformance suite meets the coverage bar declared in the spec itself.
3. A public review period of at least 30 days has elapsed with no unresolved
   blocking issues.

Ratified specs become **immutable** at their version. Changes ship as a new
version (v0.2, v0.3, v1.0).

## Why spec-first

LCA has a trust problem. Every incumbent tool is a black box; every result
"from" SimaPro or OpenLCA is trusted because the brand is trusted, not because
anyone can verify it. Arko's bet is that publishing the math, the edge cases,
and the test vectors — and inviting anyone to reimplement — is a stronger
credibility position than any marketing claim.

See commitment **C3** in the root `README.md`.
