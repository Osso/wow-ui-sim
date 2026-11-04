//! UNKNOWN scheduling, target selection, security and native trap handling.
//! These explicit simulator adapters are EMULATED, not native live guarantees.

use super::SuppliedCombatRangeResult;
use crate::lua_api::methods::borrow_state_mut;
use crate::lua_api::script_helpers::fire_named_event_state;
use crate::lua_bridge::stack_val;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};
use std::collections::HashMap;

pub(super) fn read_enable(state: &LuaState, function: &str, usage: &str) -> LuaResult<bool> {
    if matches!(stack_val(state, 2), Val::Nil) {
        return Err(crate::c_api::action_button_hover::argument_error(
            2, function, usage,
        ));
    }
    let value = crate::c_api::action_button_hover::unknown_policy::read_accessible_argument(
        state, 2, usage,
    )?;
    Ok(!matches!(value, Val::Nil | Val::Bool(false)))
}

/// Native diagnostics can trap; they are not established Lua argument errors.
/// EMULATED exceptional-path policy: report and leave the counted store intact.
pub(super) fn report_native_diagnostic(message: &str) {
    eprintln!("[WM93 EMULATED diagnostic handling; native trap policy UNKNOWN] {message}");
}

/// Explicit geometry/target seam: no snapshot means no evaluation. The initial
/// range Boolean (even when unavailable) is supplied, never synthesized.
pub(super) fn select_supplied_result(
    supplied_results: &HashMap<i32, SuppliedCombatRangeResult>,
    key: i32,
) -> Option<SuppliedCombatRangeResult> {
    let mut result = *supplied_results.get(&key)?;
    if !(0..408).contains(&key) {
        result.available = false;
    }
    Some(result)
}

/// Explicit EMULATED scheduling/delivery seam, not a native polling interval.
/// Gather state updates before Lua delivery, allowing re-entrant registration.
pub(super) fn sweep_supplied_results(state: &mut LuaState) -> LuaResult<u32> {
    let force = !matches!(stack_val(state, 1), Val::Nil | Val::Bool(false));
    let updates = {
        let mut sim = borrow_state_mut(state)?;
        super::collect_updates(&mut sim.action_combat_range, force)
    };
    for (key, result) in updates {
        fire_named_event_state(
            state,
            "ACTION_COMBAT_ACTION_RANGE_CHECK_UPDATE",
            &[
                Val::Num(f64::from(key) + 1.0),
                Val::Bool(result.in_range),
                Val::Bool(result.available),
            ],
        );
    }
    Ok(0)
}
