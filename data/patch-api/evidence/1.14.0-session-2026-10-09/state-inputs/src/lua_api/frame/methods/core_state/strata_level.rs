//! Frame strata, frame level, and toplevel methods.

use super::helpers::{arg_bool, frame_id};
use crate::lua_api::frame::methods::methods_helpers::{
    can_change_protected_state_for, emit_addon_action_blocked,
};
use crate::lua_api::methods::{borrow_state, borrow_state_mut, create_string};
use crate::lua_bridge::FromStack;
use crate::widget::Frame;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub fn set_frame_strata(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id(state, 1)?;
    let strata = String::from_stack(state, 2)?;
    let Some(strata) = crate::widget::FrameStrata::from_str(&strata) else {
        return Ok(0);
    };
    if !can_change_protected_state_for(state, id) {
        emit_addon_action_blocked(state, id, "SetFrameStrata");
        return Ok(0);
    }
    let mut sim = borrow_state_mut(state)?;
    let mut strata_changed = false;
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        strata_changed |= frame.frame_strata != strata;
        frame.frame_strata = strata;
        frame.has_fixed_frame_strata = true;
    }
    let mut queue: Vec<u64> = sim
        .widgets
        .get(id)
        .map(|f| f.children.clone())
        .unwrap_or_default();
    while let Some(child_id) = queue.pop() {
        let Some(child) = sim.widgets.get_mut_visual(child_id) else {
            continue;
        };
        if child.has_fixed_frame_strata {
            continue;
        }
        strata_changed |= child.frame_strata != strata;
        child.frame_strata = strata;
        queue.extend(child.children.iter().copied());
    }
    if strata_changed {
        sim.invalidate_strata_buckets();
        // Strata is part of the hit-grid render-order key; re-insert the
        // subtree at its new position.
        sim.queue_hit_grid_eligibility_change(id);
    }
    Ok(0)
}

pub fn get_frame_strata(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id(state, 1)?;
    let strata = frame_strata_name(state, id)?;
    let val = create_string(state, strata);
    state.push(val);
    Ok(1)
}

fn frame_strata_name(state: &mut LuaState, id: u64) -> LuaResult<&'static str> {
    let sim = borrow_state(state)?;
    let strata = sim
        .widgets
        .get(id)
        .map(|frame| frame.frame_strata.as_str())
        .unwrap_or("MEDIUM");
    Ok(strata)
}

pub fn set_fixed_frame_strata(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id(state, 1)?;
    let fixed = arg_bool(state, 2);
    if !can_change_protected_state_for(state, id) {
        emit_addon_action_blocked(state, id, "SetFixedFrameStrata");
        return Ok(0);
    }
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        frame.has_fixed_frame_strata = fixed;
    }
    Ok(0)
}

pub fn has_fixed_frame_strata(state: &mut LuaState) -> LuaResult<u32> {
    push_frame_bool(state, |frame| frame.has_fixed_frame_strata)
}

fn push_frame_bool(state: &mut LuaState, read: impl FnOnce(&Frame) -> bool) -> LuaResult<u32> {
    let id = frame_id(state, 1)?;
    let sim = borrow_state(state)?;
    let result = sim.widgets.get(id).map(read).unwrap_or(false);
    drop(sim);
    state.push(Val::Bool(result));
    Ok(1)
}

pub fn set_frame_level(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id(state, 1)?;
    let level = i32::from_stack(state, 2)?;
    if !can_change_protected_state_for(state, id) {
        emit_addon_action_blocked(state, id, "SetFrameLevel");
        return Ok(0);
    }
    let mut sim = borrow_state_mut(state)?;
    let Some(current_level) = sim.widgets.get(id).map(|frame| frame.frame_level) else {
        return Ok(0);
    };
    if current_level == level {
        return Ok(0);
    }
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        frame.frame_level = level;
    }
    super::super::methods_hierarchy::propagate_strata_level_pub(&mut sim.widgets, id);
    sim.invalidate_strata_buckets();
    // Frame level is part of the hit-grid render-order key; re-insert the
    // subtree at its new position.
    sim.queue_hit_grid_eligibility_change(id);
    Ok(0)
}

pub fn get_frame_level(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id(state, 1)?;
    let sim = borrow_state(state)?;
    let result = sim.widgets.get(id).map(|f| f.frame_level).unwrap_or(0);
    drop(sim);
    state.push(Val::Num(result as f64));
    Ok(1)
}

pub fn set_fixed_frame_level(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id(state, 1)?;
    let fixed = arg_bool(state, 2);
    if !can_change_protected_state_for(state, id) {
        emit_addon_action_blocked(state, id, "SetFixedFrameLevel");
        return Ok(0);
    }
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        frame.has_fixed_frame_level = fixed;
    }
    Ok(0)
}

pub fn has_fixed_frame_level(state: &mut LuaState) -> LuaResult<u32> {
    push_frame_bool(state, |frame| frame.has_fixed_frame_level)
}

pub fn set_toplevel(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id(state, 1)?;
    let toplevel = arg_bool(state, 2);
    if !can_change_protected_state_for(state, id) {
        emit_addon_action_blocked(state, id, "SetToplevel");
        return Ok(0);
    }
    borrow_state_mut(state)?.set_frame_toplevel(id, toplevel);
    Ok(0)
}

pub fn is_toplevel(state: &mut LuaState) -> LuaResult<u32> {
    push_frame_bool(state, |frame| frame.toplevel)
}
