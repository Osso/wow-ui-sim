//! Mists scenario eligibility supplied by the environment, not inferred policy.

use crate::lua_api::methods::{borrow_state, create_string};
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::{gc::arena::GcRef, state::LuaState, table::Table};
use rilua::{LuaResult, Val};

pub(super) fn register(state: &mut LuaState, namespace: GcRef<Table>) -> LuaResult<()> {
    // Mists 5.5.4 LFGInfoDocumentation.lua:51 declares non-nil bool/string returns.
    table_set_rust_fn_static(
        state,
        namespace,
        "CanPlayerUseScenarioFinder",
        can_player_use_scenario_finder,
    )
}

fn can_player_use_scenario_finder(state: &mut LuaState) -> LuaResult<u32> {
    let supplied = borrow_state(state)?.scenario_finder_availability.clone();
    let (can_use, failure_reason) = supplied.unwrap_or_else(|| (true, String::new()));
    let reason = create_string(state, &failure_reason);
    state.push(Val::Bool(can_use));
    state.push(reason);
    Ok(2)
}
