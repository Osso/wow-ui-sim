# Patch 8.3.0 API audit

Warcraft Wiki page **109654**, revision **6471393** (2025-09-13T10:01:45Z), retrieved 2026-10-08. Default retail carries 12.1.0; audit covers publication accounting, not historical reconstruction.

## Source boundary

Pinned raw source has **220 inventory occurrences**: Global API 123 added / 42 removed, Widgets 2 / 2, Events 39 / 6, CVars 5 / 1. The removed CVars-column item is explicitly a command (`DumpSoundKits`), not a CVar. All eight numerical headers match parsed counts.

New opt-in `--legacy-column-headers` retains prose counts and command-column identity without changing prior default outputs. New extractor opt-in `--retain-reference-notes` retains the Auction House summary and inline citation; existing normalization/example flags remain recorded. Two RED fixtures reproduce unsupported options; acceptance is pending.

Sixteen nonblank extract occurrences remain; four late-build APIs appear in two summary bullets outside the consolidated inventory and require separate accounting. Discovery, per-ID review and final proof pending.

## Sources

- [Pinned fetch](../../../data/patch-api/evidence/8.3.0-session-2026-10-08/p830-fetch.json).
- [Provenance](../../../data/patch-api/sources/8.3.0-api-changes.provenance.json), [raw](../../../data/patch-api/sources/8.3.0-api-changes.wikitext), [extract](../../../data/patch-api/sources/8.3.0-api-changes.txt), [register](../../../data/patch-api/sources/8.3.0-wikitext-register.json).
- [Specification](../../specs/patch-8-3-0-publication-sweep.md).

## See Also

- [[patch-8-3-7-api-audit]] — publication/extract accounting precedent.
- [[patch-9-0-1-api-audit]] — chronological supersession and retained boundaries.

## Bounded retirement implementation

Initial discovery finds 174 OK / 46 gaps. Six unused retail surfaces are retired: four C_ProductChoice members, C_WowTokenPublic.SellToken and GetAuctionHouseDepositRate. A separate RETIRED_8_3_0_MEMBERS list uses the existing retail-only module gate; the legacy global is excluded only under retail-12-0-0. Mists defaults and classic callers remain intact.

Qualified and bare cached-retail searches plus whole src/tests scans are retained untruncated in [consumer receipt](../../../data/patch-api/evidence/8.3.0-session-2026-10-08/p830-removal-consumers.json). Bare GetProducts matches belong to C_StoreSecure, not C_ProductChoice. ProductChoice callers are already Mists-only; no retail caller migration needed. RED tests fail at fabricated GetChoices lookup and legacy deposit-rate raw publication after unmodified cached preload. Expected final fixture has forty gaps; GREEN and final proof pending.
