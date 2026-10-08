# Patch 8.1.5 API audit

Verified 2026-10-08. Warcraft Wiki page **394493**, current revision **3789732** (2019-04-25T05:47:36Z), refetched from the revisions API. Audit current retail, not an 8.1.5 emulator. **66 inventory + 19 extract occurrences = 85 ledger IDs**, covering every nonblank raw-source line. Discovery **29 OK / 37 gaps** becomes **42 OK / 24 exact gaps** after one meaningful missing producer and twelve bounded retirements.

## Source and parsing

[Source/provenance](../../../data/patch-api/sources/8.1.5-api-changes.provenance.json) pins raw SHA-256 `fc40dccee6ebfd304bce361e9251bcaab39e79e6fe2c6e3406c55fa4d6f5d8e4`; [fetch response](../../../data/patch-api/evidence/8.1.5-session-2026-10-08/p815-fetch.json) and HTTP receipt retain the current revision, not the revision listed in the older page inventory.

This page uses level-four API/New and API/Removals bullets, plus two unqualified event additions. No caption tables or numerical headers. Independent opt-in **`--legacy-api-bullets`** in both tools preserves original occurrence line numbers and strips API/event inventory without swallowing namespace notes, source URLs or incomplete `?` markers. Generator/extractor fixtures reproduce the prior failure before the change. Defaults and prior recipes remain unchanged; 8.2.0's different opt-in caption-table work is not copied.

Extract notes naming new C_CVar, C_ClassColor, C_ItemSocketInfo and C_Texture tables are structural migration context, with members separately inventoried. The three `?` markers supply no behavioral contract; retain them as incomplete editorial context, not fabricated requirements. Diff/deprecated links are retained but not expanded.

## Coverage matrix

| Capability | Covered | Missing / disabled | Proof level |
|---|---|---|---|
| Inventory publication/absence | 42 / 66 | 24 exact reviewed IDs | Raw + ordinary lookup, event registration; not signature/output/security parity |
| Toy fanfare | Acquire, repeat, clear, uncollect, unknown ID, fifth GetToyInfo result | Native acquisition events, persistence, live-client parity | Lua API integration through existing admin producer; retail and Mists |
| Namespace retirement | Twelve consumer-free removals | Current consumers, later re-addition, legacy-global policy | Bare/repeated lookup, cached full UI; Mists remains reachable |
| Extract accounting | All 19 occurrences | Three unspecified source markers cannot define behavior | Literal raw/extract mapping, no runtime credit |
| Reproduction | 38 registers; 35 saved extracts | Three inherited 12.x extract failures unchanged | Recorded flags or explicitly inherited recipes; 194 prior files / 74 mode outcomes preserved |

## Meaningful model

`src/c_api/c_toy_box_info.rs` owns pending acquired toy IDs. Existing `A_Admin.CollectToy` marks only newly collected catalog toys; repeating collection does not rewrap a cleared toy. Uncollection removes pending fanfare. `C_ToyBoxInfo.NeedsFanfare`, `ClearFanfare` and the fifth `C_ToyBox.GetToyInfo` result use the same backing state. Clearing is idempotent and does not remove ownership; unknown IDs do not affect another toy.

This replaces the prior no-op ClearFanfare and missing raw NeedsFanfare registration, rather than adding a false-return shim. Current cached ToyBox code clears fanfare on selection and reads `hasFanfare` in the fifth tuple slot. Native NEW_TOY_ADDED timing/payload is deliberately not claimed; the existing admin collection producer remains the bounded acquisition boundary. [Behavioral test](../../../tests/patch_8_1_5_toy_fanfare.rs) covers the complete transition sequence.

## Twelve retirements and retained removals

[Pre-change scans](../../../data/patch-api/evidence/8.1.5-session-2026-10-08/p815-removal-consumers.json) and [final scans](../../../data/patch-api/evidence/8.1.5-session-2026-10-08/p815-whole-callers-after.json) retain untruncated `/usr/bin/grep -rnE` output: whole-word qualified and bare names in cached retail AddOns, excluding Documentation files/directories, and all bare occurrences in src/tests (including conditional references and pcall). All **28 removal candidates** are scanned. Host lacks rg; grep use is recorded explicitly.

Every newly retired member has **zero cached qualified/bare consumers and zero initial source/test callers**:

- C_AreaPoiInfo.GetAreaPOITimeLeft; C_Calendar.GetDate; C_ChatInfo.ReportPlayer; C_Club.AddClubStreamToChatWindow.
- C_DateAndTime.GetDateFromEpoch, GetTodaysDate, GetYesterdaysDate.
- C_PvP.GetBrawlInfo; C_ReportSystem.ReportPlayer.
- C_Social.GetLastScreenshot, GetNumCharactersPerMedia, GetScreenshotByIndex.

