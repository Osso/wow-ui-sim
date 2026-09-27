# ColorSelect RGB callback

The simulator's existing `ColorSelect:SetColorRGB` publishes changed color state through `OnColorSelect`. Source: `src/lua_api/frame/methods/widgets/slider/colorselect.rs`; see [widget system](../widget-system.md). Blizzard Lua/XML remains unchanged.

## What it must do

- [x] Commit RGB components while preserving alpha before dispatching `OnColorSelect(self, r, g, b)` through registered scripts and hooks.
- [x] Update the unchanged retail ColorPickerFrame's current swatch and caller callback during setup and hex-entry interaction.
- [x] Construct the six ColorSelect texture children declared in XML, preserving parent keys, slot getter identity, and declared properties through ordinary, inherited-template, and runtime-template creation.

Same-value RGB calls retain their previous no-notification behavior. Handler failures use the established script error handler; native same-value and reentrant policies are not established by this work.

## How it works

- [Widget system](../widget-system.md)
- [XML template system](../xml-template-system.md)
- [Event dispatch](../event-system.md)

## Implementation inventory

- `src/lua_api/frame/methods/widgets/slider/colorselect.rs`: RGB state commitment and callback delivery.
- `src/loader/xml_frame_extras.rs`: ColorSelect slot declaration resolution and shared texture property application.
- `src/loader/xml_frame/finalize.rs` and `src/lua_api/globals/create_frame/template_chain/runtime_loader_effects.rs`: ordinary and runtime-template integration.

## Tests asserting this spec

- `tests/widget_methods_colorselect.rs`: 12 RGB/HSV/alpha state and getter controls, plus exact committed-RGB script/hook callback and unchanged Retail ColorPicker setup/hex-entry consumer cases.
- `tests/xml_templates/inline_advanced/colorselect_textures.rs`: three XML construction paths. Each asserts `GetNumRegions() == 6`; all six role getters bind the matching `parentKey` texture with the correct parent/object type; declared dimensions, all eight thumb texture-coordinate values, and the Wheel's renderer-facing `color_texture` are retained.

## Evidence

`/tmp/cross-version-colorselect-verification-ledger.md` is the detailed proof record. At `e586e6d30`, 35 unique behavioral cases are valid: 18 retained XML controls, three moved XML cases, 12 ColorSelect state controls, and two exact callback/Retail-consumer cases. Fresh `cargo fmt --check` and `cargo test --test integration --no-run` passed without compiler warnings; the exact Retail consumer case asserts zero recorded Lua errors.

Evidence phases: `a5c96d373` supplied the RGB callback RED; `67d596b0b` supplied the XML-slot RED; `3397eeec9`, `69e4bd61d`, and `579770559` implemented the runtime slice; `b27e0bf75` documented the first integrated attempt, whose XML assertion was falsified because it expected vertex tint rather than `color_texture`; `fab59666e` corrected that fixture; and `e586e6d30` moved the three XML cases into the bounded sibling module. No Blizzard/vendor UI files changed.

## Known gaps

Native HSV-only and alpha-only callback semantics, native reentry/coercion behavior, physical picker dragging, GPU parity, and non-Retail profiles remain unverified. Cached Retail `Blizzard_ColorPickerFrame/Mainline/ColorPickerFrame.lua` is the consumer source; its setup and hex-entry path passes, but this is not a fresh native-client probe.

Readability: `tests/widget_methods_colorselect.rs` was already over the 750-line cap before the initial test commit (`a5c96d373^`: 897 lines); it is now 1,015 lines, a net +118 across the full slice. This is a preexisting finding, not authorization for extra cleanup. The new sibling module avoided extending `rendering_templates.rs`, which is now 669 lines.

## Out of scope

HSV-only and alpha-only callback semantics, native reentry/coercion behavior, physical picker dragging, and GPU parity. Existing HSV/RGB conversion and alpha storage remain unchanged.
