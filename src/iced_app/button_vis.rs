//! Frame visibility resolution for rendering.
//!
//! Handles button state-dependent texture visibility (NormalTexture, PushedTexture,
//! HighlightTexture, DisabledTexture) and WoW HIGHLIGHT draw layer semantics
//! (regions only visible when parent is hovered).

use rustc_hash::FxHashSet;

use crate::widget::{DrawLayer, WidgetRegistry, WidgetType};

/// Decide whether a frame should be skipped during rendering.
///
/// Checks: subtree filter, zero alpha, HIGHLIGHT draw layer hover rules,
/// and button state texture visibility.
pub fn should_skip_frame(
    f: &crate::widget::Frame,
    id: u64,
    eff_alpha: f32,
    visible_ids: &Option<FxHashSet<u64>>,
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
    if parent_draw_layer_is_disabled(f, registry) {
        return true;
    }
    // Generic HIGHLIGHT regions require hover; a locked Button slot child
    // instead renders in the regular draw pass.
    if f.draw_layer == DrawLayer::Highlight {
        let parent_hovered = f.parent_id.is_some() && hovered_frame == f.parent_id;
        if !locked_button_highlight_child(f, registry)
            && (!parent_hovered || !parent_allows_hover_highlight(f, registry))
        {
            return true;
        }
    }

    let state_override = resolve_button_visibility(f, id, registry, pressed_frame);
    if state_override.is_some() && has_hidden_ancestor(f, registry) {
        return true;
    }
    match state_override {
        Some(false) => true,
        Some(true) => false,
        None => !registry.is_ancestor_visible(id),
    }
}

fn has_hidden_ancestor(f: &crate::widget::Frame, registry: &WidgetRegistry) -> bool {
    let mut current_id = f.parent_id;
    while let Some(id) = current_id {
        let Some(parent) = registry.get(id) else {
            return true;
        };
        if !parent.visible {
            return true;
        }
        current_id = parent.parent_id;
    }
    false
}

fn parent_draw_layer_is_disabled(f: &crate::widget::Frame, registry: &WidgetRegistry) -> bool {
    if !matches!(
        f.widget_type,
        WidgetType::Texture | WidgetType::FontString | WidgetType::Line
    ) {
        return false;
    }
    let Some(parent_id) = f.parent_id else {
        return false;
    };
    let Some(parent) = registry.get(parent_id) else {
        return false;
    };
    !parent.is_draw_layer_enabled(f.draw_layer)
}

pub(super) fn should_append_hover_highlight(
    registry: &WidgetRegistry,
    button: &crate::widget::Frame,
) -> bool {
    !button.highlight_locked && registry.is_ancestor_visible(button.id)
}

fn locked_button_highlight_child(f: &crate::widget::Frame, registry: &WidgetRegistry) -> bool {
    f.parent_id
        .and_then(|id| registry.get(id))
        .is_some_and(|parent| {
            matches!(
                parent.widget_type,
                WidgetType::Button | WidgetType::CheckButton
            ) && parent.highlight_locked
                && parent.children_keys.get("HighlightTexture") == Some(&f.id)
        })
}

fn parent_allows_hover_highlight(f: &crate::widget::Frame, registry: &WidgetRegistry) -> bool {
    let Some(parent_id) = f.parent_id else {
        return false;
    };
    let Some(parent) = registry.get(parent_id) else {
        return false;
    };
    if matches!(
        parent.widget_type,
        WidgetType::Button | WidgetType::CheckButton
    ) {
        // Suppress the HighlightTexture slot child here — it is rendered exclusively
        // by append_hover_highlight, which controls timing and blend mode. Allowing
        // it through the regular draw loop produces double additive blending on hover.
        if parent.children_keys.get("HighlightTexture") == Some(&f.id) {
            return false;
        }
        return is_enabled(parent);
    }
    true
}

fn resolve_button_visibility(
    f: &crate::widget::Frame,
    id: u64,
    registry: &WidgetRegistry,
    pressed_frame: Option<u64>,
) -> Option<bool> {
    if !matches!(f.widget_type, WidgetType::Texture) {
        return None;
    }
    let parent_id = f.parent_id?;
    let parent = registry.get(parent_id)?;
    texture_visibility(parent, id, parent_id, pressed_frame)
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
) -> Option<bool> {
    if !matches!(
        parent.widget_type,
        WidgetType::Button | WidgetType::CheckButton
    ) {
        return None;
    }
    let is_disabled = !is_enabled(parent);
    let is_pressed = !is_disabled && (pressed_frame == Some(parent_id) || parent.button_state == 1);

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
        // Locked highlight renders in the regular pass; unlocked highlight
        // renders in the hover pass. Never emit the same child in both.
        return Some(parent.highlight_locked);
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

#[cfg(test)]
mod tests {
    use super::should_skip_frame;
    use crate::widget::{AttributeValue, DrawLayer, Frame, WidgetRegistry, WidgetType};

    #[test]
    fn disabled_parent_draw_layer_hides_child_region() {
        let mut registry = WidgetRegistry::new();

        let mut parent = Frame::new(WidgetType::Frame, Some("Parent".to_string()), None);
        parent.set_draw_layer_enabled(DrawLayer::Border, false);
        let parent_id = parent.id;
        registry.register(parent);

        let mut child = Frame::new(
            WidgetType::Texture,
            Some("Child".to_string()),
            Some(parent_id),
        );
        child.draw_layer = DrawLayer::Border;
        let child_id = child.id;
        registry.register(child);
        registry.add_child(parent_id, child_id);

        let child = registry.get(child_id).unwrap();
        let skipped = should_skip_frame(child, child_id, 1.0, &None, &registry, None, None);
        assert!(
            skipped,
            "child region should be skipped when its parent draw layer is disabled",
        );
    }

    #[test]
    fn disabled_button_suppresses_hover_draw_layer_child() {
        let mut registry = WidgetRegistry::new();

        let mut parent = Frame::new(WidgetType::Button, Some("Parent".to_string()), None);
        parent
            .attributes
            .insert("__enabled".to_string(), AttributeValue::Boolean(false));
        let parent_id = parent.id;
        registry.register(parent);

        let mut child = Frame::new(
            WidgetType::Texture,
            Some("HighlightChild".to_string()),
            Some(parent_id),
        );
        child.draw_layer = DrawLayer::Highlight;
        let child_id = child.id;
        registry.register(child);
        registry.add_child(parent_id, child_id);

        let child = registry.get(child_id).unwrap();
        let skipped = should_skip_frame(
            child,
            child_id,
            1.0,
            &None,
            &registry,
            None,
            Some(parent_id),
        );
        assert!(
            skipped,
            "disabled button should not render HIGHLIGHT draw-layer children on hover",
        );
    }

    #[test]
    fn hidden_button_suppresses_state_texture_child() {
        let mut registry = WidgetRegistry::new();

        let mut parent = Frame::new(WidgetType::Button, Some("Parent".to_string()), None);
        parent.visible = false;
        let parent_id = parent.id;
        registry.register(parent);

        let child = Frame::new(
            WidgetType::Texture,
            Some("NormalTexture".to_string()),
            Some(parent_id),
        );
        let child_id = child.id;
        registry.register(child);
        registry.add_child(parent_id, child_id);
        registry
            .get_mut(parent_id)
            .unwrap()
            .children_keys
            .insert("NormalTexture".to_string(), child_id);

        let child = registry.get(child_id).unwrap();
        let skipped = should_skip_frame(child, child_id, 1.0, &None, &registry, None, None);
        assert!(
            skipped,
            "state texture child should stay hidden when its button parent is hidden",
        );
    }
}
