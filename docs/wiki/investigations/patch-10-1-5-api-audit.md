# Patch 10.1.5 API page audit

Page 396196, revision 3807695 (August 2, 2023, 17:30:43 UTC), retrieved October 7, 2026 UTC. Branch `p1015-page` starts from master `368e9bb26`. Default retail carries 12.1.0, not reconstructed 10.1.5.

## Source accounting

101 inventory occurrences: 74 added, 15 removed, 12 changed. Global API source headers say 36 added / 3 removed, but enumerate 40 / 8. Preserve all occurrences and the mismatch; widgets 2/3, events 17/2 and CVars 15/2 match. Nineteen chronological later registers (10.2.0 through 12.1.0) govern current publication. A one-line placeholder reserves 10.1.7 at the list start; that branch is not a dependency.

199 unique source IDs: 101 inventory + 98 extracts. Ledger statuses: 18 bounded-coverage, 50 partial-development-green, 122 audit-pending, nine metadata-only. Pending inventory includes 32 exact publication gaps and one historical default mismatch: wmoPortalInteriorFade page 0 versus current default 1. All 89 substantive extract statements remain pending; 9 editorial/source rows grant no runtime credit. Changed signatures and event payloads are not proved by publication/registration.

The old parser silently dropped all level-two inventories and the extractor stopped before enums/constants/structures. Behavioral RED fixtures reproduce both failures; parser/extractor now reuse the exact heading handling already present in the read-only 10.1.7 worktree. No new markup rendering rule, placeholder, historical epoch or vendor mutation.

## Bounded closures

Discovery 67 OK / 34 gaps. Final 69 OK / 32 gaps:

- C_CampaignInfo.UsesNormalQuestIcons autostub is retired only for retail/PTR. Qualified cached retail search has no consumer. Bare search finds `Blizzard_ObjectAPI/Mainline/Campaign.lua:53` defining CampaignMixin:UsesNormalQuestIcons, an unrelated Lua method. GetCampaignInfo remains published.
- RequestArtifactCompletionHistory registration is omitted for retail/PTR; no current cached retail Lua consumer. Classic retains the modeled request and its availability-state test. IsArtifactCompletionHistoryAvailable/GetArtifactInfoByRace remain unchanged. Whole src/tests caller search identifies the single legacy request test, now classic-only.

Repeated bare-environment lookup and full cached Game prefork tests bound retirement evidence. Neither retirement deletes or rewrites Blizzard deprecation wrappers. All profile gating uses existing retail/profile boundaries.

## Retained gaps

[Per-ID review](../../../data/patch-api/evidence/10.1.5-session-2026-10-07/p1015-gap-review.json) assigns every discovery failure once. Commentator item cooldown lists/DTOs, trackable catalogs/navigation/text/mutation/events, journal links/map relations, loot roll timing, PvP reward tuples with role shortage bonus, super-track arbitration and reagent-aware enchant validation need modeled producers, not inert registrations. GetNumJunkItems has an inferred Forever-only policy; exposing that policy under retail would silently promote unverified count/eligibility semantics.

UpdateUIParentPosition is superseded by the later removal, but current cached retail still defines it (`Blizzard_UIParentUtil/UIParentUtil.lua:13`), hooks it (`Blizzard_EditMode/Shared/EditModeUtil.lua:88`) and calls it (`Blizzard_Game/Shared/EventImplementation.lua:48,304`). Preserve current vendor behavior and record the removal gap. [Consumer evidence](../../../data/patch-api/evidence/10.1.5-session-2026-10-07/p1015-retained-update-ui-consumers.txt) retains exact matches.

[Extract scout](../../../data/patch-api/evidence/10.1.5-session-2026-10-07/p1015-extract-scout.json) assigns every non-inventory occurrence once with its literal statement, section and proof boundary. Numeric enum changes, ContentTrackingConsts identities/capacity enforcement, populated DTO fields/renames/removals, range optimization and combat-insecure widget restrictions remain unproved. Source retention is not native compatibility credit.

Read-only comparison with 10.1.7 revision 6473483 finds no add/remove intersection with the final 32 gap IDs. Changed-symbol intersections are retained separately in [supersession evidence](../../../data/patch-api/evidence/10.1.5-session-2026-10-07/p1015-possible-1017-supersessions.json). Recompute exact fixture after integration; this snapshot cannot predict that branch's runtime changes.

