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

Top-level panels use a separate monotonic active show-order sequence. Showing a `toplevel="true"` frame assigns the next sequence value; hiding removes its active entry, and showing it again assigns a newer value. Every ID below a top-level owner is grouped with that owner even when the owner is unraised and intermediate parents use other strata. Each owner's IDs emit as one contiguous segment at its owner strata, with the owner first. Therefore a `HIGH` child remains behind an independent `MEDIUM` panel when its top-level owner is `LOW`, while independent `HIGH`, `DIALOG`, plain `TOOLTIP`, and actual `GameTooltip` remain above `MEDIUM`. Top-level visibility changes invalidate the bucket for a full regroup; ordinary child shows can use incremental repair. `UIParent` and `WorldFrame` are strata-root boundaries.

The controlled retail 12.1.0.69587 capture classifies this at physical 1440-pixel display height: case 1 BLUE, cases 2–5 RED in created, hide/show, and Raise phases. Raised getter values record local transition state, not a global z-index.

Strata order: `WORLD < BACKGROUND < LOW < MEDIUM < HIGH < DIALOG < FULLSCREEN < FULLSCREEN_DIALOG < TOOLTIP`.

### Deferred text and nested wrappers

FontString deferral must stop before a child at a higher raw frame level, using
the FontString's original owning frame level rather than the region's stored
level. Hoisting regions out of regionless Frame/ScrollFrame wrappers is confined
to same-level subtrees. Otherwise lower-level text can be emitted after an opaque
modal. Within a hoisted subtree, nested wrapper regions are suppressed exactly
once; Button/EditBox regions remain owned by those widgets.

Regression coverage in `state_render_buckets_tests.rs` exercises higher-level
opaque covers, parent text, same-level hoisting, and button labels. The
`higher_level_modal_covers_nested_text_after_repeated_show_and_bucket_repair`
test in `state_render_tests.rs` also covers the final strata buckets and repeated
modal hide/show with cache repair.

`tests/text_layer_order.rs` exercises the Lua API through the glyph atlas and
headless GPU renderer for both Frame and ScrollFrame wrappers. It checks visible
lower glyphs without the cover, no lower glyphs through an alpha-1 cover, a
visible cover caption, and repeated hide/show. Colours are compared against the
empty margin in the same image, not a platform-specific reference PNG. Fonts use
the system fallback without CASC; the fixture loads no Blizzard UI or addons.
As with the existing text-shadow test, image checks skip when no GPU adapter is
available. Bucket tests remain independent of GPU availability.

## Alpha Propagation

`collect_ancestor_visible_ids()` BFS from roots: `eff_alpha = parent_alpha * frame.alpha`. Frames with `eff_alpha <= 0` are skipped entirely.

## ScrollFrame presentation geometry

`08c43a536` and `4c39f4a5a` apply the shared layout presentation transform before quad emission and hit geometry, without changing stored layout rectangles, anchors, or logical geometry queries. Follow-up `ec6b39047` reuses existing quad clipping for hover geometry rather than keeping a duplicate clip helper. Each crossed ScrollFrame contributes its requested offset only when the ancestor's designated `scroll_child_id` is the current edge; this covers nested viewports and descendants whose anchors point outside the subtree. Render clips intersect presented ancestor viewport rectangles. A disjoint nested intersection becomes an explicit empty clip, so it suppresses content instead of removing the clip constraint. Lines, masks, and hover quads use the same presentation/clip path.

Follow-ups `4e344ab4f` and `1ecc0c98c` cache cumulative offsets only for one render-list build, hit-grid batch, or hit-test descent. Descendants reuse already-computed ancestor totals; nonclipping parent edges return before resolving presented parent geometry. The cache does not mutate logical geometry or introduce persistent invalidation state.

