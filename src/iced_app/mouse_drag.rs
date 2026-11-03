//! Standalone drag helper functions for mouse event handling.

use crate::widget::AnchorPoint;

pub(super) fn frame_motion_scripts_allowed(frame: &crate::widget::Frame) -> bool {
    frame_enabled(frame) || frame.motion_scripts_while_disabled
}

pub(super) fn frame_enabled(frame: &crate::widget::Frame) -> bool {
    frame
        .attributes
        .get("__enabled")
        .and_then(|value| match value {
            crate::widget::AttributeValue::Boolean(enabled) => Some(*enabled),
            _ => None,
        })
        .unwrap_or(true)
}

pub(super) fn moving_drag_anchor_update(
    state: &crate::lua_api::SimState,
    drag_id: u64,
    dx: f32,
    dy: f32,
    screen_width: f32,
    screen_height: f32,
) -> Option<(Option<u64>, f32, f32)> {
    let frame = state.widgets.get(drag_id)?;
    if !frame.is_moving {
        return None;
    }

    let rect = frame.layout_rect?;
    let scale = frame.effective_scale.max(1e-6);
    let parent_id = frame.parent_id;
    let (parent_x, parent_y) = parent_id
        .and_then(|id| state.widgets.get(id).and_then(|parent| parent.layout_rect))
        .map(|rect| (rect.x, rect.y))
        .unwrap_or((0.0, 0.0));

    let mut new_left = rect.x + dx;
    let mut new_top = rect.y + dy;
    if frame.clamped_to_screen {
        new_left = clamp_axis_to_viewport(new_left, rect.width, screen_width);
        new_top = clamp_axis_to_viewport(new_top, rect.height, screen_height);
    }
    Some((
        parent_id,
        (new_left - parent_x) / scale,
        -(new_top - parent_y) / scale,
    ))
}

pub(super) fn active_motion_drag_frame(
    state: &crate::lua_api::SimState,
    active_drag_id: u64,
) -> u64 {
    let mut current = Some(active_drag_id);
    while let Some(id) = current {
        let Some(frame) = state.widgets.get(id) else {
            return active_drag_id;
        };
        if frame.is_moving || frame.is_sizing {
            return id;
        }
        current = frame.parent_id;
    }
    active_drag_id
}

pub(super) fn clamp_axis_to_viewport(position: f32, size: f32, viewport_size: f32) -> f32 {
    if size >= viewport_size {
        position
    } else {
        position.clamp(0.0, viewport_size - size)
    }
}

pub(super) fn reanchor_moving_drag_frame(
    state: &mut crate::lua_api::SimState,
    drag_id: u64,
    parent_id: Option<u64>,
    x_offset: f32,
    y_offset: f32,
) {
    state.widgets.remove_all_anchor_dependents_for(drag_id);
    if let Some(parent_id) = parent_id {
        state.widgets.add_anchor_dependent(parent_id, drag_id);
    }

    if let Some(frame) = state.widgets.get_mut_visual(drag_id) {
        frame.clear_all_points();
        frame.set_point(
            AnchorPoint::TopLeft,
            parent_id.map(|id| id as usize),
            AnchorPoint::TopLeft,
            x_offset,
            y_offset,
        );
    }
    state.widgets.mark_rect_dirty(drag_id);
}

/// Calculate new size for a frame being resized via StartSizing.
/// Returns the new size in frame units, clamped to resize bounds, or None if not sizing.
pub(super) fn sizing_drag_update(
    state: &crate::lua_api::SimState,
    drag_id: u64,
    dx: f32,
    dy: f32,
) -> Option<(f32, f32)> {
    let frame = state.widgets.get(drag_id)?;
    if !frame.is_sizing {
        return None;
    }

    let rect = frame.layout_rect?;
    let sizing_point = frame.sizing_point;

    let (dw, dh) = match sizing_point {
        AnchorPoint::BottomRight => (dx, dy),
        AnchorPoint::BottomLeft => (-dx, dy),
        AnchorPoint::TopRight => (dx, -dy),
        AnchorPoint::TopLeft => (-dx, -dy),
        AnchorPoint::Right => (dx, 0.0),
        AnchorPoint::Left => (-dx, 0.0),
        AnchorPoint::Bottom => (0.0, dy),
        AnchorPoint::Top => (0.0, -dy),
        AnchorPoint::Center => return None,
    };

    // The layout rect and mouse deltas are in screen units. Resize bounds
    // and Frame::set_size use frame units.
    let scale = frame.effective_scale.max(1e-6);
    let mut new_width = (rect.width + dw) / scale;
    let mut new_height = (rect.height + dh) / scale;

    let (min_w, min_h) = frame.resize_bounds_min;
    new_width = new_width.max(min_w.max(1.0));
    new_height = new_height.max(min_h.max(1.0));

    if let Some((max_w, max_h)) = frame.resize_bounds_max {
        new_width = new_width.min(max_w);
        new_height = new_height.min(max_h);
    }

    Some((new_width, new_height))
}

