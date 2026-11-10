# Shader colour output

UI RGB inputs and image texels represent display-byte values on an sRGB output target. See the [rendering pipeline](../rendering-pipeline.md).

## What it must do

- Preserve opaque sRGB image bytes through white modulation.
- Map UI RGB input 0.5 to byte 128 and `(0.235, 0.14, 0.08)` to `(60, 36, 20)` for solid colours and vertex tints.
- Multiply image texels and UI tints once in display-value space. Do not double-encode image texels.
- Retain premultiplied-alpha and additive blend behavior after conversion to the target's linear color space.
- Leave non-sRGB targets on the existing linear-input path.

## How it works

- [Rendering pipeline](../rendering-pipeline.md)

On an sRGB target, image samples are decoded by texture sampling. The shader recovers their display values, multiplies by UI color/tint, rounds the resulting display RGB to the nearest byte, then decodes it for linear blending and the sRGB attachment. Solid and tint values use the same path. The target transfer function is selected from its format; image bytes are not modified in the atlas.

## Tests asserting this spec

- `iced_app::issue52_render_tests::issue52_headless_colour_bytes_match_lua_inputs_and_image_tints` checks Lua solid colors, vertex tints and opaque/colored images through headless PNG readback.
- `render::headless::colour_tests::colour_readback_uses_display_byte_rgb_values` checks production shader swatches, including opaque images, solids, half-alpha and additive blending. Run its documented ignored-test filter explicitly.

## Evidence limits

The input-byte results follow issue #52's reported client values. This spec does not establish WoW parity for every blend mode, non-sRGB target, or HDR output.
