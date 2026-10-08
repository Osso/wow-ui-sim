# Patch 9.0.5 API page audit

Page 447309, revision 4298844 (May 6, 2021, 12:06:16 UTC), retrieved October 8, 2026. Branch `p905-page` starts from master `7b929243e`; current retail carries 12.1.0. Historical client reconstruction is excluded.

## Source accounting

28 inventory occurrences: 24 added and four removed. Global API 16/2, widgets 1/0, events 1/0, CVars 6/2. All four numeric headers match the parsed inventories. The older page uses `16 new functions` rather than `<small>(16)</small>`; generator already parses every row but does not store those headers. [Literal header accounting](../../../data/patch-api/evidence/9.0.5-session-2026-10-08/p905-header-accounting.json) retains lines/counts without changing shared generator behavior. Master and read-only p910-page generator were inspected, including its `--skip-plain-scripts-label`; no new flag or tool change needed. Provenance records `generator_flags: []`.

Unchanged extractor retains four nonblank metadata occurrences: patch navigation, Summary heading, TOC and external diffs. No substantive summary or historical build caption occurs. [Scout](../../../data/patch-api/evidence/9.0.5-session-2026-10-08/p905-extract-scout.json) maps all four to exact source lines; linked pages are not expanded. [Coverage ledger](../../../data/patch-api/sources/9.0.5-page-coverage.json) accounts for all 32 unique IDs: six bounded-coverage, eight partial-development-green, fourteen audit-pending and four metadata-only. Publication/absence does not establish signatures, outputs, security, policies, event producers/payloads or native parity.

Initial cached sweep: 13 OK / 15 gaps. Final: 14 OK / fourteen exact retained gaps. Later 9.1.5–12.1.0 registers supersede historical additions/removals. Current deprecated SpellIsPriorityAura wrapper is credited only as a cached deprecation fallback, never deleted or monkey-patched.

## Bounded retirement and retained gaps

Retire `C_Soulbinds.GetConduitItemLevel`: both qualified whole-word and bare-name cached-retail Lua scans have zero matches; whole `src/` and `tests/` scans before the change have zero callers. [Consumer receipt](../../../data/patch-api/evidence/9.0.5-session-2026-10-08/p905-removal-consumers.json). After-change scan finds only the retirement list and new tests; no pre-existing caller migration needed. Separate `RETIRED_9_0_5_MEMBERS` const uses the existing retail-12-0-0 module gate; classic profiles do not load it. Cached RED fails on lazy fabrication; GREEN asserts raw absence and repeated ordinary lookup absence after unmodified full UI preload. Bare retail and Mists unchanged legacy lookup pass. No Blizzard deprecation wrapper, vendor file, cache file or placeholder added/modified.

Removed global QueryGuildMembersForRecipe already resolves nil. Bare cached search finds its namespaced successor at `Blizzard_Professions/Blizzard_ProfessionsGuildMemberList.lua:59` and API documentation at `Blizzard_APIDocumentationGenerated/GuildInfoDocumentation.lua:248`. Successor remains untouched and an exact missing-publication gap; it is not evidence of a live removed global.

[Per-ID review](../../../data/patch-api/evidence/9.0.5-session-2026-10-08/p905-gap-review.json) assigns all fourteen retained gaps precise model/policy boundaries: keystone/current-map eligibility; four covenant cap/catch-up queries/lifecycle; follower completed-mission relation; custom gossip description; guild recipe request/reply; quest time display policy; four weekly reward period/encounter/next-increase/interaction producers; and absent GamePadSmoothFacing value/default. Lazy `raw=nil; lookup=function` receives no producer credit. Existing challenge completion and weekly progress snapshots do not establish these different contracts. Page gives no GamePadSmoothFacing default; no invented default added.

## Verification

Runtime/test acceptance revision `da0fc967a`; later evidence/docs-only commits do not invalidate it. All 31 publication sweeps plus animation factory regression pass (32/32 cases), covering 6,740 inventory occurrences. Own sweep: 28 rows / 14 OK / fourteen exact gaps. Bare repeated-lookup test, cached-prefork retirement and Mists legacy lookup all pass. Only changed API has no pre-existing callers; every affected test is covered.

