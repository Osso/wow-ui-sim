# Patch 8.3.0 API audit

Warcraft Wiki **Patch 8.3.0/API changes**, page **109654**, revision **6471393** (2025-09-13T10:01:45Z), retrieved 2026-10-08. Default retail carries 12.1.0. This is complete publication accounting, not historical reconstruction or native behavior parity.

## Source and markup boundary

[API response](../../../data/patch-api/evidence/8.3.0-session-2026-10-08/p830-fetch.json) retains exact raw wikitext, fetched through browser navigation to the MediaWiki revisions API. [Provenance](../../../data/patch-api/sources/8.3.0-api-changes.provenance.json) records both generator and extractor flags.

**224 inventory occurrences** comprise 220 consolidated rows plus four explicit late-build additions. Consolidated columns: Global API 123 added / 42 removed; Widgets 2 / 2; Events 39 / 6; CVars 5 / 1. All eight numerical headers match parsed counts. The removed CVars-column item is expressly a command, `DumpSoundKits`, not a CVar. Four additional Diffs identities are C_AzeriteEssence.GetNumUsableEssences, C_Item.IsItemCorruptable, TargetSpellHasApplyCorruption and LFG_GROUP_DELISTED_LEADERSHIP_CHANGE. The two corruption helpers are absent under later removals; the event registers; the Azerite producer remains a gap.

New opt-in `--legacy-column-headers` retains prose numerical headers and command-column identity. New opt-in `--diff-api-additions` retains every explicit late-build API on shared source lines. Extractor opt-in `--retain-reference-notes` preserves the Auction House summary and inline citation; existing `--normalize-inventory-headings` and `--preserve-examples` are recorded too. Defaults and earlier registers are unchanged. Three concrete serialized-output fixtures have retained RED evidence; all **46** generator/extractor fixtures pass.

## Coverage matrix and remaining gaps

| Capability | Covered | Remaining | Proof boundary |
|---|---|---|---|
| Consolidated and late-build publication | 183 OK / 224 identities | 41 exact gaps | Raw/ordinary lookup, event registration, object method and command/CVar probes only |
| Retail retirement | Six formerly reachable surfaces absent | Three modeled legacy auction globals retained | Bare/repeated lookup, unmodified cached full UI and startup |
| Classic preservation | Five affected Mists tests pass | Wrath/Era/Anniversary not executed | Existing shared retail-only feature gate excludes classic |
| Non-inventory summary | Every occurrence retained | One Auction House revamp statement pending | No inferred historical overhaul parity |
| Source reproducibility | 36 registers byte-identical; own saved extract reproduces | Three inherited 12.x extract failures unchanged | Exact recorded flags where available; old inferred recipes labeled |

[Occurrence ledger](../../../data/patch-api/sources/8.3.0-page-coverage.json) accounts for **244 unique IDs**: 224 inventory, sixteen extract, four build/comparison-caption contexts. Statuses: **58 bounded-coverage**, **125 partial-development-green**, **42 audit-pending**, **19 metadata-only**. Pending rows are 41 publication gaps plus the broad Auction House revamp statement. Duplicate extract bullets link all four late-build inventory IDs and carry no independent runtime credit. [Extract scout](../../../data/patch-api/evidence/8.3.0-session-2026-10-08/p830-extract-scout.json) maps every literal to exact raw and extract lines. Numerical headers remain separate register metadata; no captions are omitted.

[Per-ID gap review](../../../data/patch-api/evidence/8.3.0-session-2026-10-08/p830-gap-review.json) retains every exact source line, expectation, observation and producer/lifecycle/migration boundary. Missing replication snapshots, calendar/editor state, GUID mappings, talent progression, corruption/GUID locks, item-interaction sessions, threat providers, token sale, power-bar metadata, timing-mode and pet stance producers are not replaced with constants/no-ops. Lazy namespace lookup fabrication receives no raw-publication credit.

## Six bounded closures and retained removals

Initial 220-row discovery has 174 OK / 46 gaps. Six bounded retirements close those six gaps; expanded accounting adds one Azerite producer gap, yielding final **183 OK / 41 gaps** across 224 rows. Four C_ProductChoice members (GetChoices, GetNumSuppressed, GetProducts, MakeSelection), C_WowTokenPublic.SellToken and GetAuctionHouseDepositRate are absent on retail. Separate `RETIRED_8_3_0_MEMBERS` uses the existing `retail-12-0-0` module/registration gate; the legacy global's stub-array entry is excluded only under that retail feature. Header patch list updated. No placeholder additions, vendor edits, Blizzard monkey-patches or deprecation-wrapper deletion.

Qualified whole-word and bare-name cached retail Lua searches plus whole `src/` and `tests/` scans are retained **untruncated** in [consumer receipt](../../../data/patch-api/evidence/8.3.0-session-2026-10-08/p830-removal-consumers.json); [after-change callers](../../../data/patch-api/evidence/8.3.0-session-2026-10-08/p830-whole-callers-after.json) covers every changed identity. Bare GetProducts hits belong to C_StoreSecure, not the removed C_ProductChoice member. Existing ProductChoice callers/tests are already Mists-only; no retail caller migration is needed. Mists empty-table/count/selection defaults and legacy token/deposit-rate lookup are preserved.

