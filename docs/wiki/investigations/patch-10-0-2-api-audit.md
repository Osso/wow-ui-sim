# Patch 10.0.2 API page audit

Page 303146, revision 2926754 (January 27, 2023, 02:58:55 UTC), retrieved October 7, 2026 UTC. The requested page exists; no 10.0.0 substitution. Branch `p1002-page` starts from read-only `p1005-page` at `73f3c2834`. Default retail carries 12.1.0, not reconstructed Dragonflight launch semantics.

## Source accounting

416 inventory occurrences: 240 added, 164 removed, 12 changed. Global API 203/63/8, widgets 7/100/0, events 23/0/4, CVars 7/1/0. Every published header count matches. All 24 chronological later registers, 10.0.5 through 12.1.0, supersede this page.

583 unique source IDs: 416 inventory + 167 nonblank extract occurrences. Ledger statuses: 137 bounded-coverage, 130 partial-development-green, 305 audit-pending, 11 metadata-only. Pending comprises 149 exact publication gaps and 156 substantive extract occurrences. Bounded means current absence/deprecation identity or CVar queries only; partial green means function/event/method publication only. Neither establishes signatures, populated DTOs, event payloads, security, native parity or historical behavior. No current CVar default mismatches observed.

The parser initially rejected the unlinked removed CVar `professionGearSlotsExampleShown`. A RED/GREEN fixture proves plain CVar identity, direction, source line and header cardinality. The change accepts only a bare identifier in the CVar section; no existing register changed. The extractor is unchanged. Literal tooltip Lua examples are retained with `--preserve-examples`; own extract reproduces in that mode. Without that flag, this page's code fences differ by design; that failure is recorded, not called reproducible.

[Literal scout](../../../data/patch-api/evidence/10.0.2-session-2026-10-07/p1002-extract-scout.json) accounts for every nonblank extract line. Native-method-to-mixin migration, tuple-to-table container output, GUID token mapping, the populated player tooltip example, SurfaceArgs, removed script names, global TooltipDataProcessor callbacks, enum value/flag identities and populated DTO additions remain pending. No unrelated existing test is credited as occurrence-specific behavior proof.

## Bounded closures

Discovery 247 OK / 169 gaps. Final 267 OK / 149 gaps. Twenty unused members retired in separate `RETIRED_10_0_2_MEMBERS`: one C_ChallengeMode tooltip setter, two C_ItemInteraction tooltip setters, C_ProfSpecs.GetUnspentPointsForSkillLine, and sixteen old C_TradeSkillUI order/tools/tooltip members. The [per-ID review](../../../data/patch-api/evidence/10.0.2-session-2026-10-07/p1002-gap-review.json) enumerates every member; no whole namespace removed.

[Initial qualified/bare cached Lua searches](../../../data/patch-api/evidence/10.0.2-session-2026-10-07/p1002-removal-consumers.json) and [exact-boundary qualified searches](../../../data/patch-api/evidence/10.0.2-session-2026-10-07/p1002-exact-removal-consumers.json) cover all 164 removed occurrences. All twenty retired namespace members have zero qualified/bare matches. Initial whole `src/` and `tests/` bare-name retirement scan is empty. [Final untruncated scan](../../../data/patch-api/evidence/10.0.2-session-2026-10-07/p1002-final-retirement-whole-caller-scan.txt) finds only retirement constants and new retail/Mists assertions. Embedded Lua, pcall function references and guards are included by searching all files with `--hidden --no-ignore`; no pre-existing integration or prefork caller requires migration. A separate [all-gap caller scan](../../../data/patch-api/evidence/10.0.2-session-2026-10-07/p1002-whole-caller-scan.txt) retains 2,145 lines, not truncated tool output.

Repeated lookup passes in bare and unmodified cached Game environments; Mists preserves all twenty legacy lookups. `src/c_api/mod.rs:302-303,336-337` gates the module/call on retail-12-0-0, excluding Wrath, Mists, Era and Anniversary. No Blizzard deprecation wrapper, vendor Lua, cache file, existing model or classic registration deleted or monkey-patched.

## Retained gaps

Every discovery failure is reviewed once: twenty closed, 149 retained. [Source scans](../../../data/patch-api/evidence/10.0.2-session-2026-10-07/p1002-gap-source-scans.json) and [current declaration excerpts](../../../data/patch-api/evidence/10.0.2-session-2026-10-07/p1002-native-contracts.json) retain file:line context. Name-matched current declaration context found for 117 retained gaps; missing declarations are not evidence that a historical API never existed.

Missing tooltip-source DTOs/comparisons, customer/crafter order lifecycle, container purchase/refund/sort/action policy, profession recipe/gear/filter inputs, item GUID linking, mail-order DTOs, reward/visibility/season mappings and fractional-power/GUID-health contracts need real behavior. Existing bag-slot flags, local PlaceNewOrder/GetMyOrders and item/unit tooltip providers are not credited as the absent API. Slot-visibility queries remain in the project's intentionally unsupported 3D domain. No new placeholder introduced.

