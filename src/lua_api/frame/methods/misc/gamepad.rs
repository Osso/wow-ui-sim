//! Gamepad button/stick enable methods.

use crate::lua_api::frame::methods::methods_helpers::{
    can_change_protected_state_for, emit_addon_action_blocked,
};
use crate::lua_api::methods::{borrow_state, borrow_state_mut, frame_id_from_stack};
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val};

pub fn register(state: &mut LuaState, mt: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, mt, "EnableGamePadButton", enable_game_pad_button)?;
    table_set_rust_fn_static(state, mt, "EnableGamePadStick", enable_game_pad_stick)?;
    table_set_rust_fn_static(
        state,
        mt,
        "IsGamePadButtonEnabled",
        is_game_pad_button_enabled,
    )?;
    table_set_rust_fn_static(
        state,
        mt,
        "IsGamePadStickEnabled",
        is_game_pad_stick_enabled,
    )?;
    table_set_rust_fn_static(
        state,
        mt,
        "ShouldButtonPassThrough",
        should_button_pass_through,
    )?;
    Ok(())
}

pub fn enable_game_pad_button(state: &mut LuaState) -> LuaResult<u32> {
    set_game_pad_enabled(state, GamePadControl::Button)
}

pub fn enable_game_pad_stick(state: &mut LuaState) -> LuaResult<u32> {
    set_game_pad_enabled(state, GamePadControl::Stick)
}

pub fn is_game_pad_button_enabled(state: &mut LuaState) -> LuaResult<u32> {
    push_game_pad_enabled(state, GamePadControl::Button)
}

pub fn is_game_pad_stick_enabled(state: &mut LuaState) -> LuaResult<u32> {
    push_game_pad_enabled(state, GamePadControl::Stick)
}

enum GamePadControl {
    Button,
    Stick,
}

fn set_game_pad_enabled(state: &mut LuaState, control: GamePadControl) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let enabled = bool::from_stack(state, 2)?;
    if !can_change_protected_state_for(state, id) {
        let method = match control {
            GamePadControl::Button => "EnableGamePadButton",
            GamePadControl::Stick => "EnableGamePadStick",
        };
        emit_addon_action_blocked(state, id, method);
        return Ok(0);
    }
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut(id) {
        match control {
            GamePadControl::Button => frame.gamepad_button_enabled = enabled,
            GamePadControl::Stick => frame.gamepad_stick_enabled = enabled,
        }
    }
    Ok(0)
}

fn push_game_pad_enabled(state: &mut LuaState, control: GamePadControl) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let enabled = borrow_state(state)?
        .widgets
        .get(id)
        .map(|frame| match control {
            GamePadControl::Button => frame.gamepad_button_enabled,
            GamePadControl::Stick => frame.gamepad_stick_enabled,
        })
        .unwrap_or(false);
    state.push(Val::Bool(enabled));
    Ok(1)
}

pub fn should_button_pass_through(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let button = String::from_stack(state, 2)?;
    let val = borrow_state(state)?
        .widgets
        .get(id)
        .map(|frame| frame_has_pass_through_button(frame, &button))
        .unwrap_or(false);
    state.push(Val::Bool(val));
    Ok(1)
}

fn frame_has_pass_through_button(frame: &crate::widget::Frame, button: &str) -> bool {
    if button.bytes().all(|byte| !byte.is_ascii_uppercase()) {
        return frame.pass_through_buttons.contains(button);
    }

    frame
        .pass_through_buttons
        .contains(&button.to_ascii_lowercase())
}

#[cfg(test)]
mod tests {
    use super::frame_has_pass_through_button;
    use crate::widget::{Frame, WidgetType};

    #[test]
    fn frame_has_pass_through_button_matches_lowercase_and_mixed_case_names() {
        let mut frame = Frame::new(WidgetType::Button, Some("GamePadButton".to_string()), None);
        frame.pass_through_buttons.insert("leftbutton".to_string());

        assert!(frame_has_pass_through_button(&frame, "leftbutton"));
        assert!(frame_has_pass_through_button(&frame, "LeftButton"));
        assert!(!frame_has_pass_through_button(&frame, "RightButton"));
    }
}
