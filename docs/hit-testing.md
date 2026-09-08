# Hit Testing

How the simulator determines which frame is under the mouse cursor.

## Overview

Hit testing runs on every mouse event (move, click, scroll). It takes a screen-space point and returns the ID of the deepest mouse-enabled frame under that point, or `None`.

## Hittable Frame Collection

`frame_collect.rs::collect_hittable_frames()` filters the renderer's flattened strata buckets. Bucket position is the hit-order key; collection does not independently sort raw frame strata or levels.

A frame is **hittable** when all four conditions are met:

1. The frame and its ancestors are shown.
2. The frame has a resolved rectangle and valid render-layout ancestry.
3. Mouse input is enabled, or the widget is an EditBox.
4. Its name is not in `HIT_TEST_EXCLUDED`, the explicit non-interactive overlay list.

Alpha is deliberately ignored: transparent mouse-enabled frames still accept input.

Iterating the list in **reverse** yields the topmost eligible frame. Parent-owned render groups, ordinary frame-level ordering and top-level raising come from the same buckets used to draw the scene. No raw-order fallback is used for frames absent from those buckets.

## Spatial Grid

The ordered hittable list feeds a `HitGrid` (64px cells) stored in `App::cached_hittable`. Each cell stores overlapping frame IDs in render-rank order; a rectangle map supports direct lookup.

Once per input-update batch, current bucket ranks replace cached keys, frames no longer in the order are removed, and changed cell ordering is refreshed. Geometry/visibility changes then update affected subtrees using those same ranks. Unchanged siblings are rekeyed too, since inserting or moving a render group changes their ordinal positions. The grid retains its spatial cells rather than rebuilding for order or layout updates; initial construction and screen-size changes still construct a grid.

### Grid Construction

`build_hittable_rects()` converts raw `LayoutRect` values to screen-space `iced::Rectangle` by:

1. Applying **hit rect insets** — shrinking the rect inward by the frame's `(left, right, top, bottom)` insets
2. Scaling by `UI_SCALE`

`HitGrid::new()` then indexes each frame into every cell its rectangle overlaps.

## Hit Test Algorithm

`hit_test(pos)` in `view.rs` runs two phases:

### Phase 1: Find topmost frame (grid lookup)

Computes which cell contains `pos`, then iterates that cell's frames in **reverse** (highest render rank first). Returns the first frame whose rectangle contains the point. This is O(1) cell lookup + O(k) scan where k is the number of frames in that cell (typically 10–30).

### Phase 2: Drill down to deepest child

Starting from the initial hit, walks down the widget tree through `frame.children`. For each child (checked in reverse order), if the child is in the hittable set (`grid.contains(cid, pos)`) and its rect contains the point, it becomes the new current frame. Repeats until no hittable child contains the point.

This produces the **deepest mouse-enabled descendant**, matching WoW's behavior where child frames receive clicks over parents regardless of frame level.

## Hit Rect Insets

`SetHitRectInsets(left, right, top, bottom)` shrinks a frame's clickable area relative to its visual bounds. Positive values move the edge inward:

```
Visual bounds:  (x, y, width, height)
Hit bounds:     (x+left, y+top, width-left-right, height-top-bottom)
```

Width and height are clamped to zero if insets exceed the frame dimensions.

Insets are stored on the `Frame` struct as `hit_rect_insets: (f32, f32, f32, f32)` and applied during hittable rect cache construction, not during the hit test itself.

### API

- `frame:SetHitRectInsets(left, right, top, bottom)` — set insets
- `frame:GetHitRectInsets()` — returns `left, right, top, bottom`
- XML: `<HitRectInsets left="10" right="10" top="5" bottom="5"/>` — calls `SetHitRectInsets` during frame creation

### Example

A 200x100 frame with `SetHitRectInsets(10, 20, 5, 15)` has an effective clickable area of 170x80, offset 10px from the left and 5px from the top.

## Mouse Event Flow

1. **Mouse move** → `hit_test(pos)` → update `hovered_frame` → fire `OnLeave`/`OnEnter`
2. **Mouse down** → `hit_test(pos)` → fire `OnMouseDown` → track for click/drag
3. **Mouse up** → `hit_test(pos)` → if same frame as mouse down, fire `OnClick` + `PostClick`; otherwise fire `OnMouseUp`
4. **Scroll** → `hit_test(pos)` → walk parent chain for `OnMouseWheel` handler
5. **Middle click** → `hit_test(pos)` → open inspector panel (simulator-only)

Drag detection uses a 5px threshold from the mouse-down position before firing `OnDragStart`.

## Key Files

- `src/iced_app/hit_grid.rs` — `HitGrid` spatial index (grid construction, `topmost_at`, `contains`)
- `src/iced_app/view.rs` — `hit_test()` implementation (Phase 1 + Phase 2)
- `src/iced_app/frame_collect.rs` — hittable list collection, `HIT_TEST_EXCLUDED`, visibility filter
- `src/iced_app/render.rs` — `build_hittable_rects()` applies insets and scales, builds grid
- `src/iced_app/mouse.rs` — mouse event handlers calling `hit_test()`
- `src/widget/frame.rs` — `hit_rect_insets` field
- `src/lua_api/frame/methods/methods_attribute.rs` — `SetHitRectInsets`/`GetHitRectInsets`
