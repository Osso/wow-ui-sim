//! Follower-level XP tables from the pinned GarrFollowerLevelXP catalogue.

use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::{create_table, table_set_num};
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    if !matches!(ACTIVE, ClientProfile::Retail | ClientProfile::Ptr) {
        return Ok(());
    }
    let namespace = super::ensure_namespace(state, "C_Garrison")?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetFollowerXPTable",
        get_follower_xp_table,
    )
}

fn get_follower_xp_table(state: &mut LuaState) -> LuaResult<u32> {
    let follower_type = i32::from_stack(state, 1)?;
    let result = create_table(state);
    let Val::Table(table) = result else {
        unreachable!()
    };
    for &(kind, level, xp) in sim_data::garrison_follower_xp::FOLLOWER_LEVEL_XP {
        if kind == follower_type {
            table_set_num(state, table, level as f64, Val::Num(xp as f64));
        }
    }
    state.push(result);
    Ok(1)
}
