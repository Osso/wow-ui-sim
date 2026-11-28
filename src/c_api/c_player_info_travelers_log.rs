//! Host-supplied Traveler's Log availability for `C_PlayerInfo`.

use crate::c_api::helpers::ensure_namespace;
use crate::lua_api::methods::{borrow_state, borrow_state_mut};
use crate::lua_bridge::{FromStack, TableBuilder, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

const INPUT_NAME: &str = "A_Admin.SetTravelersLogAvailable";

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = ensure_namespace(state, "C_PlayerInfo")?;
    table_set_rust_fn_static(
        state,
        namespace,
        "IsTravelersLogAvailable",
        is_travelers_log_available,
    )
}

pub(crate) fn register_admin(builder: TableBuilder) -> LuaResult<TableBuilder> {
    builder.set_function("SetTravelersLogAvailable", set_travelers_log_available)
}

fn is_travelers_log_available(state: &mut LuaState) -> LuaResult<u32> {
    let available = borrow_state(state)?.player.travelers_log_available;
    state.push(Val::Bool(available));
    Ok(1)
}

fn set_travelers_log_available(state: &mut LuaState) -> LuaResult<u32> {
    let supplied = Val::from_stack(state, 1)?;
    let Val::Bool(available) = supplied else {
        return Err(runtime_error(format!("{INPUT_NAME}: expected a boolean")));
    };
    borrow_state_mut(state)?.player.travelers_log_available = available;
    Ok(0)
}
