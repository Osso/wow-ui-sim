//! Active event season, separate from the player's timerunning season.
use crate::c_api::helpers::ensure_namespace;
use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(super) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = ensure_namespace(state, "C_TimerunningUI")?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetActiveTimerunningSeasonID",
        get_active_timerunning_season_id,
    )
}

fn get_active_timerunning_season_id(state: &mut LuaState) -> LuaResult<u32> {
    let season_id = borrow_state(state)?.active_timerunning_season_id;
    state.push(Val::Num(f64::from(season_id)));
    Ok(1)
}
