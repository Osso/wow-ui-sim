# Retail Patch 5.3.0 API audit

Pinned pageid 423274 revision 4065122 and separately pinned transcluded diff pageid 330804 revision 3188422. Parent pageid 38992 revision 6611399 states `|toc = 50300`, `|Release = May 21, 2013` and build 16977. Diff captions compare 5.2.0.16650 → 5.3.0.16965. This is the 2013 retail Mists line, not Classic 5.5.x; neither redirect nor stub.

## Source and parser coverage

The main page describes loot specialization, stable expansion, embedded support browsing, trade-link encoding and an InterfaceOptions ordering bug. Its diff is a separate source, not silently expanded HTML. Original responses and both raw pages are retained in the [session evidence](../../../data/patch-api/evidence/5.3.0-session-2026-10-08/).

The register contains 44 occurrences: 15 global APIs, 3 FrameXML functions, 9 events, 14 widget methods and 3 widget handlers. Every table header count matches the parsed count. `parse_mists_automated_diff` and its opt-in dispatch are copied byte-identically from p542-page at a016e9393943de703ae070d8ce33b5497113c3fc; `--mists-diff` dispatch comes byte-identically from p540-page at 9698f09862b6cd91b4f87e840295a3fce304940a. New `parse_mists_widget_handlers`/`--mists-widget-handlers` behavior is additive and opt-in. Handler symbols retain Browser ownership and `widget-script` kind.

Queued retail placeholders start `later_registers` in order: 5.4.0, 5.4.1, 5.4.2, followed by real 5.4.7, 5.4.8, 6.0.1 and subsequent retail registers. Classic registers are excluded.

## Capability coverage

| Contract | Proof level | Limit |
|---|---|---|
| Every inventory identity | Register/header reproduction; all publication cases pass | Publication is not historical output/security parity |
| GetPVPRoles / SetPVPRoles | New bare/cached tests assert stored tank/dps, healer-only and all-false transitions | Existing shared lfg_roles backing; not historical role-update event production |
| PrepVoidStorageForTransmogrify / UNIT_DYNAMIC_FLAGS | Whole-word consumer/caller scans; bare/cached absence assertions pass | Already absent; no runtime retirement added |
| Browser engine, support ticket/survey and history | Problematic native-service domain | Generic frame methods/inert navigation cannot establish a browser engine |
| Loot award policy, legacy upgrades/tracking/battleground metadata | Exact publication gaps reviewed; prose contracts pending | No fabricated default values or aliases introduced |
| Stable pages, trade-link field migration, historical InterfaceOptions bug | Pending source contracts | No historical fixtures; current client Settings is not the 2013 panel list |

No runtime source changed. No Blizzard/vendor/Wowless/WowlessData modifications or monkey-patches. Existing temporary defaults are labeled publication-only, never credited as modeled domain behavior.

## Row accounting

All 60 IDs are accounted for: 44 inventory occurrences, 11 main-page extract rows and 5 diff captions. Current discovery has 29 bounded publication/absence rows, 21 pending rows (15 publication gaps plus 6 prose contracts), and 10 metadata-only rows. [Ledger](../../../data/patch-api/sources/5.3.0-page-coverage.json) separates publication from modeled behavior.

| Publication gaps | Precise missing backing |
|---|---|
| AcknowledgeSurvey; HideKnowledgeBase; ShowKnowledgeBase | Historical support/survey/knowledge-base service and embedded-browser integration |
| GetBattlegroundPoints; GetLFGRoleUpdateBattlegroundInfo | Historical team-score/objective producer and role-update invitation metadata |
| SetLootSpecialization | Loot-selection state/award policy; constant-zero getter is not that system |
| Browser CopyExternalLink, DeleteCookies, NavigateBack/Forward/Reload/Stop, OpenTicket | Native browser history, cookies, clipboard/external-link and ticket integration |
| Browser OnEditFocusGained / OnEditFocusLost | Browser focus lifecycle is unsupported; shared callable focus methods do not provide these scripts |

The six prose limits are loot award policy, historical five-page/fifty-slot stable layout, support website integration, Browser domain, trade-link/Cooking field encoding and the old InterfaceOptions list-ordering bug. The transclusion pointer is metadata because the actual diff is separately inventoried. No missing historical contract is claimed resolved by a plausible default.

## Retirement evidence

GNU `/usr/bin/grep -RInwF` scans retain untruncated qualified and bare-name results. Cached retail AddOns scans exclude `*Documentation*` files/directories. Both removed identities had zero cached consumers and zero prior src/tests references. Final caller scans include only this audit's explicit absence assertions. The bare-name scan includes direct calls, `pcall(Name, ...)` and conditional references.

`git ls-tree`/`git show` scans at base 896086537 and pinned p542/p541/p540 revisions find no re-addition in the retail chain. No new retirement action was necessary; existing absence remains the target.

## Verification

Source reproduction: 59 registers byte-identical; 56 extracts match, with the exact inherited 12.0.5/12.0.7/12.1.0 failures preserved. All six `tools/test_*.py` scripts pass (89 fixtures). Targeted acceptance passes: 58 publication cases, one cached and one bare behavior case, one existing PvP startup unit test, warning-clean non-vendor Mists check and format. Negative control changes only GetPVPRoles to a missing name and raises the non-ok set from 15 to 16; every other observation is identical. Own fresh-checkout/later-audit validator gate is the remaining audit gate. Master baseline gate passed 43/43 clean and 44/44 later (including its synthetic audit).

The initial cold-target launch lacked an output path and preceded the first commit, so it is diagnostic only. Committed-input discovery retains every result. Long commands stream to logs and run asynchronously; no full integration suite is run.

## Sources

- [Source provenance](../../../data/patch-api/sources/5.3.0-api-changes.provenance.json).
- [Pinned register](../../../data/patch-api/sources/5.3.0-wikitext-register.json).
- [Publication spec](../../specs/patch-5-3-0-publication-sweep.md).
- [Parser origin](../../../data/patch-api/evidence/5.3.0-session-2026-10-08/parser-provenance.json).
- [Validator portability](patch-audit-validator-portability.md).

## See Also

- [[patch-5-4-7-api-audit]] — next merged retail register.
- [[patch-6-0-2-api-audit]] — later scenario bonus API supersession.
- [[patch-audit-validator-portability]] — historical proof gate.
