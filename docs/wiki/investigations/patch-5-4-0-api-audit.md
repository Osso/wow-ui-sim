# Patch 5.4.0 API audit

Verified: 2026-10-08. Retail Mists of Pandaria, not Mists Classic. API pageid 414879 revision 6200158; separately pinned automated diff pageid 389126 revision 3741686. Parent pageid 511918 revision 6902277 states TOC 50400, build 17345 and September 10, 2013 release; the diff compares 5.3.0.17128 → 5.4.0.17359. Both sources are substantive pages, not redirects or stubs.

## Coverage matrix

| Exact scope | Result | Proof level |
|---|---|---|
| Main-page API references | 17 occurrences | Register, including repeated GetInstanceInfo and both Slider methods |
| Transcluded Global API / FrameXML / Events / Widget API | 65 / 6 / 18 / 8 occurrences | 97 entries; all five numerical headers match |
| Combined inventory | 114 occurrences; 92 OK / 22 retained gaps | Current-retail publication/absence only |
| Main-page / diff extract | 30 / 15 rows | Every seeded occurrence accounted; 17 editorial/context rows |
| Instance group size at GetInstanceInfo return #9 | 17 → 22 independently of maxPlayers=25 | Concrete current world-state integration/prefork fixture; automatic flex tuning not proved |
| Frame:IsForbidden | false → true → false | Concrete current flag transitions; no historical access-enforcement claim |
| Remaining substantive extract contracts | 27 pending | Precise per-statement reasons; no fabricated signatures or datasets |
| Historical removals | All 31 already absent | No new retirement or runtime change |

## Problematic cases

[Gap review](../../../data/patch-api/evidence/5.4.0-session-2026-10-08/p540-gap-review.json) retains exact observations and reasons for all 22 IDs. Missing backing domains include flexible-raid/LFG queues, proving grounds, archaeology sites/race data, arena inspection, war games, resurrection timeout, rated rewards, realm identity and consolidated-buff tooltip selection. Three C namespace members resolve only through namespace fallback, not explicit publication. An event is unknown to the strict catalog. Two historical FrameXML helpers and securerandom lack a pinned behavior contract; no aliases or guessed security policy were introduced.

[Contract review](../../../data/patch-api/evidence/5.4.0-session-2026-10-08/p540-contract-review.json) separates historical auction return placement, contradictory GetLFGRoleUpdate numbering (#5 in prose versus #6 in the printed signature), unspecified Modified API deltas, connected-realm naming/trading, saved-variable OOM behavior, the historical Slider defect, and literal historical enum values. UnitRealmRelationship has no connected/coalesced backing; GetUnitName currently delegates to its single-name getter. Current world-elapsed constants must not be renumbered to the 2013 table after their later retirement. Existing permissive publications, including C_NewItems defaults, receive no behavioral credit.

No simulator runtime code, Blizzard Lua, vendor/Wowless/WowlessData, or Classic bootstrap was changed. There are zero new runtime models; the two behavioral cases bound existing backing systems rather than filling missing domains.

## Retirement safety

[Scans](../../../data/patch-api/evidence/5.4.0-session-2026-10-08/p540-retirement-scans.json) retain untruncated `/usr/bin/grep -RInw -F` output for each of 31 bare global names. This includes `pcall(Name, ...)` and `and Name then`; none of this page's removals is a namespaced member. Cached scans exclude only *Documentation* files/directories. GetPVPRankInfo and UnitPVPRank have cached Vanilla consumers; source/test callers also exist for three other identities. No new gate retires these names. All retail absences predate this audit.

[Later-register checks](../../../data/patch-api/evidence/5.4.0-session-2026-10-08/p540-later-register-check.json) pin master bebcc5830 and queued p547/p542/p541 tips with git ls-tree-derived complete source sets. No later register re-adds these removals. The sweep begins with placeholders for 5.4.1, 5.4.2 and 5.4.7, followed by 5.4.8, 6.0.1 and the remaining retail chain; no Classic 5.5.x input.

## Reproduction and proof

The Mists automated-inventory parser and extractor stripping function are byte-identical copies from pinned p542-page; p541-page was also read before implementation. New summary, separate-transclusion and source-markup behaviors are opt-in functions/flags. [Origin](../../../data/patch-api/evidence/5.4.0-session-2026-10-08/p540-parser-origin.json) records the exact function seals.

All 56 saved registers reproduce byte-identically. 53 of 56 main extracts reproduce; inherited 12.0.5, 12.0.7 and 12.1.0 failures are unchanged. The separately pinned diff extract also reproduces. Prior source bytes and every recorded extraction-mode outcome are preserved. All 88 tools Python fixtures pass.

Final targeted proof passes: 57 publication/factory cases, 2 prefork and 2 standalone behavior cases, 11 instance and 13 protection caller cases, 6 frame-state source-unit cases, format, and Mists compilation with zero non-vendor warnings. Six inherited vendor manifest deprecations are retained. Negative control fails with exactly 22 → 23 gaps. The initial standalone filter selected zero tests; registered wrappers and nonempty-selection gates correct that verification defect. Portable-validator acceptance remains pending; see the session proof ledger. Runtime/lib-master and addons-enabled startup comparison gates are conditional on src changes; no src change occurred here. No full integration suite was run.

## Sources

- [Coverage ledger](../../../data/patch-api/sources/5.4.0-page-coverage.json) — all 159 occurrence IDs.
- [Provenance](../../../data/patch-api/sources/5.4.0-api-changes.provenance.json) — parent/page/diff pins and opt-in recipes.
- [Spec](../../specs/patch-5-4-0-publication-sweep.md) — publication contract and exclusions.
- [Evidence](../../../data/patch-api/evidence/5.4.0-session-2026-10-08/) — original responses, complete grep scans, command receipts and portable validator.

## See Also

- [[patch-5-4-8-api-audit]] — template and next merged retail register.
- [[patch-audit-validator-portability]] — fixed historical scope and later-audit gate.
