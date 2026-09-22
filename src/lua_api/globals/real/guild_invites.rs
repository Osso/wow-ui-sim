//! Forever guild invite setter sharing the existing guild-probe state.

use crate::lua_api::methods::borrow_state_mut;
use crate::lua_bridge::stack_val;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

fn set_auto_decline_guild_invites(state: &mut LuaState) -> LuaResult<u32> {
    // Validate the caller before interpreting the documented default argument.
    let value = rilua::table_security::unwrap_secret(state, stack_val(state, 1))?;
    let decline = match value {
        Val::Nil => false,
        Val::Bool(decline) => decline,
        _ => {
            return Err(rilua::runtime_error(
                "SetAutoDeclineGuildInvites requires a boolean",
            ));
        }
    };
    borrow_state_mut(state)?.auto_decline_guild_invites = decline;
    Ok(0)
}

pub(crate) fn register_all(lua: &mut rilua::Lua) -> LuaResult<()> {
    LuaApiMut::register_function(
        lua,
        "SetAutoDeclineGuildInvites",
        set_auto_decline_guild_invites,
    )
}