/// Apply new size to a frame being resized, re-anchoring to keep the non-dragged corner fixed.
pub(super) fn apply_sizing(
    state: &mut crate::lua_api::SimState,
    drag_id: u64,
    new_width: f32,
    new_height: f32,
) {
    let sizing_point = state.widgets.get(drag_id).map(|f| f.sizing_point);
    let old_rect = state.widgets.get(drag_id).and_then(|f| f.layout_rect);

    if let Some(frame) = state.widgets.get_mut_visual(drag_id) {
        frame.set_size(new_width, new_height);
    }

    let Some(sizing_point) = sizing_point else {
        state.widgets.mark_rect_dirty(drag_id);
        return;
    };
    let Some(old_rect) = old_rect else {
        state.widgets.mark_rect_dirty(drag_id);
        return;
    };

    let fixed_anchor = opposite_anchor(sizing_point);
    let (fixed_x, fixed_y) = anchor_screen_position(fixed_anchor, &old_rect);
    reanchor_to_fixed_corner(state, drag_id, fixed_anchor, fixed_x, fixed_y);
}

/// Return the anchor point opposite to the one being dragged.
fn opposite_anchor(point: AnchorPoint) -> AnchorPoint {
    match point {
        AnchorPoint::BottomRight => AnchorPoint::TopLeft,
        AnchorPoint::BottomLeft => AnchorPoint::TopRight,
        AnchorPoint::TopRight => AnchorPoint::BottomLeft,
        AnchorPoint::TopLeft => AnchorPoint::BottomRight,
        AnchorPoint::Right => AnchorPoint::Left,
        AnchorPoint::Left => AnchorPoint::Right,
        AnchorPoint::Bottom => AnchorPoint::Top,
        AnchorPoint::Top => AnchorPoint::Bottom,
        AnchorPoint::Center => AnchorPoint::Center,
    }
}

/// Compute the absolute screen position of an anchor point on a rect.
fn anchor_screen_position(anchor: AnchorPoint, rect: &crate::LayoutRect) -> (f32, f32) {
    let x = match anchor {
        AnchorPoint::TopLeft | AnchorPoint::Left | AnchorPoint::BottomLeft => rect.x,
        AnchorPoint::Top | AnchorPoint::Center | AnchorPoint::Bottom => rect.x + rect.width / 2.0,
        AnchorPoint::TopRight | AnchorPoint::Right | AnchorPoint::BottomRight => {
            rect.x + rect.width
        }
    };
    let y = match anchor {
        AnchorPoint::TopLeft | AnchorPoint::Top | AnchorPoint::TopRight => rect.y,
        AnchorPoint::Left | AnchorPoint::Center | AnchorPoint::Right => rect.y + rect.height / 2.0,
        AnchorPoint::BottomLeft | AnchorPoint::Bottom | AnchorPoint::BottomRight => {
            rect.y + rect.height
        }
    };
    (x, y)
}

