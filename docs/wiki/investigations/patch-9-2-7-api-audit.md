# Patch 9.2.7 API page audit

Page 542130, revision 5227425 (September 4, 2022, 12:19:59 UTC), retrieved October 7, 2026. Requested page exists; no 9.2.5 substitution. Branch `p927-page` starts from master `af9ae57c2`. Current retail carries 12.1.0, not reconstructed Shadowlands behavior.

## Source accounting

Three inventory occurrences: two added events, one changed event. Added header count matches; no removals or retirement candidates. All 26 later registers, 10.0.0 through 12.1.0, supersede this page. Initial extraction retained only eight rows and silently lost the enumeration and structure sections because the page has no Global API heading. Event-only fixture reproduces the loss; level-two Events recognition restores 23 nonblank extract rows (eight metadata, fifteen pending behavior targets) without changing previous extraction-mode outcomes.

## Proof boundaries

Event registration does not prove auction purchase/delivery state or the changed notification's `auctionID` payload. Five new enum tables, three AuctionHouseError additions, ReportType.PvPScoreboard, and ItemKeyInfo's itemID/battlePetSpeciesID additions need exact identity/populated DTO behavior proof. Documentation relocation is source metadata, not an API implementation. No placeholders, vendor patches or runtime surface changes introduced.

Whole `src/` and `tests/` scan retains 43 untruncated matches, including embedded Lua and function references. Cached retail scan retains 128 matches. No global/method changed, so no caller migration or classic gating is needed. No removed row exists; qualified/bare pre-retirement scanning is vacuous.

## Verification

Targeted verification pending. Source extraction RED/GREEN: 23 extractor fixtures pass after correction. All 52 pre-existing extraction-mode outcomes unchanged. All existing source/register inputs remain untouched.

## Sources

- [Retained API response](../../../data/patch-api/evidence/9.2.7-session-2026-10-07/p927-fetch.json) — revision and original wikitext.
- [Register](../../../data/patch-api/sources/9.2.7-wikitext-register.json) — exact inventory identities and lines.
- [Spec](../../specs/patch-9-2-7-publication-sweep.md) — required proof boundaries.
- [Caller scan](../../../data/patch-api/evidence/9.2.7-session-2026-10-07/p927-whole-caller-scan.txt) — untruncated simulator callers.
- [Cached source scan](../../../data/patch-api/evidence/9.2.7-session-2026-10-07/p927-cached-source-scan.txt) — read-only retail references.

## See Also

- [[patch-10-0-0-api-audit]] — adjacent later publication register.
- [[patch-10-0-2-api-audit]] — accounting and proof conventions.
