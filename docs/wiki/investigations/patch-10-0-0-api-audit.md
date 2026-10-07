# Patch 10.0.0 API page audit

Warcraft Wiki page 530068 exists. Revision 6789768 (July 31, 2026, 04:32:53 UTC), retrieved October 7, 2026. Branch `p1000-page` starts from master `776ab3d8e`, which contains `tests/patch_10_0_2_publication_sweep.rs`. Default retail carries 12.1.0; this is not a historical Dragonflight runtime.

## Source accounting

639 inventory occurrences: 488 added, 134 removed, 17 changed. Global API 277/64/12, widgets 56/29/0, events 62/25/5, CVars including two Command records 93/16/0. Published event headers say 63 added and 26 removed, each one higher than the actual source list. Other header counts match. Missing event names are not invented to satisfy headers. All 25 chronological later registers from 10.0.2 through 12.1.0 apply.

[Coverage ledger](../../../data/patch-api/sources/10.0.0-page-coverage.json) accounts for 1,082 unique IDs: 639 inventory, 439 nonblank extract occurrences and four separately retained inventory-context lines. Statuses: 466 bounded-coverage, 600 audit-pending, 16 metadata-only. Pending comprises 173 exact publication gaps, 424 extract occurrences and three widget migration summaries. All populated outputs, annotations, payloads, security, native and historical parity remain unproved. Metadata has no runtime credit.

[Literal scout](../../../data/patch-api/evidence/10.0.0-session-2026-10-07/p1000-extract-scout.json) retains each extract line, source ID and section without attributing unrelated tests to it. Settings canvas/vertical categories and historical registration examples, stable aura instance IDs and full/incremental UNIT_AURA processing, interaction-manager show/hide events, deprecated templates, enum/DTO additions and 9.x constant migrations remain pending. Lua examples are retained verbatim using `--preserve-examples`. The same extract fails without that flag by design; it is not labeled reproducible in both modes.

Eleven CVar default comparisons differ from the historical source: eight decimal serialization differences, CMAA2HalfFloat (`0` → `1`), and two absent variables (`luaErrorExceptions`, `minimapTrackedInfov3`). These remain reported, not fixed to historical values or silently called parity.

## Source-loss boundaries and probe corrections

Fixture-backed fixes address four observed boundaries, without rewriting any prior register:

- Inline `: '''Commands'''` marks LogFps and WriteCustomizationOptions as Commands, not CVars; the removed CVar column resets this classification.
- `{{clrr}}` is editorial layout clearing, rendered as an empty line rather than rejected markup.
- Owner-qualified `UIHANDLER_OnModelCleared` preserves its source label and probes the actor's HasScript rather than constructing a frame named after the whole label.
- Shared changed lines name multiple APIs. Opt-in `--expand-shared-changes` adds C_GossipInfo.SelectAvailableQuest, C_GossipInfo.SelectOption, C_MountJournal.GetDisplayedMountInfo and GOSSIP_ENTER_CODE with their original line and shared annotations. Default generation preserves every existing register byte-for-byte. SelectOption is a real additional publication gap, not silently omitted.

The shared extractor intentionally drops inventory-section prose. [Context supplement](../../../data/patch-api/evidence/10.0.0-session-2026-10-07/p1000-inventory-context.json) retains the build caption at wikitext line 220 and all three widget migration statements at 594–596. This avoids changing previous extracts while preserving their proof boundary.

Scale, Path and FlipBook are constructed through animation groups, ScriptRegionResizing through a frame, and WorldFrame through the existing global. RED factory proof fails at invalid CreateFrame('Scale'); GREEN proves four owner/method classifications. Correct construction removes 21 false probe failures from the initial 635-row observation (442 OK / 193 gaps → 463 OK / 172 gaps). Four newly accounted shared references then yield 639 rows, 466 OK / 173 gaps. These are accounting/probe corrections, **not 21 runtime API fixes**. Missing Path methods, Scale getters and WorldFrame:OnModelCleared stay explicit gaps after successful owner construction.

## Retained gaps and caller scan

[Per-ID review](../../../data/patch-api/evidence/10.0.0-session-2026-10-07/p1000-gap-review.json) assigns all 173 final failures a reason and source boundary. [Current declaration context](../../../data/patch-api/evidence/10.0.0-session-2026-10-07/p1000-native-contracts.json) finds name-matched metadata for 136 gaps; shared member names still require namespace attribution, and missing current metadata does not prove a historical API never existed. Missing talent config lifecycle, customer-order options, layout serialization/transitions, item GUID resolution, profession path/perk/currency state, crafting quality/recraft/salvage operations, NPC validation, tracking/POI metadata, inspected PvP DTOs, graphics capabilities, quest rewards and XML template enumeration need real backing behavior. Existing namespace autostubs with `raw=nil; lookup=function` do not establish native publication. Console diagnostics and two historical CVars are not fabricated. Named movies, offscreen diagnostics and hyperlink-to-model loading remain unmodeled; 3D behavior is intentionally unsupported.

35 failures are removal mismatches. No production API change or retirement was made. Cached consumers/deprecation assignments, shared classic methods and unreviewed successor semantics must not be removed merely to reduce the gap count. Zero-match candidates remain explicitly deferred, not falsely described as live consumers.

[Initial qualified/bare scan](../../../data/patch-api/evidence/10.0.0-session-2026-10-07/p1000-removal-consumers.json) covers 200 own-removed or later-effective-absent occurrences. [Exact-boundary refinement](../../../data/patch-api/evidence/10.0.0-session-2026-10-07/p1000-exact-removal-consumers.json) excludes namespaced successors from bare-global identity; all bare matches remain retained. Exact references are not automatically loaded retail calls: declarations, comments and profile-specific cached files require attribution. Concrete examples:

