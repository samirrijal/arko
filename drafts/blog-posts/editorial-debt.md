# Editorial debt in federated LCA datasets

**Status:** seed draft — 2026-04-19
**Audience:** LCA practitioners; software vendors working with ILCD-
family data; the LCA academic community.

## Core observation

Phase 1 Week 5 of building Arko gave us two reader-level smoke results
on JRC-family ILCD data, back-to-back:

- `arko-io-ilcd` on ÖKOBAUDAT 2024-I — 3,075 processes, **~3.4%
  publisher-side cross-reference gaps** (flow UUIDs referenced by
  processes but absent from the bundle, flow-properties pointing at
  unit-groups not shipped, etc.). Gaps were classified as publisher
  data debt, not reader bugs. Reader was clean.
- `arko-io-ilcd-linker` resolver-only smoke on the JRC EF 3.1 reference
  package — 94,062 elementary-flow XMLs, **0% gaps, 0 engine
  failures**. Every flow's reference-unit chain resolved.

Same reader. Same format (ILCD). Same region (European). ~30× scale
difference. Two orders of magnitude difference in cross-reference
cleanliness.

The gap isn't a quality difference between the organizations — it's a
**structural difference in editorial model**.

## The structural argument

JRC's EF reference vocabulary is centrally curated by a single
editorial team with strict referential discipline. One team owns every
flow, every flow-property, every unit-group. Changes propagate through
one queue. Referential integrity is enforced by process.

ÖKOBAUDAT's 2,000+ entries over 15 years are federated: multiple
contributors, evolving conventions, legacy entries that reference flow
UUIDs no longer present in current exports. The content is high-quality
— German EPD data is respected for a reason — but the **graph** it
forms accumulates referential debt the way any federated dataset does.

This isn't a ÖKOBAUDAT problem. It's a federated-dataset problem. Every
LCA dataset with distributed contribution over time shows the same
pattern: broken flow references, deprecated unit groups, UUID drift as
flow concepts split or merge across publisher revisions. Ecoinvent's
change logs are full of this. ILCD Network Nodes see it. EPD
International has structural workarounds for it.

## What this means for practitioners

*(open — needs concrete examples)*

- When a calculation tool reports "flow X not found" on a published
  EPD, the bug is often in the **dataset** not the tool.
- Version-pinning the source bundle is as important as version-pinning
  the method.
- Asking a tool to accept a broken cross-reference silently is how
  dataset drift becomes silent calculation drift.

## What this means for software vendors

*(open — needs positioning)*

- Treating publisher gaps as **typed, distinct error variants** (not
  lumped into generic "parse failed") is a quality-of-life signal.
  Arko's `LinkError::Io { .. }` vs `LinkError::FlowHasNoUnitDerivation`
  vs other engine-level errors is a pattern worth naming as an
  approach.
- Surfacing publisher gaps as **first-class observational data** (the
  CHANGELOG characterisation entries are the prototype of this) lets
  downstream users make informed choices about bundle acceptance.

## What LCA datasets could learn from version-pinned software ecosystems

*(open — this is the section that could make the post interesting)*

- Cargo / npm / pip handle similar referential-integrity problems at
  scale through lockfiles, content-addressed packages, and strict
  semver.
- The ILCD world has UUIDs (content-addressing in spirit) but no
  equivalent of a lockfile — the bundle is the lockfile, which means
  any inconsistency in the bundle *is* the locked state.
- A "lockfile-shaped" ILCD bundle spec — one where referential
  integrity is a validated invariant of the bundle format, not a
  publisher discipline question — is an interesting research direction.

## Open questions this draft needs to answer before publishing

- Concrete numbers: what does "3.4% gaps" actually decompose into?
  Which gap categories dominate? (Pull from CHANGELOG + actual test
  output.)
- Is the 0% on JRC EF 3.1 reference-package a selection artifact (it's
  infrastructure-only, no processes — processes are where most gaps
  live) or a genuine editorial-discipline signal? Both are true;
  figure out how much to weight each.
- Case study: one specific ÖKOBAUDAT process where the gap cost a
  practitioner something real.
- Tone: technical essay, provocation, or practitioner-friendly
  explainer? All three work; pick one.

## Why this belongs on the Arko blog (when finished)

- Positions Arko as software that **observes its data** rather than
  just consuming it — the CHANGELOG discipline is the proof.
- Gives Arko something to say about the LCA ecosystem that isn't "we
  also have a calculation engine"; this is a systemic observation with
  novel framing.
- Invites practitioner conversation (and potential feedback loops from
  dataset publishers) in a constructive register.

## Not urgent

This belongs to Phase 4's content push. Shipping earlier is fine if
the moment arises (conference invitation, ecosystem discussion, a
publisher reaches out), but writing without a reason tends to produce
writing without an audience.
