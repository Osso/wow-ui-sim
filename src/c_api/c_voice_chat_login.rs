//! Supplied voice-service login state for the loaded Blizzard voice-toggle consumer.

use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::{borrow_state, borrow_state_mut};
use crate::lua_bridge::{TableBuilder, stack_val, table_set_rust_fn_static};
use rilua::table_security::unwrap_secret;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    if !publishes_login_state() {
        return Ok(());
    }
    let namespace = super::ensure_namespace(state, "C_VoiceChat")?;
    table_set_rust_fn_static(state, namespace, "IsLoggedIn", query_logged_in)
}

pub(crate) fn register_admin(builder: TableBuilder) -> LuaResult<TableBuilder> {
    if !publishes_login_state() {
        return Ok(builder);
    }
    builder.set_function("SetVoiceChatLoggedIn", set_logged_in)
}

fn publishes_login_state() -> bool {
    matches!(ACTIVE, ClientProfile::Retail | ClientProfile::WowForever)
}

fn query_logged_in(state: &mut LuaState) -> LuaResult<u32> {
    let logged_in = borrow_state(state)?.voice_chat.logged_in;
    state.push(Val::Bool(logged_in));
    Ok(1)
}

fn set_logged_in(state: &mut LuaState) -> LuaResult<u32> {
    let Val::Bool(logged_in) = unwrap_secret(state, stack_val(state, 1))? else {
        return Err(runtime_error(
            "A_Admin.SetVoiceChatLoggedIn: expected boolean",
        ));
    };
    borrow_state_mut(state)?.voice_chat.logged_in = logged_in;
    Ok(0)
}