- `Blizzard_ActionBarController/ActionBarController.lua:44,104,117` registers/waits for/handles SETTINGS_LOADED.
- `Blizzard_AzeriteRespecUI/Blizzard_AzeriteRespecUI.lua:1` stores C_AzeriteEmpoweredItem.CloseAzeriteEmpoweredItemRespec as a function reference.
- `Blizzard_Deprecated/11_0_0_SpellBookAPITransitionGuide.lua:142` assigns C_SpellBook.GetOverrideSpell to its C_Spell successor; the wrapper remains untouched.
- `Blizzard_UnitFrame/Shared/PartyMemberFrame.lua:202,204` calls GameTooltip:SetUnitBuffByAuraInstanceID; CompactUnitFrame.lua:2154 also calls it.
- `Blizzard_PTRFeedback/Blizzard_PTRFeedback_Tooltips.lua:76` mentions GetRecipeReagentItemLink in a comment, not a loaded native-call proof.

[Whole-tree scan](../../../data/patch-api/evidence/10.0.0-session-2026-10-07/p1000-whole-caller-scan.txt) retains 504 untruncated lines from all `src/` and `tests/`, with `--hidden --no-ignore`; embedded Lua, pcall references and guards are included. [Source excerpts](../../../data/patch-api/evidence/10.0.0-session-2026-10-07/p1000-gap-source-scans.json) retain per-gap simulator file:line context. No runtime surface changed, so no callers needed migration and no classic registration changed.

## Verification

Runtime/test revision `aa860f1d5`. The prescribed `cargo test --test prefork_full_ui -- publication_sweep` runs all 26 sweeps plus the new factory case: 26 pass, while the new 10.0.0 sweep correctly detects the newly recovered SelectOption row missing from the initial 172-ID allowance. Committing the exact 173-ID fixture and rerunning only `patch_10_0_0_publication_sweep` passes both new cases. The unchanged later 25 cases are already proven by the batch; no expensive batch is rerun merely for closure. All 26 sweeps therefore have passing current-relevant-scope evidence. Fork children use the shared full-UI preload, not integration-only sweep targets.

The negative control changes only GetUnitEmpowerHoldAtMaxTime added → removed: exactly one new failure, no resolved failures, 173 → 174, expected prefork exit 1. [Result](../../../data/patch-api/evidence/10.0.0-session-2026-10-07/p1000-negative-result.json).

All shared-helper consumers are covered: three 10.2.0 integration tests, the VertexColor probe regression, loaded deprecation alias identity regression, and one cached 12.0.0 retirement/successor prefork pass. Python parser/extractor fixtures pass 33/33. Format/fmt --check, default cargo check, Mists `cargo check --no-default-features --features sound,gui,casc,client-mists --tests`, and separate retail build pass. Mists has zero non-vendor warnings; six pre-existing iced manifest deprecations plus their summary remain unsuppressed. Bounded startup exits zero and prints `[]`.

Changed Rust manually audited: flat sweep inputs, bounded factory assertions, explicit owner factories and owner/script selection; no introduced readability violations or warning suppressions. [Proof ledger](../../../data/patch-api/evidence/10.0.0-session-2026-10-07/p1000-proof.json) records commands, revisions/scopes, exits and hashes. Cargo output is captured once, saved and searched with rg, never rerun to recover logs. Documentation/reason-only changes do not invalidate compiled proof. No independent agents/models were invoked.

## Sweep table

All rows use exact gap fixtures; publication-only OK does not mean API parity. Later results come from the shared batch; 10.0.0 acceptance comes from its subsequent targeted run.

| Patch | Rows | OK | Gaps | Case result |
|---|---:|---:|---:|---|
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

## Preservation and boundaries

[Preservation](../../../data/patch-api/evidence/10.0.0-session-2026-10-07/p1000-preservation.json) proves all 158 prior source/register/ledger/gap inputs unchanged. Before/after `--check --text-only` runs with and without `--preserve-examples` preserve every one of the 50 prior mode outcomes; own page adds two outcomes. Pre-existing 12.0.5/12.0.7/12.1.0 both-mode failures and other historical mode-specific failures remain untouched. All 26 registers regenerate byte-identically in their recorded modes, including this page's explicit shared-reference expansion.

[Artifact validator](../../../data/patch-api/evidence/10.0.0-session-2026-10-07/validate.py) checks IDs, literal scout/context, sources, exact fixture/expectations, negative control, proof hashes, warning boundary and preserved inputs without runtime reruns. No full suite, production/runtime API fixes, new placeholders, agents/models, push, merge, sibling target reuse or vendor/cache/canonical working-file edits. Every external command has explicit cwd `p1000-page` and its own target; only prescribed Git worktree metadata uses the canonical path while cwd is the initially empty destination.

## Sources

- [Provenance](../../../data/patch-api/sources/10.0.0-api-changes.provenance.json) and [raw page](../../../data/patch-api/sources/10.0.0-api-changes.wikitext).
- [Publication contract](../../specs/patch-10-0-0-publication-sweep.md).
- [Coverage ledger](../../../data/patch-api/sources/10.0.0-page-coverage.json).
- [Register reproduction](../../../data/patch-api/evidence/10.0.0-session-2026-10-07/p1000-register-reproduction.json).

## See Also

- [[patch-10-0-2-api-audit]] — immediately later source and occurrence-accounting template.
- [[patch-10-0-5-api-audit]] — reason and proof boundaries.
- [[test-suite-performance]] — isolated prefork sweeps.
