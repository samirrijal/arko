# RIVM ReCiPe 2016 characterisation factors — license analysis (primary-source reading)

**Status:** characterised, 2026-04-22. Primary-source anchor for any
Arko public claim about ReCiPe 2016 compatibility. Not legal advice.

## Tl;dr

RIVM's **ReCiPe 2016** characterisation-factor spreadsheets
(`ReCiPe2016_CFs_v1.1_20180117.xlsx` — the post-erratum canonical
release — and the companion `ReCiPe2016_country factors_v1.1_20171221.xlsx`)
are published gratis on rivm.nl with **no explicit license terms** for the
data files. The site-wide disclaimer covers image reuse and warranty
disclaimers but is silent on data redistribution. Structurally this is
the **same posture as Leiden CML-IA** — gratis with no stated grant —
and weaker than JRC EF (CC-BY-equivalent for infrastructure) or USDA
LCA Commons (CC0 1.0 explicit).

For Arko V1, the rationale for shipping ReCiPe-derived CFs in an
open-source preset is the same four-leg argument used for CML-IA:
(a) CFs are factual data points, (b) Arko's selection and arrangement
differs from RIVM's spreadsheet, (c) RIVM publishes the file gratis
with no stated redistribution prohibition, and (d) Arko preserves
attribution prominently in source comments and this license document.
Commercial-scale use should be backed by an explicit grant from RIVM
or the RIVM/Radboud ReCiPe team — parked as a Phase 2-3 task on the
same shelf as the CML-IA outreach.

| Thing | Published by | License posture |
| ----- | ------------ | --------------- |
| **ReCiPe 2016 main characterisation factors v1.1 (2018-01-17, post-erratum)** — `ReCiPe2016_CFs_v1.1_20180117.xlsx` | RIVM (Rijksinstituut voor Volksgezondheid en Milieu) | **No explicit license.** Page-level: free download, no fee, no registration. Spreadsheet-level: version + author metadata, no license clause. |
| **ReCiPe 2016 country-specific factors v1.1 (2017-12-21)** — `ReCiPe2016_country factors_v1.1_20171221.xlsx` | RIVM | Same posture as above. Companion file with country-resolved CFs for 5 of the 18 categories (POCP, PMFP, AP, EP-fw, water consumption). Not used in Arko V1 (GLO-only); reserved for V2 regionalisation bundle. |

## Sources read

All read 2026-04-22. Both spreadsheets saved locally for re-reference
under `arko/scratch/recipe2016/`.

- [RIVM — Life cycle assessment LCA downloads page][rivm-downloads]
  — institute landing page listing all ReCiPe 2016 downloadable assets
  (main CFs, country-specific factors, normalisation scores). Page
  itself states no license terms.
- [RIVM — Disclaimer and copyright][rivm-disclaimer] — site-wide
  copyright notice. Image-reuse and warranty clauses only; **no data
  reuse clause** for downloadable spreadsheets.
- **The main spreadsheet itself** (`ReCiPe2016_CFs_v1.1_20180117.xlsx`,
  21 sheets: Version + Midpoint-to-endpoint + 19 category sheets).
  `Version` sheet contains team roster (Mark Huijbregts, Rosalie van
  Zelm, Zoran Steinmann, et al., Radboud University Nijmegen) and
  publication date `2017-05-01` for the v1.1 series. **No license
  clause in the workbook.**
- **The country-specific spreadsheet** (`ReCiPe2016_country factors_v1.1_20171221.xlsx`,
  6 sheets: Version + 5 regionalised category sheets). Same Version
  metadata, no license clause.
- [Huijbregts et al. 2017 — *ReCiPe2016: a harmonised life cycle impact
  assessment method*][huijbregts-2017] (Int J LCA 22:138-147) — the
  academic paper accompanying the method. Cited as the methodological
  reference in Arko's preset.
- [RIVM Report 2016-0104][rivm-report-2016-0104] — *ReCiPe 2016: A
  harmonized life cycle impact assessment method at midpoint and
  endpoint level. Report 1: Characterization* — the institutional
  technical report. Cited alongside the Huijbregts paper.

[rivm-downloads]: https://www.rivm.nl/en/life-cycle-assessment-lca/downloads
[rivm-disclaimer]: https://www.rivm.nl/en/disclaimer-and-copyright
[huijbregts-2017]: https://link.springer.com/article/10.1007/s11367-016-1246-y
[rivm-report-2016-0104]: https://www.rivm.nl/bibliotheek/rapporten/2016-0104.pdf

Direct download URLs (verified 2026-04-22):

