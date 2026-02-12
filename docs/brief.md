# WoW UI Sim Brief - 2026-02-11

## Completed: GameMenuFrame Strata + Tiling Fix

### Problem 1: Game menu rendered behind other windows

GameMenuFrame uses `frameStrata="DIALOG"` in its XML template, but the simulator wasn't parsing `frameStrata` from XML attributes on frames.

**Fix:** Parse `frameStrata` XML attribute in `create_frame_from_xml` and call `SetFrameStrata()` on the frame. Files: `src/loader/xml_frame.rs`.

### Problem 2: Game menu appeared ~50% transparent

Two contributing factors:

1. **Template texture tiling not propagated.** The `DialogBorderTemplate` defines `<Texture parentKey="Bg" horizTile="true" vertTile="true">` for the dialog background. The XML loader path (`src/loader/xml_texture.rs`) already handled these attributes, but the template application path (`src/lua_api/globals/template/elements.rs`) did not call `SetHorizTile`/`SetVertTile` on textures created via `create_texture_from_template`. Result: the 64x64 `ui-dialogbox-background` texture was stretched over the full frame instead of tiling, making it appear washed out.

2. **Background texture has inherent 60% opacity.** The `ui-dialogbox-background.webp` texture is 64x64, all black (RGB=0,0,0) with uniform alpha=153/255≈0.6. This is by design — it's a semi-transparent dark overlay.

**Fix:** Added `horizTile`, `vertTile`, `alpha`, and `alphaMode` propagation to `append_texture_properties()` in `src/lua_api/globals/template/elements.rs`. This ensures template-created textures get these visual properties applied, matching the existing XML loader behavior.

**Template creation path:**
1. `CreateFrame("Frame", name, parent, "DialogBorderTemplate")` — Lua call
2. `apply_templates_from_registry` → `apply_single_template` → `apply_layers` — Rust template system
3. `create_texture_from_template` → `append_texture_properties` — generates Lua code for `SetHorizTile`/`SetVertTile`

## Completed: Backdrop Edge Rendering

**Problem:** BackdropTemplate nine-slice edge textures (top/bottom/left/right borders) were not rendering. Corners rendered fine.

**Root cause — three interconnected issues:**

1. **Both tile flags set**: `BackdropTemplateMixin.SetupPieceVisuals` calls `SetTexture(file, true, true)` setting both `horiz_tile` and `vert_tile` for ALL pieces, routing edges to `emit_grid_tiles` instead of directional tiling.

2. **UV values >1.0**: BackdropTemplateMixin uses UV-based tiling where `SetTexCoord` 8-arg coords contain repeat values >1.0 (e.g., `edgeRepeatX ≈ 14.19`). Our `ClampToEdge` sampler can't handle UVs >1.0.

3. **Rotated UV mapping lost**: 8-arg `SetTexCoord` reduced to axis-aligned bounding box, losing rotation. TopEdge/BottomEdge have V→horizontal rotation.

**Fix (implemented):**
1. Added `tex_coords_quad: Option<[f32; 8]>` to Frame struct — stores raw 8-arg SetTexCoord values
2. In `emit_tiled_texture`, when `tex_coords_quad` has values >1.0: `analyze_uv_repeat()` detects tiling direction and UV rotation from the raw 8-arg pattern
3. For rotated edges (TopEdge/BottomEdge): `emit_rotated_horiz_tiles()` uses `push_textured_path_uv4()` with per-vertex UV coords to handle the V→horizontal rotation
4. For standard edges (LeftEdge/RightEdge): uses existing `emit_vert_tiles()` with clamped base UVs
5. Tile pixel size: uses the non-zero dimension (edgeSize) for both axes (square tiles)

**Files modified:**
- `src/widget/frame.rs` — Added `tex_coords_quad` field
- `src/lua_api/frame/methods/methods_texture.rs` — Store raw 8-arg coords in SetTexCoord
- `src/lua_api/frame/methods/widget_slider.rs` — Clear `tex_coords_quad` on texture change
- `src/iced_app/tiling.rs` — `analyze_uv_repeat()`, `emit_uv_repeat_tiled()`, `emit_rotated_horiz_tiles()`
- `src/render/shader/quad.rs` — Added `push_textured_path_uv4()` for per-vertex UV quads

