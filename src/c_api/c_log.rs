//! Client log messages use the simulator's console sink.

use crate::c_api::ensure_namespace;
use crate::lua_api::methods::borrow_state_mut;
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::LuaResult;
use rilua::vm::state::LuaState;

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = ensure_namespace(state, "C_Log")?;
    table_set_rust_fn_static(state, namespace, "LogMessage", log_message)
}

fn log_message(state: &mut LuaState) -> LuaResult<u32> {
    let message = String::from_stack(state, 1)?;
    borrow_state_mut(state)?.console_output.push(message);
    Ok(0)
}
