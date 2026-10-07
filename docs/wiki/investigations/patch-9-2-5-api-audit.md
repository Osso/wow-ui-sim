# Patch 9.2.5 API page audit

Page 237255 revision 2301036 (2022-09-03T02:10:47Z), retrieved October 7, 2026. Branch `p925-page` starts at master `572a77d84`; retail carries 12.1.0.

## Accounting

84 inventory rows: 55 added, 18 removed, 11 changed. All headers match. 135 nonblank extract rows retained with verbatim examples; inline Structures and multiline security warning recovered by fixtures. All 27 later registers apply.

## Runtime changes

Nine unused namespace members retired using the existing retail-only module; qualified and bare cached searches have no matches. Raw/repeated lookup and cached-full-UI tests added. Live C_ReportSystem.InitiateReportPlayer and SendReportPlayer remain for cached consumers. No deprecation wrapper changed. Initial observation: 41 OK / 43 gaps; expected closure: nine retirements, 34 remaining exact gaps. Verification pending. No other model fabricated.

## Sources

- [Source and provenance](../../../data/patch-api/sources/9.2.5-provenance.json)
- [Register](../../../data/patch-api/sources/9.2.5-wikitext-register.json)
- [Retirement scans](../../../data/patch-api/evidence/9.2.5-session-2026-10-07/p925-removal-consumers.json)
- [Spec](../../specs/patch-9-2-5-publication-sweep.md)

## See Also

- [[patch-9-2-7-api-audit]] — next publication register.
- [[patch-10-0-0-api-audit]] — proof boundaries.
