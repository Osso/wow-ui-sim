# Layout System

The layout system computes screen-space rectangles for every frame by resolving anchor constraints against parent and sibling frames. It operates in screen coordinates (top-left origin, Y-down) and converts to WoW coordinates (bottom-left origin, Y-up) only at the Lua API boundary.

## AnchorPoint Enum

Nine positions on a frame rectangle (`src/widget/anchor.rs`):

```
CENTER  TOP  BOTTOM  LEFT  RIGHT  TOPLEFT  TOPRIGHT  BOTTOMLEFT  BOTTOMRIGHT
```

Corner points address frame edges; edge midpoints address side centers; Center addresses the geometric center. All map case-insensitively to/from WoW string format via `from_str()` / `as_str()`.

## Anchor Struct

```rust
pub struct Anchor {
    pub point: AnchorPoint,           // Point on THIS frame
    pub relative_to: Option<String>,  // Frame name (XML resolution)
    pub relative_to_id: Option<usize>,// Frame ID (Lua API, takes precedence)
    pub relative_point: AnchorPoint,  // Point on RELATIVE frame
    pub x_offset: f32,
    pub y_offset: f32,
}
```

ID takes precedence over name when both are present. A frame stores `Vec<Anchor>`, allowing multi-point constraints.

## Resolution Branches

Layout branches on `anchors.len()` (`src/iced_app/layout.rs`):

**0 anchors** — positioned at parent top-left with explicit `width * scale` / `height * scale`.

**1 anchor** (`resolve_single_anchor`) — resolves `relative_point` on the relative frame using `anchor_position()`, applies `(x_offset, -y_offset)` (Y sign flips coordinate system), then calls `frame_position_from_anchor()` to compute the frame's top-left. Width/height come from the frame's explicit fields.

**2+ anchors** (`resolve_multi_anchor_edges`) — each anchor maps to named edges in an `AnchorEdges` struct (`left_x`, `right_x`, `top_y`, `bottom_y`, `center_x`, `center_y`). `compute_rect_from_edges()` then derives position and dimensions: if both opposite edges are set, size equals their difference and explicit size is ignored. Position priority: left > right > center > parent center (horizontal); top > bottom > center > parent center (vertical). Inverted bounds (left > right) are swapped automatically.

## SetPoint API

`SetPoint(point, relativeTo, relativePoint, xOfs, yOfs)` (`src/lua_api/frame/methods/methods_anchor.rs`) supports flexible arg forms: `SetPoint("CENTER")`, `SetPoint("CENTER", 10, 20)`, or the full five-argument form. If an anchor with the same `point` exists it is replaced; otherwise appended. Cycle detection (`would_create_anchor_cycle()` in `registry.rs`) uses BFS and silently rejects cycles, matching WoW behavior.

`SetAllPoints(relativeTo)` sets TOPLEFT+BOTTOMRIGHT anchors to fill the target frame, clearing previous anchors.

## Display Metrics

`SimState.screen_width` and `screen_height` are literal base-canvas dimensions used by layout, before `UIParent` scale. `physical_screen_width` and `physical_screen_height` are separate physical display pixels returned by `GetPhysicalScreenSize`.

`set_screen_size(width, height)` deliberately sets both pairs one-to-one. `set_display_size(width, height)` retains physical pixels and derives the base canvas from Blizzard PixelUtil's 768-unit reference height. Frame effective scale remains independent of both inputs.

## Coordinate System

| Context | Origin | Y direction |
|---------|--------|-------------|
| LayoutRect / renderer | top-left | down |
| Lua API (GetRect, GetLeft, etc.) | bottom-left | up |

Conversion at `methods_core.rs:144`: `bottom = screen_height - rect.y - rect.height`. Y-offset sign convention: positive Y in `SetPoint` moves frame UP, which means `target_y = anchor_y - y_offset` in layout computation.

Special case: `UIParent` (id=1 or name="UIParent") always fills the base canvas.

## Sources

- [layout-system.md](../../layout-system.md) — anchor data structures, resolution algorithms, Lua API
- [anchor-resolution.md](../../anchor-resolution.md) — resolution functions with code examples
- [display-metrics.md](../../specs/display-metrics.md) — physical-display and base-canvas contract

## See Also

- [[widget-system]] — Frame struct that stores anchors and sizes
- [[lua-api]] — SetPoint, ClearAllPoints, GetRect method implementations
- [[rendering-pipeline]] — consumes LayoutRect to emit quads
- [[frame-position-baseline-drift]] — causal replay needs both display metrics and EditMode inputs
