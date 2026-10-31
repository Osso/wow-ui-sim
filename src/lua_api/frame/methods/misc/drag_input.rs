//! Drag, input movement, and clamping methods.

use crate::lua_api::frame::methods::methods_helpers::{
    can_change_protected_state_for, emit_addon_action_blocked,
};
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, extract_frame_id, frame_id_from_stack, val_to_string,
};
use crate::lua_bridge::{FromStack, stack_val, table_set_rust_fn_static};
use crate::widget::AnchorPoint;
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val};

pub fn register(state: &mut LuaState, mt: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, mt, "AbortDrag", abort_drag)?;
    table_set_rust_fn_static(state, mt, "InterceptStartDrag", intercept_start_drag)?;
    table_set_rust_fn_static(state, mt, "IsDragging", is_dragging)?;
    table_set_rust_fn_static(state, mt, "RegisterForDrag", register_for_drag)?;
    table_set_rust_fn_static(state, mt, "SetMovable", set_movable)?;
    table_set_rust_fn_static(state, mt, "IsMovable", is_movable)?;
    table_set_rust_fn_static(state, mt, "SetResizable", set_resizable)?;
    table_set_rust_fn_static(state, mt, "IsResizable", is_resizable)?;
    table_set_rust_fn_static(state, mt, "StartMoving", start_moving)?;
    table_set_rust_fn_static(state, mt, "StartSizing", start_sizing)?;
    table_set_rust_fn_static(state, mt, "StopMovingOrSizing", stop_moving_or_sizing)?;
    table_set_rust_fn_static(state, mt, "SetUserPlaced", set_user_placed)?;
    table_set_rust_fn_static(state, mt, "IsUserPlaced", is_user_placed)?;
    table_set_rust_fn_static(state, mt, "SetClampedToScreen", set_clamped_to_screen)?;
    table_set_rust_fn_static(state, mt, "IsClampedToScreen", is_clamped_to_screen)?;
    Ok(())
}

pub fn abort_drag(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let mut sim = borrow_state_mut(state)?;
    if sim.active_drag_frame == Some(id) {
        sim.set_active_drag_frame(None);
    }
    Ok(0)
}

pub fn intercept_start_drag(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let delegate_val = stack_val(state, 2);
    let delegate_id = extract_frame_id(state, delegate_val);
    let result = resolve_intercept(state, id, delegate_id)?;
    state.push(Val::Bool(result));
    Ok(1)
}

fn resolve_intercept(state: &mut LuaState, id: u64, delegate_id: Option<u64>) -> LuaResult<bool> {
    let Some(delegate_id) = delegate_id else {
        return Ok(false);
    };
    let mut sim = borrow_state_mut(state)?;
    if sim.active_drag_frame != Some(id) {
        return Ok(false);
    }
    if sim.widgets.get(delegate_id).is_none() {
        return Ok(false);
    }
    sim.set_active_drag_frame(Some(delegate_id));
    Ok(true)
}

pub fn is_dragging(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let is_dragging = borrow_state(state)?.active_drag_frame == Some(id);
    state.push(Val::Bool(is_dragging));
    Ok(1)
}

pub fn register_for_drag(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let buttons = collect_string_varargs(state, 2);
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        frame.registered_drag_buttons.clear();
        frame.registered_drag_buttons.extend(buttons);
    }
    Ok(0)
}

fn collect_string_varargs(state: &mut LuaState, start: i32) -> Vec<String> {
    let mut out = Vec::new();
    let mut index = start;
    loop {
        let value = stack_val(state, index);
        if value == Val::Nil {
            break;
        }
        if let Some(s) = val_to_string(state, value) {
            out.push(s);
        }
        index += 1;
    }
    out
}

pub fn set_movable(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let movable = bool::from_stack(state, 2)?;
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        frame.movable = movable;
    }
    Ok(0)
}

pub fn is_movable(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let movable = borrow_state(state)?
        .widgets
        .get(id)
        .map(|f| f.movable)
        .unwrap_or(false);
    state.push(Val::Bool(movable));
    Ok(1)
}

pub fn set_resizable(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let resizable = bool::from_stack(state, 2)?;
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        frame.resizable = resizable;
    }
    Ok(0)
}

pub fn is_resizable(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let resizable = borrow_state(state)?
        .widgets
        .get(id)
        .map(|f| f.resizable)
        .unwrap_or(false);
    state.push(Val::Bool(resizable));
    Ok(1)
}

pub fn start_moving(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut_visual(id)
        && frame.movable
    {
        frame.is_moving = true;
        sim.set_active_drag_frame(Some(id));
    }
    Ok(0)
}

pub fn start_sizing(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let sizing_point = val_to_string(state, stack_val(state, 2))
        .and_then(|value| AnchorPoint::from_str(&value))
        .unwrap_or(AnchorPoint::BottomRight);
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut_visual(id)
        && frame.resizable
    {
        frame.is_sizing = true;
        frame.sizing_point = sizing_point;
        sim.set_active_drag_frame(Some(id));
    }
    Ok(0)
}

pub fn stop_moving_or_sizing(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        if frame.is_moving || frame.is_sizing {
            frame.user_placed = true;
        }
        frame.is_moving = false;
        frame.is_sizing = false;
    }
    if sim.active_drag_frame == Some(id) {
        sim.set_active_drag_frame(None);
    }
    Ok(0)
}

pub fn set_user_placed(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let user_placed = bool::from_stack(state, 2)?;
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        frame.user_placed = user_placed;
    }
    Ok(0)
}

pub fn is_user_placed(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let user_placed = borrow_state(state)?
        .widgets
        .get(id)
        .map(|f| f.user_placed)
        .unwrap_or(false);
    state.push(Val::Bool(user_placed));
    Ok(1)
}

pub fn set_clamped_to_screen(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let clamped = bool::from_stack(state, 2)?;
    if !can_change_protected_state_for(state, id) {
        emit_addon_action_blocked(state, id, "SetClampedToScreen");
        return Ok(0);
    }
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        frame.clamped_to_screen = clamped;
    }
    sim.widgets.mark_rect_dirty(id);
    Ok(0)
}

pub fn is_clamped_to_screen(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let clamped = borrow_state(state)?
        .widgets
        .get(id)
        .map(|f| f.clamped_to_screen)
        .unwrap_or(false);
    state.push(Val::Bool(clamped));
    Ok(1)
}
