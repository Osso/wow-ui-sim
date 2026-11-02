//! Supplied battleground enlistment state; native cache lifecycle is not modeled.
use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::{gc::arena::GcRef, state::LuaState, table::Table};
use rilua::{LuaResult, Val};

pub(super) fn register(state: &mut LuaState, namespace: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        namespace,
        "IsBattlegroundEnlistmentBonusActive",
        query,
    )
}

fn query(state: &mut LuaState) -> LuaResult<u32> {
    let (battleground, brawl) = {
        let sim = borrow_state(state)?;
        let input = &sim.battleground_enlistment;
        let keys = match input.configuration_key {
            Some(1) => Some((10542, 13646)),
            Some(0) => Some((10537, 13647)),
            _ => None,
        };
        let enabled = input.player_level.is_some_and(|level| level >= 90)
            && input.prerequisites_present
            && input.flags_allow
            && input.cache_enabled;
        if let Some((first, second)) = keys.filter(|_| enabled) {
            let active = |id| {
                input
                    .cached_values
                    .get(&id)
                    .is_some_and(|value| *value != 0)
            };
            (active(first), active(second))
        } else {
            (false, false)
        }
    };
    state.push(Val::Bool(battleground));
    state.push(Val::Bool(brawl));
    Ok(2)
}
