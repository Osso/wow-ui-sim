//! Global-string token lookup for Retail/Forever `GetText`.
//!
//! INFERRED R88g9 (native evidence pending): the loaded `_G` value is the
//! simulator's global-string store; missing tokens pass through unchanged.

use crate::lua_api::methods::{create_string, table_get, val_to_string};
use crate::lua_bridge::stack_val;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
pub(crate) fn register_all(lua: &mut rilua::Lua) -> crate::Result<()> {
    LuaApiMut::register_function(lua, "GetText", get_text)?;
    Ok(())
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn get_text(state: &mut LuaState) -> LuaResult<u32> {
    let token = stack_val(state, 1);
    let Some(token) = val_to_string(state, token) else {
        state.push(token);
        return Ok(1);
    };
    let resolved = table_get(state, Val::Table(state.global), &token);
    if matches!(resolved, Val::Nil) {
        let unresolved = create_string(state, &token);
        state.push(unresolved);
    } else {
        state.push(resolved);
    }
    Ok(1)
}
