# Supplied TextWithState visualization

Host-supplied TextWithState input belongs to `src/c_api/ui_widget_manager/`; the [widget manager architecture](../wiki/systems/ui-widget-manager-state.md) owns supplied-record/event behavior. Complete getter projection and Retail/PTR publication follow genuine supplied-record RED at input stage `64a2a57b2`. Independent supplied-record GREEN and real cached-consumer acceptance pass for Retail/PTR at `2bb6cdde1`; the [owning packet](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/ui-widget-text-with-state-2026-10-13/README.md#bounded-saved-green) separates runtime, startup/check and bounded publication integrity. Host policy is not native catalog or transport parity.

## What it must do

- [x] `C_UIWidgetManager.GetTextWithStateWidgetVisualizationInfo(number)` returns exactly one nil for an unknown input, otherwise a fresh complete21-field snapshot preserving supplied strings/numbers/enums/bool without inferred defaults or consumer clamping. Caller mutation, host replacement/deletion and another environment cannot alter retained snapshots.
- [x] Numeric keys and strict non-number rejection follow existing `WidgetId` and `queries::read_numeric_id`, not a new ID-range policy. Real host-secret NUMBER arguments are allowed when untainted, denied before supplied/absent lookup when tainted; public IDs and public snapshot fields recover normally. Both cached schemas explicitly declare `SecretArguments = "AllowedWhenUntainted"`; no new secrecy policy is invented.
- [x] The existing typed host feed accepts TextWithState type8, validates supplied-set existence and entry/visualization type before mutation, stores complete input and membership before synchronous `UPDATE_UI_WIDGET`, and retains independent visualization after membership removal through `UPDATE_ALL_UI_WIDGETS`.
- [x] The unchanged active Retail/PTR cached type8 manager/template activates through the real `UIWidgetContainerTemplate` constructor, consumes text/tooltip/width/font/enabled-state/alignment/bottom-padding/order/scale/layout fields, processes incremental updates, suppresses Hidden, restores visible membership, and releases active frames on membership removal. Flush actual OnUpdate before pool/count assertions. Query snapshots remain independent of consumer clamping.

Literal fields from both cached `TextWithStateWidgetVisualizationInfo` declarations (all non-nil): `shownState`, `enabledState`, `text`, `tooltip`, `textSizeType`, `fontType`, `bottomPadding`, `tooltipLoc`, `hAlign`, `widgetSizeSetting`, `textureKit`, `frameTextureKit`, `hasTimer`, `orderIndex`, `widgetTag`, `inAnimType`, `outAnimType`, `widgetScale`, `layoutDirection`, `modelSceneLayer`, `scriptedAnimationEffectID`.

Explicit fixtures are host-input test policy, not native catalog records. Consumer-safe enum choices are read from both cached API-doc enum declarations and compared against loaded enums after the supplied getter boundary: Shown1/Hidden0, White3/Green4, Normal0/Shadow1, Medium16Pt1/Standard14Pt4, Top4, Left0/Right2, animation None0, scale OneHundred0, layout Vertical1, model layer None0, set Vertical0, visualization TextWithState8. Query-only texture-kit strings, timer true, fractional scripted effect ID and negative order values never enter the consumer fixture. Negative consumer padding exercises the unchanged template's clamp to zero; query snapshots preserve the supplied negative value.

## How it works

- [Widget manager state](../wiki/systems/ui-widget-manager-state.md)
- [Existing manager/feed contract](ui-widget-manager-state.md)
- [Event system](../event-system.md)

## Implementation inventory

- `src/c_api/ui_widget_manager/text_state_record.rs`: explicit DTO21 only; no Default record.
- `src/c_api/ui_widget_manager/inputs.rs`: empty-by-default `text_with_state` map keyed by existing `WidgetId`.
- `src/c_api/ui_widget_manager/feed.rs`: public `WidgetVisualization::TextWithState` type8 and insertion in existing feed before notification; no new transport/event implementation.
- `src/c_api/ui_widget_manager.rs`: public DTO export/module wiring.

- `src/c_api/ui_widget_manager/text_state.rs`: complete fresh projection, existing numeric/secret validator and Retail/PTR publication via the active-profile constant. No padding clamping.
- `src/c_api/ui_widget_manager/queries.rs`: registration beside existing supplied visualization queries.

## Tests asserting this spec

`tests/ui_widget_manager_text_state.rs` is discovered by existing `integration_tests!()` build harness; it explicitly includes `tests/widget_text_state/{fixtures,queries,security,feed,consumer}.rs`. Reuses existing `tests/widget_discrete/fixtures.rs::assert_lua` error capture and `common::blizzard_addon_harness` isolated Game environment/closure loader; no copied broad harness, vendor behavior replacement or fake global consumer.

Six unconditional cases under Retail/PTR: complete snapshot1, strict numeric1, environment isolation1, feed2, cached consumer1. A seventh real-secret case follows existing Discrete `retail-12-1-0` gate. All seven compiled and executed at `64a2a57b2`: one rejected-feed preservation PASS and six FAIL, actual wait exit101, 0.72s. Five seeded cases pass the function-publication assertion, then fail the supplied-table assertion before cached/security prerequisites. The feed callback fails its supplied-table assertion; its immediate event-result assertion fails separately. Later snapshot, numeric, isolation, security and consumer lifecycle assertions remain unreached. These are supplied-record failures, not compile or missing-publication RED. The [retained RED packet](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/ui-widget-text-with-state-2026-10-13/README.md) owns independent failure/source reports and their separate integrity status.

## Known gaps (current cycle)

- [x] Execute and independently verify supplied-record RED before adding getter projection; preserve the passed rejection control and exact unreached boundaries.
- [x] Independently verify complete21 projection, numeric/secret validation and actual Retail/PTR cached lifecycle through saved GREEN; [owning proof](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/ui-widget-text-with-state-2026-10-13/README.md#bounded-saved-green) records exact scopes and diagnostics.
- [x] Independently verify separate GREEN publication retention/privacy/link gate; the [owning packet](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/ui-widget-text-with-state-2026-10-13/README.md#bounded-saved-green) distinguishes copied TEXT integrity from runtime PASS.

## Out of scope

Native catalogs, native events, transport, rendered/pixel output, 3D, reference counting and allocation/lifetime semantics remain unknown/excluded. Native enum admissibility beyond reading declarations, real server fixtures, native quotas, unit-context behavior, unregister coverage, other profiles and full widget-system acceptance are not asserted. No vendor/cache/Wowless/WTF/settings changes. Bounded supplied Retail/PTR runtime is verified; native/render/full-parent acceptance remains unproved.
