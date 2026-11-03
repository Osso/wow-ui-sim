# Original mouse sizing capture

Original Retail DamageMeter resize callbacks exposed simulator motion-capture and release-routing defects. The bounded contract lives in [original mouse gestures](../../specs/original-mouse-gestures.md); native outside-release ordering and scaled-resize policy remain unproved.

## Root cause

`StartSizing` set native sizing flags without selecting the active motion frame. Subsequent physical movement therefore had no frame to resize. Separately, crossing the drag threshold marked a script drag even when no registered OnDragStart existed, redirecting release away from the original OnMouseUp cleanup.

`StartSizing` now captures the resized frame. `StopMovingOrSizing` releases matching capture. Threshold dispatch marks a script drag only when a registered handler starts; an ordinary sizing gesture retains ordinary mouse-up routing. Blizzard callbacks are unchanged.

## Proof boundary

The [owning proof ledger](../../specs/original-mouse-gestures.md#tests-and-proof) records Retail geometry/release cases, sizing controls, Wrath original XML/OnUpdate coverage and mutation controls. Wrath exposed the same missing capture through a sibling tab/frame pair: ancestor-only lookup could not find the `StartMoving` target. `StartMoving` now captures its actual target.

## Release outside the game canvas

In ordinary display mode, the GUI discarded button releases when the pointer
had no position inside the game canvas. The canvas could lose hover while the
game still recorded a held button. The AddOns menu scrollbar thumb, for example,
installs a cursor-driven `OnUpdate` on press and removes it on release.

Ordinary and fixed-resolution modes now route left and right releases outside
the canvas through the same cleanup. The frame that received the press gets
`OnMouseUp(button, false)`. Click and drop callbacks still require an eligible
release target. A release inside the canvas reports `upInside` from the press
recipient's hit rectangle and ancestor clipping.

## Sources

- [Gesture contract](../../specs/original-mouse-gestures.md) — acceptance and exclusions.
- [Mouse dispatch](../../../src/iced_app/mouse.rs) — physical threshold and release routing.
- [Canvas mouse events](../../../src/iced_app/render_mouse_events.rs) — outside release routing.
- [Native motion methods](../../../src/lua_api/frame/methods/misc/drag_input.rs) — sizing capture and stop state.
- [Original-consumer tests](../../../tests/f2_original_mouse_consumers.rs) — shared Retail/Wrath module.

## See Also

- [[widget-system]] — native widget state.
- [[event-system]] — original script dispatch.
- [[hit-testing]] — hit rectangles and input targets.
