# Patch 12.0.7 API Audit

### Page accounting — preparation checkpoint (2026-10-03)

12.0.7 now has a tracked `patch-page-coverage/v1` ledger at `data/patch-api/sources/12.0.7-page-coverage.json`, modeled on [[patch-12-0-5-api-audit]]. It binds `data/patch-api/sources/12.0.7-register.json` by SHA-256 `389e3b19174bf77c3646028f764cf186ccfe1b7dddaca2a3b3fcba75e3bdec60` and accounts every nonblank line of the retained [source snapshot](../../../data/patch-api/sources/12.0.7-api-changes.txt).

**0 capabilities/166 source IDs; 153 pending /0 bounded /0 partial /13 metadata**: thirteen heading/context rows are metadata-only; triage separately proposes three rows for inherited 12.0.5 capabilities, not yet promoted. Triage and batch plan: [evidence](../../../data/patch-api/evidence/12.0.7-session-2026-10-03/README.md). 73 rows need proof against existing implementations, 59 are modelable bounded work, and 18 remain evidence-blocked. Audit **IN PROGRESS**, not finished.

### Source scope and occurrence distinction

The 131 named occurrences (79 added/29 changed/23 removed) are preserved with derived section/subject/source-line IDs, plus 35 supplemental prose, wrapper, CVar completeness and editorial rows missing from that register. Section totals: Source context 14, Blue posts 16, Global API 52, ScriptObjects 36, Widgets 20, Events 17, CVars 6, Deprecated API 5. Combined lines retain all their semantic obligations. Source text is undated for the individual blue-post bullets, so `prose-undated-NNN` does not invent post chronology.

The retained text covers the recorded 12.0.5 build 67602 -> 12.0.7 build 68182 delta, but it is a crawler excerpt, not authenticated full-page extraction; it explicitly omits 14 CVar addition names and five removal names. The aggregate omission claim stays blocked. No consolidated enum/structure section is present; EncounterUnitStatus and CalendarTime claims are retained as prose, not silently discarded. Full-page provenance/completeness still needs historical source capture and reconciliation before page completion.

The occurrence manifest `data/patch-api/12.0.7.json` and [[patch-12-0-7-occurrence-inventory]] remain separate: 29 implemented/101 best-effort/1 exception-requested is not page-row conformance. Current cached generated declarations may postdate 12.0.7. `retail-12-0-7` includes `retail-12-0-5`, enabling bounded earlier capability reuse where policy is unchanged, but not changed vehicle-aura, font validation, marker-prefix or private-aura-M+ permission behavior. Default retail is newer than 12.0.7; strict epoch proof is separate.

Read-only preparation executed no builds/tests or runtime gates. No repo files changed. See staged `triage.md`, `batches.md` and `source-row-map.json` for exact per-line decisions, current provider/declaration anchors, required proofs and missing evidence. Existing bridge work below remains historical and qualified; this checkpoint does not promote it automatically.

Patch 12.0.7 API work in wow-ui-sim separates safe additive compatibility bridges from security, taint, and secret-value behavior that must be proven with live Blizzard observations before implementation.








## Wikitext completeness check and publication sweep — 2026-10-04

The page's raw wikitext (revid 6794100, [retained](../../../data/patch-api/sources/12.0.7-api-changes.wikitext)) holds five collapsed consolidated tables (Global API, ScriptObjects, Widgets, Events, CVars) with **174** entries; header counts equal parsed counts in every section. **65** are missing from the crawler register: 10 added and 4 removed globals, all 20 changed globals (C_Club opaque member IDs, C_CombatText, C_DelvesUI, C_EncounterEvents trigger/alpha, C_EncounterWarnings, `GetInstanceInfo` ret11, `SimulateMouse*`), 8 added and 4 changed script-object methods, and 14 added / 5 removed CVars (2 are console commands). This resolves row 172's unnamed CVars.

Conversely, **22** crawler-register symbols appear in no consolidated table of the current revision, the 12.0.7 (68182) revision 6745177, or the PTR (67669) revision 6733756: the `Get/SetSecurePending*Callback` globals, `C_PingSecure.Clear/SetPendingPingOffScreenCallback`, `C_QuestHub.GetDragonridingRacesForAreaPOI`, eight `DurationTextFormattingOptions`/`DurationTextRawValue` methods, `GameTooltip_AddMoneyLine` (blue-post prose only) and the four `GetAutoComplete*`/`IsRecognizedName` removals (Deprecated API prose only). Their origin is unverified; only `C_PingSecure.SetPendingPingOffScreenCallback` is declared by the cached Retail docs.

The [publication sweep](../../specs/patch-12-0-7-publication-sweep.md) probes all 174 rows on default Retail: 147 OK, 27 reviewed gaps. It retired four removed members the autostub still fabricated. No row has a later 12.1.0 add/remove, so supersession applies to none today.

