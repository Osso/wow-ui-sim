//! Helper functions for frame methods.

use crate::widget::{Anchor, AnchorPoint, Frame, WidgetType};

/// Calculate frame width from anchors or explicit size (recursive).
/// WoW behavior: anchors defining opposite edges override explicit size.
pub fn calculate_frame_width(widgets: &crate::widget::WidgetRegistry, id: u64) -> f32 {
    if let Some(frame) = widgets.get(id) {
        // Try to calculate from left+right anchors first (they override explicit size)
        use crate::widget::AnchorPoint::*;
        let left_anchors = [TopLeft, BottomLeft, Left];
        let right_anchors = [TopRight, BottomRight, Right];
        let left = frame.anchors.iter().find(|a| left_anchors.contains(&a.point));
        let right = frame.anchors.iter().find(|a| right_anchors.contains(&a.point));
        if let (Some(left_anchor), Some(right_anchor)) = (left, right) {
            // Both must anchor to same relative frame
            if left_anchor.relative_to_id == right_anchor.relative_to_id {
                let parent_id = left_anchor
                    .relative_to_id
                    .map(|id| id as u64)
                    .or(frame.parent_id);
                if let Some(pid) = parent_id {
                    // Recursively calculate parent width
                    let parent_width = calculate_frame_width(widgets, pid);
                    if parent_width > 0.0 {
                        return (parent_width - left_anchor.x_offset + right_anchor.x_offset).max(0.0);
                    }
                }
            }
        }
        // Fall back to explicit width
        frame.width
    } else {
        0.0
    }
}

/// Calculate frame height from anchors or explicit size (recursive).
/// WoW behavior: anchors defining opposite edges override explicit size.
pub fn calculate_frame_height(widgets: &crate::widget::WidgetRegistry, id: u64) -> f32 {
    if let Some(frame) = widgets.get(id) {
        // Try to calculate from top+bottom anchors first (they override explicit size)
        use crate::widget::AnchorPoint::*;
        let top_anchors = [TopLeft, TopRight, Top];
        let bottom_anchors = [BottomLeft, BottomRight, Bottom];
        let top = frame.anchors.iter().find(|a| top_anchors.contains(&a.point));
        let bottom = frame.anchors.iter().find(|a| bottom_anchors.contains(&a.point));
        if let (Some(top_anchor), Some(bottom_anchor)) = (top, bottom) {
            // Both must anchor to same relative frame
            if top_anchor.relative_to_id == bottom_anchor.relative_to_id {
                let parent_id = top_anchor
                    .relative_to_id
                    .map(|id| id as u64)
                    .or(frame.parent_id);
                if let Some(pid) = parent_id {
                    // Recursively calculate parent height
                    let parent_height = calculate_frame_height(widgets, pid);
                    if parent_height > 0.0 {
                        return (parent_height + top_anchor.y_offset - bottom_anchor.y_offset).max(0.0);
                    }
                }
            }
        }
        // Fall back to explicit height
        frame.height
    } else {
        0.0
    }
}

/// Helper to create a button texture child if it doesn't exist.
/// Also ensures existing textures have proper anchors to fill the button.
pub fn get_or_create_button_texture(
    state: &mut crate::lua_api::SimState,
    button_id: u64,
    key: &str,
) -> u64 {
    // Check if texture child already exists - copy the id to avoid borrow conflict
    let existing_tex_id = state
        .widgets
        .get(button_id)
        .and_then(|frame| frame.children_keys.get(key).copied());

    if let Some(tex_id) = existing_tex_id {
        // Ensure existing texture has anchors to fill parent
        if let Some(tex) = state.widgets.get_mut(tex_id) {
            if tex.anchors.is_empty() {
                // Add anchors to fill parent button
                tex.anchors.push(Anchor {
                    point: AnchorPoint::TopLeft,
                    relative_to: None,
                    relative_to_id: Some(button_id as usize),
                    relative_point: AnchorPoint::TopLeft,
                    x_offset: 0.0,
                    y_offset: 0.0,
                });
                tex.anchors.push(Anchor {
                    point: AnchorPoint::BottomRight,
                    relative_to: None,
                    relative_to_id: Some(button_id as usize),
                    relative_point: AnchorPoint::BottomRight,
                    x_offset: 0.0,
                    y_offset: 0.0,
                });
            }
        }
        return tex_id;
    }

    // Create new texture child
    let mut texture = Frame::new(WidgetType::Texture, None, Some(button_id));
    // Set texture to fill parent (SetAllPoints behavior)
    texture.anchors.push(Anchor {
        point: AnchorPoint::TopLeft,
        relative_to: None,
        relative_to_id: Some(button_id as usize),
        relative_point: AnchorPoint::TopLeft,
        x_offset: 0.0,
        y_offset: 0.0,
    });
    texture.anchors.push(Anchor {
        point: AnchorPoint::BottomRight,
        relative_to: None,
        relative_to_id: Some(button_id as usize),
        relative_point: AnchorPoint::BottomRight,
        x_offset: 0.0,
        y_offset: 0.0,
    });
    let texture_id = texture.id;

    state.widgets.register(texture);
    state.widgets.add_child(button_id, texture_id);

    // Store in children_keys
    if let Some(frame) = state.widgets.get_mut(button_id) {
        frame.children_keys.insert(key.to_string(), texture_id);
    }

    texture_id
}