## Completed: Effective Alpha Propagation

**Problem:** Child frames rendered using only their own `f.alpha`, ignoring ancestor alpha. In WoW, `GetEffectiveAlpha()` returns `self.alpha * parent:GetEffectiveAlpha()` — if any ancestor has alpha < 1.0, all descendants are dimmed. Our simulator computed `eff_alpha` in `frame_collect.rs` but only used it for visibility checks, not rendering.

**Fix:** Threaded `eff_alpha` through the entire rendering pipeline:
- `render.rs` → passes `eff_alpha` to `emit_frame_quads`
- `quad_builders.rs` → all builder functions accept `alpha: f32` parameter (the effective alpha), replacing direct `f.alpha` usage
- `tiling.rs` → `emit_tiled_texture` and `frame_tint` accept `alpha` parameter
- `tooltip.rs` → `build_tooltip_quads` accepts `eff_alpha`
- `message_frame_render.rs` → `emit_message_frame_text` and `render_message` accept `alpha`

## Completed: Frame Render Order Fix (Toplevel + Region Sort)

**Problem:** AccountPlayed popup content rendered on top of GameMenuFrame button textures, despite both being in DIALOG strata. Three contributing issues:

### Issue 1: Region sort key didn't group regions with parent frame

In WoW, regions (Texture/FontString) render as part of their parent frame — they don't participate in the global frame-level sort independently. Our sort key used the region's own `frame_level` (parent_level + 1), causing deeply nested regions from one hierarchy to interleave with shallow regions from another.

**Fix:** Changed `intra_strata_sort_key` in `frame_collect.rs` to use the parent frame's `frame_level` for regions, and group regions with their parent via `Reverse(parent_id)`. Sort key is now:
- Non-regions: `(frame_level, Reverse(id), 0, 0, 0, 0, Reverse(0))`
- Regions: `(parent_level, Reverse(parent_id), 1, draw_layer, sub_layer, type_flag, Reverse(id))`

This ensures all regions of a frame render immediately after that frame, before any higher-level content from other hierarchies.

### Issue 2: `toplevel` attribute was a no-op

GameMenuFrame has `toplevel="true"` in XML, which in WoW means the frame is raised above siblings when shown. Our `SetToplevel` was a no-op.

**Fix:**
- Added `toplevel: bool` field to Frame struct
- `SetToplevel`/`IsToplevel` now store/read the value
- `append_toplevel_code` in `xml_frame.rs` parses `toplevel` from XML
- `set_frame_visible` in `state.rs`: when a toplevel frame becomes visible, `raise_frame()` is called
- `raise_frame()`: finds max frame_level among siblings in the same strata, sets this frame's level to max + 1, propagates to descendants

### Issue 3: `SetFrameStrata` didn't invalidate render cache

`SetFrameStrata()` changed `frame.frame_strata` and propagated to descendants, but didn't invalidate `ancestor_visible_cache`, `strata_buckets`, or `cached_render_list`. Frames that changed strata at runtime would remain in the wrong strata bucket.

**Fix:** Added cache invalidation at the end of `SetFrameStrata()` in `methods_core.rs`.

**Files modified:**
- `src/widget/frame.rs` — Added `toplevel` field
- `src/iced_app/frame_collect.rs` — New `IntraStrataKey` type, `intra_strata_sort_key` uses parent level/id for regions
- `src/lua_api/state.rs` — `raise_frame()`, `max_sibling_level()`, toplevel raise in `set_frame_visible`
- `src/lua_api/frame/methods/methods_core.rs` — `SetToplevel`/`IsToplevel` store value, `SetFrameStrata` invalidates cache
- `src/lua_api/frame/methods/methods_meta.rs` — `Raise()` calls `state.raise_frame()` instead of just +1
- `src/lua_api/frame/methods/mod.rs` — Re-export `propagate_strata_level_pub`
- `src/lua_api/frame/mod.rs` — Re-export `propagate_strata_level_pub`
- `src/loader/xml_frame.rs` — `append_toplevel_code` parses and emits `SetToplevel(true)`

