# Patch 9.0.2 API page audit

Page 66933, revision **659986** (May 6, 2021, 09:01:47 UTC), retrieved October 8, 2026. Branch `p902-page` starts from master `1d2417266`; current retail carries 12.1.0. Original 9.0.2 page exists: no 9.0.1 substitution. Linked 9.0.1 content is not expanded; historical client reconstruction is excluded.

## Source accounting

**77 inventory occurrences**: 64 added and thirteen removed. Global API 42/10, widgets 5/0, events 8/1, CVars 9/2. All seven numeric headers match parsed counts. Existing generator already handles these tables; numeric prose headers are retained separately in [literal header accounting](../../../data/patch-api/evidence/9.0.2-session-2026-10-08/p902-header-accounting.json). No shared generator/extractor changes or new flags. Provenance records `generator_flags: []`.

Unchanged extractor retains **eleven nonblank occurrences**. [Scout](../../../data/patch-api/evidence/9.0.2-session-2026-10-08/p902-extract-scout.json) links every literal to exact raw-source lines. Eight are editorial/navigation/release/source-link context; three substantive statements remain pending: help-frame overhaul, DeleteCursorItem hardware-event enforcement, and broad Shadowlands feature additions. Publication alone does not prove these contracts. Four historical inventory build captions receive separate metadata IDs.

[Coverage ledger](../../../data/patch-api/sources/9.0.2-page-coverage.json) accounts for all **92 unique IDs**: **24 bounded-coverage**, **27 partial-development-green**, **29 audit-pending**, **twelve metadata-only**. Bounded rows prove current absence only. Partial rows prove publication or event registration only, not signatures, populated outputs, payloads, security, policies or native behavior parity. Pending rows comprise 26 exact publication gaps and three notable-change contracts.

## Bounded retirement and retained gaps

Initial cached sweep: **42 OK / 35 gaps**. Nine unused namespace retirements produce **51 OK / 26 exact retained gaps**. [Removal consumers](../../../data/patch-api/evidence/9.0.2-session-2026-10-08/p902-removal-consumers.json) retain both qualified whole-word and bare-name cached retail Lua scans plus whole `src/` and `tests/` scans, without truncation. Every retired member has zero consumers and pre-existing callers.

Retire `C_CovenantSanctumUI.GetRenownMilestones`; six old `C_Soulbinds` pending-conduit members (AddPendingConduit, GetPendingConduitID, GetPendingNodeIDInSoulbind, HasPendingConduitInSoulbind, RemovePendingConduit, ResetSoulbindConduits); `C_Spell.GetMawPowerRarityStringAndBorderAtlasBySpellID`; and `C_UIWidgetManager.GetWidgetLayoutDirectionFromWidgetSetID`. Separate `RETIRED_9_0_2_MEMBERS` const uses existing `retail-12-0-0` module gate. Classic profiles do not load it. No deprecation wrapper removed or monkey-patched, no placeholder added.

Conservatively retain removed `C_Soulbinds.MatchesCurrentSpecSet`: qualified old-namespace search is empty, but bare-name search finds **Blizzard_Soulbinds/Blizzard_SoulbindsConduitList.lua:135,396**, both calling current `C_SpecializationInfo.MatchesCurrentSpecSet`, plus generated documentation at SpecializationInfoDocumentation.lua:331. These are not falsely claimed as consumers of the old namespace identity.

[Per-ID gap review](../../../data/patch-api/evidence/9.0.2-session-2026-10-08/p902-gap-review.json) assigns every retained literal/observation a precise missing model or policy boundary: covenant access/renown catalogs, adventure effects/eligibility/healing, gossip refresh lifecycle, anima/conduit item classification, tower display/unlock policy, soulbind graph/collection/pending transactions, specialization-set membership and unsupported 3D model caching. Lazy `raw=nil; lookup=function` earns no producer credit. Existing MajorFactions renown and Soulbinds collection placeholders do not establish the different contracts. PlayerModel:ZeroCachedCenterXY remains an intentional 3D gap.

## Verification

Runtime/test acceptance revision **ebdf20c6d**; later accounting/evidence/docs-only commits do not invalidate that code scope. All **32 publication sweeps plus animation factory regression pass (33/33)**, covering **6,968 inventory occurrences**. Every exact-gap fixture matches retained results. Own 77-row sweep is 51 OK / 26 gaps.

