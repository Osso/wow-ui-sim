# ScrollFrame offsets

`ScrollFrame:SetHorizontalScroll` and `SetVerticalScroll` store the supplied offsets independently of content ranges. Source: `src/lua_api/frame/methods/widgets/slider.rs`; see [widget system](../widget-system.md).

## What it must do

- [x] Round-trip negative and beyond-range offsets through the matching getter for both axes, including when cached ranges are zero.
- [x] Commit changed offsets before synchronous `OnHorizontalScroll`/`OnVerticalScroll`, passing the supplied value and preserving the other axis.
- [x] Suppress callbacks and render dirtying for repeated identical offsets.
- [x] Keep range queries independent of requested offsets.

[Recorded client observations](../wow-client-diff/README.md#section-1-behavior-divergences) report vertical `-50` and horizontal `999` returning unchanged. This work adds simulator regressions, not a new native probe. Callback ordering and same-value suppression retain the existing simulator policy.

## How it works

- [Widget system](../widget-system.md)
- [Event dispatch](../event-system.md)

## Implementation inventory

- `src/lua_api/frame/methods/widgets/slider.rs`: offset storage, query and callback delivery.

## Tests asserting this spec

`tests/scroll_widgets.rs`, grouped `integration` target:

- `test_scrollframe_offsets_round_trip_outside_explicit_ranges_and_notify_after_commit`
- `test_scrollframe_offsets_round_trip_with_zero_cached_ranges`
- Retained `test_scrollframe_same_offsets_do_not_dirty_render_state` and explicit range-refresh control.

## Known gaps (current cycle)

None for this bounded correction. RED at `ee25b7d62` fails both new getter assertions. Independent verification after `e9b72b107` passes six ScrollFrame cases, 15 shared Slider/StatusBar controls, format/check and readability; `/tmp/cross-version-scroll-offset-verification-ledger.md` records exact scope.

## Out of scope

Render translation, clipping, implicit range-refresh timing, nonfinite inputs, argument coercion, and native callback lifecycle fidelity. No changes to vendor code or ScrollBox policies.
