# Patch 8.2.5 API page audit

Verified October 8, 2026. Warcraft Wiki page 83383, revision 6471405 (September 13, 2025, 10:05:08 UTC), is pinned with its complete MediaWiki response. Audit resumed three stopped-agent commits on `p825-page`, rebased onto `origin/master` at `3ebc8f599`, and replaced the pending 8.3.0 supersession placeholder with the actual merged register. Current-retail publication/absence is not historical/native compatibility.

## Source accounting and series endpoint

[Occurrence ledger](../../../data/patch-api/sources/8.2.5-page-coverage.json) accounts for **216 unique IDs**: **198 inventory**, **14 extract**, **four comparison captions**. Statuses: **54 bounded-coverage**, **83 partial-development-green**, **65 audit-pending**, **14 metadata-only**. Pending rows comprise **61 publication failures and four substantive prose statements**. Published methods/events remain publication-only, not populated-output, signature, payload, security or native-parity proof.

All consolidated numerical headers match their parsed counts: global functions 103 added / 35 removed, widget methods 17 added, events 29 added / nine removed, four CVars and one command. The mixed `CVar`/`Command` category labels require opt-in `--legacy-inventory-labels`; `DefragmentGPU` stays a command, not a CVar. Inline hardware-event source citation is retained with recorded extractor flags. Both rebase sides survive: 8.3.0 legacy column/diff additions and 8.2.5 mixed labels. Rebase auto-merge duplicated `--retain-reference-notes` in argparse; a concrete CLI subprocess RED reproduced the conflict before deleting the duplicate. All **29 extractor and 20 generator fixtures pass**, and prior default/example modes are unchanged.

[Exhaustive page list](../../../data/patch-api/sources/api-change-pages-remaining.json) contains **101 pages older than 8.3.0, including this page**, ending at **Patch 1.0.0/API changes**. Two retained allpages response batches exhaust continuation. After this audit, **100 listed pages remain**, next **8.2.0**. Page inventory is not an implementation claim for those pages.

[Literal extract scout](../../../data/patch-api/evidence/8.2.5-session-2026-10-08/p825-extract-scout.json) maps each nonblank extract occurrence to its exact raw line. Hardware-event protection of SendWho, partial SendChatMessage protection, the cited channel/SAY/YELL/instance policy and broad Party Sync/RAF overhaul remain pending. Build comparison, diff URLs, deprecated-file reference and References marker are metadata, not behavior. Every raw inventory identity and table caption is separately retained.

## Coverage matrix

| Capability | Covered | Remaining | Proof boundary |
|---|---|---|---|
| Current-retail publication/absence | 137 OK / 198 identities | 61 exact reviewed gaps | Raw/ordinary lookup, registration and command/CVar probes only |
| Unused-member retirement | 14 newly absent members | Three removal candidates retained | Bare/repeated lookup and unmodified cached full-UI startup |
| Classic preservation | All 14 members reachable on Mists | Other classic profiles not executed | Existing retail-only module gate unchanged |
| Non-inventory behavior | All 14 occurrences retained | Four substantive statements pending | No hardware-event/native-security or overhaul credit |
| Reproducibility | All 37 registers; 34 saved extracts | Three inherited 12.x extract failures | Recorded flags; missing historical recipes explicitly inferred |

## Fourteen bounded closures

Discovery produced **123 OK / 75 gaps**. Fourteen consumer-free namespace retirements produce final **137 OK / 61 gaps**. `RETIRED_8_2_5_MEMBERS` uses the existing retail-only module and registration path; no global registration, classic path, vendor behavior or default shim changes.

- **C_ClubFinder:** ReportPosting, ReturnCommunityApplicantList, ReturnGuildApplicantList, ReturnPendingCommunityApplicantList, ReturnPendingGuildApplicantList.
- **C_Commentator:** GetElapsedMs, GetTrackedDefensiveCooldowns, GetTrackedOffensiveCooldowns, IsTrackedDefensiveCooldown, IsTrackedOffensiveCooldown.
- **C_PvP:** GetMatchPVPStatIDs.
- **C_RecruitAFriend:** CheckEmailEnabled, IsSendingEnabled, SendRecruit.

[Pre-change scans](../../../data/patch-api/evidence/8.2.5-session-2026-10-08/p825-removal-consumers.json) and [final whole-caller scans](../../../data/patch-api/evidence/8.2.5-session-2026-10-08/p825-whole-callers-after.json) retain complete qualified whole-word, bare-name and `src/`/`tests/` outputs—including `pcall(Name, ...)` and conditional references. Host lacks rg: `/usr/bin/grep -rnE` substitutes with `\b`, excluding Documentation files and directories in the final scan. All fourteen retirements have **zero qualified/bare cached consumers**. Only ReportPosting's old 9.2.5 gap fixture matched the initial source/test scan; other candidates had no runtime caller. New test/retirement declarations appear in final scans, not historical consumer evidence.

ReportPosting also closes `wt-global-api-C_ClubFinder.ReportPosting-117` in the **9.2.5** sweep: **34 → 33 gaps**. Only that older ledger row changes from pending to bounded retail absence; original 9.2.5 proof artifacts remain historical, with this follow-up linked from its audit/spec.

Retained removals:

