# Anchor/Layout System

## Core Data Structures

### AnchorPoint Enum
**File:** `src/widget/anchor.rs:4-48`

Nine cardinal positions on a widget frame:
```
CENTER, TOP, BOTTOM, LEFT, RIGHT, TOPLEFT, TOPRIGHT, BOTTOMLEFT, BOTTOMRIGHT
```

- **Corner points** (TopLeft, TopRight, BottomLeft, BottomRight): Frame edges
- **Edge points** (Top, Bottom, Left, Right): Edge midpoints
- **Center**: Center of the frame

Maps bidirectionally to/from WoW string format (case-insensitive) via `from_str()` and `as_str()`.

### Anchor Struct
**File:** `src/widget/anchor.rs:50-78`

```rust
pub struct Anchor {
    pub point: AnchorPoint,              // Which point on THIS frame
    pub relative_to: Option<String>,     // Frame name (XML parsing)
    pub relative_to_id: Option<usize>,   // Frame ID (Lua API, takes precedence)
    pub relative_point: AnchorPoint,     // Which point on RELATIVE frame
    pub x_offset: f32,                   // X offset from anchor point
    pub y_offset: f32,                   // Y offset from anchor point
}
```

Both name and ID can be set; ID takes precedence. This supports both XML (name-based) and Lua (ID-based) anchor specification.

### Frame.anchors Field
**File:** `src/widget/frame.rs:122`

```rust
pub anchors: Vec<Anchor>,  // Multiple anchors per frame allowed
```

Multiple anchors on a single frame enable multi-point positioning (e.g., TopLeft AND BottomRight to stretch fill), flexible layout where different anchor points can conflict, and edge constraint resolution.

### LayoutRect Struct
**File:** `src/lua_api/layout.rs:6-12`

```rust
pub struct LayoutRect {
    pub x: f32,      // Left edge (screen coordinates, Y-down origin at top-left)
    pub y: f32,      // Top edge
    pub width: f32,
    pub height: f32,
}
```

**Coordinate system:** LayoutRect uses screen coordinates (origin at top-left, Y increases downward). WoW Lua uses bottom-left origin (Y increases upward). Conversion happens in rect methods like `GetRect()` (`src/lua_api/frame/methods/methods_core.rs:138-146`).

---

## Anchor Resolution: Single vs Multi-Anchor

The layout system branches based on anchor count:

### No Anchors
**File:** `src/iced_app/layout.rs:177-186`

Frame positioned at parent's top-left with explicit size:
```rust
if frame.anchors.is_empty() {
    let w = frame.width * scale;
    let h = frame.height * scale;
    return LayoutRect {
        x: parent_rect.x,
        y: parent_rect.y,
        width: w,
        height: h,
    };
}
```

### Single Anchor (`anchors.len() == 1`)
**File:** `src/iced_app/layout.rs:118-150` (`resolve_single_anchor`)

**Algorithm:**
1. Get relative frame's rectangle (`relative_rect`)
2. Calculate anchor point position on relative frame using `anchor_position()`
3. Apply x_offset/y_offset to get target position
4. Calculate frame's top-left position from anchor point using `frame_position_from_anchor()`
5. Return LayoutRect with explicit width/height (scaled)

**Example:** Frame anchored to parent CENTER with +10, -20 offset:
- Parent CENTER = (500, 300)
- Target position = (510, 280) [Y-down coordinate system]
- If frame is 100x50, TOP-LEFT = (460, 255)

### Multiple Anchors (`anchors.len() >= 2`)
**File:** `src/iced_app/layout.rs:17-116` (`resolve_multi_anchor_edges` -> `compute_rect_from_edges`)

Complex constraint resolution that treats anchors as **edge definitions** rather than simple positioning.

**Phase 1: Resolve anchors to edges** (`resolve_multi_anchor_edges()`, lines 17-60):
- For each anchor, compute its target position
- Map anchor point to constraint:
  - `TopLeft` -> sets left_x AND top_y
  - `TopRight` -> sets right_x AND top_y
  - `Left` -> sets left_x AND center_y (vertical center)
  - `Center` -> sets center_x AND center_y
  - etc.

Result: `AnchorEdges` struct with optional edge positions:
```rust
struct AnchorEdges {
    left_x: Option<f32>,
    right_x: Option<f32>,
    top_y: Option<f32>,
    bottom_y: Option<f32>,
    center_x: Option<f32>,
    center_y: Option<f32>,
}
```

**Phase 2: Resolve edges to rectangle** (`compute_rect_from_edges()`, lines 62-116):

1. **Invert bounds if necessary** (lines 73-84): If `left_x > right_x`, swap them. Same for top/bottom.
2. **Calculate dimensions** (lines 86-96): If both left/right defined: `width = right_x - left_x`. Else use explicit width or 0. Same for height.
3. **Determine position with priority hierarchy** (lines 98-113):
   - **Horizontal:** left -> right -> center -> parent center
   - **Vertical:** top -> bottom -> center -> parent center

Multiple anchors create **constraints** that resolve left-to-right. Opposite-edge anchors (TopLeft + BottomRight) override explicit size.

---

## Helper Functions

### anchor_position()
**File:** `src/iced_app/layout.rs:196-209`

Converts an anchor point enum to coordinates on a rectangle:
```rust
fn anchor_position(point: AnchorPoint, x: f32, y: f32, w: f32, h: f32) -> (f32, f32) {
    match point {
        TopLeft     => (x, y),
        Top         => (x + w/2, y),
        TopRight    => (x + w, y),
        Left        => (x, y + h/2),
        Center      => (x + w/2, y + h/2),
        Right       => (x + w, y + h/2),
        BottomLeft  => (x, y + h),
        Bottom      => (x + w/2, y + h),
        BottomRight => (x + w, y + h),
    }
}
```

