# Leiden CML-IA characterisation factors — license analysis (primary-source reading)

**Status:** characterised, 2026-04-21. Primary-source anchor for any
Arko public claim about CML-IA compatibility. Not legal advice.

## Tl;dr

Leiden CML's **CML-IA** characterisation-factor spreadsheet
(`CML-IA_aug_2016.xls`, version 4.8, August 2016) is published
gratis on Leiden University's CML server with **no explicit
license terms** — only a usage disclaimer and a suggested citation
form. Structurally weaker than JRC EF (mixed CC-BY infrastructure +
restrictive Sphera/ecoinvent EULAs, all explicit) and weaker than
USDA LCA Commons (CC0 1.0 explicit). This is a de facto "gratis
with citation expected" posture, not a license grant.

For Arko V1, the rationale for shipping CML-IA-derived CFs in an
open-source preset is defensible on the basis that: (a) CFs are
factual data points, (b) Arko's selection and arrangement differs
from Leiden's spreadsheet, (c) Leiden publishes the spreadsheet
gratis with no stated redistribution prohibition, and (d) Arko
preserves attribution prominently in source comments and this
license document. For commercial production use at scale, direct
outreach to the CML group at Leiden for an explicit written grant
is the conservative move and is parked as a Phase 2-3 task.

| Thing                                  | Published by               | License posture                                       |
| -------------------------------------- | -------------------------- | ----------------------------------------------------- |
| **CML-IA characterisation factors v4.8** — Excel spreadsheet (`CML-IA_aug_2016.xls`) | Institute of Environmental Sciences (CML), Leiden University | **No explicit license.** Page-level: "free of charge" download. Spreadsheet-level: usage disclaimer + suggested citation. No statement on commercial use, redistribution, or attribution language. |

## Sources read

All read 2026-04-21. Spreadsheet saved locally for re-reference.

- [Leiden CML — CML-IA Characterisation Factors landing][cml-landing]
  — institute page; states the spreadsheet is downloadable free of
  charge but contains no explicit license terms.
- [Leiden CML — historical download index][cml-charac] — older
  download index page on the `web.universiteitleiden.nl/cml/`
  subdomain; references the spreadsheet via relative link.
- **The spreadsheet itself** (`CML-IA_aug_2016.xls`, sheet
  `Description`, 71 rows) — the only authoritative document
  bundled with the data. Contains version metadata, suggested
  citation form, disclaimer. **No license clause.**

[cml-landing]: https://www.universiteitleiden.nl/en/research/research-output/science/cml-ia-characterisation-factors
[cml-charac]: https://web.universiteitleiden.nl/cml/ssp/projects/lca2/charac.html

Direct download URL (verified 2026-04-21):
`http://www.leidenuniv.nl/cml/ssp/databases/cmlia/cmlia.zip`
(18.7 MB ZIP containing the main spreadsheet plus a short
update-info workbook).

## Version metadata — verbatim from `Description` sheet

```
Database:        CML-IA
Version:         4.8
Date:            august 2016
previous update: january 2016
```

The spreadsheet's suggested citation row was **not updated** when
Leiden bumped 4.5 → 4.8 — it still reads:

> *"Oers, L. van, 2015. CML-IA database, characterisation and
> normalisation factors for midpoint impact category indicators.
> Version 4.5, april 2015. downloaded from
> http://www.cml.leiden.edu/software/data-cmlia.html"*

For Arko's purposes (we use v4.8 data, not v4.5), the operative
citation form is:

> *Oers, L. van. CML-IA database, characterisation and normalisation
> factors for midpoint impact category indicators. Version 4.8,
> August 2016. Institute of Environmental Sciences (CML), Leiden
> University. Downloaded 2026-04-21 from
> http://www.leidenuniv.nl/cml/ssp/databases/cmlia/cmlia.zip*

## The disclaimer — verbatim

From the `Description` sheet (rows 31-36):

> *"DISCLAIMER. The spreadsheet has been subjected to internal and
> external review. Nevertheless, this does not guarantee that the
> contents are error-free. CML and sub-contractors cannot be held
> responsible for possible errors and abuse of the data provided,
> neither for the results of applying these data in case-studies.
> Note that parts of this spreadsheet may need regular updating."*

This caveat travels with the data. Arko inherits it: the CML-IA
preset ships AS-IS, without warranty, and downstream users are
responsible for verifying applicability to their case studies.

## What is *missing* from the source

There is **no statement** in the spreadsheet or on either Leiden
landing page covering:

