//! Retail 12.0.0 secret predicates and caller-context access revocation.

use rilua::table_security::{
    can_access_secrets, is_secret_table, is_secret_value, revoke_secret_access,
};
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

pub(super) fn register(lua: &mut rilua::Lua) -> LuaResult<()> {
    LuaApiMut::register_function(lua, "canaccesssecrets", canaccesssecrets)?;
    LuaApiMut::register_function(lua, "hasanysecretvalues", hasanysecretvalues)?;
    LuaApiMut::register_function(lua, "dropsecretaccess", dropsecretaccess)?;
    LuaApiMut::register_function(lua, "issecrettable", issecrettable)
}

fn canaccesssecrets(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Bool(can_access_secrets(state)));
    Ok(1)
}

fn dropsecretaccess(state: &mut LuaState) -> LuaResult<u32> {
    revoke_secret_access(state)?;
    Ok(0)
}

fn issecrettable(state: &mut LuaState) -> LuaResult<u32> {
    let value = crate::lua_bridge::stack_val(state, 1);
    state.push(Val::Bool(is_secret_table(state, value)));
    Ok(1)
}

fn hasanysecretvalues(state: &mut LuaState) -> LuaResult<u32> {
    let secret =
        (state.base..state.top).any(|index| is_secret_value(state, state.stack_get(index)));
    state.push(Val::Bool(secret));
    Ok(1)
}
