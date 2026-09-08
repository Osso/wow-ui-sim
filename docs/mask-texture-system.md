# Mask Texture System

## Overview

WoW `MaskTexture` coverage is sampled from alpha. Visible mask regions may have black RGB, so RGB must not participate in clipping.

**Key files:**
- `src/iced_app/masking.rs` - Mask UV computation and GPU application
- `src/render/shader/primitive.rs` - Deferred RGBA/BC mask resolution and atlas UV remapping
- `src/render/shader/quad.wgsl` - Fragment shader mask sampling
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
- `mask_tex_index: i32` - GPU texture binding for the mask (`-2` = pending resolution, `-1` = no mask, `0–4` = RGBA atlas tiers, `6` = BC1 atlas, `7` = BC3 atlas)
- `mask_tex_coords: [f32; 2]` - UV coordinates into the resolved atlas slot

Mask paths are deferred like ordinary texture paths. During primitive preparation, the renderer resolves each mask from the RGBA atlas first, then the BC1/BC3 atlas, and remaps its UVs into the selected slot. If no atlas entry resolves, the pending index becomes `-1` and the mask is skipped. This BC-aware path is required for CircleMask-style compressed masks; previously a BC-only mask was cleared as unresolved.

The fragment shader samples the resolved binding and multiplies output alpha by mask alpha. Where mask alpha is zero, the pixel is fully transparent; RGB does not affect clipping.

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

---

## Ordering boundary

Mask coverage changes only fragment visibility within an already emitted quad. The alpha-only correction in `939efe88d` did not alter frame strata, parentage, raw frame levels, owner grouping, or hit order. The BetterBlizzFrames no-portrait/SpellBook overlap is instead an owner-strata render-group boundary: a `HIGH` child remains in its `LOW` top-level owner's group behind independent `MEDIUM`. See [top-level render groups](specs/toplevel-render-groups.md) and the [native investigation](wiki/investigations/betterblizzframes-no-portrait-overlay.md).