```
https://www.rivm.nl/sites/default/files/2024-10/ReCiPe2016_CFs_v1.1_20180117.xlsx
  (6.27 MB)  sha256: 7024ad5faf5916d15ec106224749b3397c3e3f9f917e14fbb7ec5a433cdc7387

https://www.rivm.nl/sites/default/files/2024-10/ReCiPe2016_country%20factors_v1.1_20171221.xlsx
  (113.7 KB) sha256: 4806afbdcbb7f184e35263a136e5043ce3fa60f01faa9eeffab5803b3d6f9d43
```

## Version metadata — verbatim from `Version` sheets

Both files report:

```
ReCiPe 2016 v1.1
Date: 2017-05-01

Team (Radboud University Nijmegen, except where noted):
  Mark Huijbregts
  Rosalie van Zelm
  Zoran Steinmann
  ... (full roster on Version sheet)
```

The `_20180117` suffix on the main file's name marks the
post-erratum re-issue: an October 2017 v1.1 release was followed by
the January 2018 erratum that corrected several CF values. **The
2018-01-17 file is the canonical v1.1 file** — this is what the
brightway `bw_recipe_2016` port uses, what Arko's preset cites in
per-factor comments, and what any parity work against third-party
ReCiPe implementations should compare against. Earlier dated v1.0
(2016-10-04) and v1.1 (2017-09-29) files exist but are superseded.

## The disclaimer — verbatim from rivm.nl

From `https://www.rivm.nl/en/disclaimer-and-copyright`:

> *"All proprietary and intellectual property rights belong to RIVM,
> its commissioning client(s) or its licensor(s)."*

> *"Images on the RIVM website may not be reused without consent."*

> *"Images on the RIVM website may not be used for commercial
> purposes."*

> *"RIVM accepts no liability in any way for direct or indirect
> damages caused by – or in any way linked to – the use of this
> website or the information provided."*

> *"this does not constitute a warranty on the part of RIVM that the
> information provided is accurate, comprehensive or up to date."*

The image-reuse clauses do not apply to data downloads. The copyright
ownership and warranty-disclaimer clauses do.

For Arko's purposes the operative citation form, mirroring the
Huijbregts paper and adding the spreadsheet release:

> *Huijbregts, M.A.J., Steinmann, Z.J.N., Elshout, P.M.F., Stam, G.,
> Verones, F., Vieira, M., Zijp, M., Hollander, A., van Zelm, R.
> (2017). ReCiPe2016: a harmonised life cycle impact assessment
> method at midpoint and endpoint level. The International Journal
> of Life Cycle Assessment 22, 138-147.
> Characterisation factors v1.1 (2018-01-17 erratum), downloaded
> 2026-04-22 from
> https://www.rivm.nl/sites/default/files/2024-10/ReCiPe2016_CFs_v1.1_20180117.xlsx*

## What is *missing* from the source

There is **no statement** in either spreadsheet or on the rivm.nl
download landing page covering:

- Commercial use (permitted? prohibited? silent)
- Redistribution of the data in derived works (permitted?
  conditional? silent)
- Required attribution language (citation is *suggested* via the
  Huijbregts paper, not legally required as a condition of use)
