//! Guild membership predicates over the host guild roster (`A_Admin.SetGuildRoster`).
//! Retail/Forever UnitDocumentation declare byte-identical `UnitIsInMyGuild` and
//! `IsPlayerInGuildFromGUID` blocks. Membership is keyed by roster GUIDs; the player
//! counts as a member of their own guild. A roster row without a GUID never matches.

use crate::lua_api::SimState;
use crate::lua_api::globals::unit_misc::existing_guid_for_unit;
use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::FromStack;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

pub(crate) fn register_all(lua: &mut rilua::Lua) -> LuaResult<()> {
    LuaApiMut::register_function(lua, "UnitIsInMyGuild", unit_is_in_my_guild)?;
    LuaApiMut::register_function(lua, "IsPlayerInGuildFromGUID", is_player_in_guild_from_guid)
}

fn guid_in_my_guild(sim: &SimState, guid: &str) -> bool {
    if sim.world.guild_name.is_none() {
        return false;
    }
    let player = existing_guid_for_unit(sim, "player");
    player.as_deref() == Some(guid)
        || sim
            .world
            .guild_members
            .iter()
            .any(|member| member.guid.as_deref() == Some(guid))
}

fn unit_is_in_my_guild(state: &mut LuaState) -> LuaResult<u32> {
    let unit = String::from_stack(state, 1)?;
    let member = {
        let sim = borrow_state(state)?;
        existing_guid_for_unit(&sim, &unit).is_some_and(|guid| guid_in_my_guild(&sim, &guid))
    };
    state.push(Val::Bool(member));
    Ok(1)
}

fn is_player_in_guild_from_guid(state: &mut LuaState) -> LuaResult<u32> {
    let guid = String::from_stack(state, 1)?;
    let member = guid_in_my_guild(&*borrow_state(state)?, &guid);
    state.push(Val::Bool(member));
    Ok(1)
}
