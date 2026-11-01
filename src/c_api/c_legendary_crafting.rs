//! Runeforge power queries for the currently empty simulator catalogue.

use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::create_table;
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::LuaResult;
use rilua::vm::state::LuaState;

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    if !matches!(
        ACTIVE,
        ClientProfile::Retail | ClientProfile::Ptr | ClientProfile::WowForever
    ) {
        return Ok(());
    }
    let namespace = super::ensure_namespace(state, "C_LegendaryCrafting")?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetRuneforgePowersByClassSpecAndCovenant",
        get_runeforge_powers,
    )
}

fn get_runeforge_powers(state: &mut LuaState) -> LuaResult<u32> {
    for index in 1..=4 {
        Option::<f64>::from_stack(state, index)?;
    }
    let result = create_table(state);
    state.push(result);
    Ok(1)
}
