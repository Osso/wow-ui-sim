# UIWidgetManager state

UIWidgetManager must expose environment-local supplied widget-set state and drive the existing cached Blizzard manager consumers. The modeled surface lives in `src/c_api/ui_widget_manager.rs` and its input module; this contract prioritizes the missing manager state system, not complete native widget coverage. See [widget architecture](../widget-system.md) and [events](../event-system.md) for the existing runtime boundaries.

## What it must do

### Supplied state

- [ ] Each environment owns independent widget-set metadata and ordered membership records containing explicit `widgetID`, widget type, and `unitToken`; changes in one environment do not affect another.
- [ ] Set queries return supplied metadata and membership in supplied order, without synthesizing membership from widget-ID ranges.
- [ ] IconAndText queries return the full supplied record, without filling missing fields with plausible defaults.
- [ ] Existing complete supplied ButtonHeader and Prey behavior remains intact; manager state does not replace or truncate those records.
- [ ] Quest-to-set and widget-to-type mappings use exact supplied associations, never numeric thresholds or inferred ID ranges.
- [ ] Existing twelve demo world quests retain explicitly stored Tooltip/BehindIcon/AdventureMapDetails sets, with their previous fixture IDs and Worldquest texture kit. Initialization derives only these fixture associations; queries never infer associations or invent IDs. No unrelated records are seeded.

### Next slice: supplied bars and HUD selection

