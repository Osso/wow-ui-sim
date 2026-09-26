# StatusBar value callbacks

StatusBar value changes notify Lua handlers so stock text labels follow the bar. The setter lives in `src/lua_api/frame/methods/widgets/slider.rs`; see [event dispatch](../event-system.md).

## What it must do

- [x] An ordinary changed value dispatches `OnValueChanged(self, value)` before `SetValue` returns; the handler observes the committed value and can update a FontString.
- [x] StatusBar dispatch does not add the Slider-only mouse-event argument.
- [x] Dispatch releases simulator borrows before invoking Lua and uses the existing protected script-dispatch/error-reporting path.

These checked behaviors have source GREEN only under the diagnostic `gui,client-wrath,aura-instance-enumeration` feature set; the normal `gui,client-wrath` lane remains blocked independently by `c_unit_auras` feature gating.

The callback requirement is corroborated by Blizzard's `Blizzard_TextStatusBar/Classic/TextStatusBar.xml` binding and `TextStatusBar_OnValueChanged` calling `TextStatusBar_UpdateTextString`. This is source-derived compatibility evidence, not an executed native-client probe. Solarity's build-12340 documentation independently describes value callbacks; it does not establish all edge semantics for the existing Wrath profile.

## How it works

- [Event system](../event-system.md)
- [Frame data flow](../frame-data-flow.md)

## Implementation inventory

- `src/lua_api/frame/methods/widgets/slider.rs` — shared Slider/StatusBar setters and callback dispatch.
- `src/lua_api/frame/methods/text_attribute_event/events.rs` — allowed StatusBar script handlers.

## Tests asserting this spec

- `tests/widget_slider.rs::statusbar_value_change_updates_text_synchronously` — changing a bar updates its label through the Lua handler before `SetValue` returns; source GREEN in the diagnostic 10/10 `widget_slider::` run recorded at `/tmp/wrath-statusbar-postfix-widget_slider__.log`.

## Known gaps (current cycle)

- [ ] Unblock normal `gui,client-wrath` compilation, then repeat the ordinary callback regression without `aura-instance-enumeration`.
- [ ] Establish Wrath-specific range/first-value initialization, callback ordering, geometry, and XML behavior; current source evidence does not close those contracts.

## Out of scope

First-zero initialization, min/max callback ordering, interpolation-event timing, and original-3.3.5a numeric limits require separate evidence. This slice does not adopt those Solarity contracts or convert the existing Wrath profile to build 12340.
