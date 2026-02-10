//! Layout computation helpers for WoW frame positioning.

use crate::widget::{AnchorPoint, WidgetRegistry};
use crate::LayoutRect;

/// Resolved edge constraints from multiple anchors.
struct AnchorEdges {
    left_x: Option<f32>,
    right_x: Option<f32>,
    top_y: Option<f32>,
    bottom_y: Option<f32>,
    center_x: Option<f32>,
    center_y: Option<f32>,
}


/// Resolve each anchor in a multi-anchor frame to edge constraints.
fn resolve_multi_anchor_edges(
    registry: &WidgetRegistry,
    frame: &crate::widget::Frame,
    parent_rect: LayoutRect,
    eff_scale: f32,
    screen_width: f32,
    screen_height: f32,
) -> AnchorEdges {
    let mut edges = AnchorEdges {
        left_x: None, right_x: None,
        top_y: None, bottom_y: None,
        center_x: None, center_y: None,
    };

    for anchor in &frame.anchors {
        let relative_rect = if let Some(rel_id) = anchor.relative_to_id {
            compute_frame_rect(registry, rel_id as u64, screen_width, screen_height)
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

        match anchor.point {
            AnchorPoint::TopLeft     => { edges.left_x = Some(target_x); edges.top_y = Some(target_y); }
            AnchorPoint::TopRight    => { edges.right_x = Some(target_x); edges.top_y = Some(target_y); }
            AnchorPoint::BottomLeft  => { edges.left_x = Some(target_x); edges.bottom_y = Some(target_y); }
            AnchorPoint::BottomRight => { edges.right_x = Some(target_x); edges.bottom_y = Some(target_y); }
            AnchorPoint::Top         => { edges.top_y = Some(target_y); edges.center_x = Some(target_x); }
            AnchorPoint::Bottom      => { edges.bottom_y = Some(target_y); edges.center_x = Some(target_x); }
            AnchorPoint::Left        => { edges.left_x = Some(target_x); edges.center_y = Some(target_y); }
            AnchorPoint::Right       => { edges.right_x = Some(target_x); edges.center_y = Some(target_y); }
            AnchorPoint::Center      => { edges.center_x = Some(target_x); edges.center_y = Some(target_y); }
        }
    }

    edges
}

/// Compute final rect from resolved edge constraints and frame size.
///
/// WoW behavior: anchors defining opposite edges override explicit size.
/// When anchors create inverted bounds, WoW swaps them to get positive dimensions.
fn compute_rect_from_edges(
    edges: AnchorEdges,
    frame: &crate::widget::Frame,
    parent_rect: LayoutRect,
    scale: f32,
) -> LayoutRect {
    // Swap inverted horizontal bounds
    let (left_x, right_x) = if let (Some(lx), Some(rx)) = (edges.left_x, edges.right_x) {
        if lx > rx { (Some(rx), Some(lx)) } else { (Some(lx), Some(rx)) }
    } else {
        (edges.left_x, edges.right_x)
    };

    // Swap inverted vertical bounds
    let (top_y, bottom_y) = if let (Some(ty), Some(by)) = (edges.top_y, edges.bottom_y) {
        if ty > by { (Some(by), Some(ty)) } else { (Some(ty), Some(by)) }
    } else {
        (edges.top_y, edges.bottom_y)
    };

    let width = match (left_x, right_x) {
        (Some(lx), Some(rx)) => rx - lx,
        _ if frame.width > 0.0 => frame.width * scale,
        _ => 0.0,
    };

    let height = match (top_y, bottom_y) {
        (Some(ty), Some(by)) => by - ty,
        _ if frame.height > 0.0 => frame.height * scale,
        _ => 0.0,
    };

    // Horizontal position priority: left > right > center > parent center
    let x = left_x.unwrap_or_else(|| {
        right_x.map(|rx| rx - width).unwrap_or_else(|| {
            edges.center_x
                .map(|cx| cx - width / 2.0)
                .unwrap_or_else(|| parent_rect.x + (parent_rect.width - width) / 2.0)
        })
    });
    // Vertical position priority: top > bottom > center > parent center
    let y = top_y.unwrap_or_else(|| {
        bottom_y.map(|by| by - height).unwrap_or_else(|| {
            edges.center_y
                .map(|cy| cy - height / 2.0)
                .unwrap_or_else(|| parent_rect.y + (parent_rect.height - height) / 2.0)
        })
    });

    LayoutRect { x, y, width, height }
}

/// Resolve a single-anchor frame's position.
fn resolve_single_anchor(
    registry: &WidgetRegistry,
    frame: &crate::widget::Frame,
    parent_rect: LayoutRect,
    eff_scale: f32,
    screen_width: f32,
    screen_height: f32,
) -> LayoutRect {
    let anchor = &frame.anchors[0];
    let width = frame.width * eff_scale;
    let height = frame.height * eff_scale;

    let relative_rect = if let Some(rel_id) = anchor.relative_to_id {
        compute_frame_rect(registry, rel_id as u64, screen_width, screen_height)
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

    let (frame_x, frame_y) =
        frame_position_from_anchor(anchor.point, target_x, target_y, width, height);

    LayoutRect { x: frame_x, y: frame_y, width, height }
}

/// Compute effective scale: product of all ancestor scales including this frame.
///
/// In WoW, `GetEffectiveScale()` returns the product of the frame's own scale
/// and all ancestor scales. The layout engine uses this to convert local-space
/// dimensions and anchor offsets to screen-space pixels.
fn effective_scale(registry: &WidgetRegistry, id: u64) -> f32 {
    let mut scale = 1.0;
    let mut current = Some(id);
    while let Some(cid) = current {
        if let Some(f) = registry.get(cid) {
            scale *= f.scale;
            current = f.parent_id;
        } else {
            break;
        }
    }
    scale
}

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
        return LayoutRect { x: 0.0, y: 0.0, width: screen_width, height: screen_height };
    }

    let parent_rect = if let Some(parent_id) = frame.parent_id {
        compute_frame_rect(registry, parent_id, screen_width, screen_height)
    } else {
        LayoutRect { x: 0.0, y: 0.0, width: screen_width, height: screen_height }
    };

    let scale = effective_scale(registry, id);

    let mut rect = if frame.anchors.is_empty() {
        let w = frame.width * scale;
        let h = frame.height * scale;
        LayoutRect {
            x: parent_rect.x,
            y: parent_rect.y,
            width: w,
            height: h,
        }
    } else if frame.anchors.len() >= 2 {
        let edges = resolve_multi_anchor_edges(registry, frame, parent_rect, scale, screen_width, screen_height);
        compute_rect_from_edges(edges, frame, parent_rect, scale)
    } else {
        resolve_single_anchor(registry, frame, parent_rect, scale, screen_width, screen_height)
    };

    // Apply animation translation offsets
    rect.x += frame.anim_offset_x;
    rect.y += frame.anim_offset_y;

    // Clamp to screen bounds when clampedToScreen is set (e.g. tooltips)
    if frame.clamped_to_screen && rect.width > 0.0 && rect.height > 0.0 {
        clamp_rect_to_screen(&mut rect, screen_width, screen_height);
    }

    rect
}

/// Shift a rect so it stays within screen bounds (WoW `clampedToScreen` behavior).
fn clamp_rect_to_screen(rect: &mut LayoutRect, screen_w: f32, screen_h: f32) {
    if rect.x + rect.width > screen_w {
        rect.x = screen_w - rect.width;
    }
    if rect.x < 0.0 {
        rect.x = 0.0;
    }
    if rect.y + rect.height > screen_h {
        rect.y = screen_h - rect.height;
    }
    if rect.y < 0.0 {
        rect.y = 0.0;
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
