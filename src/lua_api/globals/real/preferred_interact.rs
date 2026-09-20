//! Gamepad interaction preference stores a resolved identity, not a moving alias.
use crate::lua_api::globals::targeting_verbs::resolve_unit_snapshot;
use crate::lua_api::methods::borrow_state_mut;
use crate::lua_bridge::FromStack;
use rilua::{LuaApiMut, LuaResult, vm::state::LuaState};

pub fn register_all(lua: &mut rilua::Lua) -> crate::Result<()> {
    if cfg!(feature = "client-wowforever") {
        LuaApiMut::register_function(
            lua.state_mut(),
            "SetPreferredGamepadInteractTarget",
            set_preferred_target,
        )?;
    }
    Ok(())
}

fn set_preferred_target(state: &mut LuaState) -> LuaResult<u32> {
    let token = Option::<String>::from_stack(state, 1)?;
    let mut sim = borrow_state_mut(state)?;
    sim.preferred_gamepad_interact_guid = token
        .as_deref()
        .and_then(|token| resolve_unit_snapshot(&sim, token))
        .map(|unit| unit.guid);
    Ok(0)
}
