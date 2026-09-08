# Rendering Pipeline

The rendering pipeline traverses the frame hierarchy, batches geometry into a `QuadBatch`, uploads it to the GPU, and draws via custom WGSL shaders. A separate headless path renders screenshots without a window. Texture bindings cover five RGBA atlas tiers plus glyph, BC1, and BC3 atlases; mask requests can resolve through either the RGBA or BC atlases.

## QuadBatch System

**QuadVertex** (`src/render/shader/quad.rs`) — 60 bytes per vertex:

| Field | Type | Purpose |
|-------|------|---------|
| `position` | `[f32; 2]` | Screen pixels, top-left origin |
| `tex_coords` | `[f32; 2]` | UV 0.0–1.0 |
| `color` | `[f32; 4]` | RGBA premultiplied |
| `tex_index` | `i32` | RGBA tier 0–4, glyph 5, BC1 6, BC3 7, -1=solid, -2=pending |
| `flags` | `u32` | Blend mode and effect bits |
| `local_uv` | `[f32; 2]` | Quad-local UV preserved across atlas remapping |
| `mask_tex_index` | `i32` | Mask binding: -1=none, -2=pending, or atlas binding |
| `mask_tex_coords` | `[f32; 2]` | UVs into the resolved mask atlas slot |

Each quad = 4 vertices + 6 indices (two triangles). `BlendMode`: Alpha (default) or Additive (glow/highlights — zeroes output alpha so GPU produces `src + dst`).

Key batch methods: `push_solid()`, `push_textured_uv()`, `push_textured_path()` (deferred), `push_three_slice_h_path()`, `push_nine_slice()`, `push_tiled()`, `push_border()`.

## GPU Pipeline (`src/render/shader/pipeline.rs`)

Orthographic projection matrix with Y-flip: `[2/w, 0, 0, 0], [0, -2/h, 0, 0], [-1, 1, 0, 1]`. Blend state: `SrcAlpha / OneMinusSrcAlpha`. Topology: `TriangleList`. Cull: none.

Prepare phase: resize power-of-two buffers if needed, upload vertices and indices. Render phase: set viewport/scissor, bind pipeline + uniforms + textures, `draw_indexed`.

## WGSL Shaders (`src/render/shader/quad.wgsl`)

Fragment shader samples the five-tier RGBA atlas, glyph atlas, or BC1/BC3 compressed atlas according to `tex_index`; `tex_index < 0` uses vertex color directly (solid or unresolved quads). All colors are premultiplied: `color = vec4f(rgb * a, a)`. Additive quads zero output alpha. UV coords are clamped to `[0, 0.9999]` to prevent edge bleeding. No mipmapping (`textureSampleLevel(..., 0.0)`). Mask coverage is applied after sampling through mask alpha. Simulator mask paths do not use RGB as clipping coverage because valid opaque regions may be black in RGB.

## Tiered Texture Atlas (`src/render/shader/atlas.rs`)

| Tier | Cell size | Grid | Capacity |
|------|-----------|------|----------|
| 0 | 64×64 | 64×64 | 4,096 |
| 1 | 128×128 | 32×32 | 1,024 |
| 2 | 256×256 | 16×16 | 256 |
| 3 | 512×512 | 8×8 | 64 |
| 4 | 2048×2048 | 2×2 | 4 |

All RGBA tiers back a 4096×4096 texture. Glyph, BC1, and BC3 atlases are separate bindings. Deferred mask requests check the RGBA map first, then the BC map; resolved UVs are remapped into the selected slot. Tier selection: smallest tier that fits; falls back to largest with scaling if oversized.

## Strata and Draw Layer Sorting

`SimState::build_strata_buckets()` builds one bucket per frame strata. Ordinary
content is ordered by raw `frame_level`; `raise_order` is only a tie-breaker for
siblings at the same raw level. Explicit `Raise()` and `Lower()` therefore do not
cross raw frame levels. Regions remain grouped with their owning frame and then
use draw-layer order (`BACKGROUND < BORDER < ARTWORK < OVERLAY < HIGHLIGHT`)
within that frame's render group.

Top-level panels use a separate monotonic active show-order sequence. Showing a
`toplevel="true"` frame assigns the next sequence value; hiding removes its active
entry, and showing it again assigns a newer value. After ordinary per-strata
emission, every ID under an active top-level frame is assigned to its nearest
active top-level ancestor, even when intermediate parents use other strata. Each
owner's IDs are then emitted as one contiguous segment in show order, with the
owner first. Top-level visibility changes invalidate the bucket for a full regroup;
ordinary child shows can use incremental repair. `UIParent` and `WorldFrame` are
strata-root boundaries.

Strata order: `WORLD < BACKGROUND < LOW < MEDIUM < HIGH < DIALOG < FULLSCREEN < FULLSCREEN_DIALOG < TOOLTIP`.

## Alpha Propagation

`collect_ancestor_visible_ids()` BFS from roots: `eff_alpha = parent_alpha * frame.alpha`. Frames with `eff_alpha <= 0` are skipped entirely.

## Hit Testing (`src/iced_app/frame_collect.rs`, `src/iced_app/hit_grid.rs`, `src/iced_app/view.rs`)

`frame_collect` owns the shared `HitOrderKey` and collects visible, mouse-enabled frames in render order. The GUI-only `hit_grid` module imports that key for spatial indexing; headless builds do not compile the grid or its GUI dependency tree. Queries iterate in reverse (highest strata first), returning the first frame containing the cursor point. Several system frames (UIParent, Minimap, WorldFrame, chat frames) are excluded.

## Performance

- Quad batch rebuilt only when `quads_dirty` flag is set or screen resized
- Hit-test list cached, invalidated on layout changes
- Glyph atlas uploaded to GPU only when dirty
- Frame time smoothed via EMA (alpha=0.33, ~5-sample window)

## Headless GPU Rendering

The headless path is implemented in `src/render/headless.rs` and uses the same WGPU pipeline without a window: it creates an `Rgba8UnormSrgb` target, prepares atlas bindings, clears, renders, and reads pixels back with `copy_texture_to_buffer()` and `map_async()`.

`render_to_image()` creates a fresh context for one batch. Related before/after images should use `render_batches_to_images(&[&QuadBatch], ...)`, which preloads the union of primary, mask, and glyph textures and renders all batches through one device, pipeline, target, and GPU atlas. That matches the live renderer's persistent atlas; separate calls can repack slots and create bilinear/UV edge differences unrelated to the changed geometry.

## Sources

- [rendering-pipeline.md](../../rendering-pipeline.md) — QuadBatch, shaders, atlas, hit testing, alpha, text
- [state_render.rs](../../../src/lua_api/state_render.rs) — per-strata buckets, active top-level show order, nearest-owner grouping, and visibility invalidation
- [state_render_tests.rs](../../../src/lua_api/state_render_tests.rs) — cross-strata top-level grouping, repeated show ordering, and same-level Raise/Lower boundaries
- [world_map_voice_button_order.rs](../../../tests/world_map_voice_button_order.rs) — live-like top-level overlap regression

## See Also

- [[layout-system]] — produces LayoutRect consumed by quad building
- [[widget-system]] — WidgetType dispatch per frame type
- [[texture-atlas]] — TextureManager and atlas resolution feeding the GPU atlas
- [[addon-compatibility]] — Docker headless release build contract
