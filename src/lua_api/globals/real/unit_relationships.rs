//! Native controlled-player/group token classification, independent of unit existence.

use crate::lua_api::methods::val_to_string;
use crate::lua_bridge::stack_val;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

fn is_controlled_or_group_token(unit: &str) -> bool {
    if matches!(unit, "player" | "pet" | "vehicle") {
        return true;
    }
    for (prefix, maximum) in [("partypet", 4), ("raidpet", 40), ("party", 4), ("raid", 40)] {
        if let Some(suffix) = unit.strip_prefix(prefix) {
            return !suffix.starts_with('0')
                && suffix.bytes().all(|byte| byte.is_ascii_digit())
                && suffix
                    .parse::<u8>()
                    .is_ok_and(|index| (1..=maximum).contains(&index));
        }
    }
    false
}

fn unit_is_player_controlled_or_group_member(state: &mut LuaState) -> LuaResult<u32> {
    let result = val_to_string(state, stack_val(state, 1))
        .is_some_and(|unit| is_controlled_or_group_token(&unit));
    state.push(Val::Bool(result));
    Ok(1)
}

pub fn register_all(lua: &mut rilua::Lua) -> LuaResult<()> {
    LuaApiMut::register_function(
        lua,
        "UnitIsPlayerControlledOrGroupMember",
        unit_is_player_controlled_or_group_member,
    )
}
