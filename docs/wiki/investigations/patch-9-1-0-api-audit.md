# Patch 9.1.0 API page audit

Page 17181, revision 167980 (March 2, 2022, 05:11:06 UTC), retrieved October 7, 2026 (host-local date). Requested page exists; MediaWiki revision query returned HTTP 200. Branch `p910-page` starts from master `67b42dea5`. Current retail carries 12.1.0; historical Shadowlands behavior is not reconstructed.

## Source accounting

179 inventory occurrences: 77 added / 43 removed Global API, 14 added / two removed widgets, 18 added / one removed events, 19 added / five removed CVars including three Commands. All eight published header counts match. The plain widget `: Scripts` category label made the generator raise `ValueError`; master and read-only p915-page both lacked support for this label. New `--skip-plain-scripts-label` flag skips only that editorial label, retaining the `ModelScene OnDressModel` handler and source line. Default behavior is unchanged. Plain Commands and HTML CVar defaults already work by default; no redundant flag added for those behaviors. Provenance records `generator_flags`: `--expand-shared-changes --skip-plain-scripts-label`.

[Ledger](../../../data/patch-api/sources/9.1.0-page-coverage.json) accounts for all **192 unique IDs**: 179 inventory, twelve nonblank extract occurrences and one historical build-caption context. Statuses: **61 bounded-coverage, 65 partial-development-green, 56 audit-pending, ten metadata-only**. Bounded rows prove current absence only. Partial rows prove publication/registration or retained cached deprecation publication, not signatures, populated outputs or native behavior parity. Pending rows comprise 53 exact publication gaps plus three summary contracts.

[Extract scout](../../../data/patch-api/evidence/9.1.0-session-2026-10-07/p910-extract-scout.json) links every extract literal to raw source lines. Mainline TOC selection/fallback, rewritten event-trace behavior and transmogrification rework remain pending; no unrelated existing loader/model test receives source-statement credit. Resource links and source-generator HTML comment are metadata, not expanded contracts. Extractor unchanged; own extract reproduces in both modes.

## Closures and retained gaps

Initial cached sweep: 105 OK / 74 gaps. Twenty-one unused removed namespace members retired, producing **126 OK / 53 exact gaps**. Qualified whole-word and bare-name cached retail Lua searches have zero consumers for every retired member; [all removed-member scans](../../../data/patch-api/evidence/9.1.0-session-2026-10-07/p910-removal-consumers.json) are retained. Separate `RETIRED_9_1_0_MEMBERS` const in `src/c_api/patch_retired_members.rs` uses the existing `retail-12-0-0` gate; classic profiles do not load it. No deprecation wrapper deleted or patched.

Retirement groups: OldBarberShopLoaded; old Runeforge class/spec query; PetJournal GetNumMaxPets; three old PlayerChoice queries; GetConduitRankFromCollection; Transmog LoadSources/ValidateAllPending; nine obsolete TransmogCollection members; three obsolete TransmogSets members. [Repeated-lookup regression](../../../tests/patch_9_1_0_retirements.rs) lists all 21 exact identities and checks raw, ordinary and repeated lookup. Cached full-UI regression checks the same assertions after unmodified Blizzard preload.

First aggregate run exposed a second registration boundary: `GetShowMissingSourceInItemTooltips` was explicitly re-registered by the existing missing-surface module after C API retirement. Gate that existing registration and its implementation to `!retail-12-0-0`; no new placeholder added. The legacy integration caller uses the same epoch gate. [Whole src/tests scan](../../../data/patch-api/evidence/9.1.0-session-2026-10-07/p910-whole-caller-scan.txt) found this one actual caller and two unrelated `GetSetSourcesForSlot` substring hits. [Post-change scan](../../../data/patch-api/evidence/9.1.0-session-2026-10-07/p910-whole-caller-scan-after.txt) includes all embedded Lua, guard/function-reference positions and new tests without truncation.

Removed `C_Transmog.GetCost` and `C_Transmog.GetItemInfo` still fabricate functions. Qualified searches have no consumers, but conservative bare-name rule retains them: `Blizzard_ItemInteractionUI/Blizzard_ItemInteractionUI.lua:139` defines a GetCost method; `Blizzard_ActionBar/WoWLabs/ActionButtonOverrides.lua:473` calls the different `C_Item.GetItemInfo` identity. These references are not falsely claimed to use the old namespace API.

[Per-ID gap review](../../../data/patch-api/evidence/9.1.0-session-2026-10-07/p910-gap-review.json) assigns every retained literal/observation a precise boundary: alert acknowledgement, dungeon-score/color data, chat-type mapping, ordered toast state, mission targeting, item appearance/upgrade data, applicant scores, Runeforge catalog, level unlocks, quest metadata, soulbind assignments, historical SpellBook identities, appearance/illusion/inspect mappings, visualization DTOs, transcription/TTS lifecycle and two missing graphics Command records. Six DressUpModel/ModelSceneActor transmog methods remain intentionally unsupported 3D/model gaps. Lazy `raw=nil; lookup=function` receives no producer credit.

Historical defaults differ: RAIDVolumeFog current `0` versus page `1`; graphicsComputeEffects current `1` versus page `4`. Current retail state is not overwritten to historical values. Those mismatches are separate from publication gaps.

## Verification

