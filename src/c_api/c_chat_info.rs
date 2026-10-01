//! Explicit Retail chat messaging restriction query.

use crate::c_api::ensure_namespace;
use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = ensure_namespace(state, "C_ChatInfo")?;
    table_set_rust_fn_static(
        state,
        namespace,
        "InChatMessagingLockdown",
        in_chat_messaging_lockdown,
    )
}

fn in_chat_messaging_lockdown(state: &mut LuaState) -> LuaResult<u32> {
    let restricted = borrow_state(state)?.chat_messaging_lockdown;
    state.push(Val::Bool(restricted));
    Ok(1)
}
