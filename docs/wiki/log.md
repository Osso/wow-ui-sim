## [2026-10-05] evidence | 12.0.0 wikitext publication sweep

[Audit supplement](investigations/patch-12-0-0-api-audit.md#wikitext-supplement) records revision 6747189, parser and classifier corrections, 64 exact namespace retirements, all 270 initial-gap reviews, 812 OK / 198 final known gaps, source comparisons and proposed statuses. Four isolated sweeps, one-gap negative control, local check/build, fmt and startup `[]` pass. Six vendor manifest warnings match master; ledger/coverage JSON unchanged.

## [2026-10-04] evidence | 12.0.5 pending non-sweep follow-up

[Audit follow-up](investigations/patch-12-0-5-api-audit.md#pending-non-sweep-follow-up--2026-10-04) records all24 row boundaries, leadership subset RED/master/GREEN, unchanged vendor mouse proof, 55 selected GREEN, checks/startup and pinned-rilua limits finding. Coverage JSON unchanged; aggregate blockers remain explicit.

## [2026-10-04] docs | 12.1.0 publication sweep accounted

778 inventory symbols swept: 631 match, 147 gaps baselined. [Audit page](investigations/patch-12-1-0-page-audit.md#round-2--publication-sweep--2026-10-04); 1,111 rows; 391 pending /151 bounded /505 partial /64 metadata.


## [2026-10-04] docs | 12.1.0 sweep gaps closed (round 3)

Sweep gaps 147 → 21: LoD bootstrap preload fix, 31 added Global API functions, widget/event/CVar surface. [Audit page](investigations/patch-12-1-0-page-audit.md#round-3--closing-sweep-gaps--2026-10-04); 265 pending /157 bounded /625 partial /64 metadata.

## [2026-10-04] docs | 12.1.0 extract rows round 4

Enum values fixed and proven (58), struct shapes (26), deprecated wrappers (9). [Audit page](investigations/patch-12-1-0-page-audit.md#round-4--extract-rows-enums-structs-deprecated-wrappers--2026-10-04); 156 pending /224 bounded /652 partial /79 metadata.

## [2026-10-04] docs | 12.1.0 strict removals fix and deprecation-fallback sweep rule

Post-startup strict removals deleted Blizzard deprecation wrappers; fixed at namespace setup. Sweep accepts deprecation-file fallbacks for removed symbols (gaps 25 → 11). [Audit page](investigations/patch-12-1-0-page-audit.md); 135 pending /239 bounded /658 partial /79 metadata.

## [2026-10-04] docs | 12.0.7 wikitext supplement

Extract missed 65 of 174 collapsed-table symbols; shared publication sweep added (147 OK, 27 gaps). [Audit page](investigations/patch-12-0-7-api-audit.md#wikitext-supplement--2026-10-04). 12.1.0 now 22 pending after leftovers; rilua pinned at a76ffa8 (intern GC fix).

## [2026-10-04] docs | 12.0.5 wikitext supplement

Crawler register missed 216 of 363 collapsed-table symbols; sweep 306 OK / 57 gaps. [Audit page](investigations/patch-12-0-5-api-audit.md#wikitext-supplement--2026-10-04).

## 2026-10-04 | Club membership model

Added [club membership](systems/club-membership.md) and contract; retired temporary management defaults. Verification pending.

## 2026-10-05 | Club host inputs

Added synchronous member snapshots/departures and stable historical chat authors to [club membership](systems/club-membership.md). New proof pending.

## 2026-10-05 | Guild opaque-ID regression

Cached GuildRoster rank menus require `guid` independently of member IDs. Added stable projected GUIDs; no vendor/UI patch. Retest pending.

## 2026-10-05 | Club membership proof

Recorded [development proof](../specs/club-membership.md#development-proof--2026-10-05): five host/member event rows, 12 behavior tests, club/guild/communities/bnet green, publication green, startup `[]`, fmt/check green. Chat failure reproduced on untouched master. No coverage ledger edits.
