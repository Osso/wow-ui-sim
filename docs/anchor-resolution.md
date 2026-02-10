# Anchor Resolution System

## Anchor Data Structure (`src/widget/anchor.rs`)

```rust
pub struct Anchor {
    /// The point on this widget to anchor (e.g., CENTER)
    pub point: AnchorPoint,
    /// The widget name to anchor to (used for XML parsing)
    pub relative_to: Option<String>,
    /// The widget ID to anchor to (used for Lua API)
    pub relative_to_id: Option<usize>,
    /// The point on the relative widget to anchor to (e.g., LEFT)
    pub relative_point: AnchorPoint,
    /// X offset from the anchor point
    pub x_offset: f32,
    /// Y offset from the anchor point
    pub y_offset: f32,
}
```

## Anchor Point Types (`src/widget/anchor.rs`)

```rust
pub enum AnchorPoint {
    Center, Top, Bottom, Left, Right,
    TopLeft, TopRight, BottomLeft, BottomRight,
}
```

## Core Resolution Functions (`src/iced_app/layout.rs`)

### Step 1: Get the position of an anchor point on a rectangle

```rust
pub fn anchor_position(point: AnchorPoint, x: f32, y: f32, w: f32, h: f32) -> (f32, f32) {
    match point {
        AnchorPoint::TopLeft => (x, y),
        AnchorPoint::Top => (x + w / 2.0, y),
        AnchorPoint::TopRight => (x + w, y),
        AnchorPoint::Left => (x, y + h / 2.0),
        AnchorPoint::Center => (x + w / 2.0, y + h / 2.0),
        AnchorPoint::Right => (x + w, y + h / 2.0),
        AnchorPoint::BottomLeft => (x, y + h),
        AnchorPoint::Bottom => (x + w / 2.0, y + h),
        AnchorPoint::BottomRight => (x + w, y + h),
    }
}
```

### Step 2: Calculate frame position given an anchor point position

```rust
pub fn frame_position_from_anchor(
    point: AnchorPoint,
    anchor_x: f32,
    anchor_y: f32,
    w: f32,
    h: f32,
) -> (f32, f32) {
    match point {
        AnchorPoint::TopLeft => (anchor_x, anchor_y),
        AnchorPoint::Top => (anchor_x - w / 2.0, anchor_y),
        AnchorPoint::TopRight => (anchor_x - w, anchor_y),
        AnchorPoint::Left => (anchor_x, anchor_y - h / 2.0),
        AnchorPoint::Center => (anchor_x - w / 2.0, anchor_y - h / 2.0),
        AnchorPoint::Right => (anchor_x - w, anchor_y - h / 2.0),
        AnchorPoint::BottomLeft => (anchor_x, anchor_y - h),
        AnchorPoint::Bottom => (anchor_x - w / 2.0, anchor_y - h),
        AnchorPoint::BottomRight => (anchor_x - w, anchor_y - h),
    }
}
```

## Single-Anchor Resolution (`src/iced_app/layout.rs`, lines 135-168)

```rust
fn resolve_single_anchor(
    registry: &WidgetRegistry,
    frame: &crate::widget::Frame,
    parent_rect: LayoutRect,
    eff_scale: f32,
    screen_width: f32,
    screen_height: f32,
    cache: &mut LayoutCache,
) -> LayoutRect {
    let anchor = &frame.anchors[0];
    let width = frame.width * eff_scale;
    let height = frame.height * eff_scale;

    // Get the relative frame's rect (parent or custom frame)
    let relative_rect = if let Some(rel_id) = anchor.relative_to_id {
        compute_frame_rect_cached(registry, rel_id as u64, screen_width, screen_height, cache).rect
    } else {
        parent_rect
    };

    // Step 1: Resolve relative_point on the relative frame
    let (anchor_x, anchor_y) = anchor_position(
        anchor.relative_point,
        relative_rect.x, relative_rect.y,
        relative_rect.width, relative_rect.height,
    );

    // Step 2: Apply offsets (Y is inverted: WoW uses Y-up, screen uses Y-down)
    let target_x = anchor_x + anchor.x_offset * eff_scale;
    let target_y = anchor_y - anchor.y_offset * eff_scale;

    // Step 3: Resolve the child's anchor point around the target position
    let (frame_x, frame_y) =
        frame_position_from_anchor(anchor.point, target_x, target_y, width, height);

    LayoutRect { x: frame_x, y: frame_y, width, height }
}
```

