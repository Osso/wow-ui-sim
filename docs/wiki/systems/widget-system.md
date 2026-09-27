# Widget System

Every UI element is a `Frame` struct stored in the `WidgetRegistry`. The `WidgetType` enum discriminates 18 widget kinds; creation establishes default children, strata inheritance, and parent-child linkage.

## Frame Struct (~140 fields, `src/widget/frame.rs`)

Key field groups:

**Identity & hierarchy** — `id: u64` (atomic counter), `widget_type`, `name: Option<String>`, `parent_id`, `children: Vec<u64>`, `children_keys: HashMap<String, u64>` (named child refs).

**Rendering order** — `frame_strata`, `frame_level: i32`, `alpha: f32`, `scale: f32`, `draw_layer`, `draw_sub_layer`. The `BLIZZARD` input token is ignored rather than modeled as a drawable strata tier. Retail probe snapshots record effective XML strata and `HasFixedFrameStrata()` before and after selected operations; they do not expose the original XML token or internal resolution mechanism.

**Input** — `mouse_enabled`, `mouse_motion_enabled`, `keyboard_enabled`, `propagate_keyboard_input`, `movable`, `resizable`. `SimState.focused_frame_id` tracks the focused EditBox; Lua `SetFocus`/`ClearFocus` commit that state before their gained/lost callbacks. Repeated/non-owner calls are no-ops, and loss-handler reentry may replace a requested new owner. Changed EditBox `SetText`/`SetFormattedText` commits text and clamps scalar caret/selection endpoints before `OnTextSet` then `OnTextChanged(false)`; same-value assignment is a simulator no-op. At `0c5d5f047`, focused printable, Backspace, and Delete consume a nonempty selection through the same Frame helper as public `Insert`, committing text/caret/render caches before existing callbacks; rejected numeric input preserves selection. At `1933cb68c`, keyboard input validates its proposed replacement against positive scalar/byte limits before mutation; overflow preserves text, caret, selection, and caches without callbacks. This is simulator policy, not native semantics; public `Insert`/`SetText` limits remain unchanged. Tests-only `e114cafe0` is RED in three cases with one zero-default control passing, and independent post-change verification is pending. [EditBox text position and selection](../../specs/editbox-text-position-selection.md) defines this boundary. Public `Insert` callback behavior and keyboard selection creation/navigation remain unmodeled. This dispatch is simulator behavior; existing mouse focus behavior is unchanged.

**FontString fields** — `text`, `font`, `font_size` (default 14.0), `font_outline` (None/Outline/ThickOutline), `text_color` (default gold), `justify_h/v`, `word_wrap`, `max_lines`.

**Texture fields** — `texture`, `color_texture`, `vertex_color`, state textures (`normal_texture`, `pushed_texture`, `highlight_texture`, `disabled_texture`) each with `_tex_coords` companions, `tex_coords`, `atlas_tex_coords`, `blend_mode`, `nine_slice_layout/atlas`, `horiz_tile/vert_tile`, `is_mask`.

**Widget-specific** — Slider, StatusBar, EditBox, ScrollFrame, Cooldown each have dedicated field groups. Model-family state is grouped in one lazy `Option<Box<ModelWidgetState>>`; absent payloads preserve getter defaults, while mutating methods allocate only when needed and remain globally callable.

### Scripted Button click boundary

Public Lua `Button:Click()` first rejects a `ScriptedInput` forbidden aspect. It then toggles a CheckButton before the enabled/same-button recursion guard. A permitted click dispatches every registered `PreClick`, `OnClick`, and `PostClick` binding in order with `(self, mouseButton, down)`; omitted arguments become `LeftButton` and `false`. Handler errors reach the error handler without stopping later bindings or phases, and cleanup releases the recursion guard. Tests-only `8ff05b19a` is RED: six failures; two new and two existing controls pass. Its fixtures cover normal scripts and `HookScript`, not intrinsic-binding order. `/tmp/cross-version-button-click-proof.md` records the bounded evidence; independent post-change verification is pending. Wowless corroborates Button phase order, arguments, and error continuation but has no CheckButton override. Solarity corroborates toggle-before-guard but propagates handler errors. This does not claim native or physical-mouse-click parity. [Scripted Button clicks](../../specs/button-script-click.md) defines the contract.

### ScrollFrame requested-offset boundary

`SetHorizontalScroll` and `SetVerticalScroll` store the requested offset without range clamping. Commit `e9b72b107` preserves existing changed-state callback delivery after storage and same-offset suppression; `ee25b7d62` supplies the regression cases. Cached client observations record round-trips for vertical `-50` and horizontal `999`; no fresh probe ran. RED 0/2 becomes independent GREEN six ScrollFrame cases and 15 shared widget controls, with format/check/readability passing; `/tmp/cross-version-scroll-offset-verification-ledger.md`. That proof excludes renderer movement, covered separately below; implicit range-refresh timing remains unverified. [ScrollFrame offsets](../../specs/scrollframe-offsets.md) is the contract and scope.