`ed69bd136` supplies RED coverage for quad movement, hit movement, and disjoint nested clipping. Initial proof passed 27 scroll-widget, eight cache-backed hit, six strata, three mask, three line, and one hover cases, but existing 20,000-frame deep click/hover chains timed out at 90 seconds from repeated ancestry traversal. Final independent verification passes all 52 scoped cases, including the four-case direct hit-testing module in 0.477 seconds and the scaled-child quad fixture. Format/check pass; changed functions have no readability findings, while `strata_emit.rs` retains its pre-existing length-cap violation. `/tmp/cross-version-scroll-presentation-final-verification-ledger.md` records exact proof. Scroll-specific line/mask/highlight behavior, all UI paths, live GPU pixels, and native-client behavior remain unclaimed. [ScrollFrame presentation offsets](../../specs/scrollframe-presentation.md) defines the bounded contract. Solarity corroborates separation of logical geometry from presentation transforms only; no Solarity code was copied and it is not native proof.

`b6bb2f710` extends the shared translation to the public `Region:IsMouseOver` bounds query after its logical-rect, visibility, and mouse-enabled guards and before bounds/margin comparison. It leaves `GetRect` logical, preserves optional margins, and does not import this pipeline's viewport clipping/intersection semantics into the query. Independent verification passes the bounded query scope; [ScrollFrame presentation offsets](../../specs/scrollframe-presentation.md) records its 35-case and format/check evidence separately from the 52-case render/hit proof.

## Hit Testing (`src/iced_app/frame_collect.rs`, `src/iced_app/hit_grid.rs`, `src/iced_app/view.rs`)

`frame_collect` owns the shared `HitOrderKey`. Input must derive its spatial order from flattened render buckets, not a second raw strata/level sort, so a grouped child cannot steal input through a panel rendered above it. The GUI-only `hit_grid` module imports that key for spatial indexing; headless builds do not compile the grid or its GUI dependency tree. ScrollFrame presentation transforms and ancestor clips are shared with rendering, so cache-backed hit targets move and clip with presented content. Several system frames (UIParent, Minimap, WorldFrame, chat frames) are excluded.

## Font sources

`WowFontSystem` (`src/render/font.rs`) loads Blizzard's fonts (`Fonts\\FRIZQT__.TTF`, ...) from CASC at startup. A font file an addon ships (`Interface\\AddOns\\<Addon>\\...`) is loaded on first use from the AddOns directories, resolved ignoring case, and keeps the weight, style and stretch of its face, so a bold file of a family renders bold. `SetFont` with an addon path that names no file raises the client's error, `FontString:SetFont(): Invalid font asset (<path>): file not found`, and keeps the font (**UNVERIFIED native claim:** contributor observation on Forever 1.60.1; independent profile evidence is pending); a file that is not a font warns once and draws in the default font.

`WowFontSystem` (`src/render/font.rs`) loads Blizzard's fonts (`Fonts\\FRIZQT__.TTF`, ...) from CASC at startup. Without CASC every Blizzard font path maps to one fallback family: the font file `WOW_SIM_FALLBACK_FONT` names, else the first system face. The first system face differs between operating systems, so renders compared across machines pin the fallback.

The [font-source contract](../../specs/font-sources.md) separates simulator file-loading policy from native evidence. Resolved addon font files must remain under their canonical AddOns root, including when paths contain symlinks. The missing-file native assertion is currently optional; the combined-tree runtime gate has not run.

## Inline icons in text

`sim_render::inline_markup` parses texture escapes, `|Tpath-or-fileID:height:width:offsetX:offsetY:texWidth:texHeight:left:right:top:bottom:r:g:b|t`, and atlas escapes, `|A:atlas:height:width:offsetX:offsetY:r:g:b|a`, then splits text and icons for measurement and drawing.

**VERIFIED (R88d, Retail 12.1.5.70688):** With icon markup enabled, the tokenizer routes `|T...|t` and `|A:...|a` to their handlers. Atlas markup requires the colon. The first pipe after an opening tag must begin its matching lowercase closing marker. A wrong or missing marker returns the opening pipe to ordinary text handling; a later match does not repair an earlier wrong marker. A recognized token with missing required fields is consumed without an icon. Texture markup requires a colon and height position; atlas markup requires height and width positions. An empty position is not the same as a missing position. Extra fields after blue are ignored.

