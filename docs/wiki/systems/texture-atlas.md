# Texture Atlas System

Texture handling spans three interconnected layers: file I/O and caching (TextureManager), atlas name resolution and nine-slice detection (Atlas system), and GPU quad batching (rendering). The system supports BLP, PNG, WebP, and TGA formats, with WebP as the preferred local cache format.

## TextureManager (`src/texture.rs`)

```rust
pub struct TextureManager {
    addons_path: Option<PathBuf>,     // Addon directories
    cache: HashMap<String, TextureData>,
    sub_cache: HashMap<String, TextureData>,
}
```

All textures decoded to `TextureData { width, height, pixels: Vec<u8> }` in RGBA8.

**Path resolution priority**: addon textures (`Interface/AddOns/...`) → CASC via `asset-resolver` against the live WoW install at `/syncthing/World of Warcraft/Data`. Extracted BLPs are cached under `~/.cache/wow-ui-sim/casc-extract/`.

**Path normalization**: backslashes to forward slashes, extension stripped. `Interface\Buttons\UI-Panel-Button-Up.blp` → `Interface/Buttons/UI-Panel-Button-Up`. CASC lookup retries with several extensions (`blp`, `tga`, `ttf`, `otf`).

**Formats**: BLP via `image_blp` crate; the `casc` feature gates the CASC tier (default-on). Set `WOW_SIM_CASC=0` to disable.

## Atlas Lookup (`src/atlas.rs`)

The atlas database is a compiled perfect hash map (`phf_map!`) with ~50K entries built from WoW CSV exports at compile time.

```rust
pub struct AtlasInfo {
    pub file: &'static str,     // backing texture path
    pub width: u32, pub height: u32,
    pub left_tex_coord: f32, pub right_tex_coord: f32,
    pub top_tex_coord: f32, pub bottom_tex_coord: f32,
    pub tiles_horizontally: bool, pub tiles_vertically: bool,
}
```

`get_atlas_info(name)` resolution order:
1. Exact match (case-insensitive, strip/add `-2x` suffix)
2. Size-suffixed fallback: `coin-copper` → try `-16x16`, `-20x20`, `-32x32`, `-48x48`, `-64x64`
3. Underscore suffix: try `_2x` then `_1x`
4. Spelling corrections (e.g., Blizzard typo `divider` → `devider`)

`is_2x_fallback` halves reported dimensions: `width() = info.width / 2`.

## Nine-Slice Atlas Kits

Auto-detected by probing `{kit}-nineslice-cornertopleft`. If found, `get_nine_slice_atlas_info()` assembles 8 required pieces + optional center:

- Corners: `{kit}-nineslice-corner{topleft,topright,bottomleft,bottomright}`
- Edges: `_{kit}-nineslice-edge{top,bottom}`, `!{kit}-nineslice-edge{left,right}` (note prefix variations)
- Center: `{kit}-nineslice-center` (optional)

`SetAtlas()` prefers nine-slice kits over 2x fallbacks. Kit names stored in `frame.nine_slice_atlas` trigger `emit_nine_slice_atlas()` rendering (4 corners + tiled top/bottom edges + tiled left/right edges + optional center fill).

## Texture Coordinate System

WoW uses 8 UV values: `(tlx, tly, blx, bly, trx, try, brx, bry)`. Internally stored as `Option<(left, right, top, bottom)>` (packed form).

**SetTexCoord remapping**: when an atlas is active, user-supplied coords are remapped relative to the atlas sub-region:
```
result_left = atlas_left + user_left * (atlas_right - atlas_left)
```

## SetAtlas Lua API

`SetAtlas(atlasName, useAtlasSize, filterMode, resetTexCoords)` — looks up atlas info, detects nine-slice kit, sets `frame.nine_slice_atlas` or calls `apply_atlas_to_frame()` (sets texture path, UV coords, tiling hints). If `useAtlasSize`, sets frame width/height from atlas dimensions.

`SetAtlas("")` is a clear operation. This matters because ElvUI's `StripTextures()` calls `SetTexture(E.ClearTexture)` followed by `SetAtlas("")`; the generated atlas database contains an accidental empty-name entry pointing at a casting-bar flipbook, but the Lua API must treat the empty atlas name as clearing texture/atlas state rather than resolving that entry.

**Button texture propagation and visibility**: only children of `Button` or `CheckButton` with recognized state-slot keys (`NormalTexture`, `PushedTexture`, `HighlightTexture`, `DisabledTexture`, `CheckedTexture`, `DisabledCheckedTexture`) synchronize atlas and visibility with the parent button state. Custom keys and children of other widget types retain their own visibility and render independently. This prevents atlas assignment from exposing explicitly XML-hidden animation textures.

When a recognized button texture child is cleared with `SetAtlas("")`, the corresponding parent button slot is cleared too. Without that propagation, stripped Normal/Pushed/Highlight children can leave stale parent button texture fields behind for the render path.

The cached vendor `GradualAnimatedStatusBar` template provides the bounded regression case: its gain-flare and level-up textures begin XML-hidden, `SetAnimationTextures()` assigns their atlases without showing them, and the animation lifecycle owns their temporary visibility. Targeted simulator tests pass; GUI XP/reputation startup proof and final checks remain pending. This is not native-client conformance evidence.

## Deferred Texture Loading

Quad batch uses `push_textured_path()` with deferred loading: quads store a path reference; during `WowUiPrimitive::prepare()`, paths are resolved to GPU atlas bindings. Ordinary RGBA uploads use the five size tiers; eligible compressed assets may use the separate BC1 or BC3 atlas. Mask requests use the same lookup, so a BC-backed mask receives a valid compressed-atlas binding and UV remap instead of being discarded as unresolved.

The live renderer keeps its GPU atlas in the pipeline across frames. For related headless before/after images, `render_batches_to_images(&[&QuadBatch], ...)` preloads the union of all requests and renders through one persistent atlas context. Separate `render_to_image()` calls repack slots independently; with bilinear filtering, atlas-edge samples can differ even when an unchanged quad's geometry is identical.

## Sources

- [texture-atlas-system.md](../../texture-atlas-system.md) — TextureManager, AtlasInfo, nine-slice kits, tex coord remapping, StatusBar fill
- [atlas.rs](../../../src/lua_api/frame/methods/widgets/texture/atlas.rs) — `SetAtlas("")` clear semantics and recognized Button/CheckButton slot propagation
- [texture atlas visibility](../../specs/texture-atlas-visibility.md) — behavior contract and targeted proof

## See Also

- [[rendering-pipeline]] — GPU atlas tiers, QuadBatch, nine-slice and tiling rendering
- [[widget-system]] — Frame fields: texture, atlas, tex_coords, nine_slice_atlas
- [Texture atlas visibility](../../specs/texture-atlas-visibility.md) — XML-hidden animation-texture regression
