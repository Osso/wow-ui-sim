//! INFERRED legacy Battle.net friend counts from supplied friend and game-account state.

use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

pub(crate) fn register(lua: &mut rilua::Lua) -> LuaResult<()> {
    let state = lua.state_mut();
    table_set_rust_fn_static(state, state.global, "BNGetNumFriends", get_num_friends)?;
    table_set_rust_fn_static(
        state,
        state.global,
        "BNGetNumFriendInvites",
        get_num_friend_invites,
    )
}

fn get_num_friends(state: &mut LuaState) -> LuaResult<u32> {
    let counts = {
        let sim = borrow_state(state)?;
        let friends = sim.bnet_friends.iter().filter(|friend| friend.is_friend);
        let mut counts = [0, 0, 0, 0];
        for friend in friends {
            let online = friend.game_accounts.iter().any(|account| account.is_online);
            counts[0] += 1;
            counts[1] += i32::from(online);
            counts[2] += i32::from(friend.is_favorite);
            counts[3] += i32::from(online && friend.is_favorite);
        }
        counts
    };
    for count in counts {
        state.push(Val::Num(count as f64));
    }
    Ok(4)
}

fn get_num_friend_invites(state: &mut LuaState) -> LuaResult<u32> {
    let count = borrow_state(state)?.bnet_friend_invites.len();
    state.push(Val::Num(count as f64));
    Ok(1)
}
