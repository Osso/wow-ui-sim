//! Frame size methods: GetWidth, GetHeight, GetSize, SetSize, SetWidth, SetHeight.

use super::helpers::{
    apply_explicit_height, apply_explicit_size, apply_explicit_width, clear_auto_height_flag,
    clear_auto_width_flag, current_explicit_size_state, frame_id, frame_size, opt_f32,
};
use crate::lua_api::frame::methods::methods_helpers::{
    can_change_protected_state_for, emit_addon_action_blocked,
};
use crate::lua_api::frame::methods::text_attribute_event::{
    refresh_auto_text_height_after_width_change, refresh_auto_text_width_after_zero_width,
};
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, call_function_state, frame_ref, table_get_static,
};
use crate::lua_api::script_helpers::{
    get_script as get_rilua_script, set_script as set_rilua_script,
};
use crate::lua_bridge::FromStack;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub fn get_width(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id(state, 1)?;
    super::super::secret_origin::require_geometry_readable(state, id)?;
    let ignore = bool::from_stack(state, 2).ok().unwrap_or(false);
    let (width, _) = frame_size(state, id, ignore)?;
    state.push(Val::Num(width as f64));
    Ok(1)
}

pub fn get_height(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id(state, 1)?;
    super::super::secret_origin::require_geometry_readable(state, id)?;
    let ignore = bool::from_stack(state, 2).ok().unwrap_or(false);
    let (_, height) = frame_size(state, id, ignore)?;
    state.push(Val::Num(height as f64));
    Ok(1)
}

pub fn get_size(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id(state, 1)?;
    super::super::secret_origin::require_geometry_readable(state, id)?;
    let ignore = bool::from_stack(state, 2).ok().unwrap_or(false);
    let (width, height) = frame_size(state, id, ignore)?;
    state.push(Val::Num(width as f64));
    state.push(Val::Num(height as f64));
    Ok(2)
}

fn read_size_input(state: &LuaState, index: i32) -> LuaResult<(f32, bool)> {
    let (value, secret) = super::super::secret_origin::unwrap_input(
        state,
        crate::lua_bridge::stack_val(state, index),
    )?;
    let number = if secret {
        match value {
            Val::Num(number) => number as f32,
            Val::Nil => 0.0,
            _ => {
                return Err(rilua::runtime_error(
                    "secret size input must contain a number or nil",
                ));
            }
        }
    } else {
        opt_f32(state, index)
    };
    Ok((number, secret))
}

pub fn set_size(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id(state, 1)?;
    let (width, secret_width) = read_size_input(state, 2)?;
    let (height, secret_height) = read_size_input(state, 3)?;
    if !can_change_protected_state_for(state, id) {
        emit_addon_action_blocked(state, id, "SetSize");
        return Ok(0);
    }
    let mut sim = borrow_state_mut(state)?;
    let Some(current) = current_explicit_size_state(&sim, id) else {
        return Ok(0);
    };

    if let Some(frame) = sim.widgets.get_mut(id) {
        frame.secret_width = secret_width;
        frame.secret_height = secret_height;
    }
    let size_changed = current.width != width || current.height != height;
    if !size_changed {
        if current.width_is_text_auto {
            clear_auto_width_flag(&mut sim, id);
        }
        if current.height_is_text_auto {
            clear_auto_height_flag(&mut sim, id);
        }
        return Ok(0);
    }

    apply_explicit_size(&mut sim, id, width, height);
    drop(sim);
    refresh_auto_text_height_after_width_change(state, id);
    mark_nearest_layout_parent_dirty(state, id);
    super::super::widgets::refresh_scroll_frames_for_resized_frame(state, id)?;
    Ok(0)
}

pub fn set_fixed_size(state: &mut LuaState) -> LuaResult<u32> {
    set_size(state)
}