## Re-review promotes 21 rows — 2026-10-04

**37 pending / 67 bounded / 49 partial / 13 metadata** of 166 IDs.

[Re-review](../../../data/patch-api/evidence/12.0.7-session-2026-10-03/p1207-r9-r10-review.md) of the round 9 fixes and row 137: ACCEPT WITH QUALIFICATIONS. Both round 8 regressions are closed. Twenty `DurationTextBinding` rows and `GetModelUnitGUID` are bounded, each for the narrow scope in its ledger note; the broad script-object prose row stays partial. Qualification: duration and clock objects are still Lua table proxies with replaceable timing slots, so no claim that all duration state is host-owned. No tests were rerun for this step; the proof is the rounds 9–10 master run above.

## Rounds 9–10 accounted; 12.0.7 first pass complete — 2026-10-04

Nineteen capabilities; **37 pending / 46 bounded / 70 partial / 13 metadata** of 166 IDs. Every pending row carries a reason in the [ledger](../../../data/patch-api/sources/12.0.7-page-coverage.json).

- Round 9 fixed the two round 8 review defects: the text binding computes its values in Rust from timing state (`c1ded7c63`), and `SetFontString` requires a genuine FontString handle (`cbe938d38`). Both have failing-first regression tests. The 21 rows stay partial until re-reviewed.
- Round 10: `ModelSceneActorBase:GetModelUnitGUID` partial (no independent review). `ENCOUNTER_TIMELINE_EVENT_COLOR_CHANGED` stays pending: its producer is gated on 12.1.5 and the new tests are compiled out on the default build.
- Sixteen never-attempted rows were examined and are blocked: no cached declaration (duration formatting options, raw values, dragonriding races), no opaque club member identity, no limited-input model, no VM tracking of secret access per call frame.

Proof: master run at `2136907ab`, 72 passed / 0 failed over seven filters; lib `startup_globals::` 26 passed / 0 failed; startup `lua-errors` `[]`; `cargo fmt --check` exit 0 ([integration](../../../data/patch-api/evidence/12.0.7-session-2026-10-03/rounds-9-10-master-green.log.txt), [lib](../../../data/patch-api/evidence/12.0.7-session-2026-10-03/rounds-9-10-master-lib.log.txt)). Results: [round 9](../../../data/patch-api/evidence/12.0.7-session-2026-10-03/p1207-r9-result.md), [round 10](../../../data/patch-api/evidence/12.0.7-session-2026-10-03/p1207-r10-result.md).

Whole-pass limits: default feature set only; strict 12.0.7 and older client profiles were never built, although several changed producers are shared with them. No broad suite was run. Partial rows read explicit host state but lack native derivation, permissions or timing.

## Round 8 accounted — 2026-10-04

Eighteen capabilities; **38 pending / 46 bounded / 69 partial / 13 metadata** of 166 IDs.

- Partial, 21 rows: `DurationTextBinding` and the duration rows it touches, on the existing provider.
- [Review](../../../data/patch-api/evidence/12.0.7-session-2026-10-03/p1207-r8-review.md): REJECT for bounded credit, partial only. Two open defects: the binding samples by calling replaceable Lua methods on the duration instead of reading timing state, and `SetFontString` accepts any table that reports `GetObjectType() == "FontString"`.
- The zero-span `HasExpired` lib assertion now expects `true`, matching [duration-core](../../specs/duration-core.md); the cached declaration does not decide the case, so this is simulator inference, not native proof.

Proof: master run, 60 passed / 0 failed over the binding, duration and widget-aspect modules; lib `startup_globals::` 26 passed / 0 failed after a clean rebuild; startup `lua-errors` `[]`; `cargo fmt --check` exit 0 ([integration](../../../data/patch-api/evidence/12.0.7-session-2026-10-03/round-8-master-green.log.txt), [lib](../../../data/patch-api/evidence/12.0.7-session-2026-10-03/round-8-master-lib.log.txt)). The lib run also closes the rerun owed for round 7. [Result](../../../data/patch-api/evidence/12.0.7-session-2026-10-03/p1207-r8-result.md).

## Round 7 accounted — 2026-10-04

Seventeen capabilities; **59 pending / 46 bounded / 48 partial / 13 metadata** of 166 IDs.

- Partial, 8 rows: `Button` state/enabled and `ScrollFrame` scroll-offset getters and setters carry secret origin with authenticated arguments.
- Five `SetFont` rows stay pending: only the authentication prerequisite landed; no authoritative source for valid font assets and heights.

Proof: master run at `372f40db1`, new module 8 passed / 0 failed, startup `lua-errors` `[]`, `cargo fmt --check` exit 0 ([log](../../../data/patch-api/evidence/12.0.7-session-2026-10-03/round-7-master-green.log.txt)). The lib-test compile was killed by memory pressure and not rerun at accounting time. [Review](../../../data/patch-api/evidence/12.0.7-session-2026-10-03/p1207-r7-review.md): ACCEPT WITH QUALIFICATIONS — ordinary public behavior unchanged by source comparison, authentication precedes mutation and dispatch. [Result](../../../data/patch-api/evidence/12.0.7-session-2026-10-03/p1207-r7-result.md).

