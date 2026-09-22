# Slider value callbacks

`Slider:SetValue(value, treatAsMouseEvent)` synchronously delivers `OnValueChanged` after changing the stored value. Source: `src/lua_api/frame/methods/widgets/slider.rs`. See [event dispatch](../event-system.md) for script infrastructure.

## What it must do

- [x] Deliver `(self, clampedValue, treatAsMouseEvent)` before returning, with the updated value readable inside the callback; omitted mouse flag is `false`, explicit `true` is forwarded.
- [x] Retain existing clamping and suppress calls whose clamped value is unchanged, including same-value reentry from a callback.
- [x] Dispatch pre/normal/post bindings in order, including intrinsic bindings when no normal script exists.
- [x] Report each handler error through the existing error handler and continue later bindings.
- [x] Permit callbacks to update other widgets and CVars without a held simulator-state borrow; hidden sliders still dispatch.

The cached Forever `SimpleSliderAPIDocumentation.lua` documents `SetValue(value, treatAsMouseEvent=false)`. Dispatch/order/error behavior follows the simulator's existing script model and unchanged addon usage, not a new native-client probe claim.

## How it works

- [Event system](../event-system.md)
- [Widget system](../widget-system.md)

## Implementation inventory

- `src/lua_api/frame/methods/widgets/slider.rs`: changed-value application and synchronous slider script delivery.

## Tests asserting this spec

`tests/widget_slider.rs` in the grouped integration target:

- `set_value_dispatches_synchronously_with_mouse_payload`
- `set_value_clamps_suppresses_unchanged_and_allows_reentry`
- `set_value_dispatches_intrinsic_bindings_after_reported_errors`
- `set_value_updates_castbar_style_scale_and_cvar_consumer`

Frozen `d1e2487f` executes identical test bodies and XML intrinsic templates: four expected missing-dispatch failures. Committed producer `3d6017fe3` passes all nine grouped slider tests, including the four new callbacks and five existing slider regressions. Independent verification and unchanged-addon replay remain pending. Commands, source hashes and frozen executable provenance: `/tmp/forever-addon-audit/slider-value-development-ledger.json`.

## Known gaps (current cycle)

- [ ] Independently verify committed implementation and replay unchanged ClassicCastBar settings workflow.

## Out of scope

StatusBar behavior, other slider methods, mouse-drag producers, new secret handling, argument-validation fidelity and native timing conformance remain unchanged or unverified. No addon/vendor changes or manual callback invocation.
