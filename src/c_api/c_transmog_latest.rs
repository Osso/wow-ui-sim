//! Latest collected transmog appearance for the current session.

use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    if !matches!(
        ACTIVE,
        ClientProfile::Retail | ClientProfile::Ptr | ClientProfile::WowForever
    ) {
        return Ok(());
    }
    let namespace = super::ensure_namespace(state, "C_TransmogCollection")?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetLatestAppearance",
        get_latest_appearance,
    )
}

fn get_latest_appearance(state: &mut LuaState) -> LuaResult<u32> {
    let latest = borrow_state(state)?.latest_transmog_appearance;
    if let Some((visual_id, category_id)) = latest {
        state.push(Val::Num(visual_id as f64));
        state.push(Val::Num(category_id as f64));
        Ok(2)
    } else {
        Ok(0)
    }
}