## Round 6 accounted — 2026-10-04

Sixteen capabilities; **67 pending / 46 bounded / 40 partial / 13 metadata** of 166 IDs.

- Partial, 20 rows: vehicle aura marker, Mythic+ sound permission, `IsKnownFile`/`IsLooseFile`, nine `CHAT_MSG_*` events, `URL_TEXTURE_REQUEST_RESULT`, three CPU-usage getters, unsupported-unit defaults.
- Pending with notes: the "important" aura filter (cached Lua republishes it) and frame strata (cached policy is `NotAllowed`).

Proof: master run at `2f551ce7e`, 56 passed / 0 failed, startup `lua-errors` `[]`, `cargo fmt --check` exit 0 ([log](../../../data/patch-api/evidence/12.0.7-session-2026-10-03/round-6-master-green.log.txt)). [Review](../../../data/patch-api/evidence/12.0.7-session-2026-10-03/p1207-r6-review.md): ACCEPT WITH QUALIFICATIONS; [result](../../../data/patch-api/evidence/12.0.7-session-2026-10-03/p1207-r6-result.md).

Behavior changes: asset predicates answer from a host catalog (empty means false) instead of the filesystem; six core unit queries validate selectors more strictly; CPU-usage getters changed arity and reject secret arguments. Chat tests prove a four-field projection, not the cached 18-field payload.

## Rounds 4–5 accounted — 2026-10-04

Fourteen capabilities; **87 pending / 46 bounded / 20 partial / 13 metadata** of 166 IDs.

- Partial, 13 rows: six `C_PartyInfo` role/leader/removal functions, solo `GROUP_FORMED`, `/tm ~marker`, three housing queries, `C_MerchantFrame.GetMerchantCurrencies`, Mythic+ `CalendarTime`. All read explicit host state; native derivation, permissions and timing are unproved, so none is bounded.
- Ready-check rows 038/040 stay pending: they only inherit the 12.0.5 lockdown.

Proof: master run at `3557d5f1d`, 119 passed / 0 failed over 10 filters, startup `lua-errors` `[]`, `cargo fmt --check` exit 0 ([log](../../../data/patch-api/evidence/12.0.7-session-2026-10-03/rounds-4-5-master-green.log.txt)). [Round 5 review](../../../data/patch-api/evidence/12.0.7-session-2026-10-03/p1207-r5-review.md): ACCEPT WITH QUALIFICATIONS. [Round 4 review](../../../data/patch-api/evidence/12.0.7-session-2026-10-03/p1207-r4-review.md): ACCEPT WITH QUALIFICATIONS (arrived after accounting; rows unchanged). Its behavior notes: Mythic+ history entries without a completion date are now omitted, the weekly-best tuple has six slots, and the no-merchant zero-result case is unmodeled. Results: [round 4](../../../data/patch-api/evidence/12.0.7-session-2026-10-03/p1207-r4-result.md), [round 5](../../../data/patch-api/evidence/12.0.7-session-2026-10-03/p1207-r5-result.md).

Behavior changes from round 5: individual demotion no longer disables everyone-is-assistant mode; an unknown leader name no longer resets leadership to the player; roster mutations on an inactive group are no-ops; macro text and button validation is stricter.

## Rounds 1–3 accounted — 2026-10-04

Ten capabilities; **100 pending / 46 bounded / 7 partial / 13 metadata** of 166 IDs ([ledger](../../../data/patch-api/sources/12.0.7-page-coverage.json)).

- Bounded: 27 removal and deprecation-forwarding rows, `GetFileID`, bag free total, delve entrance title, manual clocks and duration predicates (8 rows), `GameTooltip_AddMoneyLine`, `ENCOUNTER_END` payload, six CVars.
- Partial: Battle.net invite (2), `GetClock`/`SetClock` (no clock type check), namespace ping setter, two prose rows.
- Left pending with reasons: world-tier difficulty (constant), eight legacy pending-callback rows (Lua-table storage), warning colours and `GetEventColor` (contradict the cached declaration).

Proof: master run at `d8aaad4f6`, 166 passed / 0 failed over 13 filters, startup `lua-errors` `[]`, `cargo fmt --check` exit 0 ([log](../../../data/patch-api/evidence/12.0.7-session-2026-10-03/rounds-1-3-master-green.log.txt)). Lib `startup_globals::` 25 passed / 1 failed: the zero-span `HasExpired` assertion, failing since before this audit. Reviews (source read, no reruns): [round 2 record](../../../data/patch-api/evidence/12.0.7-session-2026-10-03/round-2-record.md), [round 1 review](../../../data/patch-api/evidence/12.0.7-session-2026-10-03/p1207-r1-review.md). Both reviews said REJECT against the strict standard; the blocking namespace-lookup finding was fixed (`9ce93bb12`), and the rest is reflected as partial or pending rows above. The CVar slice has no independent review and no RED, since its producers pre-existed.

