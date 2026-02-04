# UI Scaling and Coordinate Systems

## WoW Coordinate System (from wowless)

WoW uses **bottom-left origin with Y increasing upward**:
```lua
local screen = {
  bottom = 0,
  left = 0,
  right = screenWidth,
  top = screenHeight,
}
```

- `(0, 0)` = bottom-left corner
- `(screenWidth, screenHeight)` = top-right corner
- Default anchor point: `TOPLEFT`
- Wowless default screen size: `1280x720`

## Current Implementation

### Screen Size
- Layout uses **canvas size** (dynamic, matches widget bounds)
- WoW coords map 1:1 to canvas pixels
- No fixed screen size - adapts to window

### Lua API
- `GetScreenWidth()` returns `1280.0`
- `GetScreenHeight()` returns `720.0`
- TODO: Should return actual canvas size dynamically

### UI_SCALE
- Defined in `src/render/texture.rs` as `1.0`
- Multiplies WoW coords to get display coords
- With UI_SCALE=1.0, no scaling applied

## Projection Matrix

The shader projection in `pipeline.rs` uses canvas bounds for the orthographic projection:
- Maps (0,0)-(width,height) to clip space (-1,-1)-(1,1)
- Viewport must match canvas bounds for correct coordinate mapping

## Key Files

- `src/iced_app.rs`: Layout calculation uses canvas `size` parameter
- `src/render/shader/pipeline.rs`: Projection matrix setup
- `src/render/texture.rs`: UI_SCALE constant
- `src/lua_api/globals.rs`: GetScreenWidth/GetScreenHeight

## Issues Fixed

1. **Hardcoded anchor override** - `main.rs` was setting `TOPLEFT (10, -10)` instead of using XML's `CENTER` anchor
2. **Screen size mismatch** - Internal screen_size was hardcoded, now uses canvas size

## TODO

- [ ] Make GetScreenWidth/GetScreenHeight return actual canvas size
- [ ] Verify Y-axis direction (WoW uses Y-up, GUI frameworks use Y-down)
- [ ] Test CENTER anchor with dynamic canvas size
- [ ] Remove debug purple border when done
