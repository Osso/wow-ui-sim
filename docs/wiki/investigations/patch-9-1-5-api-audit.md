# Patch 9.1.5 API page audit

Page 219137, revision 5920444 (January 1, 2024, 01:31:49 UTC), retrieved October 7, 2026. Branch `p915-page` starts at master `c3c3c8036`, containing the 9.2.5 sweep. Current retail carries 12.1.0; historical reconstruction is excluded.

## Source accounting

169 inventory occurrences: 109 added, 60 removed; no changed inventory. Global API 70/46, widgets 8/0, events 8/9, CVars including commands 23/5. Every published header count matches. Register provenance records `--expand-shared-changes --capture-span-defaults` (plain inline Commands labels are recognized by default since the 9.2.0 integration; the opt-in flag was removed at integration). Plain inline Commands recognition reuses p920-page's markup interpretation, behind a new opt-in flag so prior registers do not change. Twenty-eight master later registers supersede; a one-line 9.2.0 integration placeholder precedes the later-register list.

The unchanged extractor retains 18 nonblank occurrences: ten metadata and eight substantive pending statements. [Ledger](../../../data/patch-api/sources/9.1.5-page-coverage.json) accounts for all 188 unique IDs: 169 inventory, 18 extract and one historical build-caption context. Statuses: 111 bounded-coverage, eleven partial-development-green, 55 audit-pending and eleven metadata-only. Publication/absence is not signature, DTO, security or runtime policy proof. Initial sweep is 111 OK / 58 gaps; final is 122 OK / 47 gaps.

[Literal extract scout](../../../data/patch-api/evidence/9.1.5-session-2026-10-07/p915-extract-scout.json) assigns precise boundaries to tooltip inheritance, guild-bank mixin migration, legacy guild UI removal, Communities guild finder, hidden event trace logging, item-upgrade migration, hardware-event cursor deletion budget and NOSCRIPT protection. No static or unrelated test receives behavior credit. Two serialized historical CVar defaults differ: CameraFollowGamepadAdjustDelay and CameraFollowGamepadAdjustEaseIn `1.0` versus current `1.000000`. TTSUseCharacterSettings has neither value nor default (page default `1`); that remains an exact gap. Historical defaults are not overwritten.

## Bounded retirements

Eleven removed namespace members selected after qualified and bare-name cached Lua scans:

- C_Commentator.SetBlacklistedAuras and SetBlacklistedCooldowns.
- C_ItemUpgrade.GetItemLevelIncrement; C_LFGList.GetActivityInfo and GetCategoryInfo.
- C_LFGuildInfo.GetRecruitingGuildTabardInfo; C_PlayerMentorship.GetMentorOptionalAchievementIDs.
- C_Soulbinds.GetConduitChargesCapacity, GetConduitCharges, GetTotalConduitChargesPendingInSoulbind and GetTotalConduitChargesPending.

GetActivityInfo substring hits are GetActivityInfoTable; exact word-boundary scans have zero matches. GetCategoryInfo bare hits belong to other namespaces/global achievement API, not C_LFGList. No actual retired endpoint caller appears in the whole src/tests scan; no caller migration is needed. Classic profiles do not load the retirement module. Blizzard deprecation wrappers and current successor APIs remain untouched.

Cached full-UI RED reproduces fabricated C_Commentator.SetBlacklistedAuras; GREEN passes all eleven raw/ordinary/repeated lookup assertions after unmodified full UI preload. Bare integration and Mists legacy lookup pass. Eleven publication gaps closed; no modeled added API implementation or placeholder introduced.

[Per-ID review](../../../data/patch-api/evidence/9.1.5-session-2026-10-07/p915-gap-review.json) assigns all 47 remaining gaps to exact missing endpoint/model contracts. These include challenge score/map data, club focus state, commentator filters, gamepad vibration, item/upgrade metadata and transactions, LFG eligibility/keystone/title policy, mount filter mutation, MythicPlus season/keystone state, mentorship requirements, regional chat restrictions, TTS preferences, transmog source/visibility identity and widget processing GUID context. Three console commands are absent; four getters remain intentionally unsupported 3D/transmog model gaps. Dynamic `raw=nil; lookup=function` receives no native publication credit.

