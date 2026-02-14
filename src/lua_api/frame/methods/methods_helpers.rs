//! Helper functions for frame methods.

use crate::widget::{Anchor, AnchorPoint, Frame, WidgetType};
use mlua::{Lua, Value};

/// Check if a Lua mixin override exists for the given method on a frame.
///
/// When Blizzard's Mixin() applies a mixin table to a FrameHandle, methods are stored
/// in `__frame_fields[frame_id]`. Rust UserData methods (via add_method) are resolved
/// before `__index`, so they shadow mixin methods. This helper allows Rust methods to
/// detect and delegate to mixin overrides.
///
/// Returns `(function, frame_userdata)` if an override exists, None otherwise.
pub fn get_mixin_override(
    lua: &Lua,
    frame_id: u64,
    method_name: &str,
) -> Option<(mlua::Function, Value)> {
    let fields_table = crate::lua_api::script_helpers::get_frame_fields_table(lua)?;
    let frame_fields = fields_table.get::<mlua::Table>(frame_id).ok()?;
    let func = match frame_fields.get::<Value>(method_name) {
        Ok(Value::Function(f)) => f,
        _ => return None,
    };
    let frame_key = format!("__frame_{}", frame_id);
    let ud = lua.globals().get::<Value>(&*frame_key).ok()?;
    Some((func, ud))
}

/// Read the eagerly-propagated effective scale from the frame.
fn eff_scale(widgets: &crate::widget::WidgetRegistry, id: u64) -> f32 {
    widgets.get(id).map(|f| f.effective_scale).unwrap_or(1.0)
}

/// Calculate frame width from anchors or explicit size (recursive).
/// WoW behavior: anchors defining opposite edges override explicit size.
pub fn calculate_frame_width(widgets: &crate::widget::WidgetRegistry, id: u64) -> f32 {
    if let Some(frame) = widgets.get(id) {
        use crate::widget::AnchorPoint::*;
        let left_anchors = [TopLeft, BottomLeft, Left];
        let right_anchors = [TopRight, BottomRight, Right];
        let left = frame.anchors.iter().find(|a| left_anchors.contains(&a.point));
        let right = frame.anchors.iter().find(|a| right_anchors.contains(&a.point));
        if let (Some(left_anchor), Some(right_anchor)) = (left, right) {
            if left_anchor.relative_to_id == right_anchor.relative_to_id {
                // Same frame: compute from relative frame width
                let parent_id = left_anchor
                    .relative_to_id
                    .map(|id| id as u64)
                    .or(frame.parent_id);
                if let Some(pid) = parent_id {
                    let parent_width = calculate_frame_width(widgets, pid);
                    if parent_width > 0.0 {
                        return (parent_width - left_anchor.x_offset + right_anchor.x_offset).max(0.0);
                    }
                }
            }
            // Cross-frame anchors or parentless nil-target anchors (e.g. UIParent
            // with setAllPoints="true"): use pre-computed layout_rect.
            if let Some(rect) = frame.layout_rect {
                let s = eff_scale(widgets, id);
                if s > 0.0 && rect.width > 0.0 {
                    return rect.width / s;
                }
            }
        }
        frame.width
    } else {
        0.0
    }
}

/// Calculate frame height from anchors or explicit size (recursive).
/// WoW behavior: anchors defining opposite edges override explicit size.
pub fn calculate_frame_height(widgets: &crate::widget::WidgetRegistry, id: u64) -> f32 {
    if let Some(frame) = widgets.get(id) {
        use crate::widget::AnchorPoint::*;
        let top_anchors = [TopLeft, TopRight, Top];
        let bottom_anchors = [BottomLeft, BottomRight, Bottom];
        let top = frame.anchors.iter().find(|a| top_anchors.contains(&a.point));
        let bottom = frame.anchors.iter().find(|a| bottom_anchors.contains(&a.point));
        if let (Some(top_anchor), Some(bottom_anchor)) = (top, bottom) {
            if top_anchor.relative_to_id == bottom_anchor.relative_to_id {
                // Same frame: compute from relative frame height
                let parent_id = top_anchor
                    .relative_to_id
                    .map(|id| id as u64)
                    .or(frame.parent_id);
                if let Some(pid) = parent_id {
                    let parent_height = calculate_frame_height(widgets, pid);
                    if parent_height > 0.0 {
                        return (parent_height + top_anchor.y_offset - bottom_anchor.y_offset).max(0.0);
                    }
                }
            }
            // Cross-frame anchors or parentless nil-target anchors (e.g. UIParent
            // with setAllPoints="true"): use pre-computed layout_rect.
            if let Some(rect) = frame.layout_rect {
                let s = eff_scale(widgets, id);
                if s > 0.0 && rect.height > 0.0 {
                    return rect.height / s;
                }
            }
        }
        frame.height
    } else {
        0.0
    }
}

