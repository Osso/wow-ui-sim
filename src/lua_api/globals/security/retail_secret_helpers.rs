//! Retail 12.0.0 helpers using VM wrappers and the VM's access guard.
//! dropsecretaccess is intentionally absent: pinned rilua has no per-caller
//! access-revocation primitive. Setting stack taint would change issecure too.
//! issecrettable is absent: the wrapper payload-kind accessor is VM-private;
//! unwrap_secret rejects tainted callers, which must still use this predicate.

use rilua::api::state_is_secure;
use rilua::table_security::is_secret_value;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

pub(super) fn register(lua: &mut rilua::Lua) -> LuaResult<()> {
    LuaApiMut::register_function(lua, "canaccesssecrets", canaccesssecrets)?;
    LuaApiMut::register_function(lua, "hasanysecretvalues", hasanysecretvalues)
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
