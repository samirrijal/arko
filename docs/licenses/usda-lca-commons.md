# USDA LCA Commons data — license analysis (primary-source reading)

**Status:** characterised, 2026-04-20. Primary-source anchor for any
Arko public claim about LCA Commons compatibility. Not legal advice.

## Tl;dr

USDA LCA Commons datasets are dedicated to the **public domain under
CC0 1.0 Universal**, by USDA-NAL policy imposed at submission time.
Strictly more permissive than the JRC EF reference package: no
attribution is legally required, commercial use is explicitly
permitted, there is no term expiry, and there is no redistribution
restriction.

| Thing                                            | Published by                  | License posture                                       |
| ------------------------------------------------ | ----------------------------- | ----------------------------------------------------- |
| **LCA Commons datasets** — unit processes and product systems (agricultural LCI) | USDA-NAL / ARS (and contributors) | **CC0 1.0 Universal** public-domain dedication, mandatory at submission. |

Two non-copyright carve-outs in the Data Use Disclaimer Agreement
(Appendix A of the Submission Guidelines) that still matter for
Arko:

1. **Trademark-style restriction:** "The names USDA/ARS/NAL... may
   not be used in any advertising or publicity to endorse or promote
   any products or commercial entities unless specific written
   permission is obtained." CC0 waives copyright, not trademark.
2. **Indemnity + AS-IS warranty disclaimer:** the user agrees to
   indemnify the Government against any claim related to the data's
   use; data ships "AS IS" with no warranty of any kind.

Neither of those blocks Arko from shipping, parsing, or serving the
data. They shape the marketing copy and the hosting-ToS language.

## Sources read

All read 2026-04-20. Saved locally for re-reference.