CanSendAuctionQuery, CancelAuction and GetNumAuctionItems remain precise gaps: legacy registration/state-backed paths and tests require a separate profile-safe migration, not an unused namespace cleanup. Cached CancelAuction bare-name hits are the current C_AuctionHouse successor (Blizzard_AuctionHouseFrame.lua:82), not proof of the old global. Full source/caller citations and distinctions are retained in the review. Current Blizzard deprecation fallbacks are attributed by the existing shared probe and never deleted.

## Verification and proof ledger

Runtime behavior/check/startup revision **f65b5dcc8**; final expanded register/fixture proof revision **073d223fb**. Later evidence/documentation changes do not invalidate those scopes. **36 register sweeps plus animation factory regression pass (37/37)**, covering **7,977** inventory rows across all current registers. New bare and cached retirement tests pass. Cached tests assert unchanged Lua error count. REDs fail at fabricated C_ProductChoice.GetChoices lookup and raw legacy deposit-rate publication after unmodified preload.

Final negative control flips only AUCTION_CANCELED added → removed: **41 → 42 gaps**, exactly one added identity, zero resolved identities, unchanged 224 row IDs and expected test exit one. [Receipt](../../../data/patch-api/evidence/8.3.0-session-2026-10-08/p830-expanded-negative-result.json). Earlier narrower RED/acceptance results remain labeled and are not substituted for expanded acceptance.

`cargo fmt`, final `cargo fmt --check`, default `cargo check`, separate retail build and bounded retail startup pass. Startup exits zero and prints **`[]`**. Mists `cargo check --no-default-features --features sound,gui,casc,client-mists --tests` passes with **zero non-vendor warnings**; inherited iced manifest deprecations remain unsuppressed. [Changed-Rust readability](../../../data/patch-api/evidence/8.3.0-session-2026-10-08/p830-readability.md) includes six successful metric invocations and changed-line review.

All **five affected Mists tests pass**: new legacy preservation, three ProductChoice tests and existing legacy startup API-shape test. The broader 27-test Mists selection returns 26 passes / one unrelated failure in unchanged `mists_honor_frame_shared_reproduces_missing_honor_system_enabled` at tests/mists_compat_bootstrap.rs:381. Expected diagnostic contains HonorSystemEnabled; actual diagnostic is `(string):29: attempt to call a nil value`. This path does not depend on the retail-only changes. Failure is retained and reported, not suppressed or fixed outside scope. [Exact scope/result](../../../data/patch-api/evidence/8.3.0-session-2026-10-08/p830-mists-changed-surface-results.json).

Per-command `*.proof.json` files retain command, revision/scope, cwd, own target, exit, log hash and invalidation status; [aggregate](../../../data/patch-api/evidence/8.3.0-session-2026-10-08/p830-proof.json) indexes them. [Dynamic artifact validator](../../../data/patch-api/evidence/8.3.0-session-2026-10-08/validate.py) passes, deriving gap totals, ledger statuses, register/result sets, chronological supersessions and sweep table from current fixtures/sources/results. [Acceptance receipt](../../../data/patch-api/evidence/8.3.0-session-2026-10-08/p830-artifact-acceptance.json) records exact command and output hashes. No hard-coded integration counts serve as assertions.

## Preservation and scope

All **183** pre-existing source/register/ledger inputs remain byte-identical. All **70** prior default/example extraction-mode outcomes remain unchanged, including inherited failures. Every saved extract reproduces with its recorded or explicitly inferred recipe except the already non-reproducible 12.0.5/12.0.7/12.1.0 extracts, which remain untouched. All **36** registers regenerate byte-identically; recorded flags are honored and missing historical recipes labeled. [Register reproduction](../../../data/patch-api/evidence/8.3.0-session-2026-10-08/p830-register-reproduction.json), [saved extract reproduction](../../../data/patch-api/evidence/8.3.0-session-2026-10-08/p830-saved-extract-reproduction.json).

Every later master register, 8.3.7 through 12.1.0, participates in supersession. No pending sibling integration is assumed. Every command explicitly uses `/home/osso/.worktrees/wow-ui-sim-p830-page` cwd and its own target directory; branch verified `p830-page` before first commit. Canonical/sibling working files, protected caches/vendor/Wowless paths remain unmodified. No agents, models, pi/claude CLIs, push or merge.

## Sweep table

Publication/absence only. Passing requires the exact reviewed gap identities, not zero gaps. [Machine-readable summary](../../../data/patch-api/evidence/8.3.0-session-2026-10-08/p830-sweep-summary.json).

| Patch | Rows | OK | Gaps | Result |
|---|---:|---:|---:|---|
| 8.3.0 | 224 | 183 | 41 | pass |
| 8.3.7 | 2 | 1 | 1 | pass |
| 9.0.1 | 755 | 492 | 263 | pass |
| 9.0.2 | 77 | 51 | 26 | pass |
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

- [Pinned raw source](../../../data/patch-api/sources/8.3.0-api-changes.wikitext), [extract](../../../data/patch-api/sources/8.3.0-api-changes.txt), [register](../../../data/patch-api/sources/8.3.0-wikitext-register.json), [provenance](../../../data/patch-api/sources/8.3.0-api-changes.provenance.json).
- [Specification](../../specs/patch-8-3-0-publication-sweep.md).
- [Prior audit](patch-8-3-7-api-audit.md), [9.0.1 audit](patch-9-0-1-api-audit.md).

## See Also

- [[patch-8-3-7-api-audit]] — simple bullet publication and occurrence accounting.
- [[patch-9-0-1-api-audit]] — current-retail supersession, profile gates and retained model boundaries.
