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

## Known Issues

- **eff_alpha not used in rendering**: `collect_ancestor_visible_ids()` computes effective alpha per-frame but `emit_frame_quads` only uses `f.alpha` directly. All quad builders ignore ancestor alpha. Currently not visible because parent alphas are 1.0 in the default UI, but will cause incorrect rendering for frames with non-1.0 parent alpha chains.
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
