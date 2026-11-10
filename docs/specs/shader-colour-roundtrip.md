# Shader colour output

WoW UI RGB values must preserve their display-byte meaning on sRGB render targets. See the [rendering pipeline](../rendering-pipeline.md).

## What it must do

- [x] Map opaque UI RGB input `0.5` to byte 128.
- [x] Map `(0.235, 0.14, 0.08)` to `(60, 36, 20)` for solid colours and vertex tints.
- [x] Preserve opaque image bytes through white modulation and apply image tint once, without double-encoding.
- [x] Treat colored glyph RGB bytes as display values on sRGB targets while preserving sampled glyph alpha coverage.
- [x] Retain premultiplied-alpha and additive blending after conversion to target-linear RGB.
- [ ] Leave non-sRGB targets on the existing linear-input path.

## How it works

- [Issue #52 rendering contract](issue52-texture-rendering.md)
- [Rendering pipeline](../rendering-pipeline.md)

## Implementation inventory

- `src/render/shader/quad.wgsl` — converts display RGB and image modulation for sRGB outputs; uses unorm glyph RGB bytes without an extra transfer conversion.
- `src/render/shader/pipeline.rs` — selects color transfer from the output format.
- `src/render/headless_colour_tests.rs` — production shader pixel readback.
- `src/iced_app/issue52_render_tests.rs` — Lua texture state through headless PNG rendering.

## Tests asserting this spec

- `src/iced_app/issue52_render_tests.rs::issue52_headless_colour_bytes_match_lua_inputs_and_image_tints`
- `src/render/headless_colour_tests.rs::colour_readback_uses_display_byte_rgb_values` (opt-in GPU readback for solid, image, glyph, alpha and additive output)

## Known gaps (current cycle)

- [x] Verify sRGB color, image and glyph output on Retail and Forever.
- [ ] Add pixel readback for non-sRGB targets; the current linear-input branch was inspected but not separately rendered.

UNVERIFIED: native rounding for a white image texel multiplied by a `0.5` vertex tint is not measured. The test allows `(127, 128)` for that case; issue #52's exact `128` observation applies to `SetColorTexture`.

## Out of scope

- No claim that every WoW blend mode, non-sRGB target, or HDR target matches the game client.
- No vendor changes or performance-driven changes to Blizzard Lua.
