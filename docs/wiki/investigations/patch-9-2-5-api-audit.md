# Patch 9.2.5 API page audit

Page 237255, revision 2301036 (September 3, 2022, 02:10:47 UTC), retrieved October 7, 2026. Requested page exists. Branch `p925-page` starts at master `572a77d84`, containing the 9.2.7 sweep. Current retail carries 12.1.0; historical Shadowlands behavior is not reconstructed. Direct HTTP returned 403; existing Chromium successfully returned the MediaWiki revision response.

## Source accounting

84 inventory occurrences: 55 added, 18 removed, 11 changed. Global API 44/16/11, widgets 1/0/0, events 7/2/0, CVars 3/0/0. All published counts match. All 27 later registers, 9.2.7 through 12.1.0, supersede this page.

[Ledger](../../../data/patch-api/sources/9.2.5-page-coverage.json) accounts for 220 unique IDs: 84 inventory, 135 nonblank extract and one historical build-caption context. Statuses: 33 bounded-coverage, 17 partial-development-green, 154 audit-pending, 16 metadata-only. The 154 pending IDs comprise 34 exact publication gaps and 120 substantive extract occurrences. Seventeen partially covered inventory rows retain explicit annotation, payload, creating-source or historical-default boundaries. Publication credit is not full behavior or compatibility credit.

[Literal scout](../../../data/patch-api/evidence/9.2.5-session-2026-10-07/p925-extract-scout.json) records every extract ID, literal and exact raw line. UnitPopup examples and warning, cross-faction policy, filter dropdowns, recent-player autocomplete, hidden-aura exclusion, historical UNIT_AURA fields, enum identities/values/aliases and populated calendar/club/LFG/widget DTO changes remain pending. Example fences and editorial labels have no runtime credit. No monkey-patching of Blizzard menu behavior or placeholder API added.

## Recovered source boundaries

Fixture-backed changes preserve the multiline Ambox security warning, recover the unheaded `Structures` section, and stop its fields being appended to GetCategoryAppearances' annotation. Register separation is opt-in, so existing registers remain unchanged. Hidden-span CVar defaults are retained by another fixture-backed opt-in. Verbatim Lua examples require `--preserve-examples`; plain mode deliberately differs and is not called reproducible.

Two historical CVar defaults differ: cameraFov `90` versus current serialized `90.000000`; enableSourceLocationLookup `0` versus current `1`. Neither is rewritten to a historical value. Region:GetSourceLocation is published, but `src/lua_api/frame/methods/core_state/region.rs:101-133` returns owner-addon folder, not exact creating script and line. Existing folder tests do not satisfy that page statement.

## Closures and retained gaps

Initial sweep: 41 OK / 43 gaps. Nine unused namespace members retired, yielding 50 OK / 34 gaps:

- C_Calendar.ContextMenuEventComplain.
- C_Cursor.DropCursorCommunitiesStream, GetCursorCommunitiesStream and SetCursorCommunitiesStream.
- C_LFGList.ReportSearchResult.
- C_ReportSystem.OpenReportPlayerDialog, SetPendingReportPetTarget, SetPendingReportTargetByGuid and SetPendingReportTarget.

[Qualified and bare cached scans](../../../data/patch-api/evidence/9.2.5-session-2026-10-07/p925-removal-consumers.json) have zero matches for all nine. Separate const in the existing retail-only retirement module prevents raw/ordinary/repeated lookup fabrication; classic profiles do not load that module. Mists legacy lookup is tested directly. No Blizzard deprecation wrapper deleted.

Live C_ReportSystem.InitiateReportPlayer remains: `Blizzard_HelpFrame/HelpFrame.lua:146`, `Blizzard_StaticPopup_Game/Mainline/GameDialogDefs.lua:123,139`. SendReportPlayer remains for `GameDialogDefs.lua:126,142`. ComplainInboxItem's cached deprecation behavior is preserved. Bare-name successor/deprecation references for IsPlayerInGuildFromGUID, ReportPosting, IsHeirloomSourceValid and ReportApplicant are retained in the scan; unresolved old namespace fabrication stays an exact gap, not falsely described as proven native publication.

[Per-ID gap review](../../../data/patch-api/evidence/9.2.5-session-2026-10-07/p925-gap-review.json) assigns precise boundaries to all 34 failures: missing auction list DTOs, club ticket/settings/faction behavior, ignored identity lookup, SDL mapping/power telemetry, collection-filter aggregates/mutations, limited-currency and modified-instance metadata, cross-faction eligibility, report catalogs/submission, transmog category/base-set state, weekly-reward interaction and LFG title policy. Existing dynamic `raw=nil; lookup=function` results receive no publication credit.

[Whole-tree scan](../../../data/patch-api/evidence/9.2.5-session-2026-10-07/p925-whole-caller-scan.txt) retains 85 initial matches from all src/tests, including embedded Lua, function references and guards. Retirement-specific bare scan has two unrelated SetPendingReportTargetFromUnit matches in pet-battle code/tests, not callers of the retired ReportSystem identity. No pre-existing caller of the nine changed identities requires migration or classic-only test gating.

## Verification

