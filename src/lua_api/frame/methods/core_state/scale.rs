//! Scale methods: GetScale, GetEffectiveScale, SetScale,
//! SetIgnoreParentScale, GetIgnoreParentScale, IsIgnoringParentScale.

use super::helpers::{arg_bool, frame_id};
use crate::lua_api::frame::methods::methods_helpers::{
    can_change_protected_state_for, emit_addon_action_blocked,
};
use crate::lua_api::methods::{borrow_state, borrow_state_mut};
use crate::lua_api::state::SimState;
use crate::lua_bridge::stack_val;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

pub fn get_scale(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id(state, 1)?;
    let sim = borrow_state(state)?;
    let result = sim.widgets.get(id).map(|f| f.scale).unwrap_or(1.0);
    drop(sim);
    state.push(Val::Num(result as f64));
    Ok(1)
}

pub fn get_effective_scale(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id(state, 1)?;
    let sim = borrow_state(state)?;
    let result = sim
        .widgets
        .get(id)
        .map(|f| f.effective_scale)
        .unwrap_or(1.0);
    drop(sim);
    state.push(Val::Num(result as f64));
    Ok(1)
}

pub fn set_scale(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id(state, 1)?;
    let scale = match stack_val(state, 2) {
        Val::Num(n) => n as f32,
        _ => return Ok(0),
    };
    if scale <= 0.0 {
        return handle_non_positive_scale(state, id);
    }
    if !can_change_protected_state_for(state, id) {
        emit_addon_action_blocked(state, id, "SetScale");
        return Ok(0);
    }
    apply_scale_change(state, id, scale)?;
    Ok(0)
}

fn apply_scale_change(state: &LuaState, id: u64, scale: f32) -> LuaResult<()> {
    let mut sim = borrow_state_mut(state)?;
    let Some(frame) = sim.widgets.get(id) else {
        return Ok(());
    };
    if frame.scale == scale {
        return Ok(());
    }

    let parent_eff_scale = frame
        .parent_id
        .and_then(|parent_id| sim.widgets.get(parent_id))
        .map(|parent| parent.effective_scale)
        .unwrap_or(1.0);
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        frame.scale = scale;
    }
    sim.widgets.propagate_effective_scale(id, parent_eff_scale);
    sim.widgets.mark_rect_dirty(id);
    Ok(())
}

fn handle_non_positive_scale(state: &mut LuaState, id: u64) -> LuaResult<u32> {
    let sim = borrow_state(state)?;
    if is_map_canvas_scroll_child(&sim, id) {
        return Ok(0);
    }
    Err(runtime_error(non_positive_scale_message(&sim, id)))
}

fn non_positive_scale_message(sim: &SimState, id: u64) -> String {
    let frame = sim.widgets.get(id);
    let frame_name = frame
        .and_then(|frame| frame.name.as_deref())
        .unwrap_or("<unnamed>");
    let frame_type = frame
        .map(|frame| format!("{:?}", frame.widget_type))
        .unwrap_or_else(|| "<missing>".to_string());
    let parent_name = ancestor_name(sim, frame.and_then(|frame| frame.parent_id), "<no-parent>");
    let grandparent_id = frame
        .and_then(|frame| frame.parent_id)
        .and_then(|parent_id| sim.widgets.get(parent_id))
        .and_then(|parent| parent.parent_id);
    let grandparent_name = ancestor_name(sim, grandparent_id, "<no-grandparent>");
    let size = frame
        .map(|frame| format!("{}x{}", frame.width, frame.height))
        .unwrap_or_else(|| "<missing>".to_string());
    format!(
        "Frame:SetScale(): Scale must be > 0 on {frame_name} [{frame_type}] parent={parent_name} grandparent={grandparent_name} size={size}"
    )
}

fn ancestor_name(sim: &SimState, id: Option<u64>, fallback: &str) -> String {
    id.and_then(|id| sim.widgets.get(id))
        .and_then(|frame| frame.name.as_deref())
        .unwrap_or(fallback)
        .to_string()
}

fn is_map_canvas_scroll_child(sim: &SimState, id: u64) -> bool {
    let Some(frame) = sim.widgets.get(id) else {
        return false;
    };
    let Some(parent_id) = frame.parent_id else {
        return false;
    };
    let Some(parent) = sim.widgets.get(parent_id) else {
        return false;
    };
    if parent.children_keys.get("Child") != Some(&id) {
        return false;
    }
    let Some(grandparent_id) = parent.parent_id else {
        return false;
    };
    let Some(grandparent) = sim.widgets.get(grandparent_id) else {
        return false;
    };
    grandparent.children_keys.get("ScrollContainer") == Some(&parent_id)
}

pub fn set_ignore_parent_scale(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id(state, 1)?;
    let ignore = arg_bool(state, 2);
    if !can_change_protected_state_for(state, id) {
        emit_addon_action_blocked(state, id, "SetIgnoreParentScale");
        return Ok(0);
    }
    let mut sim = borrow_state_mut(state)?;
    let changed = sim
        .widgets
        .get(id)
        .map(|f| f.ignore_parent_scale != ignore)
        .unwrap_or(false);
    if !changed {
        return Ok(0);
    }
    let parent_eff_scale = sim
        .widgets
        .get(id)
        .and_then(|f| f.parent_id)
        .and_then(|pid| sim.widgets.get(pid))
        .map(|p| p.effective_scale)
        .unwrap_or(1.0);
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        frame.ignore_parent_scale = ignore;
    }
    sim.widgets.propagate_effective_scale(id, parent_eff_scale);
    sim.widgets.mark_rect_dirty(id);
    Ok(0)
}

pub fn get_ignore_parent_scale(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id(state, 1)?;
    let sim = borrow_state(state)?;
    let result = sim
        .widgets
        .get(id)
        .map(|f| f.ignore_parent_scale)
        .unwrap_or(false);
    drop(sim);
    state.push(Val::Bool(result));
    Ok(1)
}

pub fn is_ignoring_parent_scale(state: &mut LuaState) -> LuaResult<u32> {
    get_ignore_parent_scale(state)
}