Negative control flips only COVENANT_RENOWN_CATCH_UP_STATE_UPDATE added → removed: exactly fourteen → fifteen failures, one added ID, zero resolved IDs, unchanged 28 IDs and expected prefork exit 1. [Receipt](../../../data/patch-api/evidence/9.0.5-session-2026-10-08/p905-negative-result.json).

`cargo fmt` and `cargo fmt --check`, default `cargo check`, Mists `cargo check --no-default-features --features sound,gui,casc,client-mists --tests`, separate retail build and bounded startup pass. Mists has zero non-vendor warnings; six inherited iced manifest deprecations plus summary remain unsuppressed. Retail startup exits zero and prints `[]`. Forty existing generator/extractor fixtures pass. First unittest invocation omitted tools/ from Python search path and executed no fixtures; receipt marks that invocation invalid and retains corrected PYTHONPATH execution.

[Proof ledger](../../../data/patch-api/evidence/9.0.5-session-2026-10-08/p905-proof.json) retains commands, exact revisions/scopes, cwd/target, exit status and output hashes. Historical REDs are explicitly superseded, not current acceptance claims. Manual changed-Rust review: flat retirement data, one existing effect-revealing mark call, input-only sweep spec and bounded lookup tests; no new nesting, opaque side effects or suppressions. Existing long retirement registration list is outside this additive change's refactor scope.

## Preservation and 9.1.0 integration

All 158 pre-existing source/register/ledger inputs remain byte-identical. All 60 prior extraction-mode outcomes remain unchanged; sixteen inherited failures include both modes for 12.0.5/12.0.7/12.1.0 and remain untouched. Own extract reproduces in both modes. All 31 registers regenerate byte-identically; saved flags used when recorded, legacy inferred flags explicitly separated in [reproduction receipt](../../../data/patch-api/evidence/9.0.5-session-2026-10-08/p905-register-reproduction.json).

p910-page generator was consulted read-only; its worktree became unavailable before register comparison. Read-only master now supplies the integrated 9.1.0 register. [Snapshot/intersection receipt](../../../data/patch-api/evidence/9.0.5-session-2026-10-08/p905-910-supersessions.json) identifies exactly one retained-gap intersection: `wt-cvars-GamePadSmoothFacing-68`, superseded by removed `wt-cvars-GamePadSmoothFacing-239`. Main thread must prepend 9.1.0 at the one-line later-register placeholder, then regenerate own fixture/observations/ledger. Current fourteen-gap fixture deliberately excludes that pending integration. Validator derives gap/status counts from results/fixtures rather than freezing integration-sensitive totals.

Every command explicitly used this worktree cwd and its own target. No sibling/canonical working-file changes, vendor/cache/Wowless writes, agents/models, push or merge. Source was captured through MediaWiki API; tool-generated output paths were absolute.

## Sweep table

Publication/absence only; passing cases require exact fixture identities, not zero gaps.

| Patch | Rows | OK | Gaps | Result |
|---|---:|---:|---:|---|
| 9.0.5 | 28 | 14 | 14 | pass |
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

- [Pinned MediaWiki response](../../../data/patch-api/evidence/9.0.5-session-2026-10-08/p905-fetch.json).
- [Register and provenance](../../../data/patch-api/sources/9.0.5-wikitext-register.json).
- [Specification](../../specs/patch-9-0-5-publication-sweep.md).
- [Artifact validator](../../../data/patch-api/evidence/9.0.5-session-2026-10-08/validate.py).

## See Also

- [[patch-9-1-5-api-audit]] — occurrence accounting and publication-only proof conventions.
- [[patch-9-2-0-api-audit]] — conservative retirement and classic preservation.

**Integration (2026-10-08):** after p910-page merged, the 9.1.0 register was prepended to the later-register list. As predicted, `wt-cvars-GamePadSmoothFacing-68` is now superseded by its 9.1.0 removal: the exact-gap fixture drops to **13**, the ledger row moves to bounded-coverage (current absence only), and results, negative control (13 → 14), sweep summary and the register-reproduction receipt (now including 9.1.0) were regenerated. The artifact validator passes.