- Term expiry (none stated)
- Trademark carve-outs (the names "ReCiPe", "RIVM", and "Radboud
  University" appear without explicit trademark notice)
- Choice of law / jurisdiction (none stated; Dutch law would apply
  by default if challenged in NL courts)

The "free download" + "no stated prohibition" posture is the same
default-permissive shape as Leiden CML-IA. This is a posture, not a
license grant, and remains structurally weaker than the explicit
CC-BY / CC0 of JRC EF and USDA LCA Commons respectively.

## Arko's rationale for shipping ReCiPe-derived CFs

The same four-leg argument used for the CML-IA preset applies:

1. **CFs are factual data points.** A ReCiPe Hierarchist GWP100 CF
   for CH4-fossil under the underlying AR5-with-feedback convention
   is `34` kg CO2-eq per kg CH4. Numbers per se have low
   copyrightability under both US (Feist v. Rural) and EU (Database
   Directive recital 17) standards. The creative-expression layer
   sits at the *selection and arrangement* of the CF table, not at
   the values themselves.

2. **Arko's selection and arrangement differs from RIVM's.** The
   spreadsheet ships 18 midpoint categories across ~150,000+
   substance rows (the ecotoxicity sheets alone account for the
   bulk). Arko's V1 preset extracts 10 categories with a much
   smaller substance footprint, organised per-category in Rust
   source code with explicit per-factor source comments. This is
   a meaningfully different compilation, derived from but not
   equivalent to the source.

3. **RIVM publishes gratis with no stated prohibition.** Both
   spreadsheets are downloadable from rivm.nl with no fee, no
   registration, no click-through, and no redistribution clause.
   The sui-generis EU Database Right (15-year term protecting
   "substantial portions" of database content) is the strongest
   theoretical concern; Arko's selective extraction of factual data
   points across 10 categories — well under "substantial" by any
   reasonable threshold — with full attribution to RIVM and the
   Huijbregts paper, is consistent with the kind of academic re-use
   the publisher's gratis posture appears to invite.

4. **Arko preserves attribution prominently.** Each factor in the
   `arko-methods` ReCiPe preset carries a per-factor source comment
   citing the RIVM spreadsheet by file, sheet, column (Hierarchist
   variant), and substance row. The preset's module-level doc
   reproduces the Huijbregts citation form. This document is
   committed alongside the preset.

The combination is consistent with the academic-and-commercial
re-use norm RIVM's gratis posture appears to invite, while remaining
honest about the structural weakness of having no explicit license
grant.

## Arko's risk-disclosure obligations

Same as the CML-IA shelf:

- **Open-source library shipping CFs in MIT/Apache preset:** gratis-
  no-explicit-license posture is acceptable. Risk surface is low and
  consistent with how brightway, openLCA, SimaPro, and dozens of
  other LCA tools handle ReCiPe data today.

- **Commercial production use at scale** (Arko offered as paid
  hosted product, ReCiPe results embedded in client-facing reports
  under Arko branding): the conservative move is an explicit written
  grant from the RIVM ReCiPe team, ideally co-signed by the lead
  authors at Radboud. Drafted as a Phase 2-3 task alongside the
  CML-IA outreach, not a Phase 1 blocker.

## What Arko can do

- **Ship the ReCiPe 2016 V1 preset** with extracted Hierarchist CFs
  in `arko-methods`, attributed to RIVM and the Huijbregts 2017
  paper.
- **Compute impact scores** using the preset — the CFs are factual
  data and the computation is the user's own work.
- **Cite the preset by name** in release notes and documentation:
  *"ReCiPe 2016 Midpoint Hierarchist (RIVM, v1.1 post-erratum
  2018-01-17) — V1 ships 10 categories spanning EN 15804+A2 alignment
  plus three ReCiPe-distinctive midpoints (particulate matter, land
  occupation, water consumption). GLO-only; regionalised CFs deferred
  to V2."*
- **Distribute Arko-formatted derivatives** (cached registries,
  serialised method bundles, fixtures derived from the preset).
- **Run parity smokes** comparing Arko's ReCiPe preset against
  brightway's `bw_recipe_2016` port and publish the parity numbers,
  citing RIVM as the upstream factor source.

## What Arko should not do (without explicit RIVM permission)

- **Imply RIVM endorsement.** *"ReCiPe 2016 characterisation factors
  as published by RIVM"* is fine. *"Endorsed by RIVM"* or
  *"ReCiPe-certified"* is not.
- **Charge a per-CF or per-method licence fee** keyed to the ReCiPe
  factor table. Arko-side computation, hosting, support, etc. are
  Arko's own work and may be charged for; the ReCiPe factors
  themselves are gratis upstream.
- **Strip or obscure the per-factor attribution comments** in the
  preset source. Removing them would be a discipline-breach against
  this license document and against the factor-table-entry discipline
  memory.
- **Represent the data as warranted** — the rivm.nl warranty
  disclaimer travels with the CFs. Arko's own ToS must make clear
  that ReCiPe-derived results are provided AS-IS.

## Comparison to JRC EF, USDA LCA Commons, and Leiden CML-IA

| Axis | JRC EF reference package | USDA LCA Commons | Leiden CML-IA | **RIVM ReCiPe 2016** |
| ---- | ------------------------ | ---------------- | ------------- | -------------------- |
| License | EC Decision 2011/833/EU (CC-BY equiv.) | CC0 1.0 Universal | None stated | **None stated** |
| Attribution required? | Yes (legal) | No (encouraged) | None stated; moral norm | None stated; **moral norm** |
| Commercial use? | Yes | Yes (explicit) | Silent | **Silent** |
| Redistribution? | Yes with attribution | Yes, unrestricted | Silent | **Silent** |
| Term expiry? | None for reference; 2025/2030 for LCI | None | None stated | None stated |
| Trademark carve-out? | None stated | USDA/ARS/NAL — yes | None stated | None stated |
| Disclaimer / AS-IS? | Yes | Yes (Appendix A) | Yes (rows 31-36) | **Yes** (rivm.nl disclaimer page) |
| Data scope | Flows, LCIA methods, CFs (infra only) | Unit processes + product systems | LCIA CFs only | **LCIA CFs only** (Midpoint + Endpoint + Normalisation) |
| Arko-ship posture | Fine (with attribution) | Fine end-to-end | Fine for V1; commercial-scale needs explicit grant | **Fine for V1; commercial-scale needs explicit grant** |