- **C_SpellBook.IsSpellDisabled:** `Blizzard_Deprecated/11_0_0_SpellBookAPITransitionGuide.lua:130` contains a transition-alias assignment to C_Spell.IsSpellDisabled. Preserve vendor code and exact absence gap; the scan does not establish a raw runtime producer.
- **LeaveParty:** cached Classic LFGUtil calls the global; other bare matches are C_PartyInfo/C_WoWLabsMatchmaking members. Existing simulator group implementation and callers remain. Any cached consumer prohibits retirement; active-mainline global use is not inferred.
- **BNGetFriendInfo:** zero cached consumers, but cross-profile inert-global registration and calls in inert_global_defaults/system_api_seeded remain. A profile-safe registration/test migration is not unused-member cleanup.

## Recorded problematic gaps

[Per-ID review](../../../data/patch-api/evidence/8.2.5-session-2026-10-08/p825-gap-review.json) records all **61 exact IDs**, raw source literals, current expectations/observations and specific closure requirements. **No new missing producer is modeled by this audit**; the implemented runtime behavior is fourteen bounded absence corrections. Meaningful producers cannot be supplied by constants or namespace autostubs.

Battle.net gaps require populated account/friend/game-account identity indexes; calendar GUID removal requires invite ownership/mapping; ClubFinder requires selected focus flags and recruiting DTO identity. Commentator gaps require tracked/indirect spell mappings, spectator aura/cooldown snapshots and match clock/request lifecycle. LFG/PvP require active-context difficulty and ordered statistic-column DTOs. Party confirmations/invitations/conversions require request/consent/eligibility and coherent roster events, not immediate joins. Quest difficulty/triviality/replay recency require quest metadata, scaling and timestamps; QuestSession requires participant consent, pending commands and session-owned tracking. RAF claims/link/refresh/removal require reward, token and relationship producers; social mute is not voice mute; capture-zone widgets need their native payload. Retail GetExpansionForLevel cannot borrow Mists fixed pre-squish thresholds. DefragmentGPU has no native simulator GPU-defragmentation operation. Eight missing ModelSceneActor methods belong to intentionally unsupported native 3D behavior.

## Verification and proof ledger

[Proof index](../../../data/patch-api/evidence/8.2.5-session-2026-10-08/p825-proof.json) and per-command `*.proof.json` retain exact command, revision/scope, cwd, dedicated target, custom environment, exit and complete log hash. Current Rust/runtime proof remains valid after evidence/docs-only changes; no redundant broad test reruns. Every command uses explicit `/home/osso/.worktrees/wow-ui-sim-p825-page` cwd and `/home/osso/.cache/wow-ui-sim-targets/p825-page`. No session cwd tool, agent/model CLI, push or merge.

- All **37 publication sweeps plus animation factory regression pass (38/38)** over **8,175 inventory rows**. New cached retirement/sweep pass (2/2); bare absence and Mists legacy lookup pass (1/1 each).
- Scoped integration regressions: Club probes **21**, commentator API **one**, RAF surface **two**, PvP info/zone **eight**. Scoped prefork regressions: commentator **seven**, RAF **ten**, communities **six**; all pass. Initial `pvp_scoreboard` filter selected zero tests and is explicitly calibration-only, replaced by the actual PvP info scope.
- Negative control mutates only BNET_REQUEST_INVITE_CONFIRMATION added → removed: **61 → 62 gaps**, exactly one new identity, zero resolved identities, unchanged inventory IDs, expected exit one. Discovery and cached retirement RED failures remain labeled; not reported as acceptance failures.
- Format/default/Mists checks pass, **zero non-vendor warnings**; inherited iced manifest deprecations remain unsuppressed. Separate retail build and bounded startup exit zero, stdout **`[]`**. Changed-Rust readability review is manual: metric CLI absent; no changed-line violations found.
- [Dynamic validator](../../../data/patch-api/evidence/8.2.5-session-2026-10-08/validate.py) passes. Counts derive from current retained sources, fixtures and observations; no fixed integration-count assertions.

All **37 registers regenerate byte-identically**. All previously reproducible extracts reproduce; **12.0.5/12.0.7/12.1.0** remain inherited failures. **72 prior default/example mode outcomes** are unchanged. [Input preservation](../../../data/patch-api/evidence/8.2.5-session-2026-10-08/p825-input-preservation.json) accounts for **188 prior/rebased inputs**: the only after-rebase change is the single authorized 9.2.5 ledger closure; the newly merged 8.3.0 inputs are unchanged. Original pre-rebase hashes are retained, not overwritten.

Audit accounting and requested targeted gates are complete. The **61 publication gaps, four prose contracts, unsupported/native-parity boundaries and three inherited extract failures remain explicitly unfinished compatibility work**. No full integration suite or known environment-failing CASC texture tests were run.

## Sources

- [Pinned raw](../../../data/patch-api/sources/8.2.5-api-changes.wikitext), [extract](../../../data/patch-api/sources/8.2.5-api-changes.txt), [register](../../../data/patch-api/sources/8.2.5-wikitext-register.json), [provenance](../../../data/patch-api/sources/8.2.5-api-changes.provenance.json).
- [Specification](../../specs/patch-8-2-5-publication-sweep.md).
- [8.3.0 template](patch-8-3-0-api-audit.md), [9.2.5 follow-up](patch-9-2-5-api-audit.md).

## See Also

- [[patch-8-3-0-api-audit]] — pinned source, reproduction and retirement proof boundaries.
- [[patch-9-2-5-api-audit]] — older ReportPosting gap closed by this retirement.
