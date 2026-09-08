# Rendering Pipeline

## Overview

Two-tier rendering architecture:
- **Quad-based GPU rendering** via WGPU shaders (primary path)
- **Headless GPU rendering** for screenshots without GUI

The pipeline traverses the frame hierarchy, collects rendering commands into a `QuadBatch`, uploads to GPU, and renders via custom WGSL shaders with tiered texture atlases.

---

## QuadBatch System

**File:** `src/render/shader/quad.rs`

### QuadVertex Format (lines 20-33)

```rust
#[repr(C)]
pub struct QuadVertex {
    pub position: [f32; 2],      // Screen pixels, top-left origin
    pub tex_coords: [f32; 2],    // 0.0-1.0 UV space
    pub color: [f32; 4],         // RGBA, premultiplied alpha
    pub tex_index: i32,          // RGBA tier 0-4, glyph 5, BC1 6, BC3 7, -1 solid, -2 pending
    pub flags: u32,              // Blend mode and effect bits
    pub local_uv: [f32; 2],      // Quad-local UV, preserved across atlas remapping
    pub mask_tex_index: i32,     // Mask binding: -1 none, -2 pending, or atlas binding
    pub mask_tex_coords: [f32; 2], // Mask UVs, remapped during prepare
}
```

60 bytes per vertex.

### BlendMode (lines 5-14)

- **Alpha (0):** `src * alpha + dst * (1 - alpha)` (default)
- **Additive (1):** `src + dst` (highlights, glow)

### QuadBatch Structure (lines 89-100)

```rust
pub struct QuadBatch {
    pub vertices: Vec<QuadVertex>,             // 4 per quad
    pub indices: Vec<u32>,                     // 6 per quad (2 triangles)
    pub texture_requests: Vec<TextureRequest>, // Deferred texture loading
    pub mask_texture_requests: Vec<TextureRequest>, // Deferred mask resolution
}
```

### Key Methods

| Method | Purpose | Lines |
|--------|---------|-------|
| `push_quad()` | Core: 4 vertices + 6 indices | 137-183 |
| `push_solid()` | Solid color (tex_index = -1) | 186-194 |
| `push_textured_uv()` | Textured with custom UVs | 214-223 |
| `push_textured_path()` | Deferred loading via path | 229-250 |
| `push_three_slice_h_path()` | Horizontal 3-slice | 282-295 |
| `push_nine_slice()` | 9-slice (corners, edges, center) | 416-439 |
| `push_tiled()` | Tiling texture | 558-588 |
| `push_border()` | Rectangle border (4 edge quads) | 596-632 |

**TextureRequest** (lines 78-87): Stores path + vertex range for deferred GPU index resolution during `prepare()`.

---

## GPU Shader Pipeline

### quad.wgsl
**File:** `src/render/shader/quad.wgsl`

#### Vertex Shader (lines 57-71)

Transforms screen coords to clip space via orthographic projection matrix. Pass-through for tex_coords, color, tex_index, flags. Flat interpolation for tex_index and flags.

#### Fragment Shader (lines 105-131)

```wgsl
if in.tex_index < 0 {
    color = in.color;                    // Solid or pending
} else {
    let tex_color = sample_tiered_texture(in.tex_index, in.tex_coords);
    color = tex_color * in.color;        // Tinting via vertex color
}

color = vec4f(color.rgb * color.a, color.a);
if blend_mode == BLEND_ADDITIVE {
    color.a = 0.0;                       // Preserve dst, add premultiplied src
}
```

Additive quads no longer use the old 1.5x alpha-boost workaround. The shader now premultiplies
all colors, and additive quads zero their output alpha so the fixed premultiplied-alpha pipeline
produces `src + dst`.

**Texture Sampling** (lines 80-103):
- tex_index 0-4: tiered RGBA atlas (64/128/256/512/2048 cells)
- tex_index 5: glyph atlas
- tex_index 6-7: BC1 and BC3 compressed atlases
- Sampling: `textureSampleLevel(..., uv, 0.0)` (no mipmapping)
- UV clamping: `clamp(tex_coords, 0.0, 0.9999)` to avoid edge bleeding

Mask requests use the same deferred resolution path: RGBA entries are checked first, then BC1/BC3 entries. The resolved binding and UV rectangle are written to `mask_tex_index` and `mask_tex_coords`; unresolved pending masks become `-1` and are skipped.

### WowUiPipeline
**File:** `src/render/shader/pipeline.rs`

