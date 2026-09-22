//! Minimal CreateFrame helpers kept alive while the implementation moves to rilua.

use crate::lua_api::SimState;
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, create_string, frame_ref, get_or_create_frame_fields,
    sync_child_to_rilua,
};
use crate::widget::{AnchorPoint, Frame, FrameStrata, WidgetType};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub fn create_frame_instance(
    state: &mut LuaState,
    widget_type: WidgetType,
    frame_type: &str,
    name: Option<String>,
    parent_id: Option<u64>,
    parent_explicit: bool,
    id: Option<i32>,
) -> LuaResult<u64> {
    let mut frame = Frame::new(widget_type, name.clone(), parent_id);
    if widget_type == WidgetType::GameTooltip {
        frame.frame_strata = FrameStrata::Tooltip;
        frame.has_fixed_frame_strata = true;
    }
    if should_preserve_object_type_name(widget_type, frame_type) {
        frame.object_type_name = Some(frame_type.to_string());
    }
    apply_initial_visibility(state, &mut frame)?;
    apply_addon_ownership(state, parent_id, &mut frame)?;

    if let Some(user_id) = id {
        frame.user_id = user_id;
    }

    let preserve_existing_name_binding = should_preserve_existing_root_frame_name(state, &name)?;
    let frame_id = frame.id;
    register_and_attach_parent(
        state,
        frame,
        parent_id,
        parent_explicit,
        frame_id,
        preserve_existing_name_binding,
    )?;
    install_default_children(state, widget_type, frame_id)?;
    mark_loading_forbidden_object_table_scope(state, frame_id)?;
    if widget_type == WidgetType::MessageFrame {
        crate::lua_api::frame::methods::widgets::message_frame::install_message_frame_fields(
            state, frame_id,
        )?;
    }
    register_global_name(state, name, frame_id, preserve_existing_name_binding)?;

    Ok(frame_id)
}

fn mark_loading_forbidden_object_table_scope(state: &mut LuaState, frame_id: u64) -> LuaResult<()> {
    let use_forbidden_object_table = borrow_state(state)?.loading_use_forbidden_object_table;
    if use_forbidden_object_table {
        mark_frame_uses_forbidden_object_table(state, frame_id);
    }
    Ok(())
}

pub(crate) fn mark_frame_uses_forbidden_object_table(state: &mut LuaState, frame_id: u64) {
    let fields = get_or_create_frame_fields(state, frame_id);
    let marker_key = create_string(state, "__wowUseForbiddenObjectTable");
    if let Val::Table(fields_ref) = fields {
        if let Some(table) = state.gc.tables.get_mut(fields_ref) {
            let _ = table.raw_set(marker_key, Val::Bool(true), &state.gc.string_arena);
        }
        state.gc.barrier_back(fields_ref);
    }
}

fn install_default_children(
    state: &mut LuaState,
    widget_type: WidgetType,
    frame_id: u64,
) -> LuaResult<()> {
    if widget_type == WidgetType::EditBox {
        register_named_child(state, frame_id, "Text", WidgetType::FontString, false)?;
        return Ok(());
    }

    if widget_type != WidgetType::Slider {
        return Ok(());
    }

    for (key, child_type) in [
        ("Low", WidgetType::FontString),
        ("High", WidgetType::FontString),
        ("Text", WidgetType::FontString),
        ("ThumbTexture", WidgetType::Texture),
    ] {
        register_named_child(state, frame_id, key, child_type, true)?;
    }
    Ok(())
}

fn register_named_child(
    state: &mut LuaState,
    parent_id: u64,
    key: &str,
    child_type: WidgetType,
    anchor_slider_label: bool,
) -> LuaResult<u64> {
    let mut child = Frame::new(child_type, None, Some(parent_id));
    child.parent_key = Some(key.to_string());
    if anchor_slider_label {
        anchor_slider_label_child(&mut child, parent_id, key);
    }
    let child_id = child.id;
    {
        let mut sim = borrow_state_mut(state)?;
        sim.widgets.register(child);
        sim.widgets.add_child(parent_id, child_id);
        if let Some(parent) = sim.widgets.get_mut_visual(parent_id) {
            parent.children_keys.insert(key.to_string(), child_id);
        }
    }
    sync_child_to_rilua(state, parent_id, key, child_id)?;
    Ok(child_id)
}

