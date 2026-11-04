//! UNKNOWN native execution/publication policy (WM92 PTR 70668).
//!
//! EMULATED: use the runtime's AllowedWhenUntainted argument-access policy.
//! Native complete secret/combat contexts and taint outcomes are not proved.
//! No extra events, tooltip ownership, combat restrictions or taint resets are
//! synthesized: the caller implements only WM92's recovered local stores.

use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(super) fn read_accessible_argument(state: &LuaState, usage: &str) -> LuaResult<Val> {
    let original = crate::lua_bridge::stack_val(state, 1);
    rilua::table_security::unwrap_secret(state, original).map_err(|_| {
        rilua::runtime_error(format!(
            "{usage}. Secret values are only allowed during untainted execution for this argument."
        ))
    })
}