## Verification

Current runtime/test revision `2474d039a`. Twenty isolated publication sweeps pass exact fixtures; table below. Both retirement boundaries have observed RED and final GREEN; combined bare-environment assertions and one cached Game prefork pass. Eight archaeology completion-history regressions, one campaign state case and two campaign/covenant-default cases pass. The classic-only history request test passes under Mists. Whole src/tests caller searches find no other direct retired-member callers; no retail successor exists for the removed request, so its legacy request test is classic-only rather than redirected to a fabricated API.

Negative control changes only Frame:AbortDrag added → removed: exactly one new failure, no resolved failures, 32 → 33, expected exit 101. Parser/extractor fixtures: 23 GREEN after two observed RED failures. All twenty registers regenerate byte-identically; seventeen generated extracts reproduce exactly. The three later 12.0.5/12.0.7/12.1.0 plaintext files are MediaWiki/crawler captures, not extractor outputs; attempted local extraction differs (12.1.0 has an unsupported description template). Their original capture bytes, like all 122 previous source/register/ledger/fixture inputs, remain unchanged. No unsupported capture was rewritten or silently labeled reproducible.

Cargo fmt --check and Mists test check pass with zero non-vendor warnings. Six iced vendor manifest deprecations and their summary remain unsuppressed. Separate retail build passes; bounded startup exits 0 and returns `[]`. [Proof ledger](../../../data/patch-api/evidence/10.1.5-session-2026-10-07/p1015-proof.json) binds commands, exact scope/revision, logs, hashes and expected exits. Earlier parser proof remains valid at unchanged parser scope; final runtime proof supersedes initial retirement-only runs. Saved cargo outputs are inspected rather than rerun for logs. Changed Rust manually audited for readability: flat data marker, bounded profile registration and short state assertions; no changed-line violations found. No full suite, agents/models/CLIs, push, merge, sibling target reuse, canonical working-file edits or cache/vendor changes. Every command uses explicit cwd `p1015-page` and its own target. Worktree creation used only the prescribed canonical Git metadata operation with cwd in the empty destination.

| Patch | Rows | OK | Gaps | Isolated exit |
|---|---:|---:|---:|---:|
| 10.1.5 | 101 | 69 | 32 | 0 |
| 10.2.0 | 150 | 120 | 30 | 0 |
| 10.2.5 | 59 | 45 | 14 | 0 |
| 10.2.6 | 220 | 200 | 20 | 0 |
| 10.2.7 | 104 | 68 | 36 | 0 |
| 11.0.0 | 495 | 329 | 166 | 0 |
| 11.0.2 | 34 | 22 | 12 | 0 |
| 11.0.5 | 48 | 38 | 10 | 0 |
| 11.0.7 | 98 | 70 | 28 | 0 |
| 11.1.0 | 116 | 97 | 19 | 0 |
| 11.1.5 | 125 | 89 | 36 | 0 |
| 11.1.7 | 48 | 40 | 8 | 0 |
| 11.2.0 | 162 | 135 | 27 | 0 |
| 11.2.5 | 163 | 118 | 45 | 0 |
| 11.2.7 | 508 | 414 | 94 | 0 |
| 12.0.0 | 1010 | 989 | 21 | 0 |
| 12.0.1 | 225 | 222 | 3 | 0 |
| 12.0.5 | 363 | 352 | 11 | 0 |
| 12.0.7 | 174 | 171 | 3 | 0 |
| 12.1.0 | 778 | 773 | 5 | 0 |

## Sources

- [Provenance](../../../data/patch-api/sources/10.1.5-api-changes.provenance.json)
- [Coverage ledger](../../../data/patch-api/sources/10.1.5-page-coverage.json)
- [Publication contract](../../specs/patch-10-1-5-publication-sweep.md)
- [Retirement searches](../../../data/patch-api/evidence/10.1.5-session-2026-10-07/p1015-retirement-consumers.json)

## See Also

- [[patch-10-2-0-api-audit]] — occurrence and proof template.
- [[patch-10-2-5-api-audit]] — exhaustive page accounting.
- [[client-profiles]] — retail/classic boundaries.