## Example: Spark Texture

For a child texture (Spark) with anchor `CENTER → LEFT` of parent + `(sparkPosition, 0)`:

1. **Resolve `relative_point` (LEFT) on parent frame:**
   ```
   (anchor_x, anchor_y) = (parent_x, parent_y + parent_h / 2.0)
   ```

2. **Apply offset:**
   ```
   target_x = anchor_x + sparkPosition * eff_scale
   target_y = anchor_y - 0.0 * eff_scale
   // = (parent_x + sparkPosition, parent_y + parent_h / 2.0)
   ```

3. **Resolve child's point (CENTER) around target:**
   ```
   (frame_x, frame_y) = (target_x - spark_width/2, target_y - spark_height/2)
   // = (parent_x + sparkPosition - spark_width/2, parent_y + parent_h/2 - spark_height/2)
   ```

## Multi-Anchor Resolution (`src/iced_app/layout.rs`, lines 32-77)

For frames with 2+ anchors, edge constraints are resolved:

```rust
fn resolve_multi_anchor_edges(
    registry: &WidgetRegistry,
    frame: &crate::widget::Frame,
    parent_rect: LayoutRect,
    eff_scale: f32,
    screen_width: f32,
    screen_height: f32,
    cache: &mut LayoutCache,
) -> AnchorEdges {
    // Processes each anchor independently
    for anchor in &frame.anchors {
        let relative_rect = if let Some(rel_id) = anchor.relative_to_id {
            compute_frame_rect_cached(registry, rel_id as u64, ...).rect
        } else {
            parent_rect
        };

        let (anchor_x, anchor_y) = anchor_position(
            anchor.relative_point,
            relative_rect.x, relative_rect.y,
            relative_rect.width, relative_rect.height,
        );
        let target_x = anchor_x + anchor.x_offset * eff_scale;
        let target_y = anchor_y - anchor.y_offset * eff_scale;

        // Map anchor.point to edge constraints
        match anchor.point {
            AnchorPoint::TopLeft => { edges.left_x = Some(target_x); edges.top_y = Some(target_y); }
            AnchorPoint::Center => { edges.center_x = Some(target_x); edges.center_y = Some(target_y); }
            // ... etc for all 9 points
        }
    }
    edges
}
```

## Lua API: SetPoint (`src/lua_api/frame/methods/methods_anchor.rs`)

```rust
// SetPoint(point, relativeTo, relativePoint, xOfs, yOfs)
methods.add_method("SetPoint", |_, this, args: mlua::MultiValue| {
    let point_str = extract_point_str(args.first());
    let point = AnchorPoint::from_str(&point_str).unwrap_or_default();
    let (relative_to, relative_point, x_ofs, y_ofs) = parse_set_point_args(&args, point);

    if let Some(frame) = state.widgets.get_mut(this.id) {
        frame.set_point(point, relative_to, relative_point, x_ofs, y_ofs);
    }
    state.invalidate_layout(this.id);  // Triggers re-layout
    Ok(())
});
```

## Frame Struct Storage (`src/widget/frame.rs`)

```rust
pub struct Frame {
    pub anchors: Vec<Anchor>,  // Multiple anchors allowed
    pub width: f32,
    pub height: f32,
    pub scale: f32,
    pub parent_id: Option<u64>,
    pub anim_offset_x: f32,    // Added after anchor resolution
    pub anim_offset_y: f32,
    pub clamped_to_screen: bool,
    // ... 140+ other fields
}
```

## Key Insights

1. **Coordinate System**: Y-axis is inverted when converting from WoW (Y-up) to screen (Y-down): `target_y = anchor_y - y_offset`
2. **Cached Layout**: `layout.rs` caches computed rects to avoid redundant parent walks for sibling frames
3. **Anchor Chain**: Anchors can reference custom frames via `relative_to_id`, not just parent. This allows textures to anchor to any frame
4. **Frame IDs**: Used internally (`u64`) throughout Rust code; Lua API converts to/from userdata handles
5. **Animation Offset**: Applied AFTER anchor resolution (`anim_offset_x/y`) to support movement without changing anchors

All absolute screen coordinates are computed via `compute_frame_rect_cached()` which is called during quad building in `src/iced_app/frame_collect.rs`.
