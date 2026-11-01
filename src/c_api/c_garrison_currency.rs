//! Garrison currency types from the pinned GarrType catalogue.

use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    if !matches!(
        ACTIVE,
        ClientProfile::Retail | ClientProfile::Ptr | ClientProfile::WowForever
    ) {
        return Ok(());
    }
    let namespace = super::ensure_namespace(state, "C_Garrison")?;
    table_set_rust_fn_static(state, namespace, "GetCurrencyTypes", get_currency_types)
}

fn get_currency_types(state: &mut LuaState) -> LuaResult<u32> {
    let garrison_type = i32::from_stack(state, 1)?;
    let Some((primary, secondary)) = sim_data::garrison_currency::currency_types(garrison_type)
    else {
        return Ok(0);
    };
    state.push(Val::Num(primary as f64));
    state.push(Val::Num(secondary as f64));
    Ok(2)
}
