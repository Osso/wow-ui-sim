# Supplied TextWithSubtext widgets

The contract requires `C_UIWidgetManager.GetTextWithSubtextWidgetVisualizationInfo(widgetID)` to project explicit host-supplied type25 records from `src/c_api/ui_widget_manager/`. Contract grounded in the profile-scoped Retail/PTR cached `UIWidgetManagerDocumentation.lua` and unchanged `Blizzard_UIWidgetTemplateTextWithSubtext.lua`/`.xml`; records are synthetic inputs, not native catalog observations. See [widget manager state](../wiki/systems/ui-widget-manager-state.md) for architecture.

## What it must do

- [x] Unknown numeric ID returns exactly one nil; missing/non-number selectors reject through the shared required-number policy. Fractional supplied IDs remain exact numeric keys.
- [x] Populated getter returns exactly one fresh table with precisely the 27 documented non-nil keys, types and supplied values. Caller mutation cannot alter input or retained snapshots; replacement changes all fields without mutating old snapshots; environments remain isolated.
- [x] Normal type25 feeds publish complete records and membership before synchronous notifications. Wrong visualization type or absent set rejects without changing stored records/membership or notifying subscribers. Membership removal retains the independent visualization record.
- [x] Actual cached type25 registration and `UIWidgetContainerTemplate` activate both text lines through normal feeds, apply independent fonts/colors/justification, tooltip metadata, spacing, width, vertical line order, order index, layout and scale. Updates preserve frame identity and its acquisition tag while the getter projects the replacement tag. Hidden releases; Shown reacquires with the current tag; removal and active unregister release frames and subscriptions.
- [x] With `retail-12-1-0`, real present/absent secret-number wrappers permit untainted queries, reject tainted queries before lookup, preserve wrappers/caller taint and leave supplied state unchanged. Public output stays non-secret.

## How it works

- [Widget manager state](../wiki/systems/ui-widget-manager-state.md)
- [Event dispatch](../event-system.md)

## Implementation inventory

- `src/c_api/ui_widget_manager/text_with_subtext_record.rs`: complete `TextWithSubtextRecord` (enum `u32`, numeric `f64`, strings `String`, timer `bool`).
- `src/c_api/ui_widget_manager/inputs.rs`: `UiWidgetInputs.text_with_subtext` keyed by `WidgetId`.
- `src/c_api/ui_widget_manager/feed.rs`: `WidgetVisualization::TextWithSubtext` normal type25 update feed.
- `src/c_api/ui_widget_manager/text_with_subtext.rs`: fresh complete projection and shared selector-policy getter.
- `src/c_api/ui_widget_manager/queries.rs`: shared namespace registration.
- `src/c_api/ui_widget_manager.rs`: record exports and namespace subsystem.

## Tests asserting this spec

- `tests/ui_widget_text_with_subtext.rs`: Retail/PTR suite entry.
- `tests/widget_subtext/queries.rs`: unknown control, selector rejection, exact27 snapshots/replacement, two-environment isolation.
- `tests/widget_subtext/feed.rs`: state-before-event and atomic feed rejection.
- `tests/widget_subtext/consumer.rs`: unchanged cached activation and lifecycle; Lua assertion files contain observations only.
- `tests/widget_subtext/security.rs`: actual host-secret boundary using shared helper.

## Known gaps (current cycle)

- [x] Independent bounded Retail and PTR gates each passed11 unique cases at `62f115d26`: eight type25 cases including actual secret wrappers, plus three unchanged lifecycle controls. Complete snapshots, ordinary feeds and actual cached two-line lifecycle are proved only at that scope.
- [x] Default format/check gates passed; six existing manifest deprecations and reported memory-limit crossings retained. Retail normal startup at unchanged producer `e37f5426a` and fresh PTR startup at `62f115d26` returned `[]`, exit0; complete informational diagnostics and disabled-CASC/sound/addon/SavedVariables/WTF/cache limitations remain.
- [x] Owning normal Retail10.0.2 post-reconciliation replay: see the [owning-after checkpoint](patch-10-0-2-publication-sweep.md#owning-after-checkpoint--2026-10-14). Historical exact-set FAIL remains; broader/native/render/parent acceptance stays open.
- Historical results preserved: preproduction `0a3a5ac20`1 PASS/7 FAIL; producer `e37f5426a`10 PASS/1 FAIL; diagnostic `e948624b0`1 FAIL. Update failure was a fixture expectation of tag refresh during ordinary setup; cached acquisition-only tag assignment is retained, without model/vendor changes.
- Original extra-copy/path/mode and status-schema setup discrepancies remain recorded separately; corrections reuse authenticated bytes and stable metadata, with no hash reruns or invented runtime failures.

[Whole-TEXT retention packet](../../data/patch-api/evidence/widget-text-20261014/README.md) preserves39 complete derivatives at their historical scopes. Independent corrected-candidate integrity/privacy gate passed; initial privacy FAIL retained. Owning-after publication proof is separate in the checkpoint above. No broader/native/parent closure.

## Out of scope

Native catalog invention, vendor patches, animation/asset execution and visual pixel parity. `widgetWidth` is projected independently; cached two-line consumer chooses width from positive `widgetSizeSetting`, not `widgetWidth`.