Production `0745fe84f` supplies full bar producers, HUD selectors and host feeds. Historical eight-case RED and later failures remain below. At test-only `b28a45f91`, actual HUD OnLoad cases pass 2/2; the bar case fails initial neutral visibility at assertion59, leaving later lifecycle phases unreached. Prior query5/core19/startup/defaultcheck proofs are reused, not fresh b28 executions. [Retained attempts](../../data/patch-api/evidence/ui-widget-manager-state-2026-10-13/README.md#later-consumer-attempts) own exact outcomes. Later `5bd208dba` cached bar1/HUD2 pass with the explicit [atlas application-result policy](texture-atlas-application-result.md); this closes only the bounded actual consumer assertions. Affected PTR/profile and broader parent acceptance remain OPEN.

- [ ] `GetStatusBarWidgetVisualizationInfo` and `GetCaptureBarWidgetVisualizationInfo` return complete supplied 35-field StatusBar and 22-field CaptureBar records respectively, preserving field names, Lua types, enum scalars, and values from the current Retail/PTR documented schemas (equivalent except a blank line). Absent IDs return nil; IDs never infer records or catalog entries.
- [ ] Every bar query returns a fresh record. StatusBar `partitionValues` preserves supplied array order and is a fresh nested array: caller element edits, insertion/removal, and outer-record mutation cannot alter host inputs, retained snapshots, or later queries. Supplied replacements affect later queries only; environments remain independent.
- [ ] TopCenter, BelowMinimap, PowerBar and ObjectiveTracker queries return fixed set IDs 1, 2, 283 and 240 respectively. Retail build 70688 establishes local literals; their Lua publication is a best guess because the native middle path remains unknown.
- [ ] Active standard HUD containers execute their actual cached OnLoad using those fixed IDs and explicit set/widget records. Assert registration and acquired consumers; without records the selected sets remain empty.
- [x] Actual cached StatusBar/CaptureBar templates consume supplied bar fields through the manager path, without mock providers, templates, Setup, or pools. Assert concrete min/max/value, text/tooltip, partition order and geometry, capture neutral-zone geometry (including zero size), pool counts, hidden/restore transitions, incremental updates, membership removal, and unregister cleanup. Flush actual OnUpdate before asserting deferred layout/count cleanup.

### Host updates and consumer lifecycle

- [ ] Host `replace_ui_widget_set`, `update_ui_widget`, and `remove_ui_widget` publish new state before synchronous events. Unknown-set/type-mismatched updates fail without mutation or notification; membership removal preserves independent visualization snapshots.
- [ ] Host state updates become visible through the existing cached manager: `UPDATE_UI_WIDGET` performs incremental updates and `UPDATE_ALL_UI_WIDGETS` performs full reconciliation against current supplied state.
- [ ] Incremental updates publish `unitToken`, not `unit`: unitless events omit it, token events contain the token, and GUID events contain an empty string. The cached consumer's `unit` filter therefore does not filter these events. `UPDATE_ALL_UI_WIDGETS` carries no payload.
- [ ] Full reconciliation activates current members and removes consumers whose membership has been removed, preserving supplied ordering.
- [ ] Hidden visualization state and subsequent activation are handled by actual manager consumers, rather than only by query snapshots.
- [ ] IconAndText dynamic states 3/2 show the acquired dynamic child through normal supplied-state updates; ordinary state 1 hides it and hidden state 0 releases the frame. Cached-consumer child visibility is distinct from flash timing and native parity.
- [ ] Unregistering a consumer stops its updates; removal and reactivation leave no stale consumer state.
- [ ] Behavioral tests exercise activation, removal, hidden-state transitions, and unregister through the actual cached Blizzard manager consumer path.

## How it works

- [Widget architecture](../widget-system.md) — API data is consumed by Lua to create ordinary frames.
- [Event system](../event-system.md) — event dispatch architecture.

## TextWithSubtext scaffold checkpoint — 2026-10-14

Source `66514bf90` defines the complete passive 27-field `TextWithSubtextRecord`. `39a454d09` exports it and adds an initially empty environment-owned map and normal type25 host-feed storage. No inferred records or demo seeds are added.

These revisions do not implement the modeled getter, projection or query registration. Generic callable/nil fallback is not implementation evidence. Populated query, selector/security and actual cached-consumer behavior remain unproved; no compile or RED/GREEN result is claimed. Completed DoubleStatusBar/TextureAndText evidence remains revision-scoped and separate.

## TextWithSubtext producer checkpoint — 2026-10-14

[Owning contract](ui-widget-text-with-subtext.md): `e37f5426a` adds shared registration and a complete27-field projection (20 numbers, six strings, one boolean). The getter clones supplied state, releases its borrow, then roots a fresh Lua table while populating fields. Shared numeric/secret validation runs before lookup; an absent numeric ID returns one nil. No inferred catalog or demo seeds.

Independent preproduction Retail RED at `0a3a5ac20`: one unknown-ID control PASS / seven FAIL, comprising six populated-getter failures and one selector-validation failure. Consumer stopped before cached setup in that RED; activation/lifecycle were not reached then. Historical scaffold and adjoining-provider evidence remain separate.

### Current bounded TextWithSubtext verification — 2026-10-14

Independent Retail and PTR gates each passed11 unique cases at `62f115d26` (eight type25 cases including secret wrappers + three unchanged lifecycle controls). Complete27-field fresh/replacement snapshots, environment isolation, required selectors, normal feed ordering/atomicity, actual cached two-line activation/style/geometry/update, acquisition-tag retention/current-tag reacquisition, removal and live unregister are proved at that scope.

Default format/check passed, with six existing manifest deprecations and reported memory-limit crossings retained. Retail normal observation at unchanged producer `e37f5426a` and fresh PTR observation at `62f115d26`: `[]`, exit0; complete informational diagnostics, font fallback, WTF/cache reads and disabled-CASC/sound/third-party-addon/SavedVariables limits retained. No global-clean, native/render/all-profile or full-parent claim. Owning publication status is recorded only in the [owning-after checkpoint](patch-10-0-2-publication-sweep.md#owning-after-checkpoint--2026-10-14); historical FAIL and broader acceptance boundaries remain.

The cached manager assigns `widgetTag` when acquiring a frame; ordinary setup refreshes order/layout but does not replace its acquisition tag. The getter still projects every supplied replacement field. The type25 fixture must assert retained frame/tag during in-place update and the replacement tag on reacquisition, rather than altering cached Lua or truncating supplied records. Diagnostic-only `e948624b0` localized the earlier producer10 PASS/1 FAIL update failure to the tag/layout assertion. Corrected source62 fixture passes both profile gates; model/vendor code was unchanged. Preproduction RED, producer/diagnostic failures and original setup-schema discrepancies remain historical, not overwritten.

## Implementation inventory

- `src/c_api/ui_widget_manager.rs` — modeled widget identities and existing ButtonHeader records; exports manager input records.
- `src/c_api/ui_widget_manager/inputs.rs` — supplied set membership and visualization inputs.
- `src/c_api/ui_widget_manager/bar_records.rs` — complete supplied 35-field StatusBarRecord and 22-field CaptureBarRecord DTOs.
- `src/c_api/ui_widget_manager/bars.rs` — full supplied bar snapshot producers, including fresh StatusBar partition arrays; five bar/HUD query cases passed.
- `src/c_api/ui_widget_manager/hud.rs` — four fixed HUD set selectors, each returning one number.
- `src/c_api/ui_widget_manager/queries.rs` — registration calls both implemented `bars` and `hud` modules.
- `src/c_api/ui_widget_manager/text_with_subtext_record.rs` / `text_with_subtext.rs` — complete27-field supplied record and fresh projection, shared registration, owned input map and normal type25 feed; bounded Retail/PTR verification described above.
- `src/c_api/ui_widget_manager/feed.rs` — `WidgetVisualization` integration for StatusBar type2 and CaptureBar type1; actual bar-consumer adaptation uses host feeds instead of direct state changes or manual events.
- `src/c_api/ui_widget_manager/fixtures.rs` — existing demo initialization. The old PowerBar bootstrap is removed without fallback in the new implementation scope.
- `src/lua_api/state/sim_state.rs` — environment-owned manager, ButtonHeader, and Prey input state.
- `src/lua_api/globals/quest_surface/task_quest.rs` — exact supplied quest/type lookup.

## Tests asserting this spec

At `d11908dcf`, five supplied-state cases execute 0 PASS / 5 query-boundary FAIL, without fixture/setup failures. At `6a852c322`, actual cached XML container registration fails because supplied set417 is not returned; host-secret query fails on the old numeric extractor. Later lifecycle and secret-recovery assertions remain unreached in RED. The six state cases, one security case, nine ButtonHeader controls, six Path cases and one Prey control pass at `67775404d` (23/24 selected). Actual cached lifecycle initially fails a pre-unregister count assertion: incremental updates mark layout dirty, but the fixture expected a recomputed count before OnUpdate. Diagnostic execution at `3a7c3650d` identifies `restore showing count=1`; cached source defers layout. The fixture subsequently flushes actual OnUpdate before count-dependent cleanup assertions; retain this failed attempt as deferred-layout evidence. Normal startup has zero Lua errors, runtime warnings and AnimSetup diagnostics. At `96a44bb1f`, a separate actual-container unit notification test fails target-text preservation while player refresh passes; the event adapter correction follows that failed attempt. The unit fixture does not prove unit registration or context-data isolation.

Latest bounded core evidence: two lifecycle cases physically pass at `e887` (scoped PASS 2); the prior 23 passing state/security/control cases at `67775404d` are reused unchanged, not rerun at `e887`. This is 25 logical cases with separate proof scopes, not a fresh 25-case execution. No bar/HUD next-slice GREEN is included. Requirement checkboxes remain conservative: only assertions backed by those tests qualify; unit registration/context-data isolation remains unproven.

- `tests/ui_widget_manager_state.rs` — supplied-state isolation, metadata, ordered membership, complete records, and exact mappings.
- `tests/ui_widget_manager_lifecycle.rs` — cached manager incremental/full reconciliation and actual consumer lifecycle behavior.
- `tests/ui_widget_manager_security.rs` — actual host-secret caller policy and public recovery.

## Next-slice tests and proof gaps

At `1804d1c49`, actual active-PTR-cache bar1/HUD2 pass: three unique cases, both wait exits0, independent saved-evidence audit PASS. The tests use `default_blizzard_ui_addons_path` → active-profile cache, with no Retail pin or vendor/assertion changes. Earlier source34ef/production5bd PTR26/startup remains separate, reused proof, excluding these three consumers. [PTR and owning checkpoint](../wiki/investigations/integrated-source-and-factory-proof-2026-10-09.md#separate-ptr-and-owning-checkpoints--2026-10-13) owns current evidence boundaries; native parity and full-parent acceptance remain OPEN.

Main-reported genuine RED at source `87fecfd31`: eight executed cases, all FAIL. Preserve each failure boundary:

| Scope | Cases | Reached failure | Unreached boundary |
| --- | ---: | --- | --- |
| `tests/ui_widget_manager_bars.rs` | 2 | Wrong-type bar snapshot | Later complete-record/freshness assertions |
| `tests/ui_widget_manager_hud_selections.rs` | 3 | TopCenter exactly-one-return assertion | Later selector assertions |
| `tests/ui_widget_manager_bars_consumer.rs` | 1 | Supplied-record query after cached closure | Later consumer lifecycle |
| `tests/ui_widget_manager_hud_containers.rs` | 2 | Numeric query before cached OnLoad | Later HUD consumer assertions |

Earlier four sandbox-launch failures executed zero cases because bwrap could not find the target. They remain separate launch failures, not genuine RED cases. At `5a869f9a0` (production delta `0745fe84f`), eight cases execute **5 PASS / 3 FAIL**: both bar snapshot cases and all three HUD selection cases pass; cached bar and both HUD consumers fail later. Bar failure is an unlocalized assertion in the real-template/fields/geometry phase. Supplied HUD loading reaches a missing real EncounterBar prerequisite; default-zero HUD loading reaches an invalid `next(nil)` fixture assumption after confirming TopCenter is unregistered. Later lifecycle assertions remain unreached. At `39ce8969a`, the three corrected consumer cases still fail: bar assertion53 localizes the initial left-width comparison, while the combined closure still sorts UIWidgets before UnitFrame and also lacks the AuraContainer publisher. The next fixture revision establishes both real prerequisite publishers in a separate closure before selected HUD OnLoad, retaining empty overrides and no replay. Zero-state checks measure active pools/publication without demanding an initialized widget table. At `174b86505`, three cases still fail: the bar diagnostic observes initial left width `55.599998474121094` versus expected `55.6`, matching f32 geometry storage and exceeding the original `1e-6` tolerance; HUD prerequisites reach the real EncounterBar but an ordinary-global assertion incorrectly requires secure-environment AuraContainerUtil. The next revision uses a named `1e-4` logical-pixel tolerance for f32 dimensions/offsets only, preserving exact f64 bar-value assertions, and validates prerequisite consumer errors without demanding secure globals in ordinary Lua. At `b28a45f91`, three unique consumers execute 2 PASS / 1 FAIL: HUD2 pass actual OnLoad with real prerequisites; bar1 passes geometry then fails initial neutral visibility assertion59. Bar updates/hidden-restore/removal/unregister remain unreached. Exact launcher exits and deduplicated counts are retained in the linked additive packet. Production `0745fe84f` is unchanged; prior query5/core19/startup/defaultcheck proofs are reused, not fresh b28 executions. At that b28 checkpoint, the separate atlas application-result investigation had not yet established a fix. Actual bar updates use host feeds, not direct state mutation/manual events. No whole-slice or native passing claim follows from the five query passes.
- [x] Implement and prove complete fresh bar queries and supplied HUD-selection queries against the staged requirements.
- [x] Integrate and execute actual cached bar/HUD consumer tests; distinguish historical query-boundary/fixture/visibility failures from later passing assertions.
- [ ] Prove unit registration and context-data isolation separately if brought into scope; the current unit notification fixture is insufficient.

## Out of scope

- Native DB2 widget catalogs: unknown; no catalog completeness or fabricated catalog entries.
- Native server wire protocol and native interpolation parity: unknown; supplied host state is not proof of native transport or interpolation behavior.
- Whole native visualization coverage: core models manager state and full supplied IconAndText while preserving existing ButtonHeader/Prey support; the next slice adds only supplied StatusBar/CaptureBar records and HUD selections.
- Dynamic HUD rebinding events, native DB2 catalogs, and native fixed HUD/set/widget IDs: not established or required by this slice.
- Unrelated demo seeds or replacement Blizzard manager behavior: neither is needed for this bounded model contract.