## Active Investigation: Three-Slice Button Center Texture 0x0

**Problem:** Game menu buttons (ThreeSliceButtonTemplate) have Center texture rendering as 0x0, making the middle of each button transparent. Only Left and Right end-caps render.

**Root cause identified:** The Center texture uses cross-frame anchors (TOPLEFT→Left.TOPRIGHT, BOTTOMRIGHT→Right.BOTTOMLEFT) where Left and Right are sibling textures. Two separate bugs:

1. **Lua API `calculate_frame_width/height`** (`src/lua_api/frame/methods/methods_helpers.rs:42`): Requires both anchors to reference the **same** frame (`left_anchor.relative_to_id == right_anchor.relative_to_id`). Falls through to explicit width (0) when anchors reference different siblings.

2. **Rendering layout engine** (`src/iced_app/layout.rs`): `resolve_multi_anchor_edges` correctly handles cross-frame anchors — each anchor resolves its relative frame independently. But `compute_frame_rect` uses a fresh empty cache per call (uncached variant), so it must walk the full parent chain for each anchor's relative frame. The layout engine should produce correct results.

**Dump-tree confirmation:** `.CenterBG [Texture] (0x0) visible` in GameMenuFrame tree dump confirms the computed rect is zero-sized.

**Next steps:**
- Add targeted debug logging in `compute_frame_rect_cached` for multi-anchor frames where both left_x and right_x edges are resolved but rect is still 0x0
- Check if the issue is that Left/Right textures themselves compute to 0x0 (they use scale=0.28125 with TOPLEFT/TOPRIGHT anchors — single-anchor path uses `frame.width * eff_scale`)
- Fix `calculate_frame_width/height` to handle cross-frame anchors (separate relative_to_id)

## Known Issues

- Buff duration text missing in live GUI mode (OnUpdate not called during GUI startup)
- Script inheritance bug: `src/loader/helpers.rs` lines 297-299 prepend/append boolean swapped

## Key Technical Context

### Template Texture Properties

Two parallel code paths create textures from XML definitions:

| Path | Entry Point | Handles Tiling? |
|------|-------------|-----------------|
| XML loader (addon loading) | `create_texture_from_xml` in `src/loader/xml_texture.rs` | Yes (always did) |
| Template system (CreateFrame inherits) | `create_texture_from_template` in `src/lua_api/globals/template/elements.rs` | Yes (after fix) |

Both paths converge on similar Lua code generation but are maintained separately. When adding new texture properties, both paths must be updated.

### BackdropTemplateMixin UV Tiling
- `textureUVs` table (Backdrop.lua:146): edges use "repeatX"/"repeatY" placeholders replaced with computed repeat counts
- `coordStart = 0.0625`, `coordEnd = 0.9375` — edge texture has padding at borders
- TopEdge/BottomEdge: UV rotated (V→horizontal), width=0 (from anchors), height=edgeSize
- LeftEdge/RightEdge: UV standard (V→vertical), width=edgeSize, height=0 (from anchors)
- Corners: all UVs <1.0, SetSize(edgeSize, edgeSize) — work fine even with both tile flags

### Renderer Constraints
- Sampler: `address_mode_u/v: ClampToEdge` in `src/render/shader/atlas.rs`
- Shader: clamps UVs to `[0.0, 0.9999]` in quad.wgsl
- Atlas UV remapping: `vertex.tex_coords[0] = entry.uv_x + vertex.tex_coords[0] * entry.uv_width`

### WoW Region Rendering Model
- **Regions** (Texture, FontString) are NOT frames. They render as part of their parent frame's rendering unit.
- Sort order within a strata: frames sorted by frame_level, regions grouped with their parent
- `toplevel="true"`: frame is auto-raised above siblings when Show() is called
- `Raise()`: sets frame_level to max(sibling levels in same strata) + 1, propagates to descendants
