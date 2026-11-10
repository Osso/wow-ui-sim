# Issue #52 texture rendering compatibility

This change fixes texture colours, gradients and slice rendering in the renderer, and accepts evidenced gamepad button scripts. See the [rendering pipeline](../rendering-pipeline.md) for the broader renderer contract.

## What it must do

- [ ] Follow the exact RGB and image-tint contract in [shader colour output](shader-colour-roundtrip.md).
- [ ] `SetGradient` multiplies image texels and vertex tint. Vertical min is bottom/max is top; horizontal min is left/max is right.
- [ ] `SetTextureSliceMargins(left, top, right, bottom)` uses fixed pixel-size corners. Slice mode 0 stretches sides and centre; mode 1 tiles them. `ClearTextureSlice` restores single-quad rendering.
- [ ] `SetTextureSliceMode` values follow `UITextureSliceMode`: Stretched is 0, Tiled is 1. The default is Stretched.
- [ ] `OnGamePadButtonDown` and `OnGamePadButtonUp` register on Frame and Button for profiles whose pinned UI.xsd declares them. Registration does not create a gamepad input source.

## How it works

- [Rendering pipeline](../rendering-pipeline.md)
- [Shader colour output](shader-colour-roundtrip.md)

## Implementation inventory

- `src/render/shader/quad.wgsl` — applies UI display RGB and image tint in the sRGB-target path.
- `src/render/shader/pipeline.rs` — selects color transfer from the output texture format.
- `src/render/headless_colour_tests.rs` — checks production GPU byte readback.
- `src/iced_app/quad_builders_textures.rs` — applies image gradients and emits manual nine-slice quads.
- `src/widget/frame.rs`, `src/widget/frame_defaults.rs` — store image dimensions for enabled file-texture slicing.
- `src/lua_api/frame/methods/widgets/texture/atlas.rs`, `coords.rs`, `color.rs` — source-dimension cache and slice state.
- `src/lua_api/frame/methods/text_attribute_event/events.rs`, `src/lua_api/script_helpers.rs` — supported gamepad handler registration and name lookup.

## Tests asserting this spec

- `src/iced_app/issue52_render_tests.rs` — colour, image-gradient, slice-corner, tiled-edge/centre, changed-source-dimension and clear-slice headless PNG cases.
- `src/render/headless_colour_tests.rs` — solid, image, half-alpha and additive GPU readback.
- `tests/gamepad_buttons.rs` — `SetScript`, `GetScript` and `HasScript` for Frame and Button, plus unsupported object types.
- `tests/texture_methods_port.rs`, `tests/methods_texture/nine_slice.rs` — slice margin order, defaults, modes and clear behavior.

## Known gaps (current cycle)

- [ ] Run the new and updated tests on Retail and Forever, including the extra slice cases.
- [ ] Complete the requested profile, integration-baseline and manifest checks; then mark the verified contract bullets complete.

## Out of scope

- No invented gamepad input dispatch. No simulator button-event source is established.
- No native claim for destination bounds smaller than opposing margins, invalid/negative margins or slice/UI-scale behavior. Small destinations remain unsliced.
- Custom UV crops and status-bar fills combined with manual slicing remain unmodeled. Atlas metadata uses its logical entry dimensions; native parity for uncommon atlas/margin combinations is unverified.
- No vendor changes or performance-driven changes to Blizzard Lua.

## Pinned evidence

VERIFIED: pinned Retail and Forever `Blizzard_APIDocumentationGenerated/SimpleTextureBaseAPIDocumentation.lua:622-644` describes margin order as left, top, right, bottom in pixels, and describes stretched/tiled sides and centre. `UITextureConstantsDocumentation.lua:13-14` gives values 0 and 1. `Blizzard_SharedXML/UI.xsd:677-688` declares the XML fields. Exact RGB and image sampling behavior is specified in [shader colour output](shader-colour-roundtrip.md).

VERIFIED: pinned Retail/Forever `Blizzard_SharedXML/UI.xsd:444-446` declares both gamepad handlers; PTR, Mists, Era and Anniversary pins also declare them, while the cached Wrath pin does not. Retail `Blizzard_FrameXML/Shared/CinematicFrame.xml:88` binds the down handler on a Frame; Retail `Blizzard_SharedXML/CustomBindingButtonTemplate.xml:20-21` binds both on a Button; Forever `Blizzard_Kiosk/BlizzCon2026/ColdSwap.lua:66,69-70` sets both on a `CreateFrame("Frame")`, and `Blizzard_Gamepad/UI/CinematicMovie.lua:32-42` sets both on a button. These consumer types ground registration. `FrameScriptDocumentation.lua` has no gamepad-handler entry in the pinned Forever file. VERIFIED: no simulator event producer for gamepad buttons exists in `src/`; `src/lua_api/frame/methods/misc/gamepad.rs` only stores per-frame enable flags, so registration only is implemented.