#### Uniforms (lines 12-30)

Orthographic projection: `[2.0/w, 0, 0, 0], [0, -2.0/h, 0, 0], [0, 0, 1, 0], [-1, 1, 0, 1]` -- Y-flip for screen coords.

#### Pipeline Setup (lines 70-120)

- Blend state: `ALPHA_BLENDING` (src: SrcAlpha, dst: OneMinusSrcAlpha)
- Topology: `TriangleList`
- Cull mode: None (2D UI)

#### Prepare Phase (lines 123-180)

1. Update uniforms if viewport changed
2. Resize buffers (power-of-two growth) if needed
3. Upload vertices and indices
4. Store index count

#### Render Phase (lines 183-235)

Set viewport and scissor to widget bounds, bind pipeline/uniforms/textures, draw indexed. LoadOp: `Load` (preserves iced's other widgets).

---

## Iced Integration

**File:** `src/iced_app/render.rs`

### shader::Program Implementation

```rust
impl shader::Program<Message> for &App {
    fn update(&self, _state, event, bounds, cursor) -> Option<Action<Message>>
    fn draw(&self, _state, _cursor, bounds) -> WowUiPrimitive
}
```

**Events** (lines 26-77): CursorMoved, ButtonPressed/Released (Left, Middle), WheelScrolled -> canvas messages.

**Draw** (lines 79-103):
1. Rebuild quads if dirty/resized (cached otherwise)
2. Load new textures from disk
3. Create primitive
4. Attach glyph atlas if dirty
5. Update frame time EMA (alpha=0.33, ~5-sample smoothing)

### Quad Batch Building (lines 523-591)

```rust
pub fn build_quad_batch_for_registry(registry, screen_size, ...) -> QuadBatch {
    // 1. Add background
    // 2. Collect visible frames via collect_ancestor_visible_ids()
    // 3. Sort by strata/level/draw-layer
    // 4. For each frame: emit quads by widget type
}
```

### Frame Type Rendering

| Type | Handler | Behavior |
|------|---------|----------|
| Frame/StatusBar | `build_frame_quads()` | Backdrop, nine-slice |
| Button | `build_button_quads()` | State-driven textures + center text |
| Texture | `build_texture_quads()` | Image, atlas, tiling, nine-slice |
| FontString | `emit_widget_text_quads()` | Text with justify/wrap/max_lines |
| CheckButton | `build_button_quads()` | + left-aligned text offset 20px |
| EditBox | `build_editbox_quads()` | + padded text |

**Button texture fallback** (lines 169-188): If tex_coords specified, use custom UVs. Otherwise 3-slice default (4px caps).

---

## Headless GPU Rendering (Screenshots)

**File:** `src/render/headless.rs`

Headless rendering uses the same WGPU pipeline without a window or swapchain:

1. Create a headless wgpu device + queue.
2. Create an `Rgba8UnormSrgb` render target (`RENDER_ATTACHMENT | COPY_SRC`).
3. Create `WowUiPipeline` (the same pipeline used by the GUI).
4. Prepare batches: upload textures and resolve atlas indices.
5. Clear, render, and read back pixels via `copy_texture_to_buffer()` and `map_async()`.

`render_to_image()` renders one batch with a fresh headless context. Related before/after images should use `render_batches_to_images(&[&QuadBatch], ...)`: it preloads the union of primary, mask, and glyph textures, then renders every batch through one device, pipeline, render target, and GPU atlas. This models the live renderer's persistent atlas. Separate one-image calls can repack atlas slots independently; bilinear sampling and remapped UVs then produce packing-dependent edge differences outside the geometry change being tested.

---

## Text and Glyph System

### GlyphAtlas
**File:** `src/render/glyph.rs`

```rust
pub struct GlyphAtlas {
    pixels: Vec<u8>,                          // 2048x2048 RGBA
    cursor_x: u32, cursor_y: u32,
    row_height: u32,
    entries: HashMap<CacheKey, GlyphEntry>,
    dirty: bool,
}
```

**Packing** (lines 104-164): Row-based left-to-right, top-to-bottom. 1px padding between glyphs. Warns when full.

**Pixel formats** from swash:
- Mask (alpha-only): stored as `(255, 255, 255, alpha)`
- Color (RGBA): copied as-is
- SubpixelMask (RGB): stored as `(255, 255, 255, alpha)`

### Text Emission (lines 316-385)

```rust
pub fn emit_text_quads(batch, font_system, glyph_atlas, text, bounds,
    font_path, font_size, color, justify_h, justify_v,
    shadow_color, shadow_offset, outline, word_wrap, max_lines)
```

1. Strip WoW markup (color codes)
2. Shape text via cosmic-text
3. Calculate vertical offset (justify_v)
4. Render layers back to front:
   - Outline (if enabled): 8 directions at +/-1 or +/-2 pixels
   - Shadow (if enabled): at shadow_offset
   - Main text

### Font System
**File:** `src/render/font.rs`

```rust
pub struct WowFontSystem {
    pub font_system: cosmic_text::FontSystem,
    pub swash_cache: cosmic_text::SwashCache,
    font_map: HashMap<String, FontEntry>,
}
```

Supported fonts include FRIZQT__.TTF (default), ARIALN.TTF, FRIZQT___CYR.TTF, and the Mists-era specialty fonts used by Blizzard XML. They load from CASC by default. If path/FDID resolution misses a known core font, the loader retries by the known CASC encoding key; it does not ship embedded font bytes. Uppercase path normalization is used for HashMap lookup, with unknown font paths falling back to the default family.

---

## GPU Texture Atlas

**File:** `src/render/shader/atlas.rs`

### Tiered Architecture (lines 16-22)

| Tier | Cell Size | Grid | Max Textures |
|------|-----------|------|--------------|
| 0 | 64x64 | 64x64 | 4,096 |
| 1 | 128x128 | 32x32 | 1,024 |
| 2 | 256x256 | 16x16 | 256 |
| 3 | 512x512 | 8x8 | 64 |
| 4 | 2048x2048 | 2x2 | 4 |

All RGBA tiers use 4096x4096 backing textures. Optional BC1 and BC3 compressed atlases are separate bindings used when compressed uploads are available; masks can resolve through those atlases too.

**Tier Selection** (lines 209-220): Find smallest tier that fits. If larger or all tiers full, try largest tier with scaling.

**Upload** (lines 223-250): Check cache, select tier, allocate grid slot, copy to GPU, compute UV rectangle.

**Glyph Atlas**: Separate 2048x2048 texture at binding 6.

**Bind Group**: tier_64 (0), tier_128 (1), tier_256 (2), tier_512 (3), tier_2048 (4), sampler (5), glyph_atlas (6), BC1 atlas (7), BC3 atlas (8), glyph sampler (9).

---

## Frame Strata and Draw Layer Sorting

### FrameStrata (src/widget/frame.rs:559-602)

```
WORLD < BACKGROUND < LOW < MEDIUM < HIGH < DIALOG < FULLSCREEN < FULLSCREEN_DIALOG < TOOLTIP
```

The simulator ignores the `BLIZZARD` input token; it does not participate as a drawable sorting tier. The FrameStrata retail capture did not test `BLIZZARD`. During XML `OnLoad`, a direct `PARENT` child and its actual parent both reported `DIALOG`, while an explicit literal sibling reported `LOW`. `GetFrameStrata()` cannot expose the original XML token or internal resolution mechanism. Runtime-fixed retail behavior remains unproven.

### DrawLayer (lines 607-638)

```
BACKGROUND < BORDER < ARTWORK < OVERLAY < HIGHLIGHT
```

### Sorting Logic (`src/lua_api/state_render.rs`)

The simulator builds one bucket per frame strata. Ordinary frames and regions retain
raw frame-level ordering; `raise_order` is only a tie-breaker between siblings with
the same raw frame level, and `Raise()`/`Lower()` cannot move a frame across a
higher- or lower-level sibling.

`toplevel="true"` uses a separate monotonic active show-order sequence. Showing a
top-level frame assigns the next sequence value; hiding removes it, and showing it
again assigns a newer value. This is intentionally separate from `raise_order`.
After normal per-strata emission, IDs belonging to a top-level frame are grouped by their nearest top-level ancestor while walking through intermediate strata. This applies even before that owner has a positive active show-order value. Each group is emitted contiguously at the owner's raw strata, with the owner anchored first. Thus a `HIGH` child of an unraised `LOW` top-level parent remains in the LOW group behind independent `MEDIUM`; an independent `HIGH`, `DIALOG`, plain `TOOLTIP`, or actual `GameTooltip` retains its own higher-strata group. Top-level visibility changes rebuild the affected bucket grouping; ordinary shows may use the incremental repair path. A `SetParent` change also invalidates the cached grouping and hit eligibility even if the moved frame's raw strata is unchanged, so a stale prior owner cannot determine its draw or input rank. `UIParent` and `WorldFrame` remain strata-root boundaries.

This boundary comes from the controlled retail 12.1.0.69587 capture at physical 1440-pixel display height, not a 768-unit logical canvas: case 1 is blue and cases 2–5 red in created, hide/show, and Raise phases (`/tmp/pi-native-control-pixels.json`). Raw raised values are transition observations, not global z-index values.

Within ordinary content, the effective order remains:

1. Frame strata
2. Raw frame level
3. `raise_order` for same-raw-level ties
4. Draw layer for regions (frames render before regions)
5. Widget ID

---

## Mouse Hit Testing

**File:** `src/iced_app/view.rs`

### Excluded Frames (lines 21-25)

`UIParent`, `Minimap`, `WorldFrame`, `DEFAULT_CHAT_FRAME`, `ChatFrame1`, `EventToastManagerFrame`, `EditModeManagerFrame`

### Hit-Test Building (lines 31-63)

Filter visible, mouse-enabled, non-excluded frames. Hit ordering consumes flattened render-bucket rank rather than reconstructing raw strata/level order, so grouped descendants cannot steal input through an independent panel. The rank persists through coalesced Hide→Show and is invalidated when reparenting changes owner groups.

### Query (lines 512-524)

Iterate in reverse render order. Return first frame containing the point.

---

## Alpha Propagation

**File:** `src/iced_app/render.rs` lines 344-376

```rust
fn collect_ancestor_visible_ids(registry) -> HashMap<u64, f32> {
    // BFS from roots, effective_alpha = parent_alpha * frame.alpha
    // Only descend into visible children
}
```

Each frame receives `eff_alpha = parent_alpha * own_alpha`. Frames with alpha <= 0 are skipped during rendering.

---

## Performance

- **Quad batch caching**: Rebuilt only when `quads_dirty` or screen resized
- **Hit-test caching**: Rebuilt lazily when invalidated
- **Glyph atlas caching**: Upload to GPU only when dirty
- **Buffer management**: Power-of-two allocation with 4KB minimum
- **Frame time**: EMA smoothing (alpha=0.33) over ~5 frames

---

## Rendering Flow Diagram

```
User Input (Mouse/Keyboard)
    |
Event Handler (iced_app/render.rs:26-77)
    | [Compute hit test]
App::hit_test(pos) -> frame_id
    | [Rebuild if needed]
build_quad_batch_for_registry()
    | [Traverse frame tree]
collect_ancestor_visible_ids() -> HashMap<id, alpha>
SimState::get_strata_buckets() -> per-strata frame/region order
    | [Emit quads per type]
emit_frame_quads() -> match widget_type { ... }
    | [Collect texture requests]
QuadBatch { vertices, indices, texture_requests }
    | [Load new textures]
App::load_new_textures() -> Vec<GpuTextureData>
    | [Create primitive]
WowUiPrimitive { quads, textures, glyph_atlas_data }
    | [Prepare GPU]
WowUiPrimitive::prepare() -> upload textures, resolve indices
    |
WowUiPipeline::prepare() -> upload quads to buffers
    | [Render]
WowUiPipeline::render() -> draw_indexed(0..index_count)
    | [GPU Execution]
vs_main() -> transform to clip space
fs_main() -> sample tiered texture, apply tint, blend
    |
Framebuffer (presented by iced)
```

---

## Key Files

| Module | File | Purpose |
|--------|------|---------|
| Quad System | `src/render/shader/quad.rs` | QuadBatch, vertices, blend modes |
| GPU Pipeline | `src/render/shader/pipeline.rs` | Uniforms, buffers, render pass |
| Shader Source | `src/render/shader/quad.wgsl` | WGSL vertex/fragment shaders |
| Texture Atlas | `src/render/shader/atlas.rs` | Tiered atlases, UV computation |
| Primitive | `src/render/shader/primitive.rs` | WowUiPrimitive, texture resolution |
| Rendering | `src/iced_app/render.rs` | Frame traversal, quad emission |
| Hit Testing | `src/iced_app/view.rs` | Strata sorting, containment tests |
| Glyphs | `src/render/glyph.rs` | GlyphAtlas, text emission |
| Fonts | `src/render/font.rs` | cosmic-text integration |
| Headless Render | `src/render/headless.rs` | Single-image and shared-atlas screenshot pipeline |
