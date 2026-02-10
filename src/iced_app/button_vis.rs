//! Frame visibility resolution for rendering.
//!
//! Handles button state-dependent texture visibility (NormalTexture, PushedTexture,
//! HighlightTexture, DisabledTexture) and WoW HIGHLIGHT draw layer semantics
//! (regions only visible when parent is hovered).

use crate::widget::{DrawLayer, WidgetRegistry, WidgetType};

/// Decide whether a frame should be skipped during rendering.
///
/// Checks: subtree filter, zero alpha, HIGHLIGHT draw layer hover rules,
/// and button state texture visibility.
pub fn should_skip_frame(
    f: &crate::widget::Frame,
    id: u64,
    eff_alpha: f32,
    visible_ids: &Option<std::collections::HashSet<u64>>,
    registry: &WidgetRegistry,
    pressed_frame: Option<u64>,
    hovered_frame: Option<u64>,
) -> bool {
    if let Some(ids) = visible_ids {
        if !ids.contains(&id) {
            return true;
        }
    }
    if eff_alpha <= 0.0 {
        return true;
    }
    // WoW HIGHLIGHT draw layer: regions only visible when parent is hovered.
    // This is separate from the HighlightTexture button child (handled below).
    if f.draw_layer == DrawLayer::Highlight {
        let parent_hovered = f.parent_id.is_some() && hovered_frame == f.parent_id;
        if !parent_hovered {
            return true;
        }
    }

    let state_override = resolve_button_visibility(f, id, registry, pressed_frame, hovered_frame);
    match state_override {
        Some(false) => true,
        Some(true) => false,
        None => !f.visible,
    }
}

fn resolve_button_visibility(
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
    let is_pressed = !is_disabled && (pressed_frame == Some(parent_id) || parent.button_state == 1);
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
