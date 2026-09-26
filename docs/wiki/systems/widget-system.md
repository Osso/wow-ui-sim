# Widget System

Every UI element is a `Frame` struct stored in the `WidgetRegistry`. The `WidgetType` enum discriminates 18 widget kinds; creation establishes default children, strata inheritance, and parent-child linkage.

## Frame Struct (~140 fields, `src/widget/frame.rs`)

Key field groups:

**Identity & hierarchy** — `id: u64` (atomic counter), `widget_type`, `name: Option<String>`, `parent_id`, `children: Vec<u64>`, `children_keys: HashMap<String, u64>` (named child refs).

**Rendering order** — `frame_strata`, `frame_level: i32`, `alpha: f32`, `scale: f32`, `draw_layer`, `draw_sub_layer`. The `BLIZZARD` input token is ignored rather than modeled as a drawable strata tier. Retail probe snapshots record effective XML strata and `HasFixedFrameStrata()` before and after selected operations; they do not expose the original XML token or internal resolution mechanism.

**Input** — `mouse_enabled`, `mouse_motion_enabled`, `keyboard_enabled`, `propagate_keyboard_input`, `movable`, `resizable`. `SimState.focused_frame_id` tracks the focused EditBox; Lua `SetFocus`/`ClearFocus` commit that state before their gained/lost callbacks. Repeated/non-owner calls are no-ops, and loss-handler reentry may replace a requested new owner. For changed EditBox `SetText`/`SetFormattedText`, text commits and scalar caret/selection endpoints clamp before `OnTextSet` then `OnTextChanged(false)`; same-value assignment is a simulator no-op. This dispatch is simulator behavior; existing mouse focus behavior is unchanged.

**FontString fields** — `text`, `font`, `font_size` (default 14.0), `font_outline` (None/Outline/ThickOutline), `text_color` (default gold), `justify_h/v`, `word_wrap`, `max_lines`.

**Texture fields** — `texture`, `color_texture`, `vertex_color`, state textures (`normal_texture`, `pushed_texture`, `highlight_texture`, `disabled_texture`) each with `_tex_coords` companions, `tex_coords`, `atlas_tex_coords`, `blend_mode`, `nine_slice_layout/atlas`, `horiz_tile/vert_tile`, `is_mask`.

**Widget-specific** — Slider, StatusBar, EditBox, ScrollFrame, Cooldown each have dedicated field groups. Model-family state is grouped in one lazy `Option<Box<ModelWidgetState>>`; absent payloads preserve getter defaults, while mutating methods allocate only when needed and remain globally callable.

## WidgetType Enum (18 types, `src/widget/mod.rs`)

Frame, Button, FontString, Texture, EditBox, ScrollFrame, Slider, CheckButton, StatusBar, Cooldown, Model (stub), ModelScene (stub), PlayerModel, ColorSelect, MessageFrame, SimpleHTML (stub), GameTooltip, Minimap.

`from_str()` maps WoW aliases: "ItemButton" → Button, "ScrollingMessageFrame" → MessageFrame.

## WidgetRegistry (`src/widget/registry.rs`)

```rust
pub struct WidgetRegistry {
    widgets: HashMap<u64, Frame>,
    names: HashMap<String, u64>,
    render_dirty: Cell<bool>,
}
```

`get_mut()` sets `render_dirty`. `take_render_dirty()` atomically checks and clears. `would_create_anchor_cycle()` is a BFS reachability check used by `SetPoint`.

### Public parent-cycle boundary

Lua `SetParent` keeps its protected-state and forbidden-aspect guards before it calls the shared `methods_hierarchy::would_create_parent_cycle` helper. That helper walks the proposed parent's `parent_id` ancestry and rejects a self or descendant edge before animation reparenting or `reparent_widget` mutates the registry. Commit `818fe8d59` has RED evidence for both invalid graphs and GREEN evidence for those two rejections plus ordinary reparent, nil-parent, same-parent, and region-enumeration controls (4/4). The helper assumes an acyclic existing parent chain; it neither validates every malformed model graph nor covers other parent writers. The simulator error is tested only as containing `cycle`, not as native WoW wording.

