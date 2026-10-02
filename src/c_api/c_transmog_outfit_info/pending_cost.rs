//! Explicit pending-cost snapshot query; no pricing or transaction lifecycle.

use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::{gc::arena::GcRef, state::LuaState, table::Table};
use rilua::{LuaResult, Val};

// Chosen local scalar representation limit, not a native transmog cost cap.
const MAX_EXACT_LUA_INTEGER: u64 = 9_007_199_254_740_991;

pub(super) fn register(state: &mut LuaState, namespace: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        namespace,
        "GetPendingTransmogCost",
        get_pending_transmog_cost,
    )
}

fn validate_cost(cost: u64) -> LuaResult<()> {
    if cost > MAX_EXACT_LUA_INTEGER {
        return Err(rilua::runtime_error(
            "C_TransmogOutfitInfo.GetPendingTransmogCost: host cost exceeds the exact Lua scalar representation limit",
        ));
    }
    Ok(())
}

fn get_pending_transmog_cost(state: &mut LuaState) -> LuaResult<u32> {
    let snapshot = borrow_state(state)?.pending_transmog_cost;
    // Guessed absence policy; no native-client evidence establishes this arity.
    let Some(snapshot) = snapshot else {
        return Ok(0);
    };
    validate_cost(snapshot.cost)?;
    state.push(Val::Num(snapshot.cost as f64));
    state.push(Val::Num(snapshot.modifier_flags as f64));
    Ok(2)
}
