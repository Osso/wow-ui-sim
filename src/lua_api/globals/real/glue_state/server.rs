//! R88ad.8 Glue connection, realm, expansion and first-display reads.
//! None of these getters inspects Lua arguments or writes state.

use crate::lua_api::methods::{borrow_state, create_string};
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, RustFn, Val};

pub(super) fn register(state: &mut LuaState) -> LuaResult<()> {
    let globals: [(&str, RustFn); 4] = [
        ("GetServerName", get_server_name),
        ("IsConnectedToServer", is_connected_to_server),
        ("GetMinimumExpansionLevel", get_minimum_expansion_level),
        ("WasScreenFirstDisplayed", was_screen_first_displayed),
    ];
    for (name, function) in globals {
        LuaApiMut::register_function(state, name, function)?;
    }
    Ok(())
}

/// Zero values while the connection-availability flag is clear; otherwise the
/// name, both configuration Booleans, the additional Boolean and realm number.
fn get_server_name(state: &mut LuaState) -> LuaResult<u32> {
    let server = borrow_state(state)?.glue_state.server.clone();
    if !server.available {
        return Ok(0);
    }
    let name = match server.name {
        Some(name) => create_string(state, &name),
        None => Val::Nil,
    };
    let (first, second) = server.config.unwrap_or_default();
    let additional = server.config.is_some() && server.additional_connection_flag;
    state.push(name);
    state.push(Val::Bool(first));
    state.push(Val::Bool(second));
    state.push(Val::Bool(additional));
    state.push(
        server
            .realm_number
            .map_or(Val::Nil, |realm| Val::Num(f64::from(realm))),
    );
    Ok(5)
}

fn is_connected_to_server(state: &mut LuaState) -> LuaResult<u32> {
    let available = borrow_state(state)?.glue_state.server.available;
    state.push(Val::Bool(available));
    Ok(1)
}

fn get_minimum_expansion_level(state: &mut LuaState) -> LuaResult<u32> {
    let level = borrow_state(state)?.glue_state.minimum_expansion_level;
    state.push(Val::Num(f64::from(level)));
    Ok(1)
}

fn was_screen_first_displayed(state: &mut LuaState) -> LuaResult<u32> {
    let displayed = borrow_state(state)?.screen_first_display_state == 2;
    state.push(Val::Bool(displayed));
    Ok(1)
}