- Commercial use (permitted? prohibited? silent)
- Redistribution of the data in derived works (permitted?
  conditional? silent)
- Required attribution language (citation is *suggested*, not
  legally required as a condition of use)
- Term expiry (none stated, none implied — the data is published
  as a static deliverable)
- Patent / trademark carve-outs (none stated)
- Choice of law / jurisdiction (none stated)

The closest thing to a permissive grant is the page-level "can be
downloaded free of charge" framing on the institute landing page.
Free download is not, by itself, a license grant — it is a default
posture that becomes a license grant only when the publisher's
terms make that explicit.

## Arko's rationale for shipping CML-IA-derived CFs

Four things support shipping a CML-IA-derived preset in Arko V1:

1. **CFs are factual data points.** A CF for SO2-to-air under the
   Huijbregts 1999 average-Europe acidification model is a
   number — `1.2` kg SO2-eq per kg SO2. Numbers per se have low
   copyrightability under both US (Feist v. Rural) and EU (Database
   Directive recital 17) standards. The creative-expression layer
   sits at the *selection and arrangement* of the CF table, not at
   the values themselves.

2. **Arko's selection and arrangement differs from Leiden's.**
   The spreadsheet ships ~15 baseline categories across ~1,961
   substance rows × 111 columns, organised as one giant table.
   Arko's V1 preset extracts ~7 categories across ~50 substances,
   organised per-category in Rust source code with explicit per-
   factor source comments. This is a meaningfully different
   compilation, derived from but not equivalent to the source.

3. **Leiden publishes gratis with no stated prohibition.** The
   spreadsheet is downloadable from Leiden's institutional server
   with no fee, no registration, no click-through, and no
   redistribution clause. The sui-generis EU Database Right (15-year
   term protecting "substantial portions" of database content) is
   the strongest theoretical concern; Arko's selective extraction
   of factual data points across multiple categories, with full
   attribution to Leiden and the Oers citation, is consistent with
   the kind of academic re-use the publisher's gratis posture
   appears to invite.

4. **Arko preserves attribution prominently.** Each factor in the
   `arko-methods` CML-IA preset carries a per-factor source comment
   citing the Leiden spreadsheet by file, sheet, column, model
   variant, and substance row. The preset's module-level doc
   reproduces the Leiden citation form. This document is committed
   alongside the preset.

The combination is consistent with the academic-and-commercial
re-use norm Leiden's gratis posture appears to invite, while
remaining honest about the structural weakness of having no
explicit license grant to point at.

## Arko's risk-disclosure obligations

For Arko *as an open-source library shipping CFs in an MIT/Apache
preset*, the gratis-no-explicit-license posture is acceptable — the
risk surface is low and consistent with how dozens of LCA tools
handle CML-IA data today.

For **commercial production use at scale** (Arko offered as a paid
hosted product, Arko CFs embedded in client-facing reports under
Arko branding, etc.), the conservative move is to obtain an
explicit written grant from the CML group. Specifically:

- A short email to the CML group explaining Arko's use of CML-IA
  CFs, offering the attribution language used in the preset, and
  asking whether they would grant an explicit redistribution
  permission. Most academic groups respond positively to this kind
  of request, and having an explicit grant documented strengthens
  Arko's legal posture meaningfully.
- This is **not a Phase 1 blocker**. It is a Phase 2-3 task on the
  parked-items list, to be done before Arko's first paid hosted-data
  customer or before Arko CFs are used to author a commercial EPD
  that names Arko as the calculation tool.

## What Arko can do

- **Ship the CML-IA V1 preset** with extracted CFs in `arko-methods`,
  attributed to Leiden CML and to Oers per the citation form above.
- **Compute impact scores** using the preset — the CFs are factual
  data and the computation is the user's own work.
- **Cite the preset by name** in release notes and documentation:
  *"CML-IA baseline (Leiden CML, v4.8, August 2016) — EN 15804+A2-
  aligned subset shipped in V1."*
- **Distribute Arko-formatted derivatives** (cached registries,
  serialised method bundles, fixtures derived from the preset) —
  consistent with the gratis posture and the lack of redistribution
  restriction.
- **Run parity smokes** comparing Arko's CML-IA preset against
  third-party CML implementations and publish the parity numbers,
  citing Leiden as the upstream factor source.

## What Arko should not do (without explicit Leiden permission)

- **Imply Leiden CML endorsement.** *"CML-IA characterisation
  factors as published by Leiden CML"* is fine. *"Endorsed by
  Leiden CML"* / *"CML-certified"* is not.