/// Clear existing anchors and set a single anchor to keep the fixed corner in place.
fn reanchor_to_fixed_corner(
    state: &mut crate::lua_api::SimState,
    drag_id: u64,
    fixed_anchor: AnchorPoint,
    fixed_screen_x: f32,
    fixed_screen_y: f32,
) {
    let parent_id = state.widgets.get(drag_id).and_then(|f| f.parent_id);
    let scale = state
        .widgets
        .get(drag_id)
        .map(|f| f.effective_scale.max(1e-6))
        .unwrap_or(1.0);
    let (parent_x, parent_y) = parent_id
        .and_then(|id| state.widgets.get(id).and_then(|p| p.layout_rect))
        .map(|r| (r.x, r.y))
        .unwrap_or((0.0, 0.0));

    let x_offset = (fixed_screen_x - parent_x) / scale;
    let y_offset = -(fixed_screen_y - parent_y) / scale;

    state.widgets.remove_all_anchor_dependents_for(drag_id);
    if let Some(pid) = parent_id {
        state.widgets.add_anchor_dependent(pid, drag_id);
    }

    if let Some(frame) = state.widgets.get_mut_visual(drag_id) {
        frame.clear_all_points();
        frame.set_point(
            fixed_anchor,
            parent_id.map(|id| id as usize),
            AnchorPoint::TopLeft,
            x_offset,
            y_offset,
        );
    }
    state.widgets.mark_rect_dirty(drag_id);
}

pub(super) fn find_drag_script_target(
    env: &crate::lua_api::WowLuaEnv,
    frame_id: u64,
    script_name: &str,
) -> Option<u64> {
    let mut current = Some(frame_id);
    while let Some(id) = current {
        if env.has_script_handler(id, script_name) {
            return Some(id);
        }
        current = env
            .state()
            .borrow()
            .widgets
            .get(id)
            .and_then(|f| f.parent_id);
    }
    None
}

pub(super) struct SliderDragTrack {
    vertical: bool,
    start: f32,
    travel: f32,
}

impl SliderDragTrack {
    pub(super) fn new(
        widgets: &crate::widget::WidgetRegistry,
        slider: &crate::widget::Frame,
    ) -> Option<Self> {
        let track = crate::layout::apply_scroll_offsets(widgets, slider.id, slider.layout_rect?);
        let thumb = slider
            .children_keys
            .get("ThumbTexture")
            .and_then(|id| widgets.get(*id))
            .and_then(|thumb| thumb.layout_rect);
        let vertical = slider.slider_orientation == "VERTICAL";
        let (origin, track_extent, thumb_extent) = if vertical {
            (track.y, track.height, thumb.map_or(0.0, |rect| rect.height))
        } else {
            (track.x, track.width, thumb.map_or(0.0, |rect| rect.width))
        };
        let scale = crate::render::texture::UI_SCALE;
        let start = (origin + thumb_extent * 0.5) * scale;
        let travel = (track_extent - thumb_extent) * scale;
        (start.is_finite() && travel.is_finite() && travel > 0.0).then_some(Self {
            vertical,
            start,
            travel,
        })
    }

    fn cursor_axis(&self, pos: iced::Point) -> f32 {
        if self.vertical { pos.y } else { pos.x }
    }

    pub(super) fn grab_offset(
        &self,
        slider: &crate::widget::Frame,
        pos: iced::Point,
    ) -> Option<f32> {
        let span = slider.slider_max - slider.slider_min;
        if !span.is_finite() || span <= 0.0 {
            return None;
        }
        let fraction = ((slider.slider_value - slider.slider_min) / span).clamp(0.0, 1.0);
        let offset = self.cursor_axis(pos) - (self.start + fraction as f32 * self.travel);
        offset.is_finite().then_some(offset)
    }

    pub(super) fn value_at(
        &self,
        slider: &crate::widget::Frame,
        pos: iced::Point,
        grab_offset: f32,
    ) -> Option<f64> {
        let span = slider.slider_max - slider.slider_min;
        let fraction = (self.cursor_axis(pos) - grab_offset - self.start) / self.travel;
        if !span.is_finite() || span <= 0.0 || !fraction.is_finite() {
            return None;
        }
        Some(slider.slider_min + span * f64::from(fraction.clamp(0.0, 1.0)))
    }
}

#[cfg(test)]
mod tests {
    use super::clamp_axis_to_viewport;

    #[test]
    fn oversized_frame_drag_does_not_snap_axis_to_viewport_origin() {
        let position = 84.0;
        let size = 900.0;
        let viewport_size = 768.0;

        assert_eq!(
            clamp_axis_to_viewport(position, size, viewport_size),
            position,
            "oversized frames cannot fit inside the viewport, so clamping to \
             zero would make panels jump to the top or left edge on first drag"
        );
    }
}
