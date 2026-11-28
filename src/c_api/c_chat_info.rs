//! Explicit Retail chat messaging restriction query and producer guard.
// C_ChatInfo.IsLoggingChat: ChatInfoDocumentation.lua:363.

use crate::c_api::ensure_namespace;
use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = ensure_namespace(state, "C_ChatInfo")?;
    table_set_rust_fn_static(
        state,
        namespace,
        "InChatMessagingLockdown",
        in_chat_messaging_lockdown,
    )?;
    #[cfg(feature = "client-retail")]
    table_set_rust_fn_static(state, namespace, "IsLoggingChat", is_logging_chat)?;
    super::chat_expressions::register(state)
}

/// Explicit rejection is simulator policy; native error wording is unknown.
pub(crate) fn reject_chat_messaging_lockdown(
    state: &mut LuaState,
    operation: &str,
) -> LuaResult<()> {
    if borrow_state(state)?.chat_messaging_lockdown {
        return Err(runtime_error(format!(
            "{operation} is blocked during chat messaging lockdown"
        )));
    }
    Ok(())
}

fn in_chat_messaging_lockdown(state: &mut LuaState) -> LuaResult<u32> {
    let restricted = borrow_state(state)?.chat_messaging_lockdown;
    state.push(Val::Bool(restricted));
    Ok(1)
}

#[cfg(feature = "client-retail")]
fn is_logging_chat(state: &mut LuaState) -> LuaResult<u32> {
    let enabled = borrow_state(state)?.session_logging.chat;
    state.push(Val::Bool(enabled));
    Ok(1)
}

#[cfg(all(test, feature = "client-retail"))]
#[test]
fn chat_logging_query_tracks_session_toggle() {
    crate::lua_api::WowLuaEnv::new()
        .unwrap()
        .exec(
            r#"
        LoggingChat(true)
        assert(C_ChatInfo.IsLoggingChat() == true)
        LoggingChat(false)
        assert(C_ChatInfo.IsLoggingChat() == false)
    "#,
        )
        .unwrap();
}
