# JRC EF data — license analysis (primary-source reading)

**Status:** characterised, 2026-04-19. Primary-source anchor for any
Arko public claim about EF compatibility. Not legal advice.

## Tl;dr

"JRC EF data" is two different things under two different licenses.
Anyone writing Arko-public copy about "EF support" must name which
of the two they mean.

| Thing                                | Published by                      | License posture                                                     |
| ------------------------------------ | --------------------------------- | ------------------------------------------------------------------- |
| **EF reference package (infrastructure)** — flows, flow properties, unit groups, LCIA methods, characterisation factors | JRC / European Commission         | CC-BY 4.0 equivalent under EC Decision 2011/833/EU (attribution).   |
| **EF 3.1 LCI datasets (background processes)** — the ~20 000-process inventory bundle hosted at the EF node | Sphera, ecoinvent, CEPE, FEFAC    | Restrictive per-licensor EULAs. **Not open data.** Term-expired for general LCA use 2025-12-31. |

Phase 1 smoke testing touched **both**. The reference package is
safely under open reuse. The background-processes bundle is not, and
the smoke-test posture of "maintainer-downloaded, no redistribution,
no claim" is load-bearing.

## Sources read

All read 2026-04-19. Saved locally for re-reference, not redistributed.

- [Sphera EULA for EF 3.1 datasets, June 2024][sphera-eula] —
  covers the bulk of the background-processes bundle (Energy,
  Transport, Materials, etc.).
- [ecoinvent Association EULA for EF 3.1 Chemicals Part 1][eco-eula] —
  covers the Chemicals Part 1 data lot specifically.
- [EPLCA Developer EF reference package landing][dev-ef] — lists
  which data lots fall under which EULA (Sphera vs ecoinvent vs CEPE
  vs FEFAC vs JRC).
- [EPLCA EF node (Sphera-hosted)][ef-node] — the actual node the
  background processes come from.

CEPE (chemical manufacturers) and FEFAC (European feed) EULAs
not yet read in depth; same posture assumed until read. Their data
lots are narrower and unlikely to be on the critical Arko path
in Phase 1.

[sphera-eula]: https://lcdn.thinkstep.com/static/EULA_07%20June%202024_(Sphera%20-%20Final).pdf
[eco-eula]: https://19913970.fs1.hubspotusercontent-na1.net/hubfs/19913970/Knowledge%20Base/Projects/EULA_EF3.1data_ChemicalsPart1_ecoinvent.pdf
[dev-ef]: https://eplca.jrc.ec.europa.eu/LCDN/developerEF.xhtml
[ef-node]: http://lcdn.thinkstep.com/Node/index.xhtml?stock=EF3__official_data

## Side 1 — EF reference package (infrastructure)

**What it is:** The "ILCD infrastructure" bundle downloadable from
EPLCA's *Developer - Environmental Footprint* area. Flows, flow
properties, unit groups, sources, contacts, and LCIA method
definitions (characterisation factors for the 16 EF impact
categories). Zero process datasets by construction.

**Published by:** JRC, with author contacts (Cristina De Camillis
et al.) as JRC staff. This is Commission-produced material.

**License posture:** Under the Commission's Decision 2011/833/EU on
the reuse of Commission documents, JRC-published material is
reusable with attribution (CC-BY 4.0 equivalent) unless otherwise
indicated. The EF reference package itself does not bundle a
restrictive EULA; it ships as plain ILCD XML with the standard
Commission reuse disclaimer.

**What Arko can do:**

- Parse it, link against it, ship fixtures from it (with
  attribution) — fine.
- Include characterisation factors (GWP100 etc.) in published
  Arko method presets — fine, with attribution to *"European
  Commission, JRC, EF reference package 3.1 (2022)."*
- Cite it in release notes, blog posts, homepage copy — fine.

**Attribution text to use:**

> Characterisation factors: European Commission, Joint Research
> Centre, *Environmental Footprint reference package 3.1* (July
> 2022). eplca.jrc.ec.europa.eu/LCDN/developerEF.xhtml. Reused
> under Commission Decision 2011/833/EU.