- **Charge a per-CF or per-method licence fee** keyed to the CML-IA
  factor table. Arko-side computation, hosting, support, etc. are
  Arko's own work and may be charged for; the CML-IA factors
  themselves are gratis upstream and Arko should not represent them
  as licensed by Arko to downstream users.
- **Strip or obscure the per-factor attribution comments** in the
  preset source. Removing them would be a discipline-breach against
  this license document and against the factor-table-entry
  discipline memory.
- **Represent the data as warranted** — the disclaimer above
  travels with the CFs. Arko's own ToS must make clear that
  CML-IA-derived results are provided AS-IS and may need updating
  as Leiden revises the upstream spreadsheet.

## Comparison to JRC EF and USDA LCA Commons

| Axis                   | JRC EF reference package           | USDA LCA Commons         | **Leiden CML-IA**            |
| ---------------------- | ---------------------------------- | ------------------------ | ---------------------------- |
| License                | EC Decision 2011/833/EU (CC-BY equiv.) | CC0 1.0 Universal      | **None stated**              |
| Attribution required?  | Yes (legal)                        | No (encouraged)          | None stated; **moral norm**  |
| Commercial use?        | Yes                                | Yes (explicit)           | **Silent**                   |
| Redistribution?        | Yes with attribution               | Yes, unrestricted        | **Silent**                   |
| Term expiry?           | None for reference; 2025/2030 for LCI | None                  | None stated                  |
| Trademark carve-out?   | None stated                        | USDA/ARS/NAL names — yes | None stated                  |
| Disclaimer / AS-IS?    | Yes                                | Yes (Appendix A)         | **Yes** (rows 31-36)         |
| Data scope             | Flows, LCIA methods, CFs (infrastructure only) | Unit processes + product systems (agricultural LCI) | LCIA characterisation factors only |
| Arko-ship posture      | Fine (with attribution)            | Fine end-to-end          | **Fine for V1; commercial-scale needs explicit grant** |

## Implications for Arko

**For Phase 1 method preset coverage:** CML-IA V1 ships as the
fourth method preset (after IPCC AR6 GWP100, IPCC AR5 GWP100, and
EF 3.1 V1), bringing `MethodRegistry::standard()` to 4/4 — the
Phase 1 method preset exit criterion.

**For the `arko-license` crate's preset list (future work):** add
a preset capturing the gratis-with-disclaimer-no-explicit-license
posture distinct from CC0 (no rights reserved) and CC-BY
(attribution required as legal condition). Suggested name:
`cml_ia_leiden_gratis`. Captures: attribution as moral norm not
legal condition, AS-IS warranty disclaimer travels with data,
commercial use silent (default permissive in absence of stated
prohibition), redistribution silent (likewise).

**For per-factor source comments in the preset:** every CF in the
`arko-methods` CML-IA preset carries a comment of the form:

```
// Leiden CML, CML-IA_aug_2016.xls, sheet "characterisation factors",
// col <N> (<model variant verbatim from row 3>), row "<substance>"
```

For GWP100 specifically, the comment also notes the without-feedback
convention so future contributors don't "fix" the divergence from
Arko's existing AR5 preset (which uses with-feedback values):

```
// Note: differs from Arko's ipcc-ar5-gwp100 (with feedback) by design;
// CML-IA convention is AR5 without climate-carbon feedback values.
```

**For public-facing copy:** safe to say *"Arko ships the CML-IA
baseline (Leiden CML, v4.8) — EN 15804+A2-aligned subset of seven
core indicators."* Safe to say *"CML-IA characterisation factors
sourced from Leiden University CML (Oers, 2016)."* Not safe to
say *"Arko is endorsed by Leiden CML"* or *"CML-certified"*.

## Open items

- **Outreach to Leiden CML for explicit grant.** Phase 2-3 task
  before commercial production deployment at scale. Draft the
  outreach email when Arko has clearer product shape.
- **`arko-license` `cml_ia_leiden_gratis` preset.** Add when the
  license-preset crate work begins.
- **Monitor Leiden CML for newer CML-IA versions.** v4.8 has been
  the latest since August 2016 and the spreadsheet itself notes
  "parts of this spreadsheet may need regular updating." If a v4.9
  or v5.0 ships, Arko's preset version key bumps accordingly.
- **EU sui generis Database Right** — the only theoretical concern
  not addressed by Arko's rationale above. A formal opinion on this
  is overkill for V1 but worth getting before any aggressive
  commercial positioning.