**VERIFIED (R88d, Retail 12.1.5.70688):** The field reader copies at most 31 bytes per field, excludes the separator colon and closing pipe, and advances past the colon. Heights, widths and offsets are floats. Texture dimensions, crop edges and RGB channels are integers; integer fields use hexadecimal after an initial `0x` or `0X`, and decimal otherwise. Missing optional texture fields leave initialized values unchanged; crop parsing stops when either texture dimension is nonpositive. The parser does not use atlas-native dimensions to fill size fields, and it does not check whether a resource exists. Empty or invalid numeric text and final drawing for unknown resources remain UNKNOWN.

**VERIFIED (R88d, Retail 12.1.5.70688):** A missing texture width copies the resolved height. Nonpositive height and width use the caller metric; a nonpositive width does not have to copy height. An atlas with a nonpositive width uses the caller metric while retaining its resolved height. Positive width normally uses explicit-size scaling; when height selected the caller metric, positive width is multiplied by resolved height. **UNKNOWN (R88d):** The caller metric is not identified as font height or line height.

**VERIFIED (R88e.6, Retail 12.1.5.70688):** The explicit-size calculation uses single-precision arithmetic and has no fixed 0.9957 multiplier. **UNKNOWN (R88e.6):** Its runtime unit count and caller scale are not fixed by the evidence. The simulator does not apply this formula. **INFERRED (Forever 1.60.1.70938):** Exact explicit-size results in the contributor tests retain the supplied measurements and do not establish Retail dimensions.

**VERIFIED (R88e.6, Retail 12.1.5.70688):** Accepted icon width contributes to the checked FontString width-measurement path and to the wrap threshold. A taller accepted icon raises the stored line height. **VERIFIED (R88e.7, Retail 12.1.5.70688):** The FontString path snaps the icon's horizontal edge after multiplying width by drawing scale and adding 0.5 before floor. **UNKNOWN (R88e.7):** Whether the snapped operand is the quad width or the absolute right edge, and whether line advance uses the snapped value. Vertical mode 1 centers an icon against the caller metric; modes 0 and 2 keep different edges fixed. **UNKNOWN (R88e.6/R88e.7):** The default vertical mode, mapping to `JustifyV`, and the screen direction of those edges are not established. Tooltip use of the snap and behavior when an icon alone exceeds the wrap width are also UNKNOWN.

**VERIFIED (R88d, Retail 12.1.5.70688):** Cached `GetStringWidth` and `GetStringHeight` results are divided by the FontString scale. **VERIFIED (R88e.6, Retail 12.1.5.70688):** The checked width-measurement path adds accepted icon width to text advance, and the wrap test uses that width. Tooltip wrapping measures renderer-provided segments and keeps the widest. GameTooltip layout adds the region height of each shown left FontString, with row spacing and attachment overflow where required. The cache-miss height producer and its treatment of inline icon height remain UNKNOWN.

**Simulator policy:** `InlineIcon::size` maps the caller metric to the renderer's `line_height`, **INFERRED** from Forever measurements. The explicit-size formula's runtime unit count and caller scale remain UNKNOWN, so the simulator does not apply that formula. The renderer centers icons in its line box, but this is not proof of the native default vertical mode. The verified FontString snap is not modeled because the evidence does not identify its width-vs-edge operand or effect on line advance; tooltip use is also UNKNOWN. Unknown atlas and file IDs draw no quad in the simulator; their native result remains UNKNOWN.

**INFERRED (Forever 1.60.1.70938):** At font size 12, `X` measures 10.54 × 12.30; explicit-size widths include 11.95 for size 12, 31.86 for size 32 and 109.53 for width 110. `|T…|t` without a height was measured at width 0, but R88d verifies that this token is rejected, not that Retail reports a numeric width. The measured `|A:…:24:0|a` square and all other exact sizes remain Forever observations, not Retail claims.

