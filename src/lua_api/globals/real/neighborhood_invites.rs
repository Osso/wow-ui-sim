//! Per-environment neighborhood invite preference; no persistence or invented event.

use crate::lua_api::methods::{borrow_state, borrow_state_mut};
use crate::lua_bridge::stack_val;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

fn get_auto_decline_neighborhood_invites(state: &mut LuaState) -> LuaResult<u32> {
    let decline = borrow_state(state)?.auto_decline_neighborhood_invites;
    state.push(Val::Bool(decline));
    Ok(1)
}

fn set_auto_decline_neighborhood_invites(state: &mut LuaState) -> LuaResult<u32> {
    // Validate the caller before interpreting the documented default argument.
    let value = rilua::table_security::unwrap_secret(state, stack_val(state, 1))?;
    let decline = match value {
        Val::Nil => false,
        Val::Bool(decline) => decline,
        _ => {
            return Err(rilua::runtime_error(
                "SetAutoDeclineNeighborhoodInvites requires a boolean",
            ));
        }
    };
    borrow_state_mut(state)?.auto_decline_neighborhood_invites = decline;
    Ok(0)
}

pub(crate) fn register_all(lua: &mut rilua::Lua) -> LuaResult<()> {
    LuaApiMut::register_function(
        lua,
        "GetAutoDeclineNeighborhoodInvites",
        get_auto_decline_neighborhood_invites,
    )?;
    LuaApiMut::register_function(
        lua,
        "SetAutoDeclineNeighborhoodInvites",
        set_auto_decline_neighborhood_invites,
    )
}
