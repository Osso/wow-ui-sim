# Rendering Pipeline

## Overview

Two-tier rendering architecture:
- **Quad-based GPU rendering** via WGPU shaders (primary path)
- **Headless software rendering** for screenshots without GUI

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
    pub tex_index: i32,          // Texture tier (0-3) or -1 (solid) or -2 (pending)
    pub flags: u32,              // Blend mode bits
}
```

36 bytes per vertex.

### BlendMode (lines 5-14)

- **Alpha (0):** `src * alpha + dst * (1 - alpha)` (default)
- **Additive (1):** `src + dst` (highlights, glow)

### QuadBatch Structure (lines 89-100)

```rust
pub struct QuadBatch {
    pub vertices: Vec<QuadVertex>,           // 4 per quad
    pub indices: Vec<u32>,                   // 6 per quad (2 triangles)
    pub texture_requests: Vec<TextureRequest>, // Deferred texture loading
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

if (in.flags & 0xFF) == BLEND_ADDITIVE {
    color.a = min(color.a * 1.5, 1.0);  // Boost additive alpha
}
```

**Texture Sampling** (lines 80-103):
- tex_index 0-3: tiered atlas (64/128/256/512 cells)
- tex_index 4: glyph atlas
- Sampling: `textureSampleLevel(..., uv, 0.0)` (no mipmapping)
- UV clamping: `clamp(tex_coords, 0.0, 0.9999)` to avoid edge bleeding

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

## Software Rendering (Screenshots)

**File:** `src/render/software.rs`

Headless GPU rendering without window/swapchain:

1. Create headless wgpu device + queue
2. Create render target texture (`Rgba8UnormSrgb`, RENDER_ATTACHMENT | COPY_SRC)
3. Create WowUiPipeline (same as GUI)
4. Prepare: upload textures, resolve indices
5. Render to texture with `LoadOp::Clear`
6. Read back pixels via `copy_texture_to_buffer()` + `buffer_slice.map_async()` + `poll()`

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

Supported fonts: FRIZQT__.TTF (default), ARIALN.TTF, frizqt___cyr.ttf, TrajanPro3SemiBold.ttf. Uppercase path normalization for HashMap lookup, fallback to default if not found.

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

All tiers use 4096x4096 backing textures.

**Tier Selection** (lines 209-220): Find smallest tier that fits. If larger or all tiers full, try largest tier with scaling.

**Upload** (lines 223-250): Check cache, select tier, allocate grid slot, copy to GPU, compute UV rectangle.

**Glyph Atlas**: Separate 2048x2048 texture at binding 5.

**Bind Group**: tier_64 (0), tier_128 (1), tier_256 (2), tier_512 (3), sampler (4), glyph_atlas (5).

---

## Frame Strata and Draw Layer Sorting

### FrameStrata (src/widget/frame.rs:559-602)

```
WORLD < BACKGROUND < LOW < MEDIUM < HIGH < DIALOG < FULLSCREEN < FULLSCREEN_DIALOG < TOOLTIP
```

### DrawLayer (lines 607-638)

```
BACKGROUND < BORDER < ARTWORK < OVERLAY < HIGHLIGHT
```

### Sorting Logic (render.rs:398-415)

1. Primary: frame strata
2. Secondary: frame level (within strata)
3. Tertiary: draw layer for regions (frames render before regions)
4. Tie-breaker: widget ID

---

## Mouse Hit Testing

**File:** `src/iced_app/view.rs`

### Excluded Frames (lines 21-25)

`UIParent`, `Minimap`, `WorldFrame`, `DEFAULT_CHAT_FRAME`, `ChatFrame1`, `EventToastManagerFrame`, `EditModeManagerFrame`

### Hit-Test Building (lines 31-63)

Filter visible, mouse-enabled, non-excluded frames. Sort by strata/level. Cache lazily, invalidated on layout changes.

### Query (lines 512-524)

Iterate in reverse (highest strata first). Return first frame containing the point.

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
collect_sorted_frames() -> sorted by strata/level/draw-layer
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
| Software Render | `src/render/software.rs` | Headless screenshot pipeline |