### Storage accounting

`Frame::storage_estimate_bytes()` accounts for inline state, registry/index capacities, and owned collection data. The lazy model payload contributes its boxed allocation and owned capacities only when present. The settled full-game fixture has 45,002 frames and estimates **213,058,036 bytes**, below the unchanged **230,000,000-byte** budget; the prior inline model state estimated **239,185,088 bytes**.

## Default Children (`src/lua_api/globals/create_frame.rs`)

**Button/CheckButton** — 4 Textures (NormalTexture Artwork, PushedTexture Artwork, HighlightTexture Highlight additive, DisabledTexture Artwork) + 1 FontString (Text at Overlay). All textures fill the button via TOPLEFT+BOTTOMRIGHT anchors. `mouse_enabled = true`.

**Slider** — FontStrings: Low, High, Text; Texture: ThumbTexture.

**ItemButton** — 6 Textures (icon, searchOverlay, IconBorder, IconOverlay, IconOverlay2, ItemContextOverlay) + 2 FontStrings (Count, Stock).

**GameTooltip** — Inserts tooltip data into `SimState.tooltips`; sets strata to TOOLTIP.

## Strata Before/After Observations

- The retail capture recorded `HasFixedFrameStrata() == false` for the tested XML strata literals, including `PARENT`.
- During XML `OnLoad`, a direct `PARENT` child and its actual parent both reported `DIALOG`; an explicit literal sibling reported `LOW`. That equality does not identify why values match.
- Under an actual `DIALOG` parent, the base `HIGH` instance reported `HIGH`, the derived literal `LOW` instance reported `LOW`, and the derived `PARENT` instance reported `HIGH`.
- After parent `SetFrameStrata("LOW")` and, separately, reparenting to a `LOW` parent, every tested non-fixed direct child and grandchild reported `LOW`, including explicit XML `MEDIUM` fixtures. These snapshots do not establish an internal propagation mechanism.
- Runtime-fixed retail behavior using `SetFixedFrameStrata(true)` remains unproven.

## Button Text Rendering and Three-Slice Issue

Button text is emitted twice: once by `build_button_quads()` as part of the non-region frame (renders before all child regions), and once by the child `Text` FontString at Overlay draw layer (renders after Artwork textures).

Three-slice buttons (ThreeSliceButtonTemplate) define their background as child Textures at BACKGROUND draw layer. These render after the button frame but before the child FontString — meaning the step-1 text gets covered. The child FontString should fix this, but its default CENTER anchor gives it zero width, so it renders nothing.

**Fix**: replace the child Text FontString's single CENTER anchor with fill-parent anchors (`add_fill_parent_anchors()`), so `resolve_multi_anchor_edges` computes proper width from the parent button bounds.

## Visibility

`Show()` fires `OnShow` recursively on visible children. `IsVisible()` walks the parent chain; `IsShown()` checks only the frame's own `visible` flag.

## Sources

- [FrameStrataProbe](../../../docs/addons/FrameStrataProbe/README.md) — retail XML `PARENT`, template comparison, and before/after operation observations
- [widget-system.md](../../widget-system.md) — Frame struct, lazy model payload, WidgetType, WidgetRegistry, default children, strata
- [frame.rs](../../../src/widget/frame.rs) — Frame storage and model-state accessors
- [frame_size.rs](../../../src/widget/frame_size.rs) — registry storage estimate and boxed payload accounting
- [button-text-rendering.md](../../button-text-rendering.md) — three-slice rendering order problem and fix
- [EditBox focus callbacks](../../specs/editbox-focus-callbacks.md) — Lua focus-transition contract and limits
- [SetParent parent cycles](../../specs/set-parent-cycles.md) — public cycle-rejection contract and limits

## See Also

- [[layout-system]] — uses Frame.anchors to compute screen positions
- [[rendering-pipeline]] — dispatches quad emission per WidgetType
- [[event-system]] — Frame.registered_events, script handler storage, and focus callback dispatch
- [[lua-api]] — public `SetParent` dispatch boundary
