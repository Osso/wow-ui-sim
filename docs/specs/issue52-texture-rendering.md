# Issue 52 texture rendering

## Contract

- Lua solid colours and vertex colours use display RGB values: opaque 0.5 reads back as 128; (0.235,0.14,0.08) reads back as (60,36,20). White-modulated image bytes remain unchanged. Image tints multiply display values, with no second encoding.
- SetGradient multiplies image texels and vertex tint. Vertical min is bottom and max is top; horizontal min is left and max is right, as in the existing solid gradient path.
- SetTextureSliceMargins(left,top,right,bottom) enables nine-slice rendering with fixed pixel corners. SetTextureSliceMode selects stretched (0, default) or tiled (1) centre/sides. ClearTextureSlice disables it.
- Gamepad button up/down handlers register on Frame and Button in profiles whose pinned UI.xsd declares them, matching pinned consumers. Other widget registrations stay unsupported until evidence confirms them. Registration alone creates no input source.

## Evidence and design

VERIFIED: pinned Retail and Forever `Blizzard_APIDocumentationGenerated/SimpleTextureBaseAPIDocumentation.lua:622-644` describes pixel margins in L,T,R,B order and stretched/tiled sides and centre. `UITextureConstantsDocumentation.lua:13-14` gives the enum. Both `Blizzard_SharedXML/UI.xsd:677-688` declare the XML fields. Existing storage incorrectly names arguments L,R,T,B, so the setter must be corrected while retaining its internal tuple order.

VERIFIED: pinned Retail/Forever `Blizzard_SharedXML/UI.xsd:444-446` declares both handlers in `ScriptsType`; the cached PTR/Mists/Era/Anniversary schemas also declare them, while the cached Wrath schema does not. Retail `Blizzard_FrameXML/Shared/CinematicFrame.xml:88` binds the down handler on a Frame; Retail `Blizzard_SharedXML/CustomBindingButtonTemplate.xml:20-21` binds both on a Button; Forever `Blizzard_Gamepad/UI/CinematicMovie.lua:32-42` sets both on a button. These are the evidenced widget types; other widget types remain unproven. No simulator gamepad button input source has been established; dispatch is excluded.

Design: keep sRGB image atlas sampling and output attachment formats. Recover encoded image RGB after sampling, multiply encoded UI colour values, quantize to the nearest display byte, then decode for sRGB attachment output. Select decoding only for sRGB targets. Keep premultiplied alpha/additive blending unchanged; native blend-space parity is not claimed. Use the existing stretch/tile slice emitters for file textures and atlas entries. Read file dimensions only when margins are enabled; use the atlas entry's logical dimensions for atlas-backed textures. Apply gradients across emitted image vertices using positions in the complete region, including slices.

INFERRED / decomp questions: margin behaviour for a destination smaller than opposing margins, invalid/negative margins, explicit custom UVs or rotated UVs combined with slicing, non-atlas cropped paths, and pixel-vs-UI-unit behaviour under non-unit frame scale are not specified in these documents. Bar-fill plus slicing is also not modeled. Do not claim native parity for them.

## Acceptance tests

- `issue52_headless_colour_bytes_match_lua_inputs_and_image_tints`
- `issue52_headless_image_gradient_multiplies_texture_and_vertex_colour`
- `issue52_texture_slice_corners_keep_pixel_size_as_frame_grows`
- `issue52_texture_slice_tiled_mode_repeats_the_center`
- `issue52_texture_slice_source_dimensions_follow_texture_changes`
- `issue52_clear_texture_slice_restores_single_quad_rendering`
- `issue52_gamepad_button_scripts_register_on_evidenced_frame_and_button_types`
