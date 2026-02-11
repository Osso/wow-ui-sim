# WoW UI Sim Brief - 2026-02-11

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

## Also Fixed: Cargo Build Failure

`iced-layout-inspector` git dependency had stale v0.1.0 checkout in cargo cache (path dep `peercred-ipc` can't resolve from git). Cargo.toml already pointed to v0.2.0 which uses git URL. Fix: removed stale checkout from `~/.cargo/git/checkouts/`.

## Pending Tasks
- [ ] Find and fix buff duration text missing in live GUI mode (OnUpdate not called during GUI startup)
- [ ] Script inheritance bug: `src/loader/helpers.rs` lines 297-299 prepend/append boolean swapped

## Parallel Session Warning
Another Claude session may be working on the same codebase. DO NOT use `git stash` or `git revert`.

## Key Technical Context

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
