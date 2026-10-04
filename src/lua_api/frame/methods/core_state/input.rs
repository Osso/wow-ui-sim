//! Mouse and keyboard input-enable methods.

use super::helpers::{arg_bool, frame_id};
use crate::lua_api::methods::{borrow_state, borrow_state_mut, val_to_string};
use crate::lua_bridge::stack_val;
use crate::widget::Frame;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub fn enable_mouse(state: &mut LuaState) -> LuaResult<u32> {
    set_frame_input_flag(state, Frame::set_mouse_enabled)
}

fn set_frame_input_flag(
    state: &mut LuaState,
    apply: impl FnOnce(&mut Frame, bool),
) -> LuaResult<u32> {
    let id = frame_id(state, 1)?;
    let enable = arg_bool(state, 2);
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut(id) {
        let was_mouse_eligible = frame.accepts_mouse();
        apply(frame, enable);
        let is_mouse_eligible = frame.accepts_mouse();
        if was_mouse_eligible != is_mouse_eligible {
            sim.queue_hit_grid_eligibility_change(id);
        }
    }
    Ok(0)
}

pub fn is_mouse_enabled(state: &mut LuaState) -> LuaResult<u32> {
    push_frame_input_flag(state, |frame| {
        frame.mouse_enabled || frame.mouse_motion_enabled
    })
}

fn push_frame_input_flag(
    state: &mut LuaState,
    read: impl FnOnce(&Frame) -> bool,
) -> LuaResult<u32> {
    let id = frame_id(state, 1)?;
    let sim = borrow_state(state)?;
    let result = sim.widgets.get(id).map(read).unwrap_or(false);
    drop(sim);
    state.push(Val::Bool(result));
    Ok(1)
}

pub fn enable_mouse_wheel(state: &mut LuaState) -> LuaResult<u32> {
    set_frame_input_flag(state, |frame, enable| frame.mouse_wheel_enabled = enable)
}

pub fn is_mouse_wheel_enabled(state: &mut LuaState) -> LuaResult<u32> {
    push_frame_input_flag(state, |frame| frame.mouse_wheel_enabled)
}

pub fn enable_keyboard(state: &mut LuaState) -> LuaResult<u32> {
    set_frame_input_flag(state, |frame, enable| frame.keyboard_enabled = enable)
}

pub fn is_keyboard_enabled(state: &mut LuaState) -> LuaResult<u32> {
    push_frame_input_flag(state, |frame| frame.keyboard_enabled)
}

pub fn register_for_mouse(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id(state, 1)?;
    let buttons = collect_mouse_registration_args(state, 2);
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        frame.registered_mouse_buttons = buttons;
    }
    Ok(0)
}

fn collect_mouse_registration_args(
    state: &mut LuaState,
    start: i32,
) -> std::collections::HashSet<String> {
    let mut buttons = std::collections::HashSet::new();
    let mut index = start;
    loop {
        let value = stack_val(state, index);
        if value == Val::Nil {
            break;
        }
        if let Some(button) = val_to_string(state, value) {
            buttons.insert(button);
        }
        index += 1;
    }
    buttons
}

pub fn enable_mouse_motion(state: &mut LuaState) -> LuaResult<u32> {
    set_frame_input_flag(state, |frame, enable| frame.mouse_motion_enabled = enable)
}

pub fn is_mouse_motion_enabled(state: &mut LuaState) -> LuaResult<u32> {
    push_frame_input_flag(state, |frame| frame.mouse_motion_enabled)
}

pub fn set_mouse_motion_enabled(state: &mut LuaState) -> LuaResult<u32> {
    enable_mouse_motion(state)
}

pub fn set_mouse_click_enabled(state: &mut LuaState) -> LuaResult<u32> {
    set_frame_input_flag(state, |frame, enable| frame.mouse_enabled = enable)
}

pub fn is_mouse_click_enabled(state: &mut LuaState) -> LuaResult<u32> {
    push_frame_input_flag(state, |frame| frame.mouse_enabled)
}
