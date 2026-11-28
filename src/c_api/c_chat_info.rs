//! Explicit Retail chat messaging restriction query and producer guard.

use crate::c_api::ensure_namespace;
use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

// Forever 70707 ChatInfoDocumentation.lua:292.
pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = ensure_namespace(state, "C_ChatInfo")?;
    table_set_rust_fn_static(
        state,
        namespace,
        "InChatMessagingLockdown",
        in_chat_messaging_lockdown,
    )?;
    #[cfg(feature = "retail-12-0-5")]
    super::chat_expressions::register(state)?;
    Ok(())
}

#[cfg(feature = "client-wowforever")]
pub(crate) fn register_admin(
    builder: crate::lua_bridge::TableBuilder,
) -> LuaResult<crate::lua_bridge::TableBuilder> {
    builder.set_function("SetChatMessagingLockdown", set_chat_messaging_lockdown)
}

#[cfg(feature = "client-wowforever")]
fn set_chat_messaging_lockdown(state: &mut LuaState) -> LuaResult<u32> {
    let Val::Bool(restricted) = crate::lua_bridge::stack_val(state, 1) else {
        return Err(runtime_error("SetChatMessagingLockdown: expected boolean"));
    };
    crate::lua_api::methods::borrow_state_mut(state)?.chat_messaging_lockdown = restricted;
    Ok(0)
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