Runtime/test acceptance revision `ad7419b54`; documentation/data-only follow-up does not invalidate compiled proof. All 28 publication sweeps plus existing animation factory regression pass: 29/29 cases, 6,463 inventory occurrences. Nine-member raw/repeated lookup RED/GREEN, cached full-UI retirement case, two existing retained-report token/event regressions and Mists legacy lookup pass.

Negative control changes only GAME_PAD_POWER_CHANGED added to removed. Exactly 34 → 35 gaps, one new failure, zero resolved failures, unchanged 84 IDs, expected prefork exit 1. [Exact result](../../../data/patch-api/evidence/9.2.5-session-2026-10-07/p925-negative-result.json).

All 38 parser/extractor fixtures pass. cargo fmt/fmt --check, default cargo check, Mists `cargo check --no-default-features --features sound,gui,casc,client-mists --tests`, separate retail build and bounded startup pass. Mists has zero non-vendor warnings; six pre-existing iced manifest deprecations plus summary remain unsuppressed. Retail startup exits zero and prints `[]`.

[Proof ledger](../../../data/patch-api/evidence/9.2.5-session-2026-10-07/p925-proof.json) retains commands, exact revision/scope, outputs/hashes, exits and invalidation state. Expensive commands are captured once, never rerun to recover logs. Rust readability metrics/manual macro review retained; registration cognitive complexity 0, helpers at most 1. Existing fallible-registration path count remains a recorded registry-shape finding rather than scope for unrelated refactoring. Verification ran directly as explicitly requested; no agents/models, push or merge.

## Sweep table

Publication-only results; exact known-gap fixtures match each passing case. Native behavior parity remains separate.

| Patch | Rows | OK | Gaps | Result |
|---|---:|---:|---:|---|
| 9.2.5 | 84 | 50 | 34 | pass |
| 9.2.7 | 3 | 3 | 0 | pass |
| 10.0.0 | 639 | 466 | 173 | pass |
| 10.0.2 | 416 | 267 | 149 | pass |
| 10.0.5 | 93 | 66 | 27 | pass |
| 10.0.7 | 70 | 44 | 26 | pass |
| 10.1.0 | 129 | 93 | 36 | pass |
| 10.1.5 | 101 | 69 | 32 | pass |
| 10.1.7 | 48 | 34 | 14 | pass |
| 10.2.0 | 150 | 120 | 30 | pass |
| 10.2.5 | 59 | 45 | 14 | pass |
| 10.2.6 | 220 | 200 | 20 | pass |
| 10.2.7 | 104 | 68 | 36 | pass |
| 11.0.0 | 495 | 329 | 166 | pass |
| 11.0.2 | 34 | 22 | 12 | pass |
| 11.0.5 | 48 | 38 | 10 | pass |
| 11.0.7 | 98 | 70 | 28 | pass |
| 11.1.0 | 116 | 97 | 19 | pass |
| 11.1.5 | 125 | 89 | 36 | pass |
| 11.1.7 | 48 | 40 | 8 | pass |
| 11.2.0 | 162 | 135 | 27 | pass |
| 11.2.5 | 163 | 118 | 45 | pass |
| 11.2.7 | 508 | 414 | 94 | pass |
| 12.0.0 | 1010 | 989 | 21 | pass |
| 12.0.1 | 225 | 222 | 3 | pass |
| 12.0.5 | 363 | 352 | 11 | pass |
| 12.0.7 | 174 | 171 | 3 | pass |
| 12.1.0 | 778 | 773 | 5 | pass |

## Preservation and path incident

All 143 pre-existing source/register/ledger inputs remain byte-identical. All 54 prior extraction-mode exits/stdout are unchanged; fifteen existing failures remain, including both modes for 12.0.5/12.0.7/12.1.0. Traceback line numbers changed with tool edits, not outcomes. Own page reproduces with preserved examples and intentionally differs without them.

All commands explicitly used this worktree cwd and its own target directory. Initial Pyrun relative file writes nevertheless resolved against the harness's canonical cwd despite os.chdir; only newly created 9.2.5 artifacts were affected. They were moved immediately into the authorized worktree, canonical git status verified clean, harness cwd explicitly corrected, and later writes made absolute. No canonical tracked file edited. This was a path-scope violation, not compliant isolation; [incident](../../../data/patch-api/evidence/9.2.5-session-2026-10-07/p925-path-incident.md) is retained. No sibling worktree, vendor, Wowless or Blizzard cache files changed.

[Artifact validator](../../../data/patch-api/evidence/9.2.5-session-2026-10-07/validate.py) checks source reproduction, all IDs/literals/hashes, precise gaps/closures, negative control, preserved inputs/modes, sweep receipts and Mists warning boundary without runtime reruns.

## Sources

- [Pinned API response](../../../data/patch-api/evidence/9.2.5-session-2026-10-07/p925-fetch.json) — exact revision and wikitext.
- [Provenance](../../../data/patch-api/sources/9.2.5-api-changes.provenance.json) — retrieval and generation flags.
- [Register](../../../data/patch-api/sources/9.2.5-wikitext-register.json) — inventory identities, defaults and annotations.
- [Spec](../../specs/patch-9-2-5-publication-sweep.md) — required proof and exclusions.

## See Also

- [[patch-9-2-7-api-audit]] — next chronological register and accounting template.
- [[patch-10-0-0-api-audit]] — publication/proof boundaries and deprecated alias handling.
