# Issue 52 texture rendering

## Contract

- Lua solid colours and vertex colours use display RGB values: opaque 0.5 reads back as 128; (0.235,0.14,0.08) reads back as (60,36,20). White-modulated image bytes remain unchanged. Image tints multiply display values, with no second encoding.
- SetGradient multiplies image texels and vertex tint. Vertical min is bottom and max is top; horizontal min is left and max is right, as in the existing solid gradient path.
- SetTextureSliceMargins(left,top,right,bottom) enables nine-slice rendering with fixed pixel corners. SetTextureSliceMode selects stretched (0, default) or tiled (1) centre/sides. ClearTextureSlice disables it.
- Gamepad button up/down handlers register on Frame and Frame-derived types in profiles whose pinned UI.xsd declares them. Texture, FontString, Line and animation objects do not gain them. Registration alone creates no input source.

## Evidence and design

VERIFIED: pinned Retail and Forever `Blizzard_APIDocumentationGenerated/SimpleTextureBaseAPIDocumentation.lua:622-644` describes pixel margins in L,T,R,B order and stretched/tiled sides and centre. `UITextureConstantsDocumentation.lua:13-14` gives the enum. Both `Blizzard_SharedXML/UI.xsd:677-688` declare the XML fields. Existing storage incorrectly names arguments L,R,T,B, so the setter must be corrected while retaining its internal tuple order.

VERIFIED: pinned Retail/Forever `Blizzard_SharedXML/UI.xsd:444-446` declares both handlers in ScriptsType; FrameType owns Scripts and its derived types inherit it. The cached PTR/Mists/Era/Anniversary schemas also declare them; the cached Wrath schema has not established support. Texture/FontString/Line do not inherit FrameType. Forever `Blizzard_Gamepad/UI/CinematicMovie.lua:32-42` sets both handlers on a button. No simulator gamepad button input source has been established; dispatch is excluded.

Design: keep sRGB image atlas sampling and output attachment formats. Recover encoded image RGB after sampling, multiply encoded UI colour values, then decode the result for sRGB attachment output. Select decoding only for sRGB targets. Keep premultiplied alpha/additive blending unchanged; native blend-space parity is not claimed. Use the existing stretch/tile slice emitters; cache source dimensions through the environment texture manager when margins or source change. Apply gradients across emitted image vertices using their positions in the complete region, including slices.

INFERRED / decomp questions: margin behaviour for a destination smaller than opposing margins, invalid/negative margins, rotated/cropped UVs combined with slicing, and pixel-vs-UI-unit behaviour under non-unit frame scale are not specified in these documents. Do not claim native parity for them.

## Acceptance tests

- `issue52_headless_colour_bytes_match_lua_inputs_and_image_tints`
- `issue52_headless_image_gradient_multiplies_texture_and_vertex_colour`
- `issue52_texture_slice_corners_keep_pixel_size_as_frame_grows`
- `issue52_gamepad_button_scripts_register_on_frame_types`
