# Patch 10.2.6 API page audit

Page 582132 exists; no earlier-page substitution. Revision 6268742 (March 20, 2025, 19:36:42 UTC), retrieved October 7, 2026 UTC. Branch p1026-page starts from p1027-page revision 9d1a36178. Default retail carries 12.1.0, not reconstructed 10.2.6.

## Source and accounting

220 inventory occurrences: 136 added, 84 removed, zero changed. All six header counts match: global API 122/81, events 1/0, CVars/commands 13/3. Sixteen later registers, 10.2.7 through 12.1.0, supersede chronologically; fourteen inventory expectations reverse publication direction. Parser/extractor tooling unchanged; all seventeen registers regenerate byte-identically, including the sixteen existing registers. All 104 preserved later source/register/coverage/fixture inputs remain unchanged.

358 unique source IDs: 220 inventory + 138 extract. Ledger: 102 partial-development-green, 97 bounded-coverage, 142 audit-pending, 17 metadata-only. Every extract ID assigned once: one substantive summary, 25 enumeration, 19 structure, 76 deprecated API and 17 editorial rows. All 121 substantive extract occurrences remain pending. Loaded cached deprecation aliases/wrappers and explicit publication confer no signature, populated DTO, security or native behavior credit. Candidate source matches are not implementation proof.

## Bounded closures and retained gaps

Discovery: 197 OK / 23 failures. Three unused retail retirements leave 200 OK / 20 exact publication gaps:

- `C_CameraDefaults.GetCameraFOVDefaults` and `C_TaskQuest.GetUIWidgetSetIDFromQuestID` are marked removed in the existing retail-only namespace module; ordinary lookup cannot fabricate them. Qualified cached retail searches find no consumers.
- Legacy `CanSummonFriend` is no longer registered in retail/PTR. Namespace `C_RecruitAFriend.CanSummonFriend` remains published; classic retains its existing legacy registration. Cached global-call search finds no consumers. No new RAF model claim.
- Existing global `GetCameraFOVDefaults` remains a temporary camera-default placeholder, not evidence of a modeled camera. Bare-token searches disambiguate its live consumer at `Blizzard_SettingsDefinitions_Shared/Graphics.lua:1060` from the retired namespace member. RAF successor consumers remain at `Blizzard_FriendsFrame/Mainline/FriendsListTemplates.lua:1231` and `FriendsFrame.lua:139`; no legacy global reference found.

`profanityFilter` must remain: cached `Blizzard_SettingsDefinitions_Frame/Social.lua:69` calls `Settings.SetupCVarCheckbox(category, "profanityFilter", ...)`. Historical removal is a retained publication gap, not authority to break a current consumer. No Blizzard wrapper/cache edits. Classic namespace retirement module remains disabled; no 10.x/11.x epoch feature added.

[Per-ID review](../../../data/patch-api/evidence/10.2.6-session-2026-10-07/p1026-gap-review.json) assigns all 23 discovery failures, three closures and twenty precise retained boundaries. Commentator camera movement, FrameXML debug output, ordered item-trigger metadata, self-found mode, RAF linkage, display seasons, spectating sessions, map-pin visualization DTOs, world-loot classification/range/interaction, talent-master distance and testOutput fanout require missing models/contracts, not newly published inert functions. Unsupported camera/3D behavior stays explicit.

`gxMTDecals` is published but page default 1 differs from current 0. Shared classifier logs default mismatch separately from failure; this ledger leaves it pending and grants no default parity credit. Thus twenty exact publication failures plus one default-drift occurrence are distinguished, not conflated.

[Extract scout](../../../data/patch-api/evidence/10.2.6-session-2026-10-07/p1026-extract-scout.json) retains every statement and candidate. Enum numeric values/member renames, populated AreaPOI/Vignette/widget DTO fields, and each deprecated global-to-namespace argument/return/error/security contract remain pending. Cached alias identity is not wrapper behavior; future-removal prose does not authorize deleting Blizzard deprecation files.

## Verification

Runtime/test revision 1ef2ed7b4. New repeated-lookup retirement test fails before implementation and passes afterward; full cached Game prefork checks the same boundary, successor publication, retained profanityFilter and no new Lua errors. Seventeen isolated publication sweeps pass exact fixtures; [table](../../specs/patch-10-2-6-publication-sweep.md#local-proof). Negative control changes only `C_CurrencyInfo.GetCoinIcon` added → removed: exactly one new failure, no resolved failures, 20 → 21 gaps, expected exit 101. Eighteen parser/extractor fixtures pass; initial import-path failure is preserved and superseded by correct PYTHONPATH proof.

Cargo fmt --check and Mists tests check pass with zero non-vendor warnings. Six existing iced vendor manifest deprecations and their summary remain unsuppressed. Separate current-retail binary build passes; bounded startup exits 0 with JSON `[]`. All commands use explicit cwd p1026-page and its own target. Command output is captured once, saved and searched; proof ledger binds output hashes to exact revisions. No full suite, canonical working-file edits, siblings, vendor/Wowless edits, agents/models, push or merge. Worktree creation used the prescribed canonical Git metadata operation with cwd in the empty destination.

Changed Rust manually reviewed: two retirement data entries, existing marker call, a short classic-gated registration and bounded tests; no changed-line readability violations. Proof is local targeted development evidence, not native or independent acceptance. Artifact validator checks hashes, all 358 IDs, chronological expectations, exact sweep fixtures, three closures, default mismatch, negative control and preserved inputs. Artifact validation passes at 7d8d11fff: all 358 source IDs, seventeen exact chronological sweeps, three closures, twenty retained publication gaps, one default mismatch and 104 preserved inputs. [Result](../../../data/patch-api/evidence/10.2.6-session-2026-10-07/p1026-validation-result.json) records command/revision/output. Later result/documentation-only edits and supplemental read-only consumer evidence do not invalidate runtime or validated accounting proof.

## Sources

- [Provenance](../../../data/patch-api/sources/10.2.6-api-changes.provenance.json)
- [Coverage ledger](../../../data/patch-api/sources/10.2.6-page-coverage.json)
- [Publication contract](../../specs/patch-10-2-6-publication-sweep.md)
- [Retirement searches](../../../data/patch-api/evidence/10.2.6-session-2026-10-07/p1026-retirement-consumers.json)
- [Register reproduction](../../../data/patch-api/evidence/10.2.6-session-2026-10-07/p1026-register-reproduction.json)
- [Proof ledger](../../../data/patch-api/evidence/10.2.6-session-2026-10-07/p1026-proof.json)
- [Artifact validator](../../../data/patch-api/evidence/10.2.6-session-2026-10-07/p1026-validate.py)

## See Also

- [[patch-10-2-7-api-audit]] — read-only branch base and first later register.
- [[patch-11-0-0-api-audit]] — launch accounting and proof conventions.
- [[client-profiles]] — supported retail/classic boundaries.