Limits: default feature set only; strict 12.0.7 and older-epoch branches unexecuted. Round 1 and 2 integrator logs were lost in a host crash; the master run above is the surviving execution evidence.

## Content

### Source scope

The audit source was the Warcraft Wiki Patch 12.0.7 API changes page, preserved as the checked-in [12.0.7 API-change source snapshot](../../../data/patch-api/sources/12.0.7-api-changes.txt). The page covers the 12.0.5 `(67602)` to 12.0.7 `(68182)` API diff and 12.0.7 blue-post notes.

The machine register is `data/patch-api/12.0.7.json`, sourced from the categorized `data/patch-api/sources/12.0.7-register.json`; [[patch-12-0-7-occurrence-inventory]] is its human-readable inventory. It preserves 131 named occurrences—79 added, 29 changed, and 23 removed. Current occurrence-level classification is **29 implemented, 101 best-effort, 1 repository-scope-authorized impossible exception-requested, and 0 untriaged**. The crawler's unnamed CVar claims remain explicit source metadata rather than invented symbols.

### Completed compatible bridge work

The simulator now provides modeled 12.0.7 social/party mutations:

- `C_BattleNet.InviteFriend` appends a queryable offline Battle.net friend to `SimState.bnet_friends`, ignores empty/duplicate invites, and is visible through `GetNumFriends`/`GetFriendAccountInfo`.
- `C_PartyInfo.DoReadyCheck` / `ReadyCheck` start a ready-check state, dispatch `READY_CHECK`, and expose `GetReadyCheckStatus("player") == "waiting"` with positive `GetReadyCheckTimeLeft()`.
- `C_PartyInfo.ConfirmReadyCheck(ready)` records ready/not-ready state, dispatches `READY_CHECK_CONFIRM` and `READY_CHECK_FINISHED`, and clears the time-left value.
- `C_PartyInfo.IsGUIDInGroup(guid)` now reads the existing simulator party roster and synthetic `UnitGUID("partyN")` values instead of always returning false.
- `C_PartyInfo.SetEveryoneIsAssistant`, `PromoteToAssistant`, `DemoteAssistant`, and `PromoteToLeader` now mutate existing party assistant/leader state used by `IsEveryoneAssistant`, `IsGroupLeader`, and `UnitIsGroupLeader`.

The simulator also provides safe 12.0.7 additive probes for API names that can be inert without pretending to model live game state:
- `C_DelvesUI.GetDelveEntranceTitleString` (now Rust-backed by the Delves UI surface; it shares the seeded entrance header text)
- `C_DurationUtil.CreateManualClock` (now installed by the Rust `C_DurationUtil` surface alongside duration objects)
- `C_DurationUtil.CreateDurationTextBinding` (now best-effort tracks documented binding methods, enabled/default state, duration objects, formatter/text-format storage, and font-string update hooks)
- `C_EncounterTimeline.GetEventColor` (now Rust-backed best-effort delegated to `C_EncounterEvents` color state)
- `C_HousingCatalog.GetCatalogCategoryAndSubcategoryNames` (now Rust-backed deterministic nil until a catalog model exists)
- `C_HousingCustomizeMode.RoomConnectionSupportsDoorType` (now Rust-backed deterministic false until room-door compatibility data exists)
- `C_HousingLayout.CanSetViewedFloor` (now Rust-backed deterministic false until viewed-floor permissions are modeled)
- `C_MerchantFrame.GetMerchantCurrencies` (now Rust-backed deterministic empty table until merchant currency state exists)
- `C_PartyInfo.ConfirmReadyCheck`, `DoReadyCheck`, `UninviteUnit` (now Rust-backed; `UninviteUnit` mutates the existing party roster by unit token/name)
- `C_PingSecure.ClearPendingPingOffScreenCallback` (now Rust-backed through the shared PingSecure callback table)
- `C_QuestHub.GetDragonridingRacesForAreaPOI` (now Rust-backed deterministic empty table until area-POI race content exists)
- `C_UIFileAsset.GetFileID`, `IsKnownFile`, `IsLooseFile` (best-effort limited-listfile surface; `IsLooseFile` currently returns false and has no local loose-file registration model)
- `GetEventCPUUsage`, `GetFunctionCPUUsage`, `GetScriptCPUUsage` (now provided by the shared performance-metric defaults module)
- secure pending callback getters/setters: button, ping off-screen, toggle run (now Rust-backed through the shared PingSecure callback table; callback storage only, not real secure-execution enforcement)
- `GameTooltip_AddMoneyLine` — `8097a844c` removed an unsupported bootstrap prefix-text shim; cached `Blizzard_GameTooltip` owns the loaded helper. `e393e2f8e` verifies three hash-matched focused loaded helper/mail-consumer assertions and two fresh bootstrap-surface checks: concrete coin-atlas text, single-space zero output, boolean highlight/red colors, and label-before-money order. Its dependency closure has 128 distinct Lua-error headers plus 18 suppression notices, so this is not clean whole-addon/container/layout startup proof. Native historical-client/locale/rendering behavior remains open. See [[tooltip-money-line]].
- `ENCOUNTER_TIMELINE_EVENT_COLOR_CHANGED` registration under `retail-12-0-7`

