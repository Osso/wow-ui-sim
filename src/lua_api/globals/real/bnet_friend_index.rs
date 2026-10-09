//! Legacy account-ID lookup over the same ordered friend model as C_BattleNet.

use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::FromStack;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

pub(crate) fn register_all(lua: &mut rilua::Lua) -> LuaResult<()> {
    LuaApiMut::register_function(lua, "BNGetFriendIndex", get_friend_index)
}

fn get_friend_index(state: &mut LuaState) -> LuaResult<u32> {
    let account_id = i32::from_stack(state, 1)?;
    let index = borrow_state(state)?
        .bnet_friends
        .iter()
        .position(|friend| friend.bnet_account_id == account_id);
    // INFERRED: unknown account IDs return nil; native error/return parity unproven.
    state.push(index.map_or(Val::Nil, |index| Val::Num((index + 1) as f64)));
    Ok(1)
}
