# Patch 9.0.1 API page audit

Page 229972, revision 6471353 (September 13, 2025, 09:42:38 UTC), retrieved October 8, 2026. Branch `p901-page` starts from master `97578f102`; current retail carries 12.1.0. Historical client reconstruction is excluded.

## Source accounting

755 inventory occurrences: Global API 413 added / 125 removed-renamed; widgets 22 methods plus three scripts added / eight removed; events 75 added / eight removed; CVars 88 variables plus eight commands added / five removed. All eight numeric headers match their corresponding parsed inventories; [literal header accounting](../../../data/patch-api/evidence/9.0.1-session-2026-10-08/p901-header-accounting.json) preserves split method/script and variable/command counts. Existing generator handles every reference, command label and script link with its existing `--skip-plain-scripts-label`. No new generator behavior or flags. Provenance records that flag explicitly.

The extractor initially swallowed the deprecated 8.x table because spaced `== Global API ==` headings were not recognized and its unheaded-table heuristic started inventory at the earlier deprecated table. A minimal serialized-output fixture reproduces the loss. New opt-in `--normalize-inventory-headings` recognizes spaced inventory headings and disables that heuristic when a real Global API heading exists. Defaults remain unchanged for every earlier source. `--preserve-examples` retains the 22 bag-constant Lua lines verbatim with fences. Two coherent capture/follow-up commits preserve the initial incomplete extract and its correction; the final source is complete.

135 nonblank extract occurrences retained, including summary behavior, deprecated identity/successor lines, bag-constant removal prose and examples, and editorial scaffolding. [Scout](../../../data/patch-api/evidence/9.0.1-session-2026-10-08/p901-extract-scout.json) maps every extract occurrence to exact raw and extract lines: 113 substantive targets remain audit-pending; 22 context/headings/table/fence/resource rows are metadata-only. Four historical comparison captions have separate [build-context IDs](../../../data/patch-api/evidence/9.0.1-session-2026-10-08/p901-build-context.json). Linked external pages and transitive native behavior are not expanded.

[Coverage ledger](../../../data/patch-api/sources/9.0.1-page-coverage.json) accounts for all **894 unique IDs**: **147 bounded-coverage**, **335 partial-development-green**, **386 audit-pending**, **26 metadata-only**. Bounded rows prove current absence only, including later supersessions and cached deprecation fallback attribution under the shared probe. Partial rows prove current publication/event registration, not signatures, populated outputs, security, payloads, policies or native behavior parity. Pending rows comprise 273 exact publication gaps and 113 substantive extract targets.

## Bounded retirements and retained gaps

Initial cached sweep: 471 OK / 284 gaps. Eleven bounded namespace retirements yield **482 OK / 273 exact retained gaps**. Both namespace-qualified whole-word and bare-member current cached retail Lua searches, plus whole `src/` and `tests/` scans, have zero pre-existing consumers/callers for every retired member. [Consumer receipt](../../../data/patch-api/evidence/9.0.1-session-2026-10-08/p901-removal-consumers.json) retains full, untruncated output. [After-change scan](../../../data/patch-api/evidence/9.0.1-session-2026-10-08/p901-whole-callers-after.txt) finds only the new list and behavioral assertions; no existing callers need migration.

Separate `RETIRED_9_0_1_MEMBERS` retires two old C_CampaignInfo current-campaign getters; C_Commentator.GetTeamHighlightColor; C_Garrison.GetMissionInfo and GetTalentTreeInfoForID; two C_GossipInfo gossip-POI getters; C_Item.IsItemCorruptable; C_LootJournal.GetFilteredItemSets; C_MountJournal.IsMountEquipmentUnlocked; and C_TransmogCollection.GetAppearanceSourceInfoForTransmog. Existing `retail-12-0-0` module/registration gates exclude all classic profiles. Header patch list updated. No vendor, Blizzard cache, Wowless, placeholder or deprecation-wrapper edit; no 3D simulation introduced.

[Per-ID gap review](../../../data/patch-api/evidence/9.0.1-session-2026-10-08/p901-gap-review.json) preserves literal lines, observations and precise model/input/lifecycle boundaries for all 273 gaps. Added APIs require real campaign/covenant/soulbind/runeforge/adventure, map/quest/tracking, commentator/device/input, reward/service or typed visualization producers. Lazy namespace `raw=nil; lookup=function` receives no producer credit. Unsupported particle scaling remains a permanent 3D gap. Gamepad button-down/up scripts lack supported dispatch, not merely publication entries. The two VRS CVars have no values/defaults; the page supplies no defaults, so no guessed values are seeded. Eight console Command records are absent; catalog-only inventions would not implement their device/graphics/screenshot/diagnostic execution.

Removed members/globals with current cached Lua bare-name/qualified matches or existing simulator callers remain reachable gaps, each citing file:line in the review and full scan receipt. Bare names may denote successors or unrelated namespaces; qualified identities are distinguished, not falsely called live old-namespace consumers. Larger quest/watch/caller migrations are not treated as unused retirement cleanup. Existing deprecation wrappers remain intact. No runtime additions or speculative placeholder fixes.

## Verification

Runtime/test acceptance revision **cf079f5a6**; later evidence/docs-only work does not invalidate that scope. All **33 publication sweeps plus animation factory regression pass (34/34)**, covering **7,674 inventory occurrences**. Own 755-row sweep: 482 OK / 273 exact gaps. Bare repeated lookup, cached full-UI absence and Mists legacy lookup tests pass for every changed member. No pre-existing callers; all affected tests covered. Wrath/Era/Anniversary preserved by the shared profile gate, not claimed executed.