`C_UIFileAsset.IsLooseFile` has only constant-false bounded tests for known and unknown paths. No local loose-file registration/classification model exists, so this supplies no evidence for a true loose-file result, numeric IDs, normalization, invalid inputs, or native file semantics; no credit or runtime change is claimed.

Already-existing coverage from prior work included `C_Container.CalculateTotalNumberOfFreeBagSlots`, `C_DelvesUI.GetWorldTierDifficultyForActivePlayer`, `C_PingSecure.SetPendingPingOffScreenCallback`, and `URL_TEXTURE_REQUEST_RESULT` registration.

Key implementation locations:

- `src/c_api/c_battle_net.rs` — modeled `C_BattleNet.InviteFriend` backed by `SimState.bnet_friends`.
- `src/c_api/c_party_info.rs`, `src/lua_api/globals/group_verbs.rs`, `src/lua_api/state/support_types.rs` — modeled ready-check state, C_PartyInfo ready-check methods, GUID-in-group membership, party assistant/leader mutators, C_PartyInfo roster uninvite, global ready-check status/time probes, and immediate ready-check event dispatch.
- `src/lua_api/globals/missing_surface/delves_ui.rs` — seeded Delves UI probe data, including the 12.0.7 entrance title and world-tier difficulty methods.
- `src/c_api/c_housing.rs` — Rust-backed 12.0.7 housing catalog/customize/layout best-effort probes plus 12.1 housing state.
- `src/c_api/c_merchant_frame.rs` — Rust-backed deterministic empty merchant currency list.
- `src/c_api/c_quest_hub.rs` — Rust-backed deterministic empty Dragonriding race list.
- `src/c_api/c_ui_file_asset.rs` — best-effort `C_UIFileAsset` path/fileDataID lookup backed by the bundled limited listfile.
- `src/c_api/c_ping_secure.rs` — Rust-backed PingSecure namespace callbacks plus the 12.0.7 secure pending button/ping/toggle getter/setter globals, all using the shared callback table.
- `src/lua_api/globals/lua_duration_object.rs` — Rust-backed `C_DurationUtil.CreateDuration`, `CreateManualClock`, current-time/default duration object surface, and best-effort duration-object clock/lifecycle methods.
- `src/lua_api/globals/missing_surface/encounter_events.rs` — Rust-backed `C_EncounterTimeline.GetEventColor` bridge over the existing encounter-event color state.
- `src/lua_api/workarounds/temporary/formatting_utility_defaults.rs` — shared formatting helpers; it intentionally does not define the addon-owned `GameTooltip_AddMoneyLine` global.
- `src/lua_api/workarounds/temporary/performance_metric_defaults.rs` — shared CPU/framerate/download metric defaults, including 12.0.7 CPU usage probes.
- `src/c_api/duration_text_binding.rs` — table-backed `CreateDurationTextBinding` model, including [configuration assignment/copy](../../specs/duration-text-binding.md); existing best-effort formatting limits remain.
- `src/lua_api/workarounds/mod.rs`, `src/lua_api/workarounds/temporary/mod.rs` — bootstrap registration.
- `src/event/valid_events.rs` — 12.0.7 event registration gate.
- `src/loader/tests/wow_api_globals/startup_globals.rs` — regression test for safe 12.0.7 global bridges.

### Verification

Targeted proof logs:

- `/tmp/wow_12_0_7_target5_test-safe-12-0-7.out`
- `/tmp/wow_12_0_7_target5_test-events-12-0-7.out`

Full proof logs after the final bridge pass:

- `/tmp/wow_12_0_7_full_fmt-check.out`
- `/tmp/wow_12_0_7_full_check-default.out`
- `/tmp/wow_12_0_7_full_check-retail-12-0-7.out`
- `/tmp/wow_12_0_7_full_check-retail-12-1.out`
- `/tmp/wow_12_0_7_full_test-safe-12-0-7.out`
- `/tmp/wow_12_0_7_full_test-events-12-0-7.out`
- `/tmp/wow_12_0_7_full_test-safe-12-1.out`
- `/tmp/wow_12_0_7_full_build-retail-12-0-7.out`
- `/tmp/wow_12_0_7_full_lua-retail-12-0-7.out`

