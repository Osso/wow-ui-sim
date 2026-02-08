# Texture & Atlas System

## Overview

Three interconnected layers:

1. **Loading Layer** (`src/texture.rs`) - File I/O, BLP/PNG/WebP parsing, texture caching
2. **Atlas Resolution** (`src/atlas.rs`) - WoW atlas name lookup, nine-slice kit detection, size fallback logic
3. **Rendering Layer** (`src/iced_app/render.rs`, `src/iced_app/nine_slice.rs`, `src/iced_app/tiling.rs`) - Quad batch generation, GPU submission

---

## Texture Loading (TextureManager)

**File:** `src/texture.rs`

### Structure

```rust
pub struct TextureManager {
    textures_path: PathBuf,           // ./textures (local WebP cache)
    interface_path: Option<PathBuf>,  // ~/Projects/wow/Interface (extracted BLP files)
    addons_path: Option<PathBuf>,     // Addon directories
    cache: HashMap<String, TextureData>,       // Loaded texture pixels
    sub_cache: HashMap<String, TextureData>,   // Sub-region extractions
}
```

**Key type:** `TextureData { width: u32, height: u32, pixels: Vec<u8> }` -- RGBA8 format

### Path Resolution Strategy (lines 57-157)

Four-tier priority system with case-insensitive matching:

1. **Addon textures** -- `Interface/AddOns/{AddonName}/...` (lines 130-137)
2. **Local WebP cache** -- `./textures/...` (lines 145-148)
3. **Game files** -- `~/Projects/wow/Interface/...` extracted BLP (lines 151-154)
4. **Case-insensitive fallback** -- Crawls directories byte-by-byte (lines 184-217)

**Extension priority** (line 162): `webp > WEBP > PNG > png > tga > TGA > blp > BLP > jpg > JPG`

### Path Normalization (lines 233-243)

Replace backslashes with forward slashes, remove file extension. Example: `Interface\Buttons\UI-Panel-Button-Up.blp` -> `Interface/Buttons/UI-Panel-Button-Up`

### File Format Support (lines 246-275)

- **BLP**: `image_blp` crate (0.24 compat layer, since simulator uses `image` 0.25)
- **PNG/WebP/TGA/JPG**: Standard `image` crate, all converted to RGBA8

---

## Atlas System

### Atlas Data Structure (src/data/atlas.rs, auto-generated)

```rust
pub struct AtlasInfo {
    pub file: &'static str,              // e.g. "Interface\Buttons\UI-Panel-Button-Up"
    pub width: u32,
    pub height: u32,
    pub left_tex_coord: f32,             // UV coordinates (0.0-1.0)
    pub right_tex_coord: f32,
    pub top_tex_coord: f32,
    pub bottom_tex_coord: f32,
    pub tiles_horizontally: bool,
    pub tiles_vertically: bool,
}

pub struct AtlasLookup {
    pub info: &'static AtlasInfo,
    pub is_2x_fallback: bool,            // True if resolved to -2x (hi-res) variant
}

impl AtlasLookup {
    pub fn width(&self) -> u32 {
        if self.is_2x_fallback { self.info.width / 2 } else { self.info.width }
    }
}
```

The atlas database is a **perfect hash map** (`phf_map!`) with ~50K entries, compiled at build time from WoW CSV exports.

### Atlas Lookup Resolution (src/atlas.rs:82-118)

**Function:** `get_atlas_info(name: &str) -> Option<AtlasLookup>`

Resolution order:
1. **Generated lookup** (exact match, case-insensitive, strip/add `-2x` suffix)
2. **Size-suffixed fallback** (lines 89-95): e.g. `coin-copper` -> try `coin-copper-16x16`, `-20x20`, `-32x32`, `-48x48`, `-64x64`
3. **Underscore suffix fallback** (lines 97-105): try `_2x` then `_1x`
4. **Spelling corrections** (lines 112-118): Blizzard typo fix `divider` -> `devider`

### Nine-Slice Atlas Kits (src/atlas.rs:8-71)

A nine-slice kit is a set of 9 atlas entries for borders: 4 corners + 4 edges + optional center.

**Detection** (lines 37-46): Probes for `{kit}-nineslice-cornertopleft`. If it exists, it's a kit.

**Entry naming convention:**
- Corners: `{kit}-nineslice-cornertopleft`, `cornertopright`, `cornerbottomleft`, `cornerbottomright`
- Edges: `_{kit}-nineslice-edgetop`, `edgebottom`, `!{kit}-nineslice-edgeleft`, `edgeright` (note underscore/exclamation prefixes)
- Center: `{kit}-nineslice-center` (optional)

```rust
pub struct NineSliceAtlasInfo {
    pub corner_tl: NineSlicePiece,
    pub corner_tr: NineSlicePiece,
    pub corner_bl: NineSlicePiece,
    pub corner_br: NineSlicePiece,
    pub edge_top: NineSlicePiece,
    pub edge_bottom: NineSlicePiece,
    pub edge_left: NineSlicePiece,
    pub edge_right: NineSlicePiece,
    pub center: Option<NineSlicePiece>,
}

pub struct NineSlicePiece {
    pub file: &'static str,
    pub left: f32, pub right: f32,
    pub top: f32, pub bottom: f32,
    pub width: u32, pub height: u32,
}
```

---

## Texture Coordinates System

### Eight-Value TexCoord Format

WoW uses 8 texture coordinate values: **tlx, tly, blx, bly, trx, try, brx, bry** (top-left, bottom-left, top-right, bottom-right, each with x,y in UV space 0.0-1.0).

