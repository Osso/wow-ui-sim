//! Bounded secret-origin widget policy; not general secret-aspect enforcement.

use crate::lua_api::methods::borrow_state;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

pub(crate) fn require_readable(state: &LuaState, secret: bool) -> LuaResult<()> {
    if secret && !rilua::api::state_is_secure(state) {
        return Err(runtime_error(
            "secret-origin widget readout requires an untainted caller",
        ));
    }
    Ok(())
}

/// Only VM-authenticated wrappers are decoded; plain coercion stays with each API.
pub(crate) fn unwrap_input(state: &LuaState, value: Val) -> LuaResult<(Val, bool)> {
    if !cfg!(feature = "client-wowforever") {
        return Ok((value, false));
    }
    let secret = rilua::table_security::is_secret_value(state, value);
    let value = rilua::table_security::unwrap_secret(state, value)?;
    Ok((value, secret))
}

pub(crate) fn require_timing_readable(state: &LuaState, id: u64) -> LuaResult<()> {
    let secret = borrow_state(state)?
        .widgets
        .get(id)
        .is_some_and(|frame| frame.secret_timing);
    require_readable(state, secret)
}

pub(crate) fn require_text_readable(state: &LuaState, id: u64) -> LuaResult<()> {
    let secret = borrow_state(state)?
        .widgets
        .get(id)
        .is_some_and(|frame| frame.secret_text);
    require_readable(state, secret)
}

pub(crate) fn require_shown_readable(state: &LuaState, id: u64, ancestors: bool) -> LuaResult<()> {
    if rilua::api::state_is_secure(state) {
        return Ok(());
    }
    let sim = borrow_state(state)?;
    let mut current = Some(id);
    while let Some(frame) = current.and_then(|id| sim.widgets.get(id)) {
        require_readable(state, frame.secret_shown)?;
        if !ancestors {
            break;
        }
        current = frame.parent_id;
    }
    Ok(())
}