fn anchor_slider_label_child(child: &mut Frame, parent_id: u64, key: &str) {
    let anchor = match key {
        "Low" => Some((AnchorPoint::TopLeft, AnchorPoint::BottomLeft)),
        "High" => Some((AnchorPoint::TopRight, AnchorPoint::BottomRight)),
        "Text" => Some((AnchorPoint::Bottom, AnchorPoint::Top)),
        _ => None,
    };
    if let Some((point, relative_point)) = anchor {
        child.set_point(point, Some(parent_id as usize), relative_point, 0.0, 0.0);
    }
}

fn should_preserve_object_type_name(widget_type: WidgetType, frame_type: &str) -> bool {
    if widget_type.as_str().eq_ignore_ascii_case(frame_type) {
        return false;
    }

    !matches!(
        frame_type.to_ascii_lowercase().as_str(),
        "containedalertframe"
            | "dropdownbutton"
            | "dropdowntogglebutton"
            | "eventbutton"
            | "itembutton"
    )
}

fn apply_initial_visibility(state: &mut LuaState, frame: &mut Frame) -> LuaResult<()> {
    let initial_hidden = borrow_state(state)?
        .create_frame_initial_hidden
        .unwrap_or(false);
    if initial_hidden {
        frame.visible = false;
        frame.effective_alpha = 0.0;
    }
    Ok(())
}

fn apply_addon_ownership(
    state: &mut LuaState,
    parent_id: Option<u64>,
    frame: &mut Frame,
) -> LuaResult<()> {
    let sim = borrow_state_mut(state)?;
    let parent_forbidden = parent_id
        .and_then(|pid| sim.widgets.get(pid))
        .is_some_and(|parent| parent.forbidden);
    frame.owner_addon = sim
        .loading_addon_index
        .or(sim.executing_addon_index)
        .or_else(|| parent_id.and_then(|pid| sim.widgets.get(pid).and_then(|f| f.owner_addon)));
    frame.forbidden = sim.loading_forbidden || parent_forbidden;
    Ok(())
}

fn register_and_attach_parent(
    state: &mut LuaState,
    frame: Frame,
    parent_id: Option<u64>,
    parent_explicit: bool,
    frame_id: u64,
    preserve_existing_name_binding: bool,
) -> LuaResult<()> {
    let mut sim = borrow_state_mut(state)?;
    let replaced_frame_id =
        find_replaced_placeholder_id(&sim, &frame, frame_id, preserve_existing_name_binding);
    if preserve_existing_name_binding {
        sim.widgets.register_preserving_existing_name(frame);
    } else {
        sim.widgets.register(frame);
    }
    if let Some(replaced_frame_id) = replaced_frame_id {
        retire_replaced_named_frame(&mut sim, replaced_frame_id, frame_id);
    }
    let Some(parent_id) = parent_id else {
        return Ok(());
    };
    sim.widgets.add_child(parent_id, frame_id);
    inherit_parent_render_state(&mut sim, frame_id, parent_id, parent_explicit);
    sim.invalidate_strata_buckets();
    Ok(())
}

fn find_replaced_placeholder_id(
    sim: &SimState,
    frame: &Frame,
    frame_id: u64,
    preserve_existing_name_binding: bool,
) -> Option<u64> {
    if preserve_existing_name_binding {
        return None;
    }
    frame
        .name
        .as_deref()
        .and_then(|name| sim.widgets.get_id_by_name(name))
        .filter(|existing_id| *existing_id != frame_id)
        .filter(|existing_id| {
            sim.widgets
                .get(*existing_id)
                .is_some_and(|existing| existing.is_simulator_placeholder)
        })
}