Forty-six retained failures are removal mismatches. GameTooltip native removals cannot be implemented by deleting backwards-compatible mixin calls or shared classic methods. Example cached callers: `Blizzard_ActionBar/WoWLabs/ActionButtonOverrides.lua:376` uses GameTooltip:SetAction; `Blizzard_UnitFrame/Shared/PartyMemberFrame.lua:202,206` uses aura tooltip methods; `Blizzard_SharedXMLGame/Tooltip/TooltipDataHandler.lua:532-533` maps trade-tooltip delegates. The ledger distinguishes exact method calls, unrelated bare-name matches, documentation and profile-specific cached sources; it does not call every cache match a loaded retail consumer.

Legacy container globals remain exposed. `Blizzard_BoostTutorial/Blizzard_TutorialLogic.lua:200` calls GetContainerNumSlots; other bare matches include C_Container successors and documentation rather than old-global consumers. Old C_MajorFactions.IsMajorFaction has no exact qualified cache match; bare matches are C_Reputation.IsMajorFaction successor uses (`Blizzard_ActionBar/Shared/ReputationBar.lua:62`). It is deferred outside the conservative zero-bare-match retirement batch, not falsely described as a currently consumed old name. Three unused native GameTooltip comparison methods are also deferred pending native/mixin/classic separation.

## Verification

Runtime/test revision `8f7dec416`. All 25 publication sweeps run alone through the grouped integration target and match exact fixtures. Retirement RED fails at SetKeystoneTooltip; GREEN proves all twenty repeated lookups and successor publication. Cached Game prefork and Mists preservation each execute one case. Initial discovery and post-retirement empty-fixture discovery are expected RED, not acceptance.

Negative control changes only UnitTokenFromGUID added → removed: precisely one new failure, no resolved failures, 149 → 150, expected exit 101. [Result](../../../data/patch-api/evidence/10.0.2-session-2026-10-07/p1002-negative-result.json).

Cargo fmt/fmt --check, default check, Mists check --no-default-features --features sound,gui,casc,client-mists --tests and separate retail build pass. Mists has zero non-vendor warnings; six existing iced manifest deprecations and their summary remain unsuppressed. Retail rebuilt after Mists. Startup exits zero and returns `[]`. Existing extractor/parser fixtures pass 21 + eight. Changed Rust manually audited: flat retirement constant/call, bounded assertions, flat sweep specification and cached/classic tests; no changed-line readability violations.

[Proof ledger](../../../data/patch-api/evidence/10.0.2-session-2026-10-07/p1002-proof.json) records command, cwd, exact revision/scope, exit and log hashes. Output captured once, saved, and searched with rg; no cargo rerun to recover logs. Documentation/data-only follow-ups preserve runtime proof scope. No independent agent/model acceptance was run, as prohibited by the task.

| Patch | Rows | OK | Gaps | Isolated exit |
|---|---:|---:|---:|---:|
| 10.0.2 | 416 | 267 | 149 | 0 |
| 10.0.5 | 93 | 66 | 27 | 0 |
| 10.0.7 | 70 | 44 | 26 | 0 |
| 10.1.0 | 129 | 93 | 36 | 0 |
| 10.1.5 | 101 | 69 | 32 | 0 |
| 10.1.7 | 48 | 34 | 14 | 0 |
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

All 25 registers regenerate byte-identically. [Preservation](../../../data/patch-api/evidence/10.0.2-session-2026-10-07/p1002-preservation.json) proves 152 prior source/register/ledger/gap inputs unchanged. [Before](../../../data/patch-api/evidence/10.0.2-session-2026-10-07/p1002-extract-before.json) and [after](../../../data/patch-api/evidence/10.0.2-session-2026-10-07/p1002-extract-after.json) preserve all 48 previous extract-mode outcomes; there are 50 after adding this page. Pre-existing mode-specific failures for 10.1.0, 10.1.7, 10.2.0, 10.2.5, 10.2.7, 11.0.0 and 11.0.2 remain unchanged. Both-mode pre-existing failures for 12.0.5/12.0.7/12.1.0 remain untouched.

No full suite, agents/model CLIs, push, merge, sibling target reuse or canonical working-file/vendor/cache edits. Every external command uses explicit cwd `p1002-page`; cargo uses own `--target-dir target`. Only prescribed worktree-creation Git metadata used the canonical path, with cwd the empty destination.

## Sources

- [Provenance](../../../data/patch-api/sources/10.0.2-api-changes.provenance.json)
- [Coverage ledger](../../../data/patch-api/sources/10.0.2-page-coverage.json)
- [Publication contract](../../specs/patch-10-0-2-publication-sweep.md)
- [Artifact validator](../../../data/patch-api/evidence/10.0.2-session-2026-10-07/validate.py)

## See Also

- [[patch-10-0-5-api-audit]] — base and publication/accounting template.
- [[patch-10-0-7-api-audit]] — removal and supersession proof boundaries.
- [[client-profiles]] — retail/classic gates.