Additional proof for the Rust-backed trivial namespace pass:

- `/tmp/pi-pyrun-12-0-7-trivial-namespaces-proof.log` — `cargo fmt --check`, default `cargo check`, 12.0.7/12.1 `cargo check`, 12.0.7/12.1 focused safe bridge tests, 12.0.7 `wow-sim` build, and 12.0.7 `lua-errors`.

Additional proof for the encounter-color and tooltip-money pass:

- `/tmp/pi-pyrun-12-0-7-color-money-proof.log` — `cargo fmt --check`, default `cargo check`, 12.0.7/12.1 `cargo check`, 12.0.7/12.1 focused safe bridge tests, 12.0.7 `wow-sim` build, and 12.0.7 `lua-errors`.

Focused proof for recovered 12.0.7 CVar defaults:

- `/tmp/pi-pyrun-12-0-7-cvars-test.log` — profile-gated recovered CVar defaults/removals regression test.

Focused proof for duration-object best-effort methods:

- `/tmp/pi-pyrun-12-0-7-duration-object-test.log` — 12.0.7 safe bridge regression test covering DurationText no-argument construction, duration object clock state, and deterministic lifecycle queries.

Focused proof for the secure-pending callback Rust move:

- `/tmp/pi-pyrun-12-0-7-secure-callback-final-test.log` — 12.0.7 safe bridge regression test, including button/ping/toggle callback set/get behavior.

Rust readability metrics are under `/tmp/rust_readability_12_0_7`, `/tmp/rust_readability_12_0_7_trivial_namespaces`, `/tmp/rust_readability_12_0_7_color_money`, `/tmp/rust_readability_12_0_7_secure_callbacks_final`, and `/tmp/rust_readability_12_0_7_duration_object` with no high-complexity findings.

### CVar changes

The simulator now profile-gates the recovered exact 12.0.7.68235 CVar delta in `src/cvars.rs`: 17 additions, five removals, and the `gxWindowedResolution` default change from `1920x1080` to `auto`. `patch_12_0_7_cvar_defaults_match_retail` verifies all recovered defaults/removals under `retail-12-0-7`.

**Source provenance gap — untriaged, not approved:** the crawler excerpt claimed 20 additions and five removals while naming only six additions and no removals. A separate recovered historical diff identifies 17 additions, five removals, and the default change above; comparing the two leaves three addition claims unsupported. The 131-row source register therefore contains only the six CVar occurrences named by its checked-in crawler source, while all fourteen unnamed addition claims and five unnamed removal claims remain source metadata. Do not invent occurrence IDs; no exception approval is requested or recorded.

### Itemized status matrix

**Current occurrence classification:** the 35 added global APIs and named CVar rows are covered by profile-gated bridge/default tests; clock methods are directly state-tested; duration object/text methods, service-backed globals, changed events/widgets, and retained removals are documented best-effort where exact Blizzard fidelity is not proven. The eight `DurationTextFormattingOptions`/`DurationTextRawValue` rows remain best-effort stale-extraction compatibility entries because their factories return nil; this is documented behavior, not an exception request.

#### Pending classification and exception candidates

The occurrence-level checklist has no untriaged rows. One impossible exception-requested row is authorized by repository scope: `ModelSceneActorBase.GetModelUnitGUID` remains absent because the permanent project 2D/no-3D rule in `AGENTS.md#intentional-gaps` excludes the required 3D model subsystem. This is existing project-scope authority, not a newly granted user exception. Remaining payload, secrecy, and load-order limitations retain explicit best-effort notes rather than hidden claims of native fidelity.