## Verification

Compiled runtime/test acceptance revision `7d4d2588d`; subsequent data/docs and retirement-comment changes do not invalidate runtime proof. All 29 publication sweeps plus animation-owner factory regression pass: 30/30 cases, 6,632 inventory occurrences. Own sweep: 169 rows / 122 OK / 47 exact reviewed gaps. Bare retail integration, cached-prefork repeated lookup and Mists legacy assertions pass; no existing callers of changed identities require migration.

Negative control changes only GAME_PAD_ACTIVE_CHANGED added → removed. Exactly 47 → 48 failures, one new ID, none resolved, unchanged 169 IDs and expected prefork exit 1. [Receipt](../../../data/patch-api/evidence/9.1.5-session-2026-10-07/p915-negative-result.json).

All 39 extractor/register fixtures pass. cargo fmt/fmt --check, default cargo check, Mists `cargo check --no-default-features --features sound,gui,casc,client-mists --tests`, separate retail build and bounded startup pass. Mists has zero non-vendor warnings; six pre-existing iced manifest deprecations and summary remain unsuppressed. Retail startup exits zero and prints `[]`.

[Proof ledger](../../../data/patch-api/evidence/9.1.5-session-2026-10-07/p915-proof.json) retains exact revisions/scopes, commands, output hashes, exits and invalidation state. All commands used this worktree cwd and its own target; no sibling/canonical working-file edits, cache/vendor/Wowless writes, agents/models, push or merge. Verification ran directly as requested. Changed Rust manually reviewed; input-only sweep specification, flat member data and named registration calls add no nested logic or suppressions. Existing fallible-registration length/path count remains an inherited registry-shape finding, not authorization for refactoring.

## Preservation and 9.2.0 integration

All 148 prior source/register/ledger files remain byte-identical. All 56 prior extraction-mode exit/stdout outcomes are unchanged; sixteen existing failures remain, including both modes for 12.0.5/12.0.7/12.1.0. Own extract reproduces in both modes. All 29 registers regenerate byte-identically. Most historical provenance files omit generator_flags; [reproduction receipt](../../../data/patch-api/evidence/9.1.5-session-2026-10-07/p915-register-reproduction.json) explicitly separates recorded flags from verified inferred flags instead of modifying prior provenance.

Read-only p920-page tools were consulted before implementation. That worktree later became unavailable; the register is now readable from master. [Snapshot comparison](../../../data/patch-api/evidence/9.1.5-session-2026-10-07/p915-920-supersessions.json) finds no 9.2.0 inventory identity intersecting any of the 47 retained gap IDs. No predicted gap supersession. The requested one-line later-register placeholder remains for the integrating main thread; this branch does not merge or rewrite master.

## Sweep table

Publication/absence only; exact known-gap fixtures match each passing case.

| Patch | Rows | OK | Gaps | Result |
|---|---:|---:|---:|---|
| 9.1.5 | 169 | 122 | 47 | pass |
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


[Artifact validator](../../../data/patch-api/evidence/9.1.5-session-2026-10-07/validate.py) passes: all 188 unique IDs, exact literals/defaults/provenance, eleven closures, 47 precise gaps, negative control, preserved hashes/modes/registers, 29 sweep receipts and non-vendor warning boundary. [Acceptance receipt](../../../data/patch-api/evidence/9.1.5-session-2026-10-07/p915-artifact-acceptance.json).

## Sources

- [Pinned API response](../../../data/patch-api/evidence/9.1.5-session-2026-10-07/p915-fetch.json).
- [Register](../../../data/patch-api/sources/9.1.5-wikitext-register.json).
- [Qualified/bare scan](../../../data/patch-api/evidence/9.1.5-session-2026-10-07/p915-removal-consumers.json).
- [Exact-name scan](../../../data/patch-api/evidence/9.1.5-session-2026-10-07/p915-removal-exact-consumers.json).
- [Spec](../../specs/patch-9-1-5-publication-sweep.md).

## See Also

- [[patch-9-2-5-api-audit]] — accounting and later supersession.
- [[patch-9-2-7-api-audit]] — publication-only proof conventions.
