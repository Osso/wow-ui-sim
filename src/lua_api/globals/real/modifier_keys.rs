//! Modifier-key probes backed by `SimState::modifier_keys`.
//!
//! WoW exposes five globals: `IsShiftKeyDown` / `IsControlKeyDown` /
//! `IsAltKeyDown` / `IsMetaKeyDown` read individual keys; `IsModifierKeyDown`
//! returns true iff any of shift/control/alt is held (meta is *not* included
//! — Blizzard keeps that on its own probe).
//!
//! All keys default to `false` in the sim because no physical keyboard state
//! is captured in headless tests. Admin API
//! `A_Admin.SetShiftKeyDown(b)` / `SetControlKeyDown` / `SetAltKeyDown` /
//! `SetMetaKeyDown` toggles them for tests that want to exercise
//! modifier-aware UI paths (e.g. `IsModifiedClick("CHATLINK")`).

use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub fn is_key_down(state: &mut LuaState) -> LuaResult<u32> {
    let key = String::from_stack(state, 1)?;
    let key = crate::lua_api::key_dispatch::normalized_key_name(&key);
    let down = {
        let sim = borrow_state(state)?;
        match key.as_str() {
            "SHIFT" | "LSHIFT" | "RSHIFT" => sim.modifier_keys.shift,
            "CTRL" | "CONTROL" | "LCTRL" | "RCTRL" => sim.modifier_keys.control,
            "ALT" | "LALT" | "RALT" => sim.modifier_keys.alt,
            "META" | "LMETA" | "RMETA" => sim.modifier_keys.meta,
            "LEFTBUTTON" | "RIGHTBUTTON" | "MIDDLEBUTTON" | "BUTTON4" | "BUTTON5" => {
                sim.mouse_buttons.is_down(Some(&key))
            }
            _ => sim.pressed_keys.contains(&key),
        }
    };
    state.push(if key.is_empty() {
        Val::Nil
    } else {
        Val::Bool(down)
    });
    Ok(1)
}

pub fn is_shift_key_down(state: &mut LuaState) -> LuaResult<u32> {
    let down = borrow_state(state)?.modifier_keys.shift;
    state.push(Val::Bool(down));
    Ok(1)
}

pub fn is_control_key_down(state: &mut LuaState) -> LuaResult<u32> {
    let down = borrow_state(state)?.modifier_keys.control;
    state.push(Val::Bool(down));
    Ok(1)
}

pub fn is_alt_key_down(state: &mut LuaState) -> LuaResult<u32> {
    let down = borrow_state(state)?.modifier_keys.alt;
    state.push(Val::Bool(down));
    Ok(1)
}

fn push_modifier_side(
    state: &mut LuaState,
    accessor: impl FnOnce(&crate::lua_api::SimState) -> bool,
) -> LuaResult<u32> {
    let down = {
        let sim = borrow_state(state)?;
        accessor(&sim)
    };
    state.push(Val::Bool(down));
    Ok(1)
}

pub fn is_left_shift_key_down(state: &mut LuaState) -> LuaResult<u32> {
    push_modifier_side(state, |sim| sim.modifier_keys.shift)
}

pub fn is_right_shift_key_down(state: &mut LuaState) -> LuaResult<u32> {
    push_modifier_side(state, |sim| sim.modifier_keys.shift)
}

pub fn is_left_control_key_down(state: &mut LuaState) -> LuaResult<u32> {
    push_modifier_side(state, |sim| sim.modifier_keys.control)
}

pub fn is_right_control_key_down(state: &mut LuaState) -> LuaResult<u32> {
    push_modifier_side(state, |sim| sim.modifier_keys.control)
}

pub fn is_left_alt_key_down(state: &mut LuaState) -> LuaResult<u32> {
    push_modifier_side(state, |sim| sim.modifier_keys.alt)
}

pub fn is_right_alt_key_down(state: &mut LuaState) -> LuaResult<u32> {
    push_modifier_side(state, |sim| sim.modifier_keys.alt)
}

pub fn is_meta_key_down(state: &mut LuaState) -> LuaResult<u32> {
    let down = borrow_state(state)?.modifier_keys.meta;
    state.push(Val::Bool(down));
    Ok(1)
}

pub fn is_modifier_key_down(state: &mut LuaState) -> LuaResult<u32> {
    let any = borrow_state(state)?.modifier_keys.any_modifier();
    state.push(Val::Bool(any));
    Ok(1)
}

pub fn is_modified_click(state: &mut LuaState) -> LuaResult<u32> {
    let _action = crate::lua_bridge::stack_val(state, 1);
    let any = borrow_state(state)?.modifier_keys.any_modifier();
    state.push(Val::Bool(any));
    Ok(1)
}

pub fn register_all(lua: &mut rilua::Lua) -> LuaResult<()> {
    use rilua::LuaApiMut;
    let state = lua.state_mut();
    let g = state.global;
    table_set_rust_fn_static(state, g, "IsKeyDown", is_key_down)?;
    table_set_rust_fn_static(state, g, "IsShiftKeyDown", is_shift_key_down)?;
    table_set_rust_fn_static(state, g, "IsControlKeyDown", is_control_key_down)?;
    table_set_rust_fn_static(state, g, "IsAltKeyDown", is_alt_key_down)?;
    table_set_rust_fn_static(state, g, "IsLeftShiftKeyDown", is_left_shift_key_down)?;
    table_set_rust_fn_static(state, g, "IsRightShiftKeyDown", is_right_shift_key_down)?;
    table_set_rust_fn_static(state, g, "IsLeftControlKeyDown", is_left_control_key_down)?;
    table_set_rust_fn_static(state, g, "IsRightControlKeyDown", is_right_control_key_down)?;
    table_set_rust_fn_static(state, g, "IsLeftAltKeyDown", is_left_alt_key_down)?;
    table_set_rust_fn_static(state, g, "IsRightAltKeyDown", is_right_alt_key_down)?;
    table_set_rust_fn_static(state, g, "IsMetaKeyDown", is_meta_key_down)?;
    table_set_rust_fn_static(state, g, "IsModifierKeyDown", is_modifier_key_down)?;
    table_set_rust_fn_static(state, g, "IsModifiedClick", is_modified_click)?;
    Ok(())
}
