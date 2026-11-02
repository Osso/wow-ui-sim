//! `C_Ping.GetCooldownInfo` reads the ping manager's stored cooldown pair (WM51-F1).

use crate::lua_api::methods::{borrow_state, create_table, table_set_static};
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = super::ensure_namespace(state, "C_Ping")?;
    table_set_rust_fn_static(state, namespace, "GetCooldownInfo", get_cooldown_info)
}

fn get_cooldown_info(state: &mut LuaState) -> LuaResult<u32> {
    let interval = borrow_state(state)?.ping_burst.interval;
    // Release the provider borrow before allocating; root the fresh DTO before keys.
    let info = create_table(state);
    state.push(info);
    let start = Val::Num(f64::from(interval.start_time_ms));
    table_set_static(state, info, "startTimeMs", start);
    table_set_static(
        state,
        info,
        "endTimeMs",
        Val::Num(f64::from(interval.end_time_ms)),
    );
    Ok(1)
}
