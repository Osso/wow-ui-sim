//! Layout computation helpers for WoW frame positioning.

use crate::widget::{AnchorPoint, WidgetRegistry};
use crate::LayoutRect;

/// Compute frame rect with anchor resolution.
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

    // Special case: UIParent (id=1) fills the entire screen
    if frame.name.as_deref() == Some("UIParent") || (frame.parent_id.is_none() && id == 1) {
        return LayoutRect {
            x: 0.0,
            y: 0.0,
            width: screen_width,
            height: screen_height,
        };
    }

    let parent_rect = if let Some(parent_id) = frame.parent_id {
        compute_frame_rect(registry, parent_id, screen_width, screen_height)
    } else {
        LayoutRect {
            x: 0.0,
            y: 0.0,
            width: screen_width,
            height: screen_height,
        }
    };

    let scale = frame.scale;

    if frame.anchors.is_empty() {
        let w = frame.width * scale;
        let h = frame.height * scale;
        return LayoutRect {
            x: parent_rect.x + (parent_rect.width - w) / 2.0,
            y: parent_rect.y + (parent_rect.height - h) / 2.0,
            width: w,
            height: h,
        };
    }

    if frame.anchors.len() >= 2 {
        let mut left_x: Option<f32> = None;
        let mut right_x: Option<f32> = None;
        let mut top_y: Option<f32> = None;
        let mut bottom_y: Option<f32> = None;
        // Center position from TOP/BOTTOM/CENTER anchors with x offset
        let mut center_x: Option<f32> = None;
        // Center position from LEFT/RIGHT/CENTER anchors with y offset
        let mut center_y: Option<f32> = None;

        for anchor in &frame.anchors {
            let relative_rect = if let Some(rel_id) = anchor.relative_to_id {
                compute_frame_rect(registry, rel_id as u64, screen_width, screen_height)
            } else {
                parent_rect
            };

            let (anchor_x, anchor_y) = anchor_position(
                anchor.relative_point,
                relative_rect.x,
                relative_rect.y,
                relative_rect.width,
                relative_rect.height,
            );
            let target_x = anchor_x + anchor.x_offset;
            let target_y = anchor_y - anchor.y_offset;

            match anchor.point {
                AnchorPoint::TopLeft => {
                    left_x = Some(target_x);
                    top_y = Some(target_y);
                }
                AnchorPoint::TopRight => {
                    right_x = Some(target_x);
                    top_y = Some(target_y);
                }
                AnchorPoint::BottomLeft => {
                    left_x = Some(target_x);
                    bottom_y = Some(target_y);
                }
                AnchorPoint::BottomRight => {
                    right_x = Some(target_x);
                    bottom_y = Some(target_y);
                }
                AnchorPoint::Top => {
                    top_y = Some(target_y);
                    center_x = Some(target_x); // TOP anchor x offset sets horizontal center
                }
                AnchorPoint::Bottom => {
                    bottom_y = Some(target_y);
                    center_x = Some(target_x); // BOTTOM anchor x offset sets horizontal center
                }
                AnchorPoint::Left => {
                    left_x = Some(target_x);
                    center_y = Some(target_y); // LEFT anchor y offset sets vertical center
                }
                AnchorPoint::Right => {
                    right_x = Some(target_x);
                    center_y = Some(target_y); // RIGHT anchor y offset sets vertical center
                }
                AnchorPoint::Center => {
                    center_x = Some(target_x);
                    center_y = Some(target_y);
                }
            }
        }

        // WoW behavior: anchors defining opposite edges override explicit size.
        // Explicit size is only a fallback when the opposite edge anchor is missing.
        // When anchors create inverted bounds (e.g., TOPLEFT→TOPRIGHT, TOPRIGHT→TOPLEFT),
        // WoW swaps them to get positive dimensions. This is used by ButtonFrameTemplate_HidePortrait.
        let (final_left_x, final_right_x) = if let (Some(lx), Some(rx)) = (left_x, right_x) {
            if lx > rx {
                (Some(rx), Some(lx)) // Swap inverted bounds
            } else {
                (Some(lx), Some(rx))
            }
        } else {
            (left_x, right_x)
        };

        let (final_top_y, final_bottom_y) = if let (Some(ty), Some(by)) = (top_y, bottom_y) {
            if ty > by {
                (Some(by), Some(ty)) // Swap inverted bounds
            } else {
                (Some(ty), Some(by))
            }
        } else {
            (top_y, bottom_y)
        };

        let final_width = if let (Some(lx), Some(rx)) = (final_left_x, final_right_x) {
            // Both left and right edges defined by anchors - compute width from them
            rx - lx
        } else if frame.width > 0.0 {
            frame.width * scale
        } else {
            0.0
        };

        let final_height = if let (Some(ty), Some(by)) = (final_top_y, final_bottom_y) {
            // Both top and bottom edges defined by anchors - compute height from them
            by - ty
        } else if frame.height > 0.0 {
            frame.height * scale
        } else {
            0.0
        };

        // Horizontal position priority: left_x > right_x > center_x > parent center
        let final_x = final_left_x.unwrap_or_else(|| {
            final_right_x.map(|rx| rx - final_width).unwrap_or_else(|| {
                // Use center_x if set by TOP/BOTTOM/CENTER anchor with x offset
                center_x
                    .map(|cx| cx - final_width / 2.0)
                    .unwrap_or_else(|| parent_rect.x + (parent_rect.width - final_width) / 2.0)
            })
        });
        // Vertical position priority: top_y > bottom_y > center_y > parent center
        let final_y = final_top_y.unwrap_or_else(|| {
            final_bottom_y.map(|by| by - final_height).unwrap_or_else(|| {
                // Use center_y if set by LEFT/RIGHT/CENTER anchor with y offset
                center_y
                    .map(|cy| cy - final_height / 2.0)
                    .unwrap_or_else(|| parent_rect.y + (parent_rect.height - final_height) / 2.0)
            })
        });

        return LayoutRect {
            x: final_x,
            y: final_y,
            width: final_width,
            height: final_height,
        };
    }

    let anchor = &frame.anchors[0];
    let width = frame.width * scale;
    let height = frame.height * scale;

    // For single anchor, check if it has a specific relativeTo frame
    let relative_rect = if let Some(rel_id) = anchor.relative_to_id {
        compute_frame_rect(registry, rel_id as u64, screen_width, screen_height)
    } else {
        parent_rect
    };

    let (anchor_x, anchor_y) = anchor_position(
        anchor.relative_point,
        relative_rect.x,
        relative_rect.y,
        relative_rect.width,
        relative_rect.height,
    );

    let target_x = anchor_x + anchor.x_offset;
    let target_y = anchor_y - anchor.y_offset;

    let (frame_x, frame_y) =
        frame_position_from_anchor(anchor.point, target_x, target_y, width, height);

    LayoutRect {
        x: frame_x,
        y: frame_y,
        width,
        height,
    }
}

/// Get the position of an anchor point on a rectangle.
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

/// Calculate frame position given its anchor point and target position.
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