### frame_position_from_anchor()
**File:** `src/iced_app/layout.rs:211-230`

Inverse operation: given an anchor point and target position, calculate frame's TOP-LEFT corner.

---

## Lua API: SetPoint, ClearAllPoints, SetAllPoints

**File:** `src/lua_api/frame/methods/methods_anchor.rs`

### SetPoint(point, relativeTo, relativePoint, xOfs, yOfs)
**Lines:** 80-116

Adds or replaces an anchor with a specific point name.

**Argument Parsing** (`parse_set_point_args()`, lines 45-78): Flexible parsing supports multiple call patterns:
- `SetPoint("CENTER")` -> anchor to parent CENTER
- `SetPoint("CENTER", 10, 20)` -> anchor to parent with offset
- `SetPoint("CENTER", otherFrame, "TOPLEFT", 5, -10)` -> anchor to specific frame

**Cycle Detection** (lines 90-94): If a SetPoint call would create a cycle, it returns `Ok(())` (silently rejects). This matches WoW's behavior.

**Replacement Logic** (lines 96-107): If anchor with same `point` already exists, replace it. Otherwise append.

### ClearAllPoints()
**Lines:** 120-130 -- Clears all anchors. Checks if already empty before acquiring mutable borrow.

### ClearPoint(point_name)
**Lines:** 132-142 -- Removes one anchor by point name via `retain()`.

### AdjustPointsOffset(x, y)
**Lines:** 147-159 -- Adds offset to ALL anchor offsets. Used for batch repositioning.

### SetAllPoints(relativeTo)
**Lines:** 162-204 -- Clears existing anchors and sets TopLeft + BottomRight to fill the relative frame.

### GetPoint(index), GetNumPoints(), GetPointByName()
**Lines:** 208-262 -- Query methods. If anchor has no explicit `relative_to_id`, defaults to parent ID.

---

## Cycle Detection

**File:** `src/widget/registry.rs:89-124` (`would_create_anchor_cycle()`)

**Algorithm: BFS from relative_to**
1. Self-check: if frame_id == relative_to_id, return true (can't anchor to self)
2. BFS: Start from relative_to_id, follow all anchor dependencies
3. If we reach frame_id, a cycle exists
4. Visited set prevents revisiting same frame twice

If a SetPoint call would create a cycle, it silently rejects without updating anchors.

---

## Size Calculation from Anchors

**File:** `src/lua_api/frame/methods/methods_helpers.rs:33-97`

### calculate_frame_width() (lines 35-64)
1. If opposite horizontal edges anchored (LEFT + RIGHT on same relative frame): `width = parent_width - left_offset + right_offset`
2. Else: return explicit `frame.width`

### calculate_frame_height() (lines 68-97)
Mirror of width: `height = parent_height + top_offset - bottom_offset`

**Subtraction direction accounts for WoW's Y-up vs screen Y-down.** Both functions recurse to calculate parent dimensions if needed.

---

## Edge Cases & Special Behavior

### Zero-Size Frames
If both edges aren't defined and frame has no explicit size, width/height = 0. Zero-size frames render as invisible but still participate in layout.

### Inverted Bounds (Circular References)
**`compute_rect_from_edges()` lines 73-84:** If anchors create inverted bounds (left > right or top > bottom), bounds are automatically swapped.

### Missing Anchors
- **No anchors:** Frame positioned at parent top-left with explicit size
- **Partial constraints:** Priority hierarchy fills gaps
- **Anchor to non-existent frame:** Layout treats missing relative_frame as parent

### UIParent Special Case
**`src/iced_app/layout.rs:164-167`:** Frame named "UIParent" (or parentless with id=1) always fills the screen.

### Scale Factor
Effective scale is the product of frame's scale and all parent scales (`methods_core.rs:111-123`).

---

## Coordinate System Details

**WoW Game Coordinates (Lua API):**
- Origin: bottom-left of screen
- Y increases upward
- Positions returned as (left, bottom, width, height)

**Screen/Renderer Coordinates:**
- Origin: top-left of screen
- Y increases downward
- LayoutRect uses (x, y, width, height) where y is from top

**Conversion** (`methods_core.rs:144`):
```rust
let bottom = screen_height - rect.y - rect.height;
```

**Y-offset Sign Convention:**
- Positive Y offset in SetPoint -> moves frame UP (in WoW coords)
- In layout computation: `target_y = anchor_y - y_offset` (subtraction accounts for coordinate flip)

---

## Data Flow Summary

1. **Lua API call** (`SetPoint`) -> updates `Frame.anchors` Vec
2. **Render loop calls** `compute_frame_rect()` with frame ID and screen dimensions
3. **Layout branches** on `anchors.len()`:
   - 0 anchors -> position at parent top-left
   - 1 anchor -> use single anchor with explicit size
   - 2+ anchors -> resolve to edge constraints, compute bounds
4. **Result:** LayoutRect with final x, y, width, height (screen coordinates)
5. **Renderer** uses LayoutRect to draw frame quads and child regions
6. **Lua queries** (GetRect, GetWidth, etc.) convert back to WoW coordinates if needed

---

## Tests

**File:** `tests/methods_anchor.rs`

Key test areas: basic anchoring, multi-anchor, replacement, clear operations, SetAllPoints (adds exactly 2 anchors), AdjustPointsOffset, cycle detection, query methods (GetPointByName).
