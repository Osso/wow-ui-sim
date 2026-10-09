//! Nullable player orientation, independent of model-widget transforms.

use crate::lua_api::methods::borrow_state;
#[cfg(feature = "client-wowforever")]
use crate::lua_api::methods::borrow_state_mut;
#[cfg(feature = "client-wowforever")]
use crate::lua_bridge::stack_val;
#[cfg(feature = "client-wowforever")]
use rilua::runtime_error;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

fn get_player_facing(state: &mut LuaState) -> LuaResult<u32> {
    let facing = borrow_state(state)?.player.facing;
    state.push(facing.map_or(Val::Nil, Val::Num));
    Ok(1)
}

#[cfg(feature = "client-wowforever")]
pub(crate) fn set_player_facing(state: &mut LuaState) -> LuaResult<u32> {
    let facing = match stack_val(state, 1) {
        Val::Nil => None,
        Val::Num(value) if value.is_finite() => Some(value),
        _ => {
            return Err(runtime_error(
                "A_Admin.SetPlayerFacing expects a finite number or nil",
            ));
        }
    };
    borrow_state_mut(state)?.player.facing = facing;
    Ok(0)
}

pub(crate) fn register_all(lua: &mut rilua::Lua) -> LuaResult<()> {
    LuaApiMut::register_function(lua, "GetPlayerFacing", get_player_facing)
}