1. **Strict API removals:** exact 12.0.7 startup proof finds 11 names absent: `BNInviteFriend`, `ConfirmReadyCheck`, `DemoteAssistant`, `DoReadyCheck`, `GetMerchantCurrencies`, `IsGUIDInGroup`, `PromoteToAssistant`, `PromoteToLeader`, `SetEveryoneIsAssistant`, `GetAutoCompletePresenceID`, and `IsRecognizedName`. Six compatibility functions remain: `C_ClickBindings.GetStringFromModifiers`, `C_ClickBindings.MakeModifiers`, `C_Spell.GetMawPowerBorderAtlasBySpellID`, `UninviteUnit`, `GetAutoCompleteResults`, and `GetAutoCompleteRealms`. This is best-effort availability evidence; retained wrappers still need source/lifecycle review before strict hiding.
2. **Minimap method removals:** `SetBlipTexture`, `SetCorpsePOIArrowTexture`, `SetIconTexture`, `SetPOIArrowTexture`, `SetPlayerTexture`, `SetStaticPOIArrowTexture`. The simulator retains these methods as compatibility behavior; focused widget-surface coverage records the retained availability as best-effort.
3. **Secret/aspect widget changes:** focused 12.0.7 proof verifies the four Button methods, four ScrollFrame methods, five `SetFont` methods, and all six proposed removed Minimap texture setters remain callable compatibility functions. Exact secret/aspect/asset semantics remain best-effort. `ModelSceneActorBase:GetModelUnitGUID` remains absent under the repository-validated permanent no-3D scope; its impossible exception is scope-authorized rather than exposed as a guessed method.
4. **Restricted unit-token nil/default matrix:** 12.0.7 changed `UnitGUID`, `UnitAura`, health/power, and related PvP-restricted token errors. Need exact token, caller-taint, and return-shape probes.
5. **Encounter behavior:** `ENCOUNTER_END.encounterUnitStatus` now has a bounded 12.0.7 simulator producer: `A_Admin.SimulateBossKill` accepts an explicit copied status list (or emits a fresh empty list). Independent proof passed fourteen 12.0.7 admin cases and twelve 12.0.5 controls at `9a8c05992`. `ENCOUNTER_TIMELINE_EVENT_COLOR_CHANGED` firing and five-second warning/custom-color semantics still need live encounter state.
6. **Secure/taint behavior:** `SimulateMouse` focus/combat/protected restrictions; `debugstack`/`debuglocals` secret propagation; secure `raidtarget set-unmarked` and `/tm ~marker`; `SetFrameStrata` secret-value error change.
7. **Mythic+/group/aura behavior:** `C_MythicPlus` `CalendarTime` result structs; solo follower-dungeon/delve `GROUP_FORMED`; `AuraData.isFromPlayerOrPlayerPet` vehicle ownership; `C_UnitAuras.AddPrivateAuraAppliedSound` active-M+ gate; `AuraFilters` removal of `IMPORTANT`.
8. **Event payload/security changes:** secret-chat-lockdown behavior for the nine changed `CHAT_MSG_*` events; `ClubMemberOpaqueId` second argument for `CLUB_MEMBER_ADDED`, `CLUB_MEMBER_PRESENCE_UPDATED`, `CLUB_MEMBER_REMOVED`, `CLUB_MEMBER_ROLE_UPDATED`, and `CLUB_MEMBER_UPDATED`.
9. **Duration fidelity:** exact `DurationTextBinding` formatter/component/secret semantics. Current object is documented **best-effort**, not a native-fidelity claim.
10. **Source conflicts:** three unnamed CVar additions claimed by the local snapshot; stale `DurationTextFormattingOptions`/`DurationTextRawValue` entries. The latter names only materialize as universal-fallback no-op functions returning nil, not documented factories. Need a corrected authoritative export before implementation.
11. **Probe reconciliation:** `EditModeLayoutProbe` has no captured/reconciled result in the audit. Need a probe result before classifying its layout behavior.

### Best-effort modeled guesses needing later probes

- **C_UIFileAsset path semantics** — `GetFileID` and `IsKnownFile` are backed by `data/wow-ui-sim-listfile.csv` through `limited_listfile`, with slash/case normalization and numeric IDs passed through. `IsLooseFile` currently returns false because loose-file install/source semantics are not modeled. Replace this with exact client behavior if PTR probes show different extension handling or loose-file rules.
- **Encounter timeline color state** — `C_EncounterTimeline.GetEventColor` is Rust-backed and mirrors the existing `C_EncounterEvents` color table, including alpha, then falls back to white when no event color is configured. Exact five-second-warning/custom-color behavior and event firing still need probes.
- **DurationTextBinding formatting** — the 12.0.7 patch bootstrap supplies a documented best-effort binding object for non-secret state, duration-object storage, formatter/text-format storage, and font-string update hooks. Exact Blizzard formatting/component semantics and secret-value handling still need live probes.
- **Tooltip money line formatting** — `GameTooltip_AddMoneyLine` uses the existing `GetMoneyString` fallback with thousands grouping and optional prefix text. Exact embedded-atlas/MoneyFormatter output remains a later fidelity improvement if addon screenshots require it.
- **Party GUID membership** — `C_PartyInfo.IsGUIDInGroup` treats the local player and synthetic party member GUIDs as in-group only while `SimState.party_group_active` is true. Exact instance-party category filtering and cross-realm GUID details remain future fidelity work if addons depend on them.
- **Party role mutators** — under `retail-12-0-7`, individual roles now use explicit member-name sets and exclusions rather than toggling the coarse everyone flag. Live restrictions, GUID-category inputs, solo formation and conditional markers have bounded default-Retail development proof. [Party slice contract and proof](../../specs/party-12-0-7-audit.md) owns behavior and exclusions; native permissions/identity/timing and historical-cache parity remain unverified.

### Paused / blocked items

Security/error-shape-sensitive items still need live Blizzard behavior, generated docs, or targeted probe addons:

