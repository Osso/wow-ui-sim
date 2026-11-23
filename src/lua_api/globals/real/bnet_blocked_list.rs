//! Legacy Battle.net block-list queries for the FriendsFrame ignore window.
//! R88eh: one-based rows, identifier-backed selection, and native invalid-index errors.

use crate::lua_api::methods::{borrow_state, borrow_state_mut, create_string};
use crate::lua_bridge::FromStack;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

pub(crate) fn register_all(lua: &mut rilua::Lua) -> LuaResult<()> {
    LuaApiMut::register_function(lua, "BNGetNumBlocked", get_num_blocked)?;
    LuaApiMut::register_function(lua, "BNGetBlockedInfo", get_blocked_info)?;
    LuaApiMut::register_function(lua, "BNGetSelectedBlock", get_selected_block)?;
    LuaApiMut::register_function(lua, "BNSetSelectedBlock", set_selected_block)
}

fn get_num_blocked(state: &mut LuaState) -> LuaResult<u32> {
    let count = borrow_state(state)?.bnet_blocked_entries.len();
    state.push(Val::Num(count as f64));
    Ok(1)
}

fn get_blocked_info(state: &mut LuaState) -> LuaResult<u32> {
    let index = i32::from_stack(state, 1)?;
    let entry = {
        let sim = borrow_state(state)?;
        index
            .checked_sub(1)
            .and_then(|index| usize::try_from(index).ok())
            .and_then(|index| sim.bnet_blocked_entries.get(index).cloned())
    };
    match entry {
        Some(entry) => {
            let name = create_string(state, &entry.name);
            state.push(Val::Num(entry.blocked_id as f64));
            state.push(name);
        }
        None => return Err(rilua::runtime_error("BNGetBlockedInfo: invalid index")),
    }
    Ok(2)
}

fn get_selected_block(state: &mut LuaState) -> LuaResult<u32> {
    let sim = borrow_state(state)?;
    let selected_index = sim
        .selected_bnet_block
        .and_then(|identity| {
            sim.bnet_blocked_entries
                .iter()
                .position(|entry| entry.blocked_id == identity)
        })
        .map_or(0, |index| index + 1);
    state.push(Val::Num(selected_index as f64));
    Ok(1)
}

fn set_selected_block(state: &mut LuaState) -> LuaResult<u32> {
    let index = i32::from_stack(state, 1)?;
    let mut sim = borrow_state_mut(state)?;
    sim.selected_bnet_block = index
        .checked_sub(1)
        .and_then(|index| usize::try_from(index).ok())
        .and_then(|index| sim.bnet_blocked_entries.get(index))
        .map(|entry| entry.blocked_id);
    Ok(0)
}