**Caveats:**

- The EF method documentation PDFs (the methodology reports on JRC
  Publications Repository) are explicitly CC-BY 4.0 with JRC
  numbering (e.g. JRC130796). Cite them directly when asserting
  anything about *how* characterisation was derived.
- "Commission reuse" is not identical to CC-BY 4.0 in every
  jurisdiction; in practice the differences are minor, but a
  legal reviewer would want to see EC Decision 2011/833/EU
  cited, not "CC-BY" as shorthand.

## Side 2 — EF 3.1 LCI datasets (background processes)

**What it is:** The ~20 000-process inventory bundle hosted at the
EF node (`lcdn.thinkstep.com` and partner nodes). This is what
Arko's Phase 1 Week 5 single-process calc-correctness smoke pulled
the carpet process from.

**Published by:** **Not JRC.** The LCI datasets were procured from
commercial LCA database owners (Sphera, ecoinvent, CEPE, FEFAC)
under a tender the Commission ran; those owners retain copyright
and impose their own EULAs as a condition of redistribution by
the Commission's node infrastructure.

**License posture — the hard constraints:**

From the Sphera EULA (the largest data lot), verbatim:

> *"You are not allowed to use the DATASET for any other purpose
> than the PERMITTED USE as defined under this EULA, including for
> commercial, non-commercial or educational purposes."* (§6, "No
> Rental, no Hosting")

> *"Distribution, licensing, lease or selling of any original,
> modified or derived DATASETS against charges or fees is not
> permitted."* (§3)

> *"The license granted to 'Policy implementing End Users: other
> EU policies' is valid until 31 December 2030. […] for the
> 'PEF/OEF study End User' until 31st December 2025."* (§2, §11)

The ecoinvent EULA is stricter still — all permitted-use categories
expired 2025-12-31, requiring a new agreement with ecoinvent for
any use from 2026-01-01 onward.

**Permitted End User categories (Sphera):**

1. **PEF/OEF study End Users** — natural/legal persons carrying
   out a PEF or OEF study **in compliance with a valid
   PEFCR/OEFSR** developed by the Commission. Term: **expired
   2025-12-31.**
2. **Policy implementing End Users: Taxonomy & New Consumer
   Agenda** — natural/legal persons using datasets under EU
   Regulation 2020/852 or COM/2020/696. Term: **expired
   2025-12-31.**
3. **Policy implementing End Users: Other EU policies** — narrow
   list including batteries (EU 2023/1542), solar PV ecodesign
   (Directive 2009/125/EC), critical raw materials, ESPR,
   energy labelling (EU 2017/1369), EU Ecolabel, audiovisual
   carbon footprint calculator. Term: **expires 2030-12-31.**

**Arko is not in any of these three categories.** A general-purpose
LCA engine is not itself a PEFCR/OEFSR study, not a Taxonomy
implementation, and not a narrow EU-policy implementation.

**What Arko cannot do:**

- Cannot ship or bundle EF 3.1 LCI datasets (original, modified,
  or derived) with the Arko distribution. Prohibited.
- Cannot host EF 3.1 LCI datasets as part of a paid or hosted
  Arko service. §6 prohibits use for "commercial, non-commercial
  or educational purposes" outside the permitted-use list.
- Cannot publish EF 3.1 LCI dataset characterisation results
  (impact numbers tied to specific processes) in marketing copy
  or public-facing material without the permitted-use nexus.
- Cannot redistribute derived datasets unless free-of-charge
  **and** served from a SODA4LCA node listed in the EPLCA contact
  list. Neither condition is compatible with a commercial Arko
  SaaS.

**What Arko can do:**

- **Internal engineering smoke tests** on maintainer-downloaded
  copies: defensible under the tender's interpretation of
  "dataset/software developer" category, which Sphera permits
  for the purpose of enabling PEFCR/OEFSR studies. This is what
  the Phase 1 Week 5 carpet smoke did. The smoke-test code
  reads the bundle from an environment variable (never checked
  in), does not redistribute, and records results in an internal
  CHANGELOG — not marketing copy.
