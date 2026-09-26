# StatusBar texture rotation

`StatusBar:SetRotatesTexture` stores a boolean on the bar; its fill texture uses quarter-turn UVs. This is simulator behavior informed by local Classic API definitions, the Solarity build-12340 source-derived assertion, `~/Repos/wow-ui-schema/UI.xsd` (`StatusBarType` declares `rotatesTexture` as boolean, default `false`), and the cached Classic Blizzard UI consumer `~/.cache/wow-ui-sim/blizzard-ui/Blizzard_ActionBar/Classic/OverrideActionBar.xml` (healthBar and powerBar use `rotatesTexture="true"`). **Not independently verified native client behavior**. See [rendering pipeline](../rendering-pipeline.md).

## What it must do

- [x] New bars report `false`; `SetRotatesTexture(true/false)` round-trips, including when set before or across bar texture replacement.
- [x] The owned texture exposes the rotated eight-coordinate tuple `(0,1,1,1,0,0,1,0)`; replacing a raw source with an atlas retains rotation. Adopting an unrotated texture retains its existing custom texture coordinates.
- [x] Rendered fill quads rotate texture samples and clip partial fills without changing horizontal fill geometry.
- [x] Vertical StatusBars clip normal fills bottom-up and reverse fills top-down, including rotated textures and atlas-backed UVs. This direction is inferred from local Solarity source and conventional UI, **not native-verified**.
- [x] XML StatusBars and runtime `CreateFrame` templates apply `rotatesTexture` before `OnLoad`; unset attributes inherit the template value while explicit `false` overrides inherited `true`.

## How it works

- [Rendering pipeline](../rendering-pipeline.md)
- [Texture atlas system](../texture-atlas-system.md)

## Implementation inventory

- `src/widget/frame.rs`, `src/widget/frame_defaults.rs` — bar property and default.
- `src/lua_api/frame/methods/widgets/statusbar.rs` — methods and texture source lifecycle.
- `src/iced_app/statusbar.rs` — collect orientation from shared bar state into fill data.
- `src/iced_app/quad_builders_textures.rs` — bar-fill geometry/UV clipping and atlas remapping.
- `src/xml/types.rs` — optional XML rotation attribute.
- `src/lua_api/globals/template/direct.rs`, `src/loader/xml_frame/setup.rs`, `src/lua_api/globals/create_frame/template_chain/runtime.rs` — shared attribute resolution applied in static and runtime creation before lifecycle scripts.

## Tests asserting this spec

- `tests/widget_methods_colorselect.rs` — source GREEN: two rotation/replacement cases plus one adopted-custom-coordinate case (3/3 total) under the diagnostic feature set.
- `src/iced_app/quad_builders_textures.rs` — normal `gui,client-wrath` source GREEN at `e0720b992`: vertical normal/reverse 2/2 and rotated atlas 1/1. Horizontal rotation controls remain GREEN 2/2 within the overlapping `rotated_statusbar_` 3/3 filter.
- `src/iced_app/statusbar.rs` — normal `gui,client-wrath` source GREEN: Lua-created vertical bar orientation and value reach collected fill (1/1).
- `tests/xml_templates/inline_advanced/rendering_templates.rs` — normal `gui,client-wrath` RED 2 static + 1 runtime, then GREEN 3/3: direct XML `true` is visible in `OnLoad`; an inherited `true` remains visible in `OnLoad`; explicit XML `false` overrides it; runtime `CreateFrame` template rotation is visible in `OnLoad`.

The five earlier non-XML rotation cases passed only with `gui,client-wrath,aura-instance-enumeration`; they do not accept that diagnostic lane as normal-profile acceptance. Vertical source GREEN at `e0720b992` is separately targeted under normal `gui,client-wrath`: normal/reverse 2/2, rotated atlas 1/1, collector 1/1, with horizontal controls preserved 2/2. Bottom-up normal and top-down reverse remain inferred rather than native-verified. The XML 3/3 and vertical results are not replacements for independent final normal `fmt`/`check`.

## Known gaps (current cycle)

- [ ] Repeat the five earlier non-XML rotation cases under normal `gui,client-wrath`; `aura-instance-enumeration` was diagnostic only. Run independent final normal `fmt`/`check` after `e0720b992`.
- [ ] Confirm exact default and rotation semantics on an actual Classic Wrath client; Solarity assertions concern original build 12340, not this target.
- [ ] Verify vertical fill direction and rotation against a native Classic Wrath client; bottom-up normal/top-down reverse is an explicit inference, and Lua/renderer fixtures do not prove native conformance.

## Out of scope

- Range and callback behavior belongs to the separate value-callback slice.
- XML orientation and geometry remain separate; cached Blizzard usage corroborates the XML attribute but does not prove native rotation coordinates.
