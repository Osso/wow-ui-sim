//! Pure frame helper functions shared by loader code and rilua method modules.

use crate::lua_api::globals::state_backed_queries::dispatch_event_now;
use crate::lua_api::methods::borrow_state;
use crate::widget::{Anchor, AnchorPoint, Frame, WidgetRegistry, WidgetType};
use rilua::vm::state::LuaState;

pub fn set_all_points_anchors_pub(frame: &mut Frame, parent_id: u64) {
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

pub fn get_or_create_button_texture(
    state: &mut crate::lua_api::SimState,
    button_id: u64,
    key: &str,
) -> u64 {
    if let Some(tex_id) = button_texture_child_id(state, button_id, key) {
        return refresh_button_texture_child(state, button_id, key, tex_id);
    }
    create_button_texture_child(state, button_id, key)
}

pub fn can_change_protected_state_for(state: &mut LuaState, id: u64) -> bool {
    if rilua::api::state_is_secure(state) {
        return true;
    }
    let Ok(sim) = borrow_state(state) else {
        return true;
    };
    !sim.in_combat_lockdown() || !frame_blocks_protected_state(&sim.widgets, id)
}

pub fn frame_blocks_protected_state(widgets: &WidgetRegistry, id: u64) -> bool {
    frame_is_protected(widgets, id)
        || frame_has_protected_descendant(widgets, id)
        || frame_anchor_references_protected_state(widgets, id)
}

pub fn emit_addon_action_blocked(state: &mut LuaState, frame_id: u64, action: &str) {
    let blocked_action = borrow_state(state)
        .ok()
        .and_then(|sim| {
            sim.widgets
                .get(frame_id)
                .and_then(|frame| frame.name.as_deref())
                .map(|name| format!("{name}:{action}()"))
        })
        .unwrap_or_else(|| action.to_string());
    let owner = state.call_stack[..=state.ci]
        .iter()
        .rev()
        .find_map(|frame| frame.taint.clone())
        .expect("blocked protected action must have a tainted caller");
    let owner = crate::lua_api::methods::create_string(state, &owner);
    let blocked_action = crate::lua_api::methods::create_string(state, &blocked_action);
    let _ = dispatch_event_now(state, "ADDON_ACTION_BLOCKED", &[owner, blocked_action]);
}

fn button_texture_child_id(
    state: &crate::lua_api::SimState,
    button_id: u64,
    key: &str,
) -> Option<u64> {
    let button = state.widgets.get(button_id)?;
    if let Some(tex_id) = button.children_keys.get(key).copied() {
        return Some(tex_id);
    }

    button.children.iter().copied().find(|child_id| {
        state
            .widgets
            .get(*child_id)
            .is_some_and(|child| child.parent_key.as_deref() == Some(key))
    })
}

fn refresh_button_texture_child(
    state: &mut crate::lua_api::SimState,
    button_id: u64,
    key: &str,
    tex_id: u64,
) -> u64 {
    let needs_children_key = state
        .widgets
        .get(button_id)
        .map(|button| button.children_keys.get(key).copied() != Some(tex_id))
        .unwrap_or(false);
    let needs_anchors = state
        .widgets
        .get(tex_id)
        .map(|texture| texture.anchors.is_empty())
        .unwrap_or(false);
    let needs_parent_key = state
        .widgets
        .get(tex_id)
        .map(|texture| texture.parent_key.as_deref() != Some(key))
        .unwrap_or(false);
    if needs_children_key || needs_anchors || needs_parent_key {
        if needs_children_key {
            if let Some(button) = state.widgets.get_mut_visual(button_id) {
                button.children_keys.insert(key.to_string(), tex_id);
            }
        }
        if let Some(texture) = state.widgets.get_mut_visual(tex_id) {
            if needs_anchors {
                set_all_points_anchors_pub(texture, button_id);
            }
            texture.parent_key = Some(key.to_string());
        }
        state.widgets.mark_rect_dirty(button_id);
    }
    tex_id
}

fn create_button_texture_child(
    state: &mut crate::lua_api::SimState,
    button_id: u64,
    key: &str,
) -> u64 {
    let texture = new_button_texture_child(state, button_id, key);
    let texture_id = texture.id;
    state.widgets.register(texture);
    state.widgets.add_child(button_id, texture_id);

    if let Some(frame) = state.widgets.get_mut_visual(button_id) {
        frame.children_keys.insert(key.to_string(), texture_id);
    }
    state.widgets.mark_rect_dirty(button_id);
    texture_id
}

fn new_button_texture_child(state: &crate::lua_api::SimState, button_id: u64, key: &str) -> Frame {
    let mut texture = Frame::new(WidgetType::Texture, None, Some(button_id));
    set_all_points_anchors_pub(&mut texture, button_id);
    texture.parent_key = Some(key.to_string());
    apply_button_texture_defaults(&mut texture, state, button_id, key);
    texture
}

fn apply_button_texture_defaults(
    texture: &mut Frame,
    state: &crate::lua_api::SimState,
    button_id: u64,
    key: &str,
) {
    if key == "HighlightTexture" {
        texture.draw_layer = crate::widget::DrawLayer::Highlight;
        texture.alpha_mode = Some("ADD".to_string());
        texture.blend_mode = crate::render::BlendMode::Additive;
    }
    if let Some(parent) = state.widgets.get(button_id) {
        texture.frame_strata = parent.frame_strata;
        texture.frame_level = parent.frame_level + 1;
        texture.layout_rect = parent.layout_rect;
    }
}

fn frame_is_protected(widgets: &WidgetRegistry, id: u64) -> bool {
    widgets.get(id).is_some_and(|frame| frame.is_protected)
}

fn frame_has_protected_descendant(widgets: &WidgetRegistry, id: u64) -> bool {
    widgets.iter_ids().any(|child_id| {
        widgets.get(child_id).is_some_and(|frame| {
            frame.parent_id == Some(id)
                && (frame.is_protected || frame_has_protected_descendant(widgets, child_id))
        })
    })
}

fn frame_anchor_references_protected_state(widgets: &WidgetRegistry, id: u64) -> bool {
    widgets.get(id).is_some_and(|frame| {
        frame.anchors.iter().any(|anchor| {
            anchor
                .relative_to_id
                .is_some_and(|relative_to_id| frame_is_protected(widgets, relative_to_id as u64))
        })
    })
}
