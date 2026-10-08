# Patch 6.0.2 API audit

Pinned main pageid 556401 revision 5363123 (2015-01-12) and independently fetched transcluded diff pageid 372865 revision 3594774. The diff adds 787 lines to the 106-line main page. Four inventory captions reproduce exactly: global 374/36, FrameXML 30/14, events 113/9, widgets 42/7 added/removed. Main summary contributes 32 occurrences: 657 total API rows.

## Coverage matrix

| Scope | Result | Proof level |
|---|---|---|
| API publication/absence | 382 bounded, 275 exact pending gaps | Current retail cached sweep, later-register supersession; no historical tuple/signature/security parity |
| Scenario bonus steps | Two new modeled queries | Existing ordered `ScenarioState.steps`; two bare/cached query tests cover IDs 4/9, removal, optional rewards, output isolation, inactive state and bad arguments |
| Namespace retirements | Three consumer-free members | Raw and normal lookup absence, complete grep scans and pinned later-register checks |
| Retained main prose | 100 rows: five metadata, 95 pending contracts | Exact retained text and one reason per statement; no invented behavior credit |
| Diff enum members | 79 pending values/removals, 18 group headings | Exact source lines; no current numeric equality or historical native-default claim |

Ledger totals: 854 unique IDs, 382 bounded / 449 pending / 23 metadata. Discovery had 280 API gaps; two modeled queries and three retirements close exactly five.

`C_Scenario.GetBonusSteps` enumerates actual bonus IDs, including completed steps, in supplied order. `GetBonusStepRewardQuestID` reads only matching bonus rewards. Current cached BonusObjectiveTracker calls both. No state schema, existing scenario-info behavior or vendor Lua changed.

## Problematic cases

The 275 exact API gaps remain named in [gap review](../../../data/patch-api/evidence/6.0.2-session-2026-10-08/p602-gap-review.json). Reasons distinguish missing garrison building/follower/mission/recruitment/shipment providers, applicant/social matchmaking operations, monument persistence, historical toy filters, indexed quest/task/geographic catalogs, retired combat stats and talent contracts. Current-published functions conflicting with later removal remain explicit gaps, not forced retirements.

Main prose separately records historical tuples/boolean returns, combat-log payloads, GUID/item-link schemas, item-bonus DB2 evaluation, source-era difficulty mapping, XML/atlas semantics and disputed zero-size anchors. Unknown item-bonus actions remain unknown. Cinematic 3D rendering is intentionally unsupported. No WoW install is available for native/CASC texture acceptance. Enum values, including the source's `MULITSTRIKE` spelling, are retained literally rather than corrected or assigned fabricated current semantics.

## Retirements

New: `C_Scenario.GetBonusCriteriaInfo`, `C_Scenario.GetBonusStepInfo`, `C_Vignettes.GetVignetteInstanceID`. Their four qualified/bare cached/source-test scans are empty. No pinned later register adds them. Marking prevents namespace lookup from fabricating functions.

[Scan records](../../../data/patch-api/evidence/6.0.2-session-2026-10-08/p602-retirement-scans.json): **276 untruncated whole-word `/usr/bin/grep` scans across 69 symbols**, including three enum removals. Cached scans exclude `*Documentation*`. Source/tests scans include unadorned callbacks, `pcall(Name, ...)`, guards and references because the bare name is scanned, not just call syntax. Retain all consumed/called members; unused existing absences are not new retirements. No global, event, widget or enum runtime removal was added.

Historical later scans pin master `8dd11c1b9`, p610 `0d3f53791` and p620 `73c8261c2`. Their original records remain intact. Integration commit `08de02e35` replaces 6.1.0/6.2.0/6.2.2 placeholders with real registers, in that order before 6.2.4/7.0.1/rest; no retirement was reversed or expanded.

## Integrated proof (2026-10-08)

Base master `d0fabed03`; runtime/test/tool scope pinned at `89d359e65`. [Integrated receipts](../../../data/patch-api/evidence/6.0.2-session-2026-10-08/integrated/): 53 registers and 50 saved extracts reproduce with recorded flags, including the separately pinned Warlords diff, 6.1.0 colon bullets and 6.2.4 indented lists. Three inherited extract failures remain exactly unchanged. Every master source blob and prior extraction mode is preserved.

Own 275 gaps and all 657 observations equal historical results: no supersession/accounting replacements needed. Branch publication sweeps/factory pass **54/54**; pinned master passes **53/53**. All observations on **52 other pages** match master exactly, including every later page affected by the three namespace retirements. [Gap comparison](../../../data/patch-api/evidence/6.0.2-session-2026-10-08/integrated/gap-comparison.json) retains each page's complete gap IDs.

