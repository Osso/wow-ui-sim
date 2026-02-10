# Mask Texture System

## Overview

WoW uses MaskTextures to clip child textures to specific shapes (rounded squares, circles, etc.). The mask's alpha channel determines which parts of the child texture are visible: alpha=1 shows the texture, alpha=0 hides it.

**Key files:**
- `src/iced_app/masking.rs` - Mask UV computation and GPU application
- `src/render/shader/quad.wgsl` - Fragment shader mask sampling (lines 148-151)
- `src/loader/xml_texture.rs` - XML MaskTexture creation
- `src/lua_api/globals/template/elements.rs` - Template MaskTexture creation

---

## How Masks Work

### XML Definition

MaskTextures are defined in XML templates. Example from ActionButtonTemplate:

```xml
<MaskTexture parentKey="IconMask" atlas="UI-HUD-ActionBar-IconFrame-Mask"
             hWrapMode="CLAMPTOBLACKADDITIVE" vWrapMode="CLAMPTOBLACKADDITIVE">
    <Anchors>
        <Anchor point="CENTER" relativeKey="$parent.icon"/>
    </Anchors>
    <MaskedTextures>
        <MaskedTexture childKey="icon"/>
    </MaskedTextures>
</MaskTexture>
```

This creates a mask that clips the `icon` child texture. The mask is centered on the icon and uses a rounded-square alpha texture.

### Wiring

1. MaskTexture is created via `CreateMaskTexture()` (Lua)
2. The `<MaskedTextures>` block calls `icon:AddMaskTexture(mask)` on each referenced sibling
3. The icon's `mask_textures` vec stores the mask frame ID
4. During rendering, `apply_mask_texture()` sets up GPU mask sampling for the icon's quads

### GPU Pipeline

Each quad vertex carries:
- `mask_tex_index: i32` - Texture tier index for the mask (-2 = pending resolution, -1 = none)
- `mask_tex_coords: [f32; 2]` - UV coordinates into the mask texture

The fragment shader (quad.wgsl:148-151):
```wgsl
if in.mask_tex_index >= 0 {
    let mask_color = sample_tiered_texture(in.mask_tex_index, in.mask_tex_coords);
    color *= mask_color.a;
}
```

Premultiplied alpha: the final color is multiplied by the mask's alpha. Where mask alpha=0, the pixel is fully transparent.

---

## Mask Sizing and UV Mapping

### Critical: Mask Must Be Larger Than Icon

The mask frame should be **larger** than the icon it clips. This is because the mask texture typically has transparent borders (rounded corners, feathered edges). When the mask extends beyond the icon:

- The icon only samples the **opaque center** of the mask texture
- Transparent edges of the mask fall outside the icon bounds and don't affect rendering
- Result: icon fills its slot almost completely, with subtle rounded corners

### UV Computation (`compute_mask_uvs_from_rects`)

The mask UV computation maps the icon's screen position to coordinates within the mask's screen area:

```
icon_bounds (45x45)    mask_screen (64x64, centered on icon)
┌─────────────┐        ┌───────────────────┐
│             │        │   ┌─────────────┐ │
│   icon      │  →     │   │  icon area  │ │
│             │        │   │  UV 0.15-   │ │
│             │        │   │     0.85    │ │
│             │        │   └─────────────┘ │
└─────────────┘        └───────────────────┘
```

For a 64x64 mask centered on a 45x45 icon:
- `dx = 9.5` (icon left relative to mask left)
- UV range: `(9.5/64, 54.5/64)` = `(0.148, 0.852)` on both axes
- The icon samples only the center 70% of the mask texture

### useAtlasSize Default for MaskTextures

MaskTextures default to `useAtlasSize=true` when no explicit value is set in XML. This matches WoW behavior where mask frames auto-size from their atlas.

**Why this matters:** Without atlas-derived sizing, the mask frame has 0x0 dimensions. The fallback UV mapping then maps the FULL mask texture (0-1) to the icon, including the transparent borders. This shrinks the visible icon area because the mask's edge transparency clips pixels that should be fully visible.

| Mask Size | UV Range | Effect |
|-----------|----------|--------|
| 0x0 (broken) | 0.0 - 1.0 | Full mask mapped to icon; transparent borders shrink visible area |
| 64x64 (correct) | 0.148 - 0.852 | Only opaque center mapped; icon fills slot properly |

### SmallActionButtonMixin Override

For small buttons (30x30), `SmallActionButtonMixin_OnLoad` explicitly sets the mask to 45x45:

```lua
self.IconMask:SetSize(45, 45)
self.IconMask:ClearAllPoints()
self.IconMask:SetPoint("CENTER", 0.5, -0.5)
```

This overrides the atlas-derived 64x64 size. The 45x45 mask on a 30x30 button extends 7.5px beyond on each side, so the icon samples UV (0.167, 0.833) — similar proportional coverage.

---

## Action Bar Icon Example

The action bar icon rendering chain:

1. **Icon texture** (45x45, `SetAllPoints=true` on parent button)
   - Set via `icon:SetTexture(GetActionTexture(slot))`
   - Fills entire button area as a square

2. **IconMask** (64x64 from atlas, centered on icon)
   - Atlas: `UI-HUD-ActionBar-IconFrame-Mask` — 64x64 rounded square
   - Clips icon corners to create rounded-square shape
   - Icon samples center UV (0.148-0.852), so rounded corners are subtle

3. **SlotBackground** (setAllPoints, dark background)
   - Shows through where mask clips the icon corners

4. **SlotArt** (setAllPoints, decorative slot frame)
   - Golden border art overlaying everything

5. **NormalTexture** (46x45, button state border)
   - Renders on top as the frame's standard border
