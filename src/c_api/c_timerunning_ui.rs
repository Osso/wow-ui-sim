//! Active event season, separate from the player's timerunning season.
use crate::c_api::helpers::ensure_namespace;
use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

// Forever 70707 TimerunningUIDocumentation.lua:10.
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

#[cfg(feature = "client-wowforever")]
pub(crate) fn register_admin(
    builder: crate::lua_bridge::TableBuilder,
) -> LuaResult<crate::lua_bridge::TableBuilder> {
    builder.set_function(
        "SetActiveTimerunningSeasonID",
        set_active_timerunning_season_id,
    )
}

#[cfg(feature = "client-wowforever")]
fn set_active_timerunning_season_id(state: &mut LuaState) -> LuaResult<u32> {
    let value = crate::lua_bridge::stack_val(state, 1);
    let id = super::helpers::validate_supplied_relation_id(
        value,
        "SetActiveTimerunningSeasonID: expected a nonnegative integer",
    )?;
    let id = id.filter(|id| *id >= 0).ok_or_else(|| {
        rilua::runtime_error("SetActiveTimerunningSeasonID: expected a nonnegative integer")
    })?;
    crate::lua_api::methods::borrow_state_mut(state)?.active_timerunning_season_id = id;
    Ok(0)
}