### ScrollFrame child-ownership boundary

`07afe68ae` clears the current designation and detaches its old child with shared `reparent_widget` before assigning a different child; nil takes the same clear path. The helper invalidates old-child layout and presentation before replacement assignment, while reassigning the same child avoids detachment. Tests-only `3eea04853` is RED for two ownership failures with one same-child control passing; independent post-change proof is pending. Local Wowless `data/uiobjects/ScrollFrame/SetScrollChild.lua` corroborates only the unparent-before-replace model, not native callbacks or anchor-reset behavior. Unrelated custom Lua `ScrollChild` properties remain unchanged. [ScrollFrame child ownership](../../specs/scrollframe-child-ownership.md) is the bounded contract.

### ScrollFrame presentation boundary

Commits `08c43a536` and `4c39f4a5a` leave stored anchors and logical layout immutable, then apply a shared presentation translation at each ScrollFrame edge whose `scroll_child_id` names the current descendant. The designated subtree moves once per crossed viewport, including externally anchored descendants; nested viewports inherit outer translation but apply their own offset only below their own scroll child. The transform uses the crossed scroll child's effective scale. [ScrollFrame presentation offsets](../../specs/scrollframe-presentation.md) is the bounded render/input contract. `ed69bd136` records three RED cases. Final independent verification passes 52 scoped cases, including scaled-child quads, nested cached hits and both 20,000-frame depth controls; format/check pass. See `/tmp/cross-version-scroll-presentation-final-verification-ledger.md`. Live GPU pixels and all UI/native behavior remain unclaimed.

`b6bb2f710` applies that same translation to public `Region:IsMouseOver` bounds after its logical-rect, visibility, and mouse-enabled guards and before bounds/margin comparison. `GetRect` stays logical and optional margins remain preserved. This adds neither viewport clipping nor intersection semantics to the query; native behavior remains unverified. Independent verification passes the bounded query scope; [ScrollFrame presentation offsets](../../specs/scrollframe-presentation.md) records its 35-case and format/check evidence separately from the 52-case presentation proof. Cached MapCanvas queries its viewport rather than a scrolled child, so it is not evidence of this mismatch.

Ownership when `SetScrollChild` replaces or clears its designation is outside this presentation-only boundary: [ScrollFrame child ownership](../../specs/scrollframe-child-ownership.md) covers the pending detach contract. It makes no native callback or anchor-reset claim.

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

### Public SetParent visibility boundary

After the existing protected-state, forbidden-aspect, and cycle guards, public Lua `SetParent` compares the moved frame's effective visibility before and after hierarchy mutation. When it changed, `676cda72e` recursively dispatches `OnShow` or `OnHide` children first without mutating any frame's local `visible` flag. Locally hidden descendants receive no callback. Reentry validates the expected parent and effective visibility before each callback, so a redirected reparent suppresses stale delivery; an erroring child binding still allows the parent binding to run. `/tmp/cross-version-reparent-final-targeted.log` is GREEN 6/6, including those cases and existing self/descendant cycle controls. Independent verification after `1fcdb52b2` passes 19 scoped cases plus format/check/readability; exact scope is recorded in `/tmp/cross-version-reparent-visibility-proof.md`. Wowless is source corroboration, not a native probe; other parent writers remain outside this public method boundary.

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
- [EditBox text position and selection](../../specs/editbox-text-position-selection.md) — byte offsets, selected edits, and proof limits
- [SetParent parent cycles](../../specs/set-parent-cycles.md) — public cycle-rejection contract and limits
- [Visibility script dispatch](../../specs/visibility-script-dispatch.md) — public reparent visibility callbacks and proof limits
- [ScrollFrame offsets](../../specs/scrollframe-offsets.md) — requested-offset, callback, and range boundary
- [ScrollFrame presentation offsets](../../specs/scrollframe-presentation.md) — presentation-only descendant translation and bounded proof
- [ScrollFrame child ownership](../../specs/scrollframe-child-ownership.md) — designated-child replacement and clearing boundary
- [Scripted Button clicks](../../specs/button-script-click.md) — public click lifecycle and bounded evidence

## See Also

- [[layout-system]] — uses Frame.anchors to compute screen positions
- [[rendering-pipeline]] — dispatches quad emission per WidgetType
- [[event-system]] — Frame.registered_events, script handler storage, and focus callback dispatch
- [ScrollFrame offsets](../../specs/scrollframe-offsets.md) — public offset setter contract and proof status
- [ScrollFrame presentation offsets](../../specs/scrollframe-presentation.md) — presentation-only scroll-child subtree contract
- [ScrollFrame child ownership](../../specs/scrollframe-child-ownership.md) — pending designated-child ownership contract
- [[lua-api]] — Lua method surface
- [EditBox text position and selection](../../specs/editbox-text-position-selection.md) — selected keyboard-edit boundary
- [Scripted Button clicks](../../specs/button-script-click.md) — public Button/CheckButton click lifecycle