- **Unit identity restricted-token behavior** — 12.0.7 changed restricted unit APIs such as `UnitGUID`, `UnitAura`, and health/power APIs from Lua errors to nil/default returns for unsupported PvP-restricted tokens. Need exact token matrix, return values, and addon-vs-Blizzard behavior.
- **Encounter payloads** — `ENCOUNTER_END` includes `encounterUnitStatus` tables with `creatureID`, `creatureName`, and `remainingHealthPercent`. `ee979b81b` adds bounded explicit simulator input rather than fabricated boss state: callers provide a dense list, omitted input emits a fresh empty list, and records are copied before dispatch. Independent proof passed fourteen 12.0.7 admin cases and twelve 12.0.5 controls at `9a8c05992`; native tracking, timing, validation, security, and consumer fidelity remain unverified. See [encounter-end unit status](../../specs/encounter-end-unit-status.md).
- **Encounter Events color event semantics** — `C_EncounterEvents` color state and timeline color reads are best-effort bridged, but exact five-second-warning custom-color behavior, persistence rules, and `ENCOUNTER_TIMELINE_EVENT_COLOR_CHANGED` firing need live behavior.
- **SimulateMouse taint and focus restrictions** — 12.0.7 changed taint propagation and imposed forbidden/locked/script-inaccessible/protected focus restrictions. This overlaps secure input and combat lockdown; implement only after exact behavior is known.
- **debugstack/debuglocals secret propagation** — returning secret values based on current/caller stack secret access requires rilua secret-value semantics, not a simple Lua stub.
- **Secure `raidtarget` `set-unmarked` and `/tm ~N` behavior** — [bounded public marker and cached-click proof](../../specs/party-12-0-7-audit.md) now passes. Native secure-click authorization and `SetRaidTarget` secret-argument policy remain unproved.
- **C_MythicPlus CalendarTime return structs** — `GetRunHistory`, `GetWeeklyBestForMap`, and `GetSeasonBestForMap` changed return struct shape. Need backing M+ data and exact CalendarTime fields.
- **GROUP_FORMED solo follower dungeon/delve behavior** — [explicit host-input tick producer](../../specs/party-12-0-7-audit.md) now proves entry/reentry payload and latch behavior. Actual server join input, stable identities and unobserved between-tick transitions remain unproved.
- **AuraData vehicle ownership and AddPrivateAuraAppliedSound allowance** — aura state/security behavior needs real aura model and M+ combat state.
- **SetFrameStrata secret-value error fix** — secret argument behavior needs live tests before changing method guards.
- **Button/scroll secret aspects and font asset validation** — focused 12.0.7 proof verifies current method availability, but exact secret aspect and asset-validation semantics remain best-effort and need probes.
- **Removed Minimap texture setters** — simulator method availability must be checked against active Blizzard sources before hiding anything; removing too early could break cached UI code.
- **Deprecated wrappers / removed globals** — exact 12.0.7 startup proof now records the per-name matrix: 11 names are absent and six remain functions. Strict removal timing for the six retained wrappers should only change if current Blizzard UI no longer needs them.

### Practical next step

If the exact-behavior work resumes, create live PTR probe addons for restricted-unit returns, encounter payloads, SimulateMouse taint/focus restrictions, debug secret propagation, secure `raidtarget` actions, DurationTextBinding object semantics, and widget secret-aspect behavior.

## Sources

- `/tmp/warcraft_patch_12_0_7_api_changes.txt` — local snapshot of the patch API-change source.
- `src/c_api/c_battle_net.rs` — modeled Battle.net friend-list APIs.
- `src/c_api/c_party_info.rs`, `src/lua_api/globals/group_verbs.rs`, `src/lua_api/state/support_types.rs` — modeled ready-check behavior.
- `src/c_api/c_ui_file_asset.rs` — best-effort UI file-asset lookup.
- `src/cvars.rs` — recovered exact 12.0.7 profile CVar additions/removals/default override.
- `src/c_api/duration_text_binding.rs` — duration text-binding model and configuration copying; formatting remains best-effort as documented.
- `src/c_api/c_ping_secure.rs` — Rust-backed secure pending callback storage globals.
- `src/lua_api/globals/missing_surface/encounter_events.rs` — Rust-backed encounter timeline color bridge.
- `src/lua_api/workarounds/temporary/formatting_utility_defaults.rs` — shared money formatting and tooltip money-line helper.
- `src/event/valid_events.rs` — 12.0.7 event gate.
- `src/loader/tests/wow_api_globals/startup_globals.rs` — 12.0.7 safe bridge regression coverage.
- `docs/wiki/investigations/patch-12-1-api-audit.md` — adjacent 12.1 audit workflow and blocked-item pattern.

## See Also

- [[patch-12-1-api-audit]] — same audit pattern for Patch 12.1.
- [[client-profiles]] — retail epoch features used to gate patch-specific API surface.
- [[lua-api]] — Lua runtime surface and C API bridge context.
- [[event-system]] — event registration/dispatch behavior.
- [[taint-system]] — secure/taint behavior related to SimulateMouse, debug secret propagation, and secure actions.