## FontString smooth scaling

`0624da720` adds independent `Frame.font_string_smooth_scaling`, shared fractional line-height shaping, mode-keyed caches, and ordinary/runtime/inherited XML handling under retail 12.0.5+. `3c8b48012` adds wrapped render/cache measurement assertions. Contract, inferred-policy boundaries, RED provenance, current GREEN status, earlier XML epoch gap, and future native recorder live in [FontString smooth scaling](../../specs/fontstring-smooth-scaling.md). No native/GPU visual proof is claimed.

## Performance

- Quad batch rebuilt only when `quads_dirty` flag is set or screen resized
- Hit-test list cached, invalidated on layout changes
- Glyph atlas uploaded to GPU only when dirty
- Frame time smoothed via EMA (alpha=0.33, ~5-sample window)

## Headless GPU Rendering

The headless path is implemented in `src/render/headless.rs` and uses the same WGPU pipeline without a window: it creates an `Rgba8UnormSrgb` target, prepares atlas bindings, clears, renders, and reads pixels back with `copy_texture_to_buffer()` and `map_async()`.

`render_to_image()` creates a fresh context for one batch. Related before/after images should use `render_batches_to_images(&[&QuadBatch], ...)`, which preloads the union of primary, mask, and glyph textures and renders all batches through one device, pipeline, target, and GPU atlas. That matches the live renderer's persistent atlas; separate calls can repack slots and create bilinear/UV edge differences unrelated to the changed geometry.

## Sources

- [font.rs](../../../src/render/font.rs) — CASC fonts, addon fonts and the pinned fallback family
- [inline_markup.rs](../../../crates/sim-render/src/inline_markup.rs) — inline `|T`/`|A` escape fields, sizes and first-line width
- [inline_emit.rs](../../../src/render/glyph/inline_emit.rs) — inline icon layout and quads
- R88d / R88e.6 / R88e.7 (Retail 12.1.5.70688) — byte-verified native parser, measurement and layout findings
- [TextureUtil.lua](https://github.com/Gethe/wow-ui-source/blob/live/Interface/AddOns/Blizzard_SharedXMLBase/TextureUtil.lua) — `CreateTextureMarkup` and `CreateAtlasMarkup`, the escape formats Blizzard writes
- [FontString smooth scaling](../../specs/fontstring-smooth-scaling.md) — inferred height policy, tests, scope and future native probe

- [rendering-pipeline.md](../../rendering-pipeline.md) — QuadBatch, shaders, atlas, hit testing, alpha, text
- [state_render.rs](../../../src/lua_api/state_render.rs) — per-strata buckets, active top-level show order, nearest-owner grouping, and visibility invalidation
- [state_render_tests.rs](../../../src/lua_api/state_render_tests.rs) — cross-strata top-level grouping, repeated show ordering, and same-level Raise/Lower boundaries
- [world_map_voice_button_order.rs](../../../tests/world_map_voice_button_order.rs) — live-like top-level overlap regression
- [top-level render groups](../../specs/toplevel-render-groups.md) — native controlled matrix and pending integration status
- [ScrollFrame presentation offsets](../../specs/scrollframe-presentation.md) — bounded presentation transform, clipping, and proof status
- [[betterblizzframes-no-portrait-overlay]] — root cause and private native evidence

## See Also

- [[layout-system]] — produces LayoutRect consumed by quad building
- [[widget-system]] — WidgetType dispatch per frame type and ScrollFrame state
- [ScrollFrame presentation offsets](../../specs/scrollframe-presentation.md) — immutable logical geometry with shared render/hit presentation transforms
- [[texture-atlas]] — TextureManager and atlas resolution feeding the GPU atlas
- [[addon-compatibility]] — Docker headless release build contract
