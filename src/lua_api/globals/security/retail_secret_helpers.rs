//! Retail 12.0.0 helpers using VM wrappers and the VM's access guard.
//! dropsecretaccess is intentionally absent: pinned rilua has no per-caller
//! access-revocation primitive. Setting stack taint would change issecure too.

use rilua::api::state_is_secure;
use rilua::table_security::is_secret_value;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

pub(super) fn register(lua: &mut rilua::Lua) -> LuaResult<()> {
    LuaApiMut::register_function(lua, "canaccesssecrets", canaccesssecrets)?;
    LuaApiMut::register_function(lua, "hasanysecretvalues", hasanysecretvalues)?;
    LuaApiMut::register_function(lua, "issecrettable", issecrettable)
}

fn canaccesssecrets(state: &mut LuaState) -> LuaResult<u32> {
    // This is exactly the guard used by rilua's unwrap_secret.
    state.push(Val::Bool(state_is_secure(state)));
    Ok(1)
}

fn hasanysecretvalues(state: &mut LuaState) -> LuaResult<u32> {
    let secret =
        (state.base..state.top).any(|index| is_secret_value(state, state.stack_get(index)));
    state.push(Val::Bool(secret));
    Ok(1)
}

fn issecrettable(state: &mut LuaState) -> LuaResult<u32> {
    let value = state.stack_get(state.base);
    // Trusted inspection of wrapper *kind*, never its table contents. Do not
    // unwrap: this predicate must remain callable by tainted code.
    let secret_table = match value {
        Val::Userdata(reference) => state
            .gc
            .userdata
            .get(reference)
            .and_then(|userdata| userdata.secret_value())
            .is_some_and(|payload| matches!(payload, Val::Table(_))),
        // SecretWrapContents (option 2) is rejected by pinned rilua. The other
        // security flags restrict indexing, but do not produce secret contents.
        _ => false,
    };
    state.push(Val::Bool(secret_table));
    Ok(1)
}
