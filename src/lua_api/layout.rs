//! Layout computation helpers for frame positioning.

use crate::widget::{AnchorPoint, WidgetRegistry};

/// Simple layout rect for frame positioning.
#[derive(Debug, Default, Clone, Copy)]
pub struct LayoutRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// Get depth in parent hierarchy (for indentation).
pub fn get_parent_depth(registry: &WidgetRegistry, id: u64) -> usize {
    let mut depth = 0;
    let mut current = id;
    while let Some(frame) = registry.get(current) {
        if let Some(parent_id) = frame.parent_id {
            depth += 1;
            current = parent_id;
        } else {
            break;
        }
    }
    depth
}

/// Get the parent rect, falling back to the screen rect if no parent.
fn parent_rect(
    registry: &WidgetRegistry,
    parent_id: Option<u64>,
    screen_width: f32,
    screen_height: f32,
) -> LayoutRect {
    match parent_id {
        Some(pid) => compute_frame_rect(registry, pid, screen_width, screen_height),
        None => LayoutRect {
            x: 0.0,
            y: 0.0,
            width: screen_width,
            height: screen_height,
        },
    }
}

/// Compute frame rect for debugging (same algorithm as renderer).
pub fn compute_frame_rect(
    registry: &WidgetRegistry,
    id: u64,
    screen_width: f32,
    screen_height: f32,
) -> LayoutRect {
    let frame = match registry.get(id) {
        Some(f) => f,
        None => return LayoutRect::default(),
    };

    let width = frame.width;
    let height = frame.height;
    let pr = parent_rect(registry, frame.parent_id, screen_width, screen_height);

    if frame.anchors.is_empty() {
        return LayoutRect {
            x: pr.x,
            y: pr.y,
            width,
            height,
        };
    }

    let anchor = &frame.anchors[0];
    let (pax, pay) = anchor_position(anchor.relative_point, pr.x, pr.y, pr.width, pr.height);
    let target_x = pax + anchor.x_offset;
    // WoW uses Y-up coordinate system, screen uses Y-down
    let target_y = pay - anchor.y_offset;
    let (frame_x, frame_y) =
        frame_position_from_anchor(anchor.point, target_x, target_y, width, height);

    LayoutRect {
        x: frame_x,
        y: frame_y,
        width,
        height,
    }
}

/// Get the position of an anchor point on a rect.
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

/// Get frame position given an anchor point position.
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
