# Hit Testing

How the simulator determines which frame is under the mouse cursor.

## Overview

Hit testing runs on every mouse event (move, click, scroll). It takes a screen-space point and returns the ID of the deepest mouse-enabled frame under that point, or `None`.

## Hittable Frame Collection

During the quad batch build (`render.rs` → `build_quad_batch`), the system collects all frames eligible for hit testing as a side output of `collect_sorted_frames()` in `frame_collect.rs`.

A frame is **hittable** when all three conditions are met:

1. `frame.visible == true`
2. `frame.mouse_enabled == true` (set via `EnableMouse(true)` in Lua or `enableMouse="true"` in XML)
3. Not in `HIT_TEST_EXCLUDED` — a hardcoded list of full-screen non-interactive overlays: `UIParent`, `WorldFrame`, `Minimap`, `ChatFrame1`, `EventToastManagerFrame`, `EditModeManagerFrame`

The hittable list is sorted by `(frame_strata, frame_level, id)` — lowest first. Iterating in **reverse** yields the topmost frame.

## Hittable Rect Cache

The raw hittable list contains `LayoutRect` values in unscaled WoW coordinates. Before caching, `build_hittable_rects()` converts these to screen-space `iced::Rectangle` values by:

1. Applying **hit rect insets** — shrinking the rect inward by the frame's `(left, right, top, bottom)` insets
2. Scaling by `UI_SCALE`

The cached list is stored as `Vec<(u64, Rectangle)>` in `App::cached_hittable` and rebuilt on every quad batch rebuild. It is **not** cleared on layout invalidation to avoid losing hover state between frames.

## Hit Test Algorithm

`hit_test(pos)` in `view.rs` runs two phases:

### Phase 1: Find topmost frame

Iterates the cached hittable list in **reverse** (highest strata/level first). Returns the first frame whose rectangle contains the point.

### Phase 2: Drill down to deepest child

Starting from the initial hit, walks down the widget tree through `frame.children`. For each child (checked in reverse order), if the child is in the hittable set and its rect contains the point, it becomes the new current frame. Repeats until no hittable child contains the point.

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

- `src/iced_app/view.rs` — `hit_test()` implementation
- `src/iced_app/frame_collect.rs` — hittable list collection and `HIT_TEST_EXCLUDED`
- `src/iced_app/render.rs` — `build_hittable_rects()` applies insets and scales
- `src/iced_app/mouse.rs` — mouse event handlers calling `hit_test()`
- `src/widget/frame.rs` — `hit_rect_insets` field
- `src/lua_api/frame/methods/methods_attribute.rs` — `SetHitRectInsets`/`GetHitRectInsets`
