# Minimap System

## Current State

The minimap exists as a first-class widget type but renders as a **solid dark green rectangle** — no map content, no circular clipping.

### What Works

- **Widget type**: `WidgetType::Minimap` in the enum (`src/widget/mod.rs`)
- **XML parsing**: `<Minimap>` elements parse correctly via `FrameElement::Minimap`
- **Frame creation**: `CreateFrame("Minimap", ...)` creates a proper frame
- **Child hierarchy**: Zoom buttons, backdrop, border texture render as children
- **Lua methods**: 25+ stub methods prevent addon errors (zoom, blips, blob textures)
- **Global registration**: `Minimap` is registered as a Lua global in `global_frames.rs`

### What's Missing

- **Map content**: No texture displayed inside the minimap area
- **Circular clipping**: Renders as a rectangle, not clipped to a circle
- **Mask texture**: `SetMaskTexture` is a no-op stub
- **Zoom**: `SetZoom`/`GetZoom` are stubs returning constants
- **Player arrow**: Not rendered
- **Blips/POIs**: Not rendered

## Key Files

| File | Purpose |
|------|---------|
| `src/widget/mod.rs:23-42` | `WidgetType::Minimap` enum variant |
| `src/iced_app/render.rs:296-300` | `build_minimap_quads()` — currently a solid fill |
| `src/iced_app/render.rs:469` | Dispatch: `WidgetType::Minimap => build_minimap_quads(...)` |
| `src/lua_api/frame/methods/methods_misc.rs:17-74` | All minimap Lua method stubs |
| `src/lua_api/globals/global_frames.rs` | `Minimap` global registration |
| `src/render/shader/quad.wgsl` | WGSL shader (no circular clip support yet) |
| `Interface/BlizzardUI/Blizzard_Minimap/Mainline/Minimap.xml` | Blizzard minimap XML definition |

## How WoW Clips the Minimap to a Circle

WoW uses **three layers** stacked on top of each other:

1. **Map content** (bottom): rectangular texture showing the zone map
2. **Mask texture** (middle): `UIMinimapMask.BLP` — a white circle on black, applied via `SetMaskTexture()`. The black areas make the map transparent, clipping it to the circle shape. The mask follows the compass frame contour (slightly indented at the 4 cardinal points).
3. **Border frame** (top): `UIMinimap.BLP` (`ui-hud-minimap-frame` atlas) — the decorative compass ring with gold cardinal markers. Purely cosmetic overlay, transparent both inside and outside the ring.

The border does **not** do the masking. The corners of the border texture are transparent, so without the mask, a rectangular map would show through. The mask is the essential piece.

### Mask Texture: `UIMinimapMask.BLP`

- Atlas entry: `ui-hud-minimap-frame-mask` (256x256, full UV)
- Source: `Interface/HUD/UIMinimapMask.BLP`
- White = opaque (show map), Black = transparent (hide map)
- Shape matches the compass frame contour (not a perfect circle — has indentations at N/S/E/W)

### Border Frame: `UIMinimap.BLP`

- Atlas entry: `ui-hud-minimap-frame` (438x460, sub-region of 512x512 texture)
- Source: `Interface/HUD/UIMinimap.BLP`
- Contains: compass ring, gold cardinal markers, other UI icons in the atlas
- Rendered via `MinimapCompassTexture` child at OVERLAY layer, sublevel 2

## Available Textures

### In `textures/minimap/`

| Texture | Purpose |
|---------|---------|
| `ui-minimap-background.webp` | Background fill behind map |
| `ui-minimap-border.webp` | Old-style decorative ring border |
| `minimaparrow.webp` | Player direction arrow |
| `compassring.webp` | Compass ring overlay |
| `partyraidblips.webp` / `partyraidblipsv2.webp` | Party/raid member markers |
| `poiicons.webp` | Points of interest icons |
| `ui-minimap-zoominbutton-*.webp` | Zoom button states |
| `ui-minimap-zoomoutbutton-*.webp` | Zoom button states |