| Regression scope | Result | Proof boundary |
|---|---|---|
| Own bonus/retirement behavior | Bare 2/2, cached 2/2 plus own sweep | Ordered state, updates, inactive state, invalid input and both lookup forms |
| Existing scenario-info queries | 11/11 | Bare integration fixture state |
| Objective tracker | Integration 10/10, prefork 13/13 | Loading, module publication, titles, visibility/layout and glyph behavior |
| Vignette consumer modules | Integration 34/34, prefork 24/24 | Shared-map providers, flight-map, navigation and POI consumers; no historical vignette tuple parity |
| Broader scenario selector | 9/10, identical on pinned master | `c_api_surface::scenario_defaults_are_not_c_api_temporary_shims` rejects the existing `c_scenario_info` module by source substring; assertion preserved, not weakened |

No cases match prefork `scenario` or shared-map-provider selectors. Cached scenario behavior is covered by own bonus tests and tracker module tests; cached vignette consumers by flight-map/navigation/POI tests. Zero-case runs receive no behavior credit.

Five Python fixture scripts pass **84/84** (generator 33, extractor 36, validation helper 8, portability 4, Warlords 3). `cargo fmt --check` and Mists `cargo check --no-default-features --features sound,gui,casc,client-mists --tests` pass; only seven unchanged vendor manifest warnings remain. All **37 prior validators**, enumerated with `git ls-tree` at pinned master, pass. Final startup comparison and two-phase portability receipts are recorded after their asynchronous jobs finish.

[Rebase mapping](../../../data/patch-api/evidence/6.0.2-session-2026-10-08/integrated/rebase-mapping.json) records 12 original/rebased commit patch IDs, pinned old/new blobs and the queued 6.1.0 pin's merged identity. Two historical generator blobs are retained where additive later parsers changed shared bytes. The original validator is archived and every invariant replays against mapped pins and recorded sweep/register inventories; its original seals, 276 scans, accounting, negative control and command proofs remain enforced. Shared inputs are pinned; current-byte seals cover only this session. No ignored/uncommitted validation input or external live comparison is used.

## Historical targeted proof

Runtime `c0f58dad8`: all publication sweeps plus factory **51/51**, own cached cases **3/3**, bare cases **2/2**, existing scenario-info regressions **11/11**. Dedicated own result capture uses the compiled prefork binary because the aggregate invocation omitted per-sweep output variables. Negative control adds exactly one failing row (275 to 276). Initial in-flight discovery is explicitly invalidated; expanded discovery and cached bonus RED retained.

Python fixtures: extractor 36, generator 32, validation helper 8, portability gate 4, new compact/diff parser 3; all pass. **50 registers / 47 extracts reproduce**. Three inherited extract failures (`12.0.5`, `12.0.7`, `12.1.0`) are unchanged from the base audit, not new failures. Before/after saved-input and extraction-mode results preserved. No `__pycache__` generated.

Both addons-enabled `wow-sim --no-saved-vars lua-errors` runs finish with `[]`: branch matches pinned master. That master has byte-identical runtime/Cargo/Interface inputs to the initial base. `cargo fmt --check` passes. Mists `cargo check --no-default-features --features sound,gui,casc,client-mists --tests` passes with zero non-vendor warnings (seven unchanged vendor warnings). `tools/check_patch_validators.py` passes at `cfee5d4ce`: clean 29/29 retained validators; later-audit 30/30 including the synthetic validator. [Finalization](../../../data/patch-api/evidence/6.0.2-session-2026-10-08/p602-finalization.json) records these results.

Validator reads shared source/test/tool/register inputs from recorded Git revisions. Complete register scope uses `historical_registers`; prior validators use `git ls-tree` at the base. Own immutable session files alone carry current-byte seals. No ignored scratch dependency or absolute checkout equality. Required history must be present.

## Sources

- [Spec](../../specs/patch-6-0-2-publication-sweep.md).
- [Register](../../../data/patch-api/sources/6.0.2-wikitext-register.json), [ledger](../../../data/patch-api/sources/6.0.2-page-coverage.json).
- [Historical evidence gate](../../../data/patch-api/evidence/6.0.2-session-2026-10-08/validate.py), [integrated validator](../../../data/patch-api/evidence/6.0.2-session-2026-10-08/integrated/validate.py).
- [Exact integrated commands and proof scope](../../../data/patch-api/evidence/6.0.2-session-2026-10-08/integrated/command-ledger.md).

## See Also

- [[patch-audit-validator-portability]] — historical/fresh-checkout gate.
- [[patch-6-2-4-api-audit]] — later source supersession and inherited reproduction boundaries.
- [[patch-7-0-3-api-audit]] — larger prepatch audit template.
