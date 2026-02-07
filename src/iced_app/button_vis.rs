//! Button state-dependent texture visibility.
//!
//! WoW buttons have child Texture widgets (NormalTexture, PushedTexture,
//! HighlightTexture, DisabledTexture) whose visibility depends on button state.

use crate::widget::{WidgetRegistry, WidgetType};

/// Check if a frame is a button state texture and return its state-driven visibility.
///
/// Returns `Some(true/false)` for state textures (overrides frame.visible),
/// or `None` for all other frames (use normal visibility rules).
pub fn resolve_visibility(
    f: &crate::widget::Frame,
    id: u64,
    registry: &WidgetRegistry,
    pressed_frame: Option<u64>,
    hovered_frame: Option<u64>,
) -> Option<bool> {
    if !matches!(f.widget_type, WidgetType::Texture) {
        return None;
    }
    let parent_id = f.parent_id?;
    let parent = registry.get(parent_id)?;
    texture_visibility(parent, id, parent_id, pressed_frame, hovered_frame)
}

/// Determine if a Texture child of a Button should render based on button state.
///
/// Returns `Some(true)` if the texture should render (overrides frame.visible),
/// `Some(false)` if it should be hidden, or `None` if this is not a button
/// state texture (use normal visibility rules).
///
/// WoW button state texture rules:
/// - Disabled: DisabledTexture shown, all others hidden
/// - Pressed: PushedTexture shown, NormalTexture hidden
/// - Hovered: HighlightTexture shown (overlays NormalTexture)
/// - Normal: NormalTexture shown, all others hidden
fn texture_visibility(
    parent: &crate::widget::Frame,
    texture_id: u64,
    parent_id: u64,
    pressed_frame: Option<u64>,
    hovered_frame: Option<u64>,
) -> Option<bool> {
    if !matches!(parent.widget_type, WidgetType::Button | WidgetType::CheckButton) {
        return None;
    }
    let is_disabled = !is_enabled(parent);
    let is_pressed = !is_disabled && pressed_frame == Some(parent_id);
    let is_hovered = !is_disabled && hovered_frame == Some(parent_id);

    if parent.children_keys.get("DisabledTexture") == Some(&texture_id) {
        return Some(is_disabled);
    }
    if parent.children_keys.get("NormalTexture") == Some(&texture_id) {
        return Some(!is_disabled && !is_pressed);
    }
    if parent.children_keys.get("PushedTexture") == Some(&texture_id) {
        return Some(is_pressed);
    }
    if parent.children_keys.get("HighlightTexture") == Some(&texture_id) {
        return Some(is_hovered);
    }
    None
}

/// Check whether a button's `__enabled` attribute is true (default: true).
fn is_enabled(frame: &crate::widget::Frame) -> bool {
    frame
        .attributes
        .get("__enabled")
        .and_then(|v| match v {
            crate::widget::AttributeValue::Boolean(b) => Some(*b),
            _ => None,
        })
        .unwrap_or(true)
}
