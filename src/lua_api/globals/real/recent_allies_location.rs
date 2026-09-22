//! Per-environment location preference; no cross-process persistence policy.

use crate::lua_api::methods::{borrow_state, borrow_state_mut};
use crate::lua_api::script_helpers::fire_named_event_state;
use crate::lua_bridge::stack_val;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

fn get_allow_recent_allies_see_location(state: &mut LuaState) -> LuaResult<u32> {
    let allow = borrow_state(state)?.allow_recent_allies_see_location;
    state.push(Val::Bool(allow));
    Ok(1)
}

fn set_allow_recent_allies_see_location(state: &mut LuaState) -> LuaResult<u32> {
    // Reuse VM caller validation for the documented AllowedWhenUntainted argument.
    let value = rilua::table_security::unwrap_secret(state, stack_val(state, 1))?;
    let Val::Bool(allow) = value else {
        return Err(rilua::runtime_error(
            "SetAllowRecentAlliesSeeLocation requires a boolean",
        ));
    };
    {
        let mut sim = borrow_state_mut(state)?;
        // Inferred: Settings writes the current value back while handling this event.
        if sim.allow_recent_allies_see_location == allow {
            return Ok(0);
        }
        sim.allow_recent_allies_see_location = allow;
    }
    fire_named_event_state(state, "LET_RECENT_ALLIES_SEE_LOCATION_SETTING_UPDATED", &[]);
    Ok(0)
}

pub(crate) fn register_all(lua: &mut rilua::Lua) -> LuaResult<()> {
    LuaApiMut::register_function(
        lua,
        "GetAllowRecentAlliesSeeLocation",
        get_allow_recent_allies_see_location,
    )?;
    LuaApiMut::register_function(
        lua,
        "SetAllowRecentAlliesSeeLocation",
        set_allow_recent_allies_see_location,
    )
}
