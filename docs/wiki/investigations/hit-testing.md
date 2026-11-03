# Hit Testing

How the simulator determines which frame is under the mouse cursor.

## Algorithm

Hit testing runs on every mouse event (move, click, scroll) in two phases:

**Phase 1 — Grid lookup.** A `HitGrid` indexes the hittable frames in render order. The selector finds the topmost candidate at the pointer for the current input event.

**Phase 2 — Drill down.** The selector descends through visible children, using scrolled bounds and clipping. The child must qualify for the current input event to become the target.

## Hittable Frame Conditions

The grid includes shown frames with resolved bounds that accept mouse clicks, mouse motion, or wheel input. EditBoxes also qualify. The explicit overlay exclusion list still applies.

## Hit Rect Insets

`SetHitRectInsets(left, right, top, bottom)` shrinks the clickable area inward. Applied during grid construction, not during hit testing. Stored as `frame.hit_rect_insets: (f32, f32, f32, f32)`.

## Mouse Event Flow

| Event | Action |
|---|---|
| Move | `hit_test` → update `hovered_frame` → fire `OnLeave`/`OnEnter` |
| Down | `hit_test` → fire `OnMouseDown`, track for click/drag |
| Up | Use the release hit to decide clicks and drops. |
| Scroll | Select a target with the wheel hit test and walk its parents for an eligible `OnMouseWheel` handler. |
| Middle click | `hit_test` → open inspector panel (simulator-only) |

Drag: 5px threshold from mouse-down position before firing `OnDragStart`.

## Forever scroll controls

RestedXP's legacy guide inherits its wheel handler from Blizzard XML and uses a native Slider for the scrollbar. XML wheel handlers enable wheel input by default, with `autoEnableInput="false"` as the opt-out. Native Slider motion updates the value and dispatches `OnValueChanged`, which the Blizzard template uses to scroll the guide.

EllesmereUI's settings viewport enables only wheel input. The hover lookup could reach it through a button and then skip it when the button scrolled out from under the pointer. Wheel targeting now includes that viewport directly.

The wheel regression holds the pointer stationary as a row scrolls away and checks that later wheel events continue moving the viewport.

## Key Files

- `src/iced_app/hit_grid.rs` — `HitGrid` spatial index
- `src/iced_app/view/hit_testing.rs` — hover, button, and wheel selectors
- `src/iced_app/frame_collect.rs` — hittable list collection and `HIT_TEST_EXCLUDED`
- `src/iced_app/render.rs` — `build_hittable_rects()` (applies insets, builds grid)
- `src/iced_app/mouse.rs` — mouse event handlers

## Sources

- [hit-testing.md](../../hit-testing.md) — full system description
- [Mouse dispatch](../../../src/iced_app/mouse.rs) — wheel delivery
- [Native Slider methods](../../../src/lua_api/frame/methods/widgets/slider.rs) — value and drag callbacks

## See Also

- [[on-update-dirty]] — rendering pipeline context
