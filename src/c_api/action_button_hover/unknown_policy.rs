//! UNKNOWN native execution/publication policy (WM92 PTR 70668).
//!
//! EMULATED: use the runtime's AllowedWhenUntainted argument-access policy.
//! Native complete secret/combat contexts and taint outcomes are not proved.
//! No extra events, tooltip ownership, combat restrictions or taint resets are
//! synthesized: the caller implements only WM92's recovered local stores.

use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(crate) fn read_accessible_argument(
    state: &LuaState,
    index: i32,
    usage: &str,
) -> LuaResult<Val> {
    let original = crate::lua_bridge::stack_val(state, index);
    rilua::table_security::unwrap_secret(state, original).map_err(|_| {
        rilua::runtime_error(format!(
            "{usage}. Secret values are only allowed during untainted execution for this argument."
        ))
    })
}

/// UNKNOWN extreme protected-path consequences in WM93. EMULATED signed
/// narrowing outside i32 range; ordinary converted keys retain their exact value.
pub(super) fn narrow_converted_slot(converted: f64) -> i32 {
    (converted as i64) as i32
}
