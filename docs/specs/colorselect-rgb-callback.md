# ColorSelect RGB callback

The simulator's existing `ColorSelect:SetColorRGB` publishes changed color state through `OnColorSelect`. Source: `src/lua_api/frame/methods/widgets/slider/colorselect.rs`; see [widget system](../widget-system.md). Blizzard Lua/XML remains unchanged.

## What it must do

- [ ] Commit RGB components while preserving alpha before dispatching `OnColorSelect(self, r, g, b)` through registered scripts and hooks.
- [ ] Update the unchanged retail ColorPickerFrame's current swatch and caller callback during setup and hex-entry interaction.
- [ ] Construct the ColorSelect texture children already declared in Blizzard XML, preserving their parent keys, slot getter identity and declared properties in ordinary and template-based creation.

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

- `tests/widget_methods_colorselect.rs`: committed RGB/alpha callback payloads and unchanged retail ColorPicker setup/hex-entry consumer.
- `tests/xml_templates/inline_advanced/rendering_templates.rs`: six parsed ColorSelect texture roles through inline XML, inherited XML, and runtime template creation.

## Known gaps (current cycle)

Tests-only `a5c96d373` proves missing RGB callback delivery. Its real retail consumer test exposes an independent missing `Alpha` child, not a clean callback-only RED. Tests-only `67d596b0b` reproduces dropped special-texture declarations in three construction paths; each currently stops at the first Wheel binding. Ledgers: `/tmp/cross-version-colorselect-callback-proof.md` and `/tmp/cross-version-colorselect-xml-proof.md`. Integrated verification is pending.

Retail `Blizzard_ColorPickerFrame/Mainline/ColorPickerFrame.lua` installs the callback in `OnLoad`; setup and hex entry invoke the native setter rather than manually updating the current swatch. This grounds the consumer repair without modifying that Lua or creating substitute UI. It is not a fresh native-client probe.

## Out of scope

HSV-only and alpha-only callback semantics, native reentry/coercion behavior, physical picker dragging, and GPU parity. Existing HSV/RGB conversion and alpha storage remain unchanged.