Cached retirement RED fails on fabricated C_CampaignInfo.GetCurrentCampaignChapterID after unmodified Blizzard preload; GREEN asserts all eleven raw/ordinary/repeated absences and no added Lua errors. Initial empty-fixture discovery and historical REDs are not acceptance claims. New extractor fixture fails on unsupported opt-in flag before implementation, then passes corrected deprecated retention.

Negative control flips only CURSOR_CHANGED added → removed: **273 → 274 failures**, exactly one added ID, zero resolved IDs, unchanged 755 IDs, expected prefork exit one. [Receipt](../../../data/patch-api/evidence/9.0.1-session-2026-10-08/p901-negative-result.json).

`cargo fmt`, `cargo fmt --check`, default `cargo check`, Mists `cargo check --no-default-features --features sound,gui,casc,client-mists --tests`, separate retail build and bounded startup pass. Mists has **zero non-vendor warnings**; six inherited iced manifest deprecations and their summary remain unsuppressed. Rebuilt retail startup exits zero and prints **`[]`**. All **42** generator/extractor fixtures pass. [Changed-Rust readability](../../../data/patch-api/evidence/9.0.1-session-2026-10-08/p901-readability.md) records additive flat data and explicit registration call; all five metric invocations exit zero.

[Proof ledger](../../../data/patch-api/evidence/9.0.1-session-2026-10-08/p901-proof.json) records exact commands/revisions/scopes, cwd/own target, exit statuses and output hashes. Artifact validator passes at accounting revision `8c43b16d8`; [acceptance receipt](../../../data/patch-api/evidence/9.0.1-session-2026-10-08/p901-artifact-acceptance.json) retains exact command/result. Its totals, register set and sweep set derive from source/fixture/result files, not frozen receipts.

## Preservation and integration

All **168 pre-existing source/register/ledger inputs remain byte-identical**. All **64 prior extraction-mode exit/stdout outcomes remain unchanged**, including sixteen inherited failures and both modes for 12.0.5/12.0.7/12.1.0. Own saved extract reproduces with recorded normalization/example flags; without `--preserve-examples` it deliberately differs because fenced Lua literals are retained, and that exact mode failure is reported rather than hidden. All **33 registers regenerate byte-identically**; provenance flags used when recorded, old inferred flags explicitly distinguished in [register reproduction](../../../data/patch-api/evidence/9.0.1-session-2026-10-08/p901-register-reproduction.json). Earlier provenance untouched.

Read-only p902-page register snapshot identifies eleven retained gap intersections with 9.0.2 removals. [Snapshot/intersection receipt](../../../data/patch-api/evidence/9.0.1-session-2026-10-08/p901-902-supersessions.json) records exact IDs/hash. One-line 9.0.2 placeholder is first in later-register list; main thread inserts it and recomputes fixture, observations, gap review, ledger and evidence at integration. Intersection is not automatic closure: C_Soulbinds.MatchesCurrentSpecSet still needs a retained-removal decision. Pending 9.0.2 runtime changes may close the other superseded additions. No sibling files or targets were modified.

| Retained 9.0.1 gap ID | 9.0.2 direction |
|---|---|
| `wt-global-api-C_CovenantSanctumUI.GetRenownMilestones-246` | removed |
| `wt-global-api-C_Soulbinds.AddPendingConduit-452` | removed |
| `wt-global-api-C_Soulbinds.GetPendingConduitID-467` | removed |
| `wt-global-api-C_Soulbinds.GetPendingNodeIDInSoulbind-468` | removed |
| `wt-global-api-C_Soulbinds.HasPendingConduitInSoulbind-473` | removed |
| `wt-global-api-C_Soulbinds.MatchesCurrentSpecSet-479` | removed |
| `wt-global-api-C_Soulbinds.RemovePendingConduit-480` | removed |
| `wt-global-api-C_Soulbinds.ResetSoulbindConduits-481` | removed |
| `wt-global-api-C_Spell.GetMawPowerRarityStringAndBorderAtlasBySpellID-486` | removed |
| `wt-global-api-C_UIWidgetManager.GetWidgetLayoutDirectionFromWidgetSetID-512` | removed |
| `wt-events-SOULBIND_CONDUITS_RESET-806` | removed |

## Sweep table

Publication/absence only; passing requires exact fixture identities, not zero gaps. [Machine-readable summary](../../../data/patch-api/evidence/9.0.1-session-2026-10-08/p901-sweep-summary.json) derives from results/fixtures.

| Patch | Rows | OK | Gaps | Result |
|---|---:|---:|---:|---|
| 9.0.1 | 755 | 482 | 273 | pass |
| 9.0.5 | 28 | 15 | 13 | pass |
| 9.1.0 | 179 | 128 | 51 | pass |
| 9.1.5 | 169 | 122 | 47 | pass |
| 9.2.0 | 80 | 54 | 26 | pass |
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

## Sources

- [Pinned MediaWiki response](../../../data/patch-api/evidence/9.0.1-session-2026-10-08/p901-fetch.json).
- [Register](../../../data/patch-api/sources/9.0.1-wikitext-register.json) and [provenance](../../../data/patch-api/sources/9.0.1-api-changes.provenance.json).
- [Specification](../../specs/patch-9-0-1-publication-sweep.md).
- [Artifact validator](../../../data/patch-api/evidence/9.0.1-session-2026-10-08/validate.py).

## See Also

- [[patch-9-0-5-api-audit]] — exact occurrence/fixture and retained-source conventions.
- [[patch-9-1-0-api-audit]] — retail retirement, qualified/bare consumer scans and classic gates.

Every command explicitly used p901-page cwd and its own target. No canonical/sibling working-file writes, agents/models/CLIs, push or merge.
