# Patch 4.2.0 API audit

Pinned historical retail inventory, page 315583/revision 3045158 (2021-08-22T03:01:41Z), retrieved 2026-10-09. Development accounting only; coordinator owns final gates.

## Source boundary

65 global occurrences: 63 added, two removed. Both literal headers match parsed counts. One extracted navigation metadata row; zero behavior prose or signature rows. Existing generator with `--client-line retail` and default extractor suffice; no parser changes.

## Publication scope

Historical development probe used 5.0.1–12.1.0 retail successors and queued 4.3.0/4.3.4 placeholders. Integrated probe now includes both actual registers; Classic histories excluded. Own targeted prefork GREEN passes 1/1 at `f3bdb8a26`, with all 65 observations retained and the exact 31-gap set. One bounded retail friend-index model change; no retirements.

## Bounded model fix

`BNGetFriendIndex(accountID)` now reads the current position in `SimState.bnet_friends`, consistent with `C_BattleNet.GetFriendAccountInfo(index)` rather than stale `friend_index` fields. Retail-only; Classic and PTR registration unchanged. Cached `Blizzard_FriendsFrame/Mainline/FriendsFrame.lua:2545` passes a Battle.net account ID. Two local model tests failed on missing global before implementation. Unknown-ID nil is inferred; this page provides no signatures and no native parity is claimed. No shims/fallbacks added. Targeted integration GREEN passes 2/2 at `6b89034a3`. Initial publication RED observed 32 mismatches; the local model closes one, leaving 31 observed by the own-sweep GREEN run.

## Historical development coverage matrix

Counts derive from [results](../../../data/patch-api/evidence/4.2.0-session-2026-10-09/development-results.json) and [per-ID ledger](../../../data/patch-api/sources/4.2.0-page-coverage.json), not native receipts.

| Source / capability | Accounting | Proof scope |
|---|---:|---|
| Global inventory | 63 additions + 2 removals = 65 | Literal source references, matching headers |
| Current retail publication/absence | 34 bounded, 31 gaps | Own cached-Game prefork case only |
| Bounded observations | 22 published, 8 absent, 4 cached deprecation wrappers/aliases | No signature/output/gameplay credit |
| Non-inventory | 1 navigation metadata row | Exact default extract |
| Behavior prose / signatures | 0 / 0 | Not present in pinned page; linked references not expanded |
| Local Battle.net friend lookup | 1 gap closed, 2 tests pass | Current list order/ID lookup and mutation; inferred unknown-ID nil |
| Retail successor history | 64 actual registers, 2 pending placeholders | 5.0.1 onward through 12.1.0; no Classic registers |
| Native / Classic runtime parity | Unproven / untested | No such claims |

## Historical retained gaps

Historical 31-gap breakdown below remains development evidence. Current ledger and `tests/data/patch_4_2_0_sweep_known_gaps.json` retain 30 gaps after the integrated removal described next.

| Group | Rows | Boundary |
|---|---:|---|
| Raid profiles | 14 | No legacy profile CRUD/options/copy/position model; Classic defaults untouched |
| War games | 7 | No legacy list/header/selection model; cached Mists subpaths are not loaded-retail proof |
| Encounter journal | 3 | Missing map query, historical section tuple, difficulty-mask contract |
| Existing legacy retirement mismatch | 1 | `EJ_GetLootInfoByIndex` callable despite later removal expectation; retained, not retired in this slice |
| Display refresh | 2 | Refresh-rate state/control not equivalent to frame/update rate |
| OS locale | 1 | OS locale input not proven by game locale |
| PVP rewards | 1 | Historical reward tuple/state unspecified |
| Travel pass | 1 | Historical eligibility/service policy unmodeled |
| URL launch | 1 | URL-index catalog and launch behavior unmodeled; actual cached callers retained |

## Consumers and retirement boundary

[Full scan](../../../data/patch-api/evidence/4.2.0-session-2026-10-09/caller-scan.json): 1,652 own source text files, 2,149 own test text files (embedded Lua included), and 2,551 cached-retail Lua files. Scanned both source removals and initial gap identities without truncating matches. All are globals, so fully qualified and bare names coincide. `KeyRingButtonIDToInvSlotID` and `strreplace` have no matches and are already absent; no retirement edits or Classic caller migrations. Cached root includes Mists/Glue/documentation paths; a match does not prove that the runtime loaded that file. Existing deprecated wrappers are unmodified.

## Integrated successors

Actual 4.3.0/4.3.4 registers replace both placeholders. Sole overlap: `EJ_SetDifficultyByMask`, removed by 4.3.0 ID `wt-global-api-EJ_SetDifficultyByMask-88`. Current absence satisfies that expectation, moving one row to bounded publication/absence: 35 matching observations, 30 gaps. No difficulty-mask model or native parity credit. Historical [successor pins](../../../data/patch-api/evidence/4.2.0-session-2026-10-09/later-register-pins.json), 31-gap results and 32-gap negative remain unchanged; separate [current observations](../../../data/patch-api/evidence/4.2.0-session-2026-10-09/integrated/current-results.json) record the closure. Updated accounting fixtures pass 6/6; broader integrated acceptance remains pending.

## Development proof

Own publication RED failed with 32 mismatches; local model RED failed 2/2 on missing `BNGetFriendIndex`, then GREEN passed 2/2. Own publication GREEN passed 1/1 with 31 retained gaps. Added-row negative was rejected at the row-count boundary (66 versus 65), without result output; a same-size replacement negative reached classification, added exactly `negative-p420-missing-global` (31 → 32 gaps), and failed. Both controls and historical RED bytes are retained. The initial copied zero-row count was rejected before observation; fixed to derive the count from register data. Source/accounting fixtures GREEN pass 6/6 at `a0db0fa0d`; [development proof ledger](../../../data/patch-api/evidence/4.2.0-session-2026-10-09/development-proof.json) records commands, exact revisions/scopes, exits and artifact/input hashes.

Only targeted development tests and changed-file formatting executed. No check/lint/type/readability/coverage, all-publication sweep, smoke, full suite, final acceptance, push/merge/deploy/delegation. Existing native/vendor warnings in build logs are not a clean-warning claim. Coordinator owns integration and final gates.

## Sources

- [Pinned source](../../../data/patch-api/sources/4.2.0-api-changes.wikitext)
- [Source pin](../../../data/patch-api/evidence/4.2.0-session-2026-10-09/source-pin.json)
- [Coverage](../../../data/patch-api/sources/4.2.0-page-coverage.json)
- [Spec](../../specs/patch-4-2-0-publication-sweep.md)

## See Also

- [[patch-5-0-1-api-audit]] — first available later retail register.