ReCiPe and CML-IA share the gratis-no-explicit-license posture. The
risk frame and Arko-ship rationale are consequently the same shape.

## V2 region-vocabulary verification — captured in scoping pass

Done while the country-specific xlsx was open to avoid a return trip
in the V2 implementation session. The companion file
`ReCiPe2016_country factors_v1.1_20171221.xlsx` was inspected: it
contains 5 category sheets (POCP, PMFP, AP, EP-fw, water consumption)
and the region encoding **is not uniform across categories**:

- **POCP and PMFP:** "Source region" + "Continent" columns; values
  are atmospheric-model source-region groupings (e.g., *"Austria,
  Slovenia, Liechtenstein, Czechia, Hungary"*) — **not ISO codes**;
  these correspond to TM5 / atmospheric-transport-model regions.
- **Terrestrial acidification, freshwater eutrophication, water
  consumption:** single "Country" column; values are **English
  country names** (e.g., *"Afghanistan"*, *"Albania"*) — also not
  ISO codes.

V2 implication: the regionalisation bundle (`CasRegion` matcher +
per-process `C` build + Eq. 3 restructure) must accommodate **two
parallel region vocabularies**, not one. Either Arko normalises both
to a unified internal region key (likely ISO 3166-1 alpha-3 for
country-resolved categories plus a synthetic key set for atmospheric
source regions), or the matcher carries a vocabulary tag and matches
against the appropriate region encoding per category. Decision
deferred to V2 scope.

## Implications for Arko

**For Phase 1 method preset coverage:** ReCiPe 2016 V1 is the fifth
preset registered (after AR6, AR5, EF 3.1, CML-IA baseline) and the
fourth named-slate method, satisfying the 4-of-4 named-slate Phase 1
exit criterion (AR6, EF 3.1, CML 2001 via CML-IA baseline, ReCiPe
2016).

**For the `arko-license` crate's preset list (future work):** the
`cml_ia_leiden_gratis` preset proposed for CML-IA also fits ReCiPe.
Either share the preset or add a parallel `recipe_2016_rivm_gratis`
with identical posture flags — to be decided when the license-preset
crate work begins.

**For per-factor source comments in the preset:** every CF in the
`arko-methods` ReCiPe preset carries a comment of the form:

```
// RIVM ReCiPe 2016, ReCiPe2016_CFs_v1.1_20180117.xlsx, sheet
// "<sheet name>", col <N> (Hierarchist), row "<substance>"
```

For the 5 regionalised-in-source categories (POCP-h, PMFP, AP,
EP-fw, water consumption), the V1 GLO-default values from the **main
xlsx** are used. Each carries an additional inline note:

```
// GLO default; country-specific CFs from companion xlsx
// (ReCiPe2016_country factors_v1.1_20171221.xlsx) deferred to V2
// regionalisation bundle.
```

**For public-facing copy:** safe to say *"Arko ships ReCiPe 2016
Midpoint Hierarchist (RIVM, v1.1) — 10 categories spanning EN
15804+A2-aligned core indicators plus particulate matter, land
occupation, and water consumption, GLO-only in V1."* Safe to say
*"ReCiPe characterisation factors sourced from RIVM (Huijbregts
et al., 2017)."* Not safe to say *"Arko is endorsed by RIVM"* or
*"ReCiPe-certified"*.

## Open items

- **Outreach to RIVM ReCiPe team for explicit grant.** Phase 2-3 task
  before commercial production deployment at scale. Combine with the
  CML-IA outreach into a single *"academic LCIA factor providers
  outreach"* shelf item; both groups respond to the same kind of
  request.
- **`arko-license` `recipe_2016_rivm_gratis` preset (or shared
  `academic_lcia_gratis` preset with CML-IA).** Add when the
  license-preset crate work begins.
- **Monitor RIVM for newer ReCiPe versions.** v1.1 has been the
  latest since the 2018-01-17 erratum. If v1.2 / v2.0 ships, Arko's
  preset version key bumps accordingly.
- **EU sui generis Database Right** — same theoretical concern as
  CML-IA. Same shelf item; one formal opinion can cover both.
- **V2 regionalisation bundle** — `CasRegion` matcher + per-process
  `C` build + Eq. 3 restructure + reader region-extraction +
  5 country-specific CF tables, with the **two-vocabulary** finding
  above as a load-bearing design constraint.
