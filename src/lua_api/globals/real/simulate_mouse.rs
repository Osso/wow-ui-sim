//! `SimulateMouseClick/Down/Up/Wheel` queue input for the GUI mouse dispatch path.
//!
//! Retail 12.0.7 restrictions (cached InputDocumentation + patch notes):
//! - Insecure callers must consume a gamepad limited-input hardware event. The
//!   simulator has no gamepad hardware event source, so insecure calls are refused.
//! - Every current mouse focus must not be forbidden, script inaccessible, or a
//!   protected frame while in combat. INFERRED mapping: forbidden = `Frame.forbidden`,
//!   script inaccessible = any `access_restrictions`, locked down = `is_protected`
//!   during `InCombatLockdown()`.
//!
//! INFERRED refused calls return nothing and queue nothing. Only LeftButton and
//! RightButton reach the dispatcher, which handles no other buttons.

use crate::lua_api::SimState;
use crate::lua_api::methods::{borrow_state, borrow_state_mut, val_to_string};
use crate::lua_bridge::stack_val;
use rilua::table_security::unwrap_secret;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val, runtime_error};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SimulatedMouseButton {
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SimulatedMouseInput {
    Down(SimulatedMouseButton),
    Up(SimulatedMouseButton),
    Wheel(f32),
}

fn read_button(state: &LuaState, function: &str) -> LuaResult<SimulatedMouseButton> {
    let value = unwrap_secret(state, stack_val(state, 1))?;
    match val_to_string(state, value).as_deref() {
        Some("LeftButton") => Ok(SimulatedMouseButton::Left),
        Some("RightButton") => Ok(SimulatedMouseButton::Right),
        _ => Err(runtime_error(format!(
            "{function}: button must be \"LeftButton\" or \"RightButton\""
        ))),
    }
}

fn read_wheel_delta(state: &LuaState) -> LuaResult<f32> {
    match unwrap_secret(state, stack_val(state, 1))? {
        Val::Num(delta) if delta.is_finite() => Ok(delta as f32),
        _ => Err(runtime_error(
            "SimulateMouseWheel: delta must be a finite number",
        )),
    }
}

fn focus_blocks_simulation(sim: &SimState) -> bool {
    let Some(frame) = sim.hovered_frame.and_then(|id| sim.widgets.get(id)) else {
        return false;
    };
    frame.forbidden
        || frame.access_restrictions != 0
        || (frame.is_protected && sim.player.in_combat)
}

fn simulation_allowed(state: &LuaState) -> LuaResult<bool> {
    if !rilua::api::state_is_secure(state) {
        return Ok(false);
    }
    Ok(!focus_blocks_simulation(&*borrow_state(state)?))
}

fn queue(state: &LuaState, inputs: &[SimulatedMouseInput]) -> LuaResult<u32> {
    if simulation_allowed(state)? {
        borrow_state_mut(state)?
            .simulated_mouse_inputs
            .extend(inputs.iter().copied());
    }
    Ok(0)
}

fn simulate_mouse_click(state: &mut LuaState) -> LuaResult<u32> {
    let button = read_button(state, "SimulateMouseClick")?;
    use SimulatedMouseInput::{Down, Up};
    queue(state, &[Down(button), Up(button)])
}

fn simulate_mouse_down(state: &mut LuaState) -> LuaResult<u32> {
    let button = read_button(state, "SimulateMouseDown")?;
    queue(state, &[SimulatedMouseInput::Down(button)])
}

fn simulate_mouse_up(state: &mut LuaState) -> LuaResult<u32> {
    let button = read_button(state, "SimulateMouseUp")?;
    queue(state, &[SimulatedMouseInput::Up(button)])
}

fn simulate_mouse_wheel(state: &mut LuaState) -> LuaResult<u32> {
    let delta = read_wheel_delta(state)?;
    queue(state, &[SimulatedMouseInput::Wheel(delta)])
}

pub fn register_all(lua: &mut rilua::Lua) -> crate::Result<()> {
    LuaApiMut::register_function(lua, "SimulateMouseClick", simulate_mouse_click)?;
    LuaApiMut::register_function(lua, "SimulateMouseDown", simulate_mouse_down)?;
    LuaApiMut::register_function(lua, "SimulateMouseUp", simulate_mouse_up)?;
    LuaApiMut::register_function(lua, "SimulateMouseWheel", simulate_mouse_wheel)?;
    Ok(())
}