/// Add fill-parent anchors (TopLeft + BottomRight) to a frame, equivalent to SetAllPoints.
fn set_all_points_anchors(frame: &mut Frame, parent_id: u64) {
    frame.anchors.push(Anchor {
        point: AnchorPoint::TopLeft,
        relative_to: None,
        relative_to_id: Some(parent_id as usize),
        relative_point: AnchorPoint::TopLeft,
        x_offset: 0.0,
        y_offset: 0.0,
    });
    frame.anchors.push(Anchor {
        point: AnchorPoint::BottomRight,
        relative_to: None,
        relative_to_id: Some(parent_id as usize),
        relative_point: AnchorPoint::BottomRight,
        x_offset: 0.0,
        y_offset: 0.0,
    });
}

/// Helper to create a button texture child if it doesn't exist.
/// Also ensures existing textures have proper anchors to fill the button.
pub fn get_or_create_button_texture(
    state: &mut crate::lua_api::SimState,
    button_id: u64,
    key: &str,
) -> u64 {
    let existing_tex_id = state
        .widgets
        .get(button_id)
        .and_then(|frame| frame.children_keys.get(key).copied());

    if let Some(tex_id) = existing_tex_id {
        let needs_anchors = state.widgets.get(tex_id).map(|t| t.anchors.is_empty()).unwrap_or(false);
        if needs_anchors
            && let Some(tex) = state.widgets.get_mut(tex_id) {
                set_all_points_anchors(tex, button_id);
            }
        return tex_id;
    }

    let mut texture = Frame::new(WidgetType::Texture, None, Some(button_id));
    set_all_points_anchors(&mut texture, button_id);
    // Inherit strata and level from parent button
    if let Some(parent) = state.widgets.get(button_id) {
        texture.frame_strata = parent.frame_strata;
        texture.frame_level = parent.frame_level + 1;
    }
    let texture_id = texture.id;

    state.widgets.register(texture);
    state.widgets.add_child(button_id, texture_id);

    if let Some(frame) = state.widgets.get_mut(button_id) {
        frame.children_keys.insert(key.to_string(), texture_id);
    }

    texture_id
}

/// Resolve a Lua value that can be a file path (string) or a file data ID (number).
///
/// WoW's SetTexture/SetNormalTexture/etc. accept either a string path like
/// `"Interface\\Icons\\Spell_Holy_CrusaderStrike"` or a numeric file data ID like
/// `135891`. This helper handles both cases, looking up numeric IDs in the manifest.
pub fn resolve_file_data_id_or_path(value: &Value) -> Option<String> {
    match value {
        Value::String(s) => {
            let s = s.to_str().ok()?;
            // A string that parses as an integer is a file data ID passed as string
            if let Ok(id) = s.parse::<u32>() {
                let path = crate::manifest_interface_data::get_texture_path(id)?;
                Some(format!("Interface\\{}", path.replace('/', "\\")))
            } else {
                Some(s.to_string())
            }
        }
        Value::Integer(n) => {
            let path = crate::manifest_interface_data::get_texture_path(*n as u32)?;
            Some(format!("Interface\\{}", path.replace('/', "\\")))
        }
        Value::Number(n) => {
            let path = crate::manifest_interface_data::get_texture_path(*n as u32)?;
            Some(format!("Interface\\{}", path.replace('/', "\\")))
        }
        _ => None,
    }
}
