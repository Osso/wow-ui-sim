//! UI visibility globals (`SetUIVisibility`, `SetInWorldUIVisibility`).
//!
//! Blizzard panel manager calls `SetUIVisibility(true)` when returning from a
//! fullscreen panel back to normal UI layout. Missing this global causes the
//! maximize world-map flow to throw and leave panel state half-updated.
//!
//! `SetInWorldUIVisibility` is currently a compatibility no-op: the simulator
//! does not model a separate in-world UI layer yet.

use crate::lua_api::methods::borrow_state_mut;
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult};

fn set_ui_parent_visibility(state: &mut LuaState, visible: bool) -> LuaResult<()> {
    let mut sim = borrow_state_mut(state)?;
    let Some(ui_parent_id) = sim.widgets.get_id_by_name("UIParent") else {
        return Ok(());
    };
    sim.set_frame_visible(ui_parent_id, visible);
    Ok(())
}

/// `SetUIVisibility(visible)` — toggles UIParent visibility.
fn set_ui_visibility(state: &mut LuaState) -> LuaResult<u32> {
    let visible = Option::<bool>::from_stack(state, 1)?.unwrap_or(false);
    set_ui_parent_visibility(state, visible)?;
    Ok(0)
}

/// `SetInWorldUIVisibility(visible)` — currently a no-op compatibility shim.
fn set_in_world_ui_visibility(state: &mut LuaState) -> LuaResult<u32> {
    let _ = Option::<bool>::from_stack(state, 1)?;
    Ok(0)
}

pub fn register_all(lua: &mut rilua::Lua) -> LuaResult<()> {
    let state = lua.state_mut();
    table_set_rust_fn_static(state, state.global, "SetUIVisibility", set_ui_visibility)?;
    table_set_rust_fn_static(
        state,
        state.global,
        "SetInWorldUIVisibility",
        set_in_world_ui_visibility,
    )?;
    Ok(())
}
