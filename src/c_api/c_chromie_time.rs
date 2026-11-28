//! `C_ChromieTime` empty-state surface for Retail, PTR and Forever clients.

use crate::c_api::helpers::ensure_namespace;
use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::create_table;
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(crate) fn register_c_chromie_time_surface(state: &mut LuaState) -> LuaResult<()> {
    // Forever ChromieTimeUIDocumentation.lua declares the same four blocks.
    if !matches!(
        ACTIVE,
        ClientProfile::Retail | ClientProfile::Ptr | ClientProfile::WowForever
    ) {
        return Ok(());
    }

    let chromie_time = ensure_namespace(state, "C_ChromieTime")?;
    table_set_rust_fn_static(state, chromie_time, "CloseUI", close_ui)?;
    table_set_rust_fn_static(
        state,
        chromie_time,
        "GetChromieTimeExpansionOption",
        get_chromie_time_expansion_option,
    )?;
    table_set_rust_fn_static(
        state,
        chromie_time,
        "GetChromieTimeExpansionOptions",
        get_chromie_time_expansion_options,
    )?;
    table_set_rust_fn_static(
        state,
        chromie_time,
        "SelectChromieTimeOption",
        select_chromie_time_option,
    )
}

fn close_ui(_state: &mut LuaState) -> LuaResult<u32> {
    Ok(0)
}

fn get_chromie_time_expansion_option(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Nil);
    Ok(1)
}

fn get_chromie_time_expansion_options(state: &mut LuaState) -> LuaResult<u32> {
    let options = create_table(state);
    state.push(options);
    Ok(1)
}

fn select_chromie_time_option(_state: &mut LuaState) -> LuaResult<u32> {
    Ok(0)
}