fn inherit_parent_render_state(
    sim: &mut SimState,
    frame_id: u64,
    parent_id: u64,
    parent_explicit: bool,
) {
    let Some(parent) = sim.widgets.get(parent_id) else {
        return;
    };
    let parent_strata = parent.frame_strata;
    let parent_level = parent.frame_level;
    let parent_alpha = parent.effective_alpha;
    let parent_scale = parent.effective_scale;

    let Some(child) = sim.widgets.get_mut_visual(frame_id) else {
        return;
    };
    if !child.has_fixed_frame_strata {
        child.frame_strata = parent_strata;
    }
    if parent_explicit {
        child.frame_level = parent_level + 1;
    }
    child.effective_alpha = if child.visible {
        parent_alpha * child.alpha
    } else {
        0.0
    };
    child.effective_scale = parent_scale * child.scale;
}

fn retire_replaced_named_frame(sim: &mut SimState, old_frame_id: u64, new_frame_id: u64) {
    let old_parent_id = sim
        .widgets
        .get(old_frame_id)
        .and_then(|frame| frame.parent_id);
    let (old_children, old_children_keys) = match sim.widgets.get_mut_visual(old_frame_id) {
        Some(old_frame) => {
            old_frame.visible = false;
            old_frame.effective_alpha = 0.0;
            old_frame.parent_id = None;
            (
                std::mem::take(&mut old_frame.children),
                std::mem::take(&mut old_frame.children_keys),
            )
        }
        None => return,
    };

    if let Some(old_parent_id) = old_parent_id
        && let Some(old_parent) = sim.widgets.get_mut(old_parent_id)
    {
        old_parent
            .children
            .retain(|child_id| *child_id != old_frame_id);
        old_parent
            .children_keys
            .retain(|_, child_id| *child_id != old_frame_id);
    }

    for child_id in old_children {
        sim.widgets.add_child(new_frame_id, child_id);
    }

    if let Some(new_frame) = sim.widgets.get_mut(new_frame_id) {
        for (key, child_id) in old_children_keys {
            new_frame.children_keys.entry(key).or_insert(child_id);
        }
    }
    sim.visible_on_update_cache = None;
    sim.invalidate_strata_buckets();
}

fn register_global_name(
    state: &mut LuaState,
    name: Option<String>,
    frame_id: u64,
    preserve_existing_name_binding: bool,
) -> LuaResult<()> {
    let Some(name) = name else {
        return Ok(());
    };
    if preserve_existing_name_binding {
        return Ok(());
    }
    let frame_val = frame_ref(state, frame_id)?;
    let (hide_from_global_env, add_to_secure_env) = {
        let sim = borrow_state(state)?;
        (
            sim.loading_hide_from_global_env,
            sim.loading_add_to_secure_env,
        )
    };
    if !hide_from_global_env {
        let key = state.gc.intern_string(name.as_bytes());
        let global = state.global;
        if let Some(globals) = state.gc.tables.get_mut(global) {
            let _ = globals.raw_set(Val::Str(key), frame_val, &state.gc.string_arena);
        }
        state.gc.barrier_back(global);
        crate::lua_api::global_slots::refresh_installed_slots_for_name(state, &name);
        crate::lua_api::globals::compat_overrides::record_public_global_publication(state, &name)?;
    }
    if add_to_secure_env {
        crate::lua_api::globals::security::set_secure_env_key_state(state, &name, frame_val)?;
    }
    Ok(())
}

fn should_preserve_existing_root_frame_name(
    state: &mut LuaState,
    name: &Option<String>,
) -> LuaResult<bool> {
    let Some(name) = name.as_deref() else {
        return Ok(false);
    };
    if !matches!(name, "UIParent" | "WorldFrame") {
        return Ok(false);
    }
    Ok(borrow_state(state)?.widgets.get_id_by_name(name).is_some())
}

pub(crate) fn apply_parent_sub(name: &str, parent_id: Option<u64>, state: &SimState) -> String {
    if name.len() < 7 || !name[..7].eq_ignore_ascii_case("$parent") {
        return name.to_string();
    }

    let mut current_id = parent_id;
    while let Some(id) = current_id {
        let Some(frame) = state.widgets.get(id) else {
            break;
        };
        if let Some(frame_name) = &frame.name
            && !frame_name.is_empty()
            && frame_name != "UIParent"
        {
            return format!("{frame_name}{}", &name[7..]);
        }
        current_id = frame.parent_id;
    }

    format!("Top{}", &name[7..])
}
