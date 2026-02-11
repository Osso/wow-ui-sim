# Action Bar Spell Icons — Investigation & Fix

## Problem

Spell icons didn't render on the main action bar buttons. The action bar frame displayed correctly, but icon textures were invisible.

## Root Causes Found & Fixed

### 1. `SetDrawLayer` Overridden by No-Op (FIXED)

In `widget_model.rs`, `add_model_scene_rendering_stubs` registered a no-op `SetDrawLayer`:

```rust
methods.add_method("SetDrawLayer", |_, _this, _args: mlua::MultiValue| Ok(()));
```

This was registered AFTER the real `SetDrawLayer` in `methods_texture.rs`. In mlua, later `add_method` with the same name overrides the earlier one. Result: `BaseActionButtonMixin_OnLoad` calling `self.NormalTexture:SetDrawLayer("OVERLAY")` was silently ignored.

**Fix**: Removed the no-op from `widget_model.rs`.

### 2. Draw Order Within Same Layer (FIXED)

Icon and SlotArt were both at `BACKGROUND` sub-level 0. SlotArt (opaque dark wing texture, atlas `ui-hud-actionbar-iconframe-slot`) rendered ON TOP of the icon because it had a higher widget ID.

Per [WoW documentation](https://warcraft.wiki.gg/wiki/Layer), the render order of textures within the same draw layer and sublevel is **undefined**. WoW's engine happens to render earlier-created textures on top.

**Fix**: Reversed the ID tiebreaker in `intra_strata_sort_key` — lower IDs (earlier-created) now sort last (render on top). Also added a `type_flag` so FontStrings always render above Textures in the same layer, matching WoW's guaranteed rule.

### 3. `SetDrawLayer` Sublevel Parameter (FIXED)

`SetDrawLayer(layer, sublevel)` ignored the second argument. `GetDrawLayer()` always returned 0 for sublevel.

**Fix**: Both now properly read/write `frame.draw_sub_layer`.

### 4. XML `textureSubLevel` Not Parsed (FIXED)

The `<Layer textureSubLevel="N">` XML attribute was not parsed, so all textures in the same layer got sublevel 0.

**Fix**: `LayerXml` now parses `textureSubLevel` and passes it through to `create_texture_from_xml` / `create_fontstring_from_xml`, which call `SetDrawLayer` with the sublevel.

## Architecture

### Action Button Layer Structure

```
BACKGROUND layer:
  icon (spell texture, e.g. Spell_Holy_FlashHeal)
  IconMask (masks icon to rounded shape)
  SlotBackground (hidden when bar art shown)
  SlotArt (dark wing motif, shown when bar art shown)

OVERLAY layer:
  NormalTexture (frame border with transparent center)
  PushedTexture (pressed state border)
```

The NormalTexture frame border renders at OVERLAY with a transparent center, showing the BACKGROUND layer beneath. The icon must render above SlotArt within BACKGROUND for the spell icon to be visible through the frame border's center hole.

### Render Sort Key

```
(frame_level, region_flag, draw_layer, draw_sub_layer, type_flag, Reverse(id))
```

- `region_flag`: 0 for frames, 1 for textures/fontstrings
- `type_flag`: 0 for Texture, 1 for FontString (text always above textures)
- `Reverse(id)`: Earlier-created regions render on top within same layer

## Files Modified

- `src/lua_api/frame/methods/widget_model.rs` — Removed no-op SetDrawLayer override
- `src/lua_api/frame/methods/methods_texture.rs` — SetDrawLayer/GetDrawLayer sublevel support
- `src/iced_app/frame_collect.rs` — Reversed ID tiebreaker, added FontString type_flag
- `src/xml/types_elements.rs` — Parse textureSubLevel from XML Layer
- `src/loader/xml_frame.rs` — Pass sub_level to texture/fontstring creation
- `src/loader/xml_texture.rs` — Accept and apply sub_level parameter
- `src/loader/xml_fontstring.rs` — Accept and apply sub_level parameter