### In game files (BLP, not yet converted)

| BLP File | Atlas | Purpose |
|----------|-------|---------|
| `Interface/HUD/UIMinimap.BLP` | `ui-hud-minimap-frame` | New compass border frame (512x512) |
| `Interface/HUD/UIMinimapMask.BLP` | `ui-hud-minimap-frame-mask` | Circular clip mask (256x256) |
| `Interface/HUD/UIMinimapBackground.BLP` | — | Background fill |

## Rendering Architecture

### Current Flow

```
build_minimap_quads(batch, bounds, frame)
  → batch.push_solid(bounds, [0.05, 0.08, 0.05, alpha])  // dark green rect
```

### Shader Pipeline

The WGSL shader (`src/render/shader/quad.wgsl`) processes quads with:
- **Vertex attributes**: position, tex_coords, color, tex_index, flags
- **tex_index**: `-1` = solid color, `0-3` = tiered texture atlas, `4` = glyph atlas
- **flags**: blend mode (alpha or additive)
- No mask/clip support — all quads are axis-aligned rectangles

## Implementation Plan: Map Texture with Mask

Render a static map texture clipped by the mask texture, matching WoW's approach.

### Recommended Approach: Dual-Texture Mask in Shader

Add a shader feature that samples a mask texture alongside the main texture, multiplying the output alpha by the mask value.

**Shader change** — add a mask texture binding and flag:
```wgsl
const FLAG_MASK_CLIP: u32 = 0x100u;  // bit 8

// Additional vertex attribute for mask UV (or reuse tex_coords if mask covers same bounds)
// In fs_main, after computing color:
if (in.flags & FLAG_MASK_CLIP) != 0u {
    let mask_value = textureSampleLevel(mask_texture, texture_sampler, in.tex_coords, 0.0);
    color.a *= mask_value.r;  // white = show, black = hide
}
```

**Rust change** — in `build_minimap_quads()`:
1. Load `UIMinimapMask.BLP` as a texture (or use the atlas entry `ui-hud-minimap-frame-mask`)
2. Load a static map image as the main texture
3. Emit a textured quad with the `FLAG_MASK_CLIP` bit set
4. The shader multiplies alpha by the mask, clipping to the contoured circle

**Pros**: Matches WoW's actual masking behavior, smooth anti-aliased edges from the mask texture, shape matches the compass frame contour exactly.

**Cons**: Requires a dedicated mask texture binding in the shader pipeline.

### Alternative: Smoothstep Circle in Shader

If adding a mask texture binding is too complex, approximate with a mathematical circle:

```wgsl
const FLAG_CIRCLE_CLIP: u32 = 0x100u;

if (in.flags & FLAG_CIRCLE_CLIP) != 0u {
    let centered = in.tex_coords * 2.0 - 1.0;
    let dist = length(centered);
    color.a *= 1.0 - smoothstep(0.96, 1.0, dist);
}
```

**Pros**: No extra texture needed, single flag bit.
**Cons**: Perfect circle doesn't match the compass frame's indented contour at cardinal points.

### Map Content Source

Options for the static map texture:

1. **User-provided image**: Load an image (like the Westguard Keep map) as a texture
2. **Procedural noise**: Generate terrain-like texture (green/brown patches)
3. **Solid with overlays**: Keep the dark fill but add compass ring and player arrow

Option 1 requires loading an arbitrary image into the GPU atlas. A dedicated texture slot outside the tiered atlas would be simplest.

### Minimal Implementation Steps

1. Convert mask: `wow-cli convert-texture Interface/HUD/UIMinimapMask.BLP -o textures/hud/uiminimapmask.webp`
2. Add mask texture support to the shader (new binding or flag + atlas lookup)
3. In `build_minimap_quads()`, emit a textured quad with the mask flag
4. Load a placeholder map image at startup
5. The compass border (`MinimapCompassTexture`) already renders as a child on top
