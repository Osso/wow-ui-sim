//! INFERRED host-declared unsupported tokens; no automatic PvP token classifier.

use crate::lua_api::methods::{borrow_state, val_to_string};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

/// Authenticate every argument, including ignored extras, before validating any.
/// AllowedWhenUntainted is cached for GUID/vitals and INFERRED for legacy UnitAura.
pub(crate) fn authenticate_unit_arguments(state: &mut LuaState) -> LuaResult<String> {
    let arguments: Vec<Val> = state.stack[state.base..state.top].to_vec();
    let authenticated = arguments
        .into_iter()
        .map(|value| rilua::table_security::unwrap_secret(state, value))
        .collect::<LuaResult<Vec<_>>>()?;
    let unit = match authenticated.first() {
        Some(value @ Val::Str(_)) => val_to_string(state, *value),
        _ => None,
    }
    .ok_or_else(|| rilua::runtime_error("unit token must be a string"))?;
    state.stack[state.base..state.top].copy_from_slice(&authenticated);
    Ok(unit)
}

pub(crate) fn is_unsupported(state: &LuaState, unit: &str) -> LuaResult<bool> {
    Ok(borrow_state(state)?.unsupported_unit_tokens.contains(unit))
}
