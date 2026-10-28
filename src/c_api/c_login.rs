//! `C_Login` glue-screen connection state helpers.

use crate::c_api::{ensure_namespace, permanent_shims};
use crate::lua_api::methods::{borrow_state, create_table, table_set};
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub fn register_c_login(state: &mut LuaState) -> LuaResult<()> {
    let ns = ensure_namespace(state, "C_Login")?;
    table_set_rust_fn_static(state, ns, "IsLoginReady", c_login_is_login_ready)?;
    table_set_rust_fn_static(state, ns, "GetState", c_login_get_state)?;
    permanent_shims::c_login::register_c_login_defaults(state, ns)?;
    Ok(())
}

fn c_login_is_login_ready(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Bool(true));
    Ok(1)
}

fn c_login_get_state(state: &mut LuaState) -> LuaResult<u32> {
    let screen_kind = borrow_state(state)?.screen_kind;
    let (aurora_state, connected_to_wow, wow_connection_state, has_realm_list) =
        screen_kind.login_state();
    if crate::client_profile::ACTIVE == crate::client_profile::ClientProfile::Ptr {
        let result = create_table(state);
        for (key, value) in [
            ("auroraState", Val::Num(f64::from(aurora_state))),
            ("connectedToWoW", Val::Bool(connected_to_wow)),
            (
                "wowConnectionState",
                Val::Num(f64::from(wow_connection_state)),
            ),
            ("hasRealmList", Val::Bool(has_realm_list)),
            ("waitingForRealmList", Val::Bool(false)),
            // User-approved scenario defaults; native login policy is not modeled.
            ("superDistrictChoicePending", Val::Bool(false)),
            ("keepingCharacterListOnRealmJoin", Val::Bool(false)),
            ("waitingForRuleSets", Val::Bool(false)),
            ("hasSelectedExperiencePreset", Val::Bool(true)),
        ] {
            table_set(state, result, key, value);
        }
        state.push(result);
        return Ok(1);
    }
    state.push(Val::Num(f64::from(aurora_state)));
    state.push(Val::Bool(connected_to_wow));
    state.push(Val::Num(f64::from(wow_connection_state)));
    state.push(Val::Bool(has_realm_list));
    state.push(Val::Bool(false));
    Ok(5)
}