- [USDA LCA Commons Submission Guidelines, Final 2018-07-25][submission-guidelines] —
  canonical policy document that binds every dataset in the Commons.
  Contains the CC0 1.0 requirement (§ "Placing Your Data in the
  Public Domain"), the Data Use Disclaimer Agreement (Appendix A),
  the Data Contributor's Content License Agreement (Appendix B), and
  the full CC0 1.0 Universal legal code (Appendix C).
- [LCA Commons — catalog.data.gov entry][catalog-gov] — government
  catalog listing naming the Commons and the submission guidelines
  by URL.
- [Federal LCA Commons landing][lcacommons] — public face of the
  Commons, maintained by USDA-NAL.

[submission-guidelines]: https://www.lcacommons.gov/sites/default/files/2018-07/LCA_Commons_Submission_Guidelines_2018-07-25_FINAL.pdf
[catalog-gov]: https://catalog.data.gov/dataset/lca-commons
[lcacommons]: https://www.lcacommons.gov/

## The CC0 1.0 dedication — verbatim

From "Placing Your Data in the Public Domain":

> *"USDA-NAL is requiring that all datasets submitted to the LCA
> Commons be placed in the public domain under the terms of the
> Creative Commons Zero, Public Domain Dedication License (CC0 1.0
> Universal (CC0 1.0)). By placing your datasets in the public
> domain, you are, according to the CC0 1.0 license, removing 'all
> of [your] rights to the work worldwide under copyright law,
> including all related and neighboring rights, to the extent
> allowed by law.'"*

This is mandatory at submission: every dataset in the Commons is
CC0 by construction. There is no parallel commercial tier, no
"enhanced" dataset behind a paywall, no per-licensor EULA wrapper
around individual data lots (contrast EF 3.1 LCI, where the
Commission aggregates four different commercial EULAs).

From Appendix C (the CC0 1.0 legal code itself), § 2 (Waiver):

> *"Affirmer hereby overtly, fully, permanently, irrevocably and
> unconditionally waives, abandons, and surrenders all of Affirmer's
> Copyright and Related Rights and associated claims and causes of
> action... (iv) for any purpose whatsoever, including without
> limitation commercial, advertising or promotional purposes."*

The CC0 text is explicit on commercial use and on
"irrevocable/unconditional." There is no term, no revocation path,
no permitted-use matrix.

## The Data Use Disclaimer Agreement — what it adds

Appendix A of the Submission Guidelines, verbatim on the two
non-copyright carve-outs:

> *"The names USDA/ARS/NAL, however, may not be used in any
> advertising or publicity to endorse or promote any products or
> commercial entities unless specific written permission is obtained
> from USDA/ARS/NAL."*

> *"YOU AGREE TO INDEMNIFY THE GOVERNMENT AND USDA/ARS/NAL, AND ITS
> CONTRIBUTORS, SUBSIDIARIES, AFFILIATES, OFFICERS, AGENTS, AND
> EMPLOYEES AGAINST ANY CLAIM OR DEMAND, INCLUDING REASONABLE
> ATTORNEYS' FEES, RELATED TO YOUR USE OF THESE DATA. THESE DATA ARE
> PROVIDED BY USDA/ARS/NAL AND ITS CONTRIBUTORS 'AS IS'..."*

And on support:

> *"The user also understands that USDA/ARS/NAL is not obligated to
> provide the user with any support, consulting, training or
> assistance of any kind with regard to the use of these Data or to
> provide the user with any updates, revisions or new versions of
> these Data."*

These obligations travel with the data: they are terms of access,
not copyright restrictions. CC0 1.0 § 4(a) explicitly preserves
trademark and patent rights, so the USDA/ARS/NAL name restriction
is consistent with CC0.

## Dataset citations — encouraged, not required

From the Submission Guidelines, "Dataset Citations":

> *"It should be made clear that although the legal requirement to
> cite datasets is removed under the terms of the CC0 1.0 Universal
> license, individuals who use these data sets are not absolved
> from institutional and scholarly norms requiring dataset
> citation. Individuals who use LCA Commons datasets are strongly
> encouraged to cite these datasets..."*

Citation is an institutional/scholarly norm, not a legal condition
of use. Arko should still cite as good practice (and as a
product-quality signal), but omitting a citation is not a license
violation.

## What Arko can do

- **Parse, link against, ship fixtures from** LCA Commons datasets
  in the Arko distribution — fine. No redistribution clause, no
  per-licensor EULA.
- **Bundle LCA Commons datasets** into an Arko SaaS surface and let
  paying customers consume them — fine. CC0 § 2(iv) explicitly
  permits commercial use.
- **Run conformance/parity smokes** against LCA Commons datasets
  and publish numbers in release notes, CHANGELOG, blog posts,
  homepage — fine. No hosting restriction, no term expiry.
- **Cite datasets by DOI** in release notes and marketing copy as a
  good-practice scholarly citation. Each Commons dataset gets a DOI
  at publication.
- **Distribute derivatives** (cached parsed forms, re-serialised
  fixtures, LU-factorised caches, aggregated summaries) — fine. CC0
  waives the right to control derivatives.

## What Arko must not do

- **Imply USDA/ARS/NAL endorsement.** *"LCA Commons data supported
  in Arko"* is fine. *"USDA-endorsed LCA platform"* or *"Powered by
  USDA-NAL"* is not, without written permission.
- **Name USDA/ARS/NAL in commercial entity promotion.** Product
  pages, pricing pages, sales copy, and similar surfaces must not
  use the USDA/ARS/NAL names as an endorsement signal. Citing the
  data source by name on a fact page or in release notes is not
  "advertising or publicity to endorse" and is fine.
- **Represent the data as warranted / fit-for-purpose by USDA.**
  The Appendix A AS-IS clause disclaims all warranties. Arko's own
  ToS must make clear that Arko inherits that posture: the LCA
  Commons data is AS-IS, and Arko does not assert it is fit for
  regulatory submission, compliance, or any specific use.
- **Shift USDA indemnity onto end users without flagging it.** The
  indemnity in Appendix A runs from "the user" (whoever accesses
  the data) to the Government. If Arko is the immediate accessor
  and then provides the data to customers, Arko is on the hook
  unless the hosting ToS passes the indemnity through. Flag for
  legal review before the first paid hosted-data customer.

## Attribution template (good practice, not required)

For a specific LCA Commons dataset, cite by DOI:

> *[Submitter(s)] ([year]). [Dataset title]. USDA-NAL LCA Commons.
> [DOI]. Accessed YYYY-MM-DD. Public domain (CC0 1.0 Universal).*

For a general reference to the Commons as a data source:

> *Datasets sourced from the USDA National Agricultural Library's
> LCA Commons (lcacommons.gov). Public domain under CC0 1.0
> Universal.*

Using the word "sourced from" rather than "endorsed by" /
"powered by" / "in partnership with" keeps the Appendix A
trademark-style restriction clean.

## Comparison to JRC EF

| Axis                        | JRC EF reference package        | USDA LCA Commons                |
| --------------------------- | ------------------------------- | ------------------------------- |
| License                     | EC Decision 2011/833/EU (CC-BY equiv.) | CC0 1.0 Universal              |
| Attribution required?       | Yes (legal)                     | No (encouraged, not legal)      |
| Commercial use?             | Yes                             | Yes (explicit in CC0 § 2(iv))   |
| Redistribution?             | Yes with attribution            | Yes, unrestricted               |
| Term expiry?                | None for reference package; 2025/2030 for LCI | None                            |
| Trademark carve-out?        | None stated                     | USDA/ARS/NAL names — yes        |
| Data scope                  | Flows, LCIA methods, CFs (infrastructure only) | Unit processes + product systems (agricultural LCI) |
| Arko-ship posture           | Fine (reference package); LCI is NOT free | Fine end-to-end                 |

LCA Commons is the most permissive license posture Arko has
encountered in the first three foreground-free databases. It is
the one that lets Arko ship a complete, free-for-the-customer
starter kit with no attribution plumbing, no term-expiry
monitoring, and no per-dataset EULA acceptance friction.

## Implications for Arko

**For Phase 1 exit criterion "three free databases importable":**
LCA Commons is cleanly eligible as the third slot. CC0 satisfies
every Arko-ship operation without qualification — unlike the EF
reference package (fine but needs attribution) and unlike the
EF 3.1 LCI bundle (not free in any Arko-ship sense).

The three-database slate after this read is:

1. ÖKOBAUDAT 2024-I (CC-BY-style, with attribution — already
   imported).
2. JRC EF reference package (infrastructure only, EC Decision
   2011/833/EU reuse — already imported).
3. **USDA LCA Commons (CC0 1.0, fully unrestricted — to be
   imported).**

ProBas (Umweltbundesamt) remains a future candidate, not needed for
the Phase 1 exit claim.

**For the `arko-license` crate's preset list:** add a preset
named along the lines of `usda_lca_commons_cc0` that encodes: no
attribution legally required, unrestricted commercial use,
unrestricted redistribution, no term expiry, trademark carve-out on
USDA/ARS/NAL names. This is the most permissive preset and can
serve as the "baseline free" calibration.

**For the reader work:** the Submission Guidelines confirm LCA
Commons publishes in ILCD XML (openLCA-compatible), which Arko's
ILCD reader already handles. The actual import work should be
incremental from the existing ÖKOBAUDAT / EF reader surface, not
net-new parsing.

**For public-facing copy (arko.earth homepage, blog posts, docs
site):** safe to say *"Arko ships ready-to-use datasets from the
USDA LCA Commons (public domain, CC0 1.0)."* Safe to say *"Arko
sources agricultural LCI data from USDA-NAL's LCA Commons."* Not
safe to say *"Arko is USDA-endorsed"*, *"Arko is powered by USDA"*,
*"Official USDA LCA platform"* — those all trip the Appendix A
trademark carve-out.

## Open items

- **Which specific LCA Commons datasets will Arko import for Phase
  1?** The Commons hosts hundreds of datasets across crops, animal
  products, biofuels, etc. A smaller, curated starter lot is
  probably the Phase 1 target — pick a representative set once the
  reader work begins.
- **Appendix A indemnity pass-through in the Arko hosting ToS.**
  Draft language before the first paid hosted-data customer. Not a
  Phase 1 blocker.
- **ProBas primary-source license read** — still unread. Only
  needed if the fourth-slot question comes up. Not a Phase 1 task.
- **CC0 1.0 § 4(b) warranty disclaimer** is redundant with the
  Appendix A AS-IS clause but worth noting for the arko-license
  preset — both independently disclaim warranty.