pub fn set_width(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id(state, 1)?;
    let (width, secret_width) = read_size_input(state, 2)?;
    if !can_change_protected_state_for(state, id) {
        emit_addon_action_blocked(state, id, "SetWidth");
        return Ok(0);
    }
    let mut sim = borrow_state_mut(state)?;
    let Some(current) = current_explicit_size_state(&sim, id) else {
        return Ok(0);
    };

    if let Some(frame) = sim.widgets.get_mut(id) {
        frame.secret_width = secret_width;
    }
    if current.width == width {
        if current.width_is_text_auto {
            clear_auto_width_flag(&mut sim, id);
        }
        drop(sim);
        refresh_auto_text_width_after_zero_width(state, id);
        return Ok(0);
    }

    apply_explicit_width(&mut sim, id, width);
    drop(sim);
    refresh_auto_text_width_after_zero_width(state, id);
    mark_nearest_layout_parent_dirty(state, id);
    refresh_auto_text_height_after_width_change(state, id);
    super::super::widgets::refresh_scroll_frames_for_resized_frame(state, id)?;
    Ok(0)
}

pub fn set_height(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id(state, 1)?;
    let (height, secret_height) = read_size_input(state, 2)?;
    if !can_change_protected_state_for(state, id) {
        emit_addon_action_blocked(state, id, "SetHeight");
        return Ok(0);
    }
    let mut sim = borrow_state_mut(state)?;
    let Some(current) = current_explicit_size_state(&sim, id) else {
        return Ok(0);
    };

    if let Some(frame) = sim.widgets.get_mut(id) {
        frame.secret_height = secret_height;
    }
    if current.height == height {
        if current.height_is_text_auto {
            clear_auto_height_flag(&mut sim, id);
        }
        return Ok(0);
    }

    apply_explicit_height(&mut sim, id, height);
    drop(sim);
    refresh_auto_text_height_after_width_change(state, id);
    mark_nearest_layout_parent_dirty(state, id);
    super::super::widgets::refresh_scroll_frames_for_resized_frame(state, id)?;
    Ok(0)
}

pub(crate) fn mark_nearest_layout_parent_dirty(state: &mut LuaState, id: u64) {
    if is_loading_addon(state) {
        return;
    }

    let Some(helper) = layout_dirty_helper(state) else {
        return;
    };

    let ancestors = {
        let Ok(sim) = borrow_state(state) else {
            return;
        };
        let mut ancestors = Vec::new();
        let mut current = sim.widgets.get(id).and_then(|frame| frame.parent_id);
        while let Some(parent_id) = current {
            ancestors.push(parent_id);
            current = sim.widgets.get(parent_id).and_then(|frame| frame.parent_id);
        }
        ancestors
    };

    for ancestor_id in ancestors {
        let Ok(frame) = frame_ref(state, ancestor_id) else {
            continue;
        };
        let previous_on_updates = on_update_snapshot(state, ancestor_id);
        let result = call_function_state(state, helper, &[frame]);
        restore_custom_on_updates(state, previous_on_updates);
        if let Ok(Val::Bool(true)) = result {
            break;
        }
    }
}

fn on_update_snapshot(state: &mut LuaState, id: u64) -> Vec<(u64, Val)> {
    let ancestor_ids = {
        let Ok(sim) = borrow_state(state) else {
            return Vec::new();
        };
        let mut ancestor_ids = vec![id];
        let mut current = sim.widgets.get(id).and_then(|frame| frame.parent_id);
        while let Some(parent_id) = current {
            ancestor_ids.push(parent_id);
            current = sim.widgets.get(parent_id).and_then(|frame| frame.parent_id);
        }
        ancestor_ids
    };

    ancestor_ids
        .into_iter()
        .filter_map(|frame_id| {
            get_rilua_script(state, frame_id, "OnUpdate")
                .map(|previous_on_update| (frame_id, previous_on_update))
        })
        .collect()
}

fn restore_custom_on_updates(state: &mut LuaState, previous_on_updates: Vec<(u64, Val)>) {
    for (frame_id, previous_on_update) in previous_on_updates {
        let current_on_update = get_rilua_script(state, frame_id, "OnUpdate");
        if current_on_update != Some(previous_on_update) {
            set_rilua_script(state, frame_id, "OnUpdate", previous_on_update);
        }
    }
}

fn layout_dirty_helper(state: &mut LuaState) -> Option<Val> {
    let helper = table_get_static(
        state,
        Val::Table(state.global),
        "__wow_mark_layout_frame_dirty",
    );
    matches!(helper, Val::Function(_)).then_some(helper)
}

fn is_loading_addon(state: &mut LuaState) -> bool {
    borrow_state(state)
        .map(|sim| sim.loading_addon_index.is_some())
        .unwrap_or(false)
}