Bare repeated-lookup retirement RED fails on fabricated GetRenownMilestones, then GREEN passes. Cached full-UI regression checks all nine raw/ordinary/repeated lookup absences after unmodified Blizzard preload. Mists legacy function lookup for all nine identities passes. Whole pre-change scans found no callers, so no existing caller migration is required; [after-change scan](../../../data/patch-api/evidence/9.0.2-session-2026-10-08/p902-whole-callers-after.txt) finds only the retirement constant and new tests. Wrath/Era/Anniversary are preserved by the shared module gate, not claimed executed.

First retirement-test invocation failed in build.rs because its prefork marker parameter was qualified rather than exact `env: &WowLuaEnv`; receipt marks it invalid, corrected before behavioral RED. Initial discovery with empty fixture is likewise historical RED, not acceptance.

Negative control changes only GOSSIP_OPTIONS_REFRESHED added → removed: **26 → 27 gaps**, exactly one added failure, zero resolved failures, unchanged 77 IDs and expected prefork exit one. [Exact receipt](../../../data/patch-api/evidence/9.0.2-session-2026-10-08/p902-negative-result.json).

`cargo fmt`, `cargo fmt --check`, default `cargo check`, Mists `cargo check --no-default-features --features sound,gui,casc,client-mists --tests`, separate retail build and bounded startup pass. Mists has **zero non-vendor warnings**; six inherited iced manifest deprecations plus summary remain unsuppressed. Rebuilt retail startup exits zero and prints **`[]`**. All **41** unchanged generator/extractor fixtures pass.

[Proof ledger](../../../data/patch-api/evidence/9.0.2-session-2026-10-08/p902-proof.json) retains exact commands, revisions/scopes, cwd/target, exits and output hashes. [Changed-Rust review](../../../data/patch-api/evidence/9.0.2-session-2026-10-08/p902-readability.md) covers flat retirement data, explicit mark call, input-only sweep and bounded regressions. All four metrics invocations exit zero. Existing registration function cyclomatic 27 is driven by its fallible call sequence; no new branching/refactor scope is introduced. Artifact validation is separately recorded in p902-artifact-acceptance.json.

## Preservation and integration

All **163 pre-existing source/register/ledger inputs remain byte-identical**. All **62 prior extraction-mode exit/stdout outcomes remain unchanged**, including sixteen inherited failures and both modes for 12.0.5/12.0.7/12.1.0. Own extract passes both modes. All **32 registers regenerate byte-identically**: saved generator flags used when present; historical inferred flags are distinguished in [reproduction receipt](../../../data/patch-api/evidence/9.0.2-session-2026-10-08/p902-register-reproduction.json). No earlier provenance rewritten.

Read-only [9.0.5 register snapshot/intersection receipt](../../../data/patch-api/evidence/9.0.2-session-2026-10-08/p902-905-supersessions.json) finds **zero retained-gap intersections** with later add/remove rows. No predicted 9.0.5 gap supersessions. One-line placeholder is first in the later-register list; main thread inserts 9.0.5 at integration. Validator derives gap counts, status totals and register lists from fixture/results/sources files rather than freezing integration-sensitive values.

Every command explicitly used this worktree cwd and own target directory. No canonical/sibling working-file changes, vendor/cache/Wowless writes, agents/models/CLIs, push or merge. Existing tracked PLAN.md was temporarily replaced locally while planning, then restored byte-for-byte before completion; never committed.

## Sweep table

Publication/absence only. Passing cases require exact fixture identities, not zero gaps. Runtime revision/scope retained in [proof ledger](../../../data/patch-api/evidence/9.0.2-session-2026-10-08/p902-proof.json); [machine-readable table](../../../data/patch-api/evidence/9.0.2-session-2026-10-08/p902-sweep-summary.json) derives counts from results.

| Patch | Rows | OK | Gaps | Result |
|---|---:|---:|---:|---|
| 9.0.2 | 77 | 51 | 26 | pass |
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

- [Pinned MediaWiki response](../../../data/patch-api/evidence/9.0.2-session-2026-10-08/p902-fetch.json).
- [Register](../../../data/patch-api/sources/9.0.2-wikitext-register.json) and [provenance](../../../data/patch-api/sources/9.0.2-api-changes.provenance.json).
- [Specification](../../specs/patch-9-0-2-publication-sweep.md).
- [Artifact validator](../../../data/patch-api/evidence/9.0.2-session-2026-10-08/validate.py).

## See Also

- [[patch-9-1-0-api-audit]] — publication-only accounting and conservative retirement.
- [[patch-9-1-5-api-audit]] — chronological supersession and classic preservation.
