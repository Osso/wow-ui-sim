//! Host-supplied charter edit capability; native eligibility remains unknown.
use crate::c_api::helpers::ensure_namespace;
use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(super) fn register(state: &mut LuaState) -> LuaResult<()> {
    let housing = ensure_namespace(state, "C_Housing")?;
    table_set_rust_fn_static(state, housing, "CanEditCharter", can_edit_charter)
}

fn can_edit_charter(state: &mut LuaState) -> LuaResult<u32> {
    let can_edit = borrow_state(state)?.housing.charter_can_edit;
    state.push(Val::Bool(can_edit));
    Ok(1)
}