- **Cite the bundle as a format-compatibility reference** ("Arko
  parses ILCD XML, the format used by the EF node") — this is a
  statement about format support, not data redistribution or
  data reuse.
- **Execute PEFCR/OEFSR studies on behalf of an Arko customer who
  is themselves a PEF/OEF End User** — **only** if the customer has
  accepted the applicable EULA, the use is within permitted term
  bounds, and the execution path keeps the dataset out of Arko's
  redistributable surface. Probably a V2+ question after legal
  review.

**Attribution (if ever publishing results):** Sphera §3 requires
*verbatim*:

> *"Sphera Solutions GmbH (2019-2025): Official EF 3.1 secondary
> LCI datasets for EU Environmental Footprint (EF) implementation
> 2019-2025. <DATASET's complete Name>, <DATASET's Location>,
> <DATASET's Reference year>, <DATASET's UUID>, <DATASET's Version
> number>. http://lcdn.thinkstep.com. Date of download XXX."*

Plus the dataset citation "in a prominent and well visible place"
on any derived communication.

## Implications for Arko

**For Phase 1 evidence (internal):** the carpet calc smoke
(2026-04-19) on a Sphera-EULA'd process is defensible under
maintainer-download + no-redistribution + no-commercial-use
posture. The CHANGELOG entry for it does name the carpet process
UUID and the impact number — that's internal evidence, not a
marketing claim. Keep it framed that way.

**For Phase 1 exit criterion "three free databases importable":**
the EF reference *package* (infrastructure) is one. The EF 3.1
LCI *datasets* are **not** a free database in the Arko-ship sense,
because Arko cannot ship them and cannot let customers consume
them without each customer accepting the relevant EULA. If the
Phase 1 exit claim is "three free databases," the LCI side of EF
should not be one of the three. `D-0010` classifies "the JRC EF
reference package" as foreground-free; that claim is correct for
the infrastructure bundle and incorrect for the LCI bundle. A
future decision entry should refine this split explicitly.

**For public-facing copy (arko.earth homepage, blog posts, docs
site, release-note blog):** safe to say *"Arko parses the ILCD XML
format used by the EF reference package"* and *"Arko ships the
16 EF 3.1 impact-category characterisation factors as a method
preset, reused under EC Decision 2011/833/EU."* Not safe to say
*"Arko ships the EF 3.1 LCI datasets"*, *"Arko supports EF 3.1
out of the box"* (ambiguous — reads as LCI support to
practitioners), or *"Arko is an open EF 3.1 platform."*

**For the `arko-license` crate's preset list:** the existing
`presets::sphera_strict` naming is accurate for the posture the
Sphera EULA requires. Worth adding a preset named along the lines
of `ef_reference_package_lci` that encodes: no redistribution,
no commercial hosting, term-expiry check, PEFCR-nexus
verification. Parking for Phase 2 — not Phase 1 work.

## Open items

- **CEPE and FEFAC EULAs unread.** Lower priority because Arko's
  current smoke paths don't touch chemical-manufacturer or feed
  data lots, but should be read before any claim about EF LCI
  compatibility at all.
- **Commission Decision 2011/833/EU wording:** the phrase
  *"reuse allowed with attribution"* is my shorthand. Worth
  pulling the actual decision text and citing it directly on the
  docs site rather than the shorthand.
- **"Dataset/software developer or provider" clause in Sphera
  §3** is the narrowest category Arko plausibly fits; it requires
  the add-on be *solely for the purpose of enabling PEFCR/OEFSR
  studies* and be *"without additional charge or license/use fee."*
  A hosted Arko that charges a subscription and enables PEFCR
  studies as one of many use cases is not obviously inside this
  clause. Flag for legal review before any Arko-consumes-EF-LCI
  product path.
- **Term-expiry as of 2026-04-19:** ecoinvent Chemicals Part 1
  permitted use has expired; Sphera PEF/OEF study user permitted
  use has expired. Only Sphera "Other EU policies" runs until
  2030-12-31. Future smoke tests on the LCI bundle should note
  this: even the internal-engineering posture has weakened since
  2025-12-31.