All **30 publication sweeps** have passing current-scope proof, covering **6,722 inventory occurrences**. The aggregate command at `13080ddd3` exits one: 29 later sweeps plus the existing animation-owner factory case pass (30 cases), while 9.1.0 fails solely because of the post-retirement tooltip re-registration. After the registration fix at `395bb4081`, focused 9.1.0 sweep and cached-retirement prefork both pass. No later register contains the changed tooltip member and its cached consumer scan is empty, so their prior passing scope is unchanged; the broad command was not redundantly rerun. This is combined scope evidence, not a claim that the first aggregate command exited zero.

Bare 21-member retirement RED/GREEN, cached full-UI retirements, Mists legacy lookup of all 21 identities and the affected Mists legacy tooltip integration caller pass. Changing the caller gate from `!client-retail` to `!retail-12-0-0` subsequently matches its production gate; default retail and Mists select identical source paths under both predicates. No PTR execution claim.

DISPLAY_EVENT_TOASTS added → removed negative control changes exactly one row: **53 → 54 gaps**, no resolved failures, unchanged 179 IDs, expected prefork exit one. [Exact receipt](../../../data/patch-api/evidence/9.1.0-session-2026-10-07/p910-negative-result.json).

Fifteen generator and 25 extractor fixtures pass. `cargo fmt`, `cargo fmt --check`, default `cargo check`, Mists `cargo check --no-default-features --features sound,gui,casc,client-mists --tests`, separate retail build and bounded startup pass. Mists has **zero non-vendor warnings**; six inherited iced manifest deprecations plus summary remain unsuppressed. Separately rebuilt retail binary exits zero and prints `[]`.

[Proof ledger](../../../data/patch-api/evidence/9.1.0-session-2026-10-07/p910-proof.json) retains exact commands, revisions, scope and exits. [Readability review](../../../data/patch-api/evidence/9.1.0-session-2026-10-07/p910-readability.md) covers flat retirement data, named mark call, input-only sweep, profile gate and bounded regressions. Retirement metrics command passes; transmog-module analyzer exits one without saved diagnostics, so manual review—not an analyzer pass—is claimed. No warning suppression, new model abstraction or placeholder added.

## Sweep table

Publication/absence only. Exact known-gap fixtures match each retained result; runtime revision/scope retained in the [machine-readable table](../../../data/patch-api/evidence/9.1.0-session-2026-10-07/p910-sweep-table.json).

| Patch | Rows | OK | Gaps | Result |
|---|---:|---:|---:|---|
| 9.1.0 | 179 | 126 | 53 | pass |
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

## Preservation and integration

All **153 pre-existing source/register/ledger inputs remain byte-identical**. All **58 prior extract-mode exit/stdout outcomes are unchanged**, including sixteen existing failures and both modes for 12.0.5/12.0.7/12.1.0. Own extract passes both modes. All 30 registers regenerate byte-identically. [Reproduction receipt](../../../data/patch-api/evidence/9.1.0-session-2026-10-07/p910-register-reproduction.json) distinguishes explicitly recorded flags from verified inferred flags where historical provenance lacks them; no earlier provenance rewritten.

Concurrent p915-page became unavailable after its integration. Read-only `git show master:...` captured its register and audit without changing this branch. [Intersection report](../../../data/patch-api/evidence/9.1.0-session-2026-10-07/p910-p915-intersections.json) finds two possible supersessions:

- `wt-global-api-AcknowledgeAADCAlert-23` → 9.1.5 removed `wt-global-api-AcknowledgeAADCAlert-103`; current nil lookup would satisfy absence.
- `wt-global-api-C_ItemUpgrade.GetItemLevelIncrement-42` → 9.1.5 removed `wt-global-api-C_ItemUpgrade.GetItemLevelIncrement-106`; the 9.1.5 audit also retires this unused endpoint. Integration is expected to close this gap, but no combined-runtime proof is claimed here.

No other retained gap identity intersects that register. The requested one-line placeholder remains first in this branch's later-register list; integrating main thread inserts 9.1.5 and recomputes the fixture (likely 53 → 51). This branch does not merge, rebase or integrate sibling runtime changes.

All commands explicitly use `/home/osso/.worktrees/wow-ui-sim-p910-page` as cwd; own `target` only. No agents/models, push, merge, canonical working-file edits, sibling edits, vendor/cache/Wowless edits or hardware changes. Worktree creation was the explicitly requested Git operation.

## Sources

- [Pinned response](../../../data/patch-api/evidence/9.1.0-session-2026-10-07/p910-fetch.json) — page revision and original wikitext.
- [Provenance](../../../data/patch-api/sources/9.1.0-api-changes.provenance.json) — generator flags and hash.
- [Register](../../../data/patch-api/sources/9.1.0-wikitext-register.json) — all inventory occurrences.
- [Spec](../../specs/patch-9-1-0-publication-sweep.md) — proof requirements and exclusions.
- [Artifact validator](../../../data/patch-api/evidence/9.1.0-session-2026-10-07/validate.py) — ID/hash/gap/proof-boundary validation without runtime reruns.

## See Also

- [[patch-9-2-0-api-audit]] — publication accounting and conservative retirement template.
- [[patch-9-2-5-api-audit]] — provenance flags and source/proof boundaries.