The existing retail-only tombstone mechanism prevents ordinary lookup from fabricating these members. No global, vendor, classic-registration or default-shim deletion. Mists lookup preserves all twelve.

Retain CanReportPlayer: qualified scan is empty, but two current bare-name consumers under C_ReportSystem trigger the conservative prohibition. Retain all CVar globals, HasInspectHonorData and RequestInspectHonorData with cached matches; GetAtlasInfo remains an accepted deprecation fallback. InActiveBattlefield has a Mists caller and cross-profile inert registration. gcinfo is a native rilua Lua 5.1 primitive and needs a separate retail-global publication decision.

**SetCommunityID is not retired.** Its only initial test match is the 10.2.7 known-gap fixture, not a runtime caller, but 10.2.7 explicitly re-adds it. A trial retirement incorrectly treated the fixture as a removable absence gap; all-sweep RED demonstrated both expectations remained published. The trial was reverted in the next coherent commit. Original 10.2.7 ledger and gap fixture are byte-identical to the base; no other-page closure remains.

## Recorded problematic contracts

[Per-ID gap review](../../../data/patch-api/evidence/8.1.5-session-2026-10-08/p815-gap-review.json) retains each literal, expected current publication, observed lookup and closure reason for all **24** gaps.

Missing producers require Battle.net account GUID/display identity, journal encounter completion, merchant refund eligibility/expiry, remote unit realm relationships, available brawl schedule DTOs, community selection, double-state icon-row visualization data, and unit PvP classification/selection/widget-set state. Fixed local realm names, generic widget defaults or boolean shims cannot model those contracts.

Legacy reporting InitiateReportPlayer/SendReportPlayer remains used by modeled report-token flows and help/report/XML tests despite later 9.2.5 removals; migrate successors explicitly before retirement. CVar globals, CanReportPlayer, InActiveBattlefield and gcinfo retain consumer/profile/VM publication blockers. Successful publication rows—including existing compatibility defaults—receive no populated-output or native-behavior credit.

## Proof and integration

[Proof index](../../../data/patch-api/evidence/8.1.5-session-2026-10-08/p815-proof.json) and individual receipts retain command, revision, explicit worktree cwd, isolated target, custom environment, exit and complete log hash. Main thread performed verification because user prohibited agents. No push, merge, session-cwd changes, sibling-worktree mutation, vendor writes or full integration suite.

All **38 publication sweeps + factory regression pass (39/39)** across **8,241 inventory rows**. Bare retirement/toy tests pass 2/2; cached retirement 1/1; Mists classic lookup/toy transition 2/2. Targeted integration regressions: toy 29, area POI eight, calendar ten, club 21, PvP eight, reporting two, social eight, date/time one. Cached community regressions pass six/six. The date/time test is existing classification evidence, not additional behavioral parity.

Parser fixtures pass **30 extractor / 21 generator**. Format, default and Mists tests check pass with **zero non-vendor warnings**; seven iced manifest warning lines remain unsuppressed in each check. Separate retail build and bounded startup pass: exit zero, stdout `[]`, zero unique/total Lua errors. Native texture tests are not run; host has no WoW install.

Negative control changes only NEW_TOY_ADDED added → removed: **24 → 25 gaps**, exactly one new ID, zero resolved IDs, unchanged inventory identity and expected exit one. Discovery, behavioral RED and trial-retirement RED receipts remain explicitly historical; not acceptance evidence.

[Dynamic validator](../../../data/patch-api/evidence/8.1.5-session-2026-10-08/validate.py) **PASS**. Counts derive from retained files and observations. All 38 registers regenerate byte-identically; all formerly reproducible extracts remain identical. Inherited **12.0.5 / 12.0.7 / 12.1.0** extract failures remain unchanged, not hidden or fixed by this page. All 194 earlier source files and 74 default/example outcomes are preserved.

The sweep starts with a one-line **8.2.0 register placeholder**, then 8.2.5 and the remaining later master registers. Main thread must integrate the newer page first, substitute its register, reconcile any exact gap supersession, and refresh affected receipts. No 8.2.0 behavior is inferred from the unmerged branch.

## Sources

- [Specification](../../specs/patch-8-1-5-publication-sweep.md).
- [Pinned register](../../../data/patch-api/sources/8.1.5-wikitext-register.json) and [ledger](../../../data/patch-api/sources/8.1.5-page-coverage.json).
- [Evidence directory](../../../data/patch-api/evidence/8.1.5-session-2026-10-08/).
- [Binding handoff](../../../data/patch-api/evidence/handoff-laptop-to-agent-server/HANDOFF.md).
- [Collection producer documentation](../../admin-api/collections.md).

## See Also

- [[patch-8-2-5-api-audit]] — template, conservative consumer scan and proof boundaries.