**Rust representation:** `Option<(f32, f32, f32, f32)>` = `(left, right, top, bottom)` (packed, unpacks to 8 values in Lua).

**Code:** `src/lua_api/frame/methods/methods_texture.rs:386-413`

### Texture Coordinate Remapping (lines 425-446)

When `SetTexCoord()` is called on an atlas-based texture, coordinates are remapped relative to the atlas sub-region:

```rust
fn remap_tex_coords(atlas_tex_coords, left, right, top, bottom) -> (f32, f32, f32, f32) {
    if let Some((al, ar, at, ab)) = atlas_tex_coords {
        let aw = ar - al;
        let ah = ab - at;
        (al + left * aw, al + right * aw, at + top * ah, at + bottom * ah)
    } else {
        (left, right, top, bottom)
    }
}
```

**Example:** Atlas coords (0.5, 0.75, 0.2, 0.4), user calls `SetTexCoord(0.0, 0.5, 0.0, 1.0)`:
Result: `(0.5, 0.625, 0.2, 0.4)`

---

## SetAtlas Lua API

**File:** `src/lua_api/frame/methods/methods_texture.rs:125-171`

### SetAtlas(atlasName, useAtlasSize, filterMode, resetTexCoords)

1. Look up atlas info
2. Prefer nine-slice kits over 2x fallbacks
3. If nine-slice kit detected: store in `frame.nine_slice_atlas`
4. If standard atlas: apply via `apply_atlas_to_frame()`, propagate to button parent if applicable
5. If atlas not found: store name anyway

### apply_atlas_to_frame (lines 227-254)

Sets texture path, UV coordinates, and tiling hints on the frame. If `use_atlas_size` is true, sets frame width/height from atlas dimensions.

### Button Texture Propagation (lines 256-319)

When a child texture with parentKey `NormalTexture`, `PushedTexture`, etc. calls `SetAtlas()`, the texture path and UVs propagate to the parent button's state fields.

**Custom parentKeys do NOT propagate** -- the child renders independently.

---

## Tiling System

**File:** `src/iced_app/tiling.rs`

### Tile Dimensions (lines 6-11)

If frame has explicit size, use it. Otherwise estimate from UV region (scaled by 128px baseline, minimum 8px).

### Tiling Modes (lines 13-31)

- `horiz_tile && !vert_tile` -> horizontal tiles
- `vert_tile && !horiz_tile` -> vertical tiles
- Both -> grid tiles

### Horizontal Tiling (lines 33-44)

Loop across bounds.width in steps of tile_w. Crop UV width for partial tiles at the right edge. Push each as a separate quad.

---

## Nine-Slice Rendering

**File:** `src/iced_app/nine_slice.rs`

### emit_nine_slice_atlas (lines 94-114)

Renders 4 corners + 4 tiled edges + optional center:
1. `emit_corners()` -- 4 fixed-size corner quads
2. `emit_horiz_edges()` -- top/bottom edges tiled horizontally
3. `emit_vert_edges()` -- left/right edges tiled vertically
4. Center fill if `ns.center` exists and area is positive

### Corner Layout (lines 27-50)

Top-left at bounds origin, top-right at right edge, bottom-left at bottom edge, bottom-right at opposite corner.

### Edge Tiling (lines 52-92)

Horizontal edges (top/bottom) tiled horizontally; vertical edges (left/right) tiled vertically.

---

## StatusBar Fill Rendering

**File:** `src/iced_app/statusbar.rs`

```rust
pub(super) struct StatusBarFill {
    pub fraction: f32,      // 0.0-1.0
    pub reverse: bool,      // Fill from right to left
    pub color: Option<Color>,
}
```

### Color Tinting (render.rs:225-231)

Bar texture tinted with StatusBar color. If no color, white (no tint).

### Fill UV Clipping (render.rs:276-292)

Both bounds and UV coordinates are clipped to the fill fraction. Normal: left portion. Reversed: right portion.

---

## Key Design Patterns

1. **Parallel Lua/Rust State:** Textures/atlas info exist in both `widget::Frame` fields and Lua `FrameHandle`. Changes via Lua update Rust immediately.
2. **Deferred Texture Loading:** Quads created with path references (`push_textured_path_uv`), resolved to GPU indices during `prepare()`.
3. **Atlas-Relative Remapping:** `SetTexCoord()` remaps relative to atlas sub-region when atlas is active.
4. **Nine-Slice Auto-Detection:** `SetAtlas()` probes for corner-top-left entry. Kit names preferred over 2x fallbacks.
5. **Button Texture Propagation:** Standard parentKey children sync atlas info to parent button. Custom parentKeys render independently.

---

## File Reference

| System | File | Key Items |
|--------|------|-----------|
| Texture Loading | `src/texture.rs` | `TextureManager`, `load()`, `resolve_path()` |
| Atlas Lookup | `src/atlas.rs` | `get_atlas_info()`, `get_nine_slice_atlas_info()` |
| Atlas Data | `data/atlas.rs` (auto-gen) | `AtlasInfo`, `ATLAS_DB` |
| Lua Texture API | `src/lua_api/frame/methods/methods_texture.rs` | `SetAtlas()`, `SetTexture()`, `SetTexCoord()` |
| Quad Batching | `src/render/shader/quad.rs` | `QuadBatch`, `QuadVertex` |
| Rendering | `src/iced_app/render.rs` | `build_texture_quads()`, `build_button_quads()` |
| Tiling | `src/iced_app/tiling.rs` | `emit_tiled_texture()` |
| Nine-Slice | `src/iced_app/nine_slice.rs` | `emit_nine_slice_atlas()` |
| StatusBar | `src/iced_app/statusbar.rs` | `StatusBarFill` |
