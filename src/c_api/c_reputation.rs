//! State-backed paragon reputation queries.

use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::stack_val;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

/// Seeded paragon payload; presence in `SimState.faction_paragon` marks a faction
/// as paragon. Storage level is explicit state, not derived from reputation.
#[derive(Clone, Debug)]
pub struct FactionParagonInfo {
    pub current_value: i32,
    pub threshold: i32,
    pub reward_quest_id: i32,
    pub has_reward_pending: bool,
    pub too_low_level_for_paragon: bool,
    pub paragon_storage_level: i32,
}

pub(crate) fn get_faction_paragon_info(state: &mut LuaState) -> LuaResult<u32> {
    let Val::Num(faction_id) = stack_val(state, 1) else {
        return Ok(0);
    };
    let Some(info) = borrow_state(state)?
        .faction_paragon
        .get(&(faction_id as i32 as i64))
        .cloned()
    else {
        return Ok(0);
    };
    state.push(Val::Num(info.current_value as f64));
    state.push(Val::Num(info.threshold as f64));
    state.push(Val::Num(info.reward_quest_id as f64));
    state.push(Val::Bool(info.has_reward_pending));
    state.push(Val::Bool(info.too_low_level_for_paragon));
    if cfg!(feature = "retail-12-0-0") {
        state.push(Val::Num(info.paragon_storage_level as f64));
        Ok(6)
    } else {
        Ok(5)
    }
}
