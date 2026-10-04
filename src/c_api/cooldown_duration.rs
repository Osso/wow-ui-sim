//! Shared cooldown-duration selection; interval policies are simulator inferences.

use crate::lua_api::SimState;
use crate::lua_api::globals::action_bar_api::spell_cooldown_times;
use crate::lua_bridge::stack_val;
use rilua::Val;
use rilua::vm::state::LuaState;

pub(crate) fn read_ignore_gcd(state: &LuaState, index: i32) -> bool {
    cfg!(feature = "retail-12-0-5") && matches!(stack_val(state, index), Val::Bool(true))
}

/// Action/book flags use the cached AllowedWhenUntainted argument policy.
pub(crate) fn read_untainted_ignore_gcd(state: &LuaState, index: i32) -> rilua::LuaResult<bool> {
    if !cfg!(feature = "retail-12-0-5") {
        return Ok(false);
    }
    let input = rilua::table_security::unwrap_secret(state, stack_val(state, index))?;
    // Inferred compatibility: retain the existing true-only rule for public values.
    Ok(matches!(input, Val::Bool(true)))
}

pub(crate) fn select_cooldown_duration_times(
    sim: &SimState,
    spell_id: u32,
    now: f64,
    ignore_gcd: bool,
) -> (f64, f64) {
    if !ignore_gcd {
        return spell_cooldown_times(sim, spell_id, now);
    }
    sim.spell_cooldowns
        .get(&spell_id)
        .filter(|cooldown| cooldown.start + cooldown.duration > now)
        .map(|cooldown| (cooldown.start, cooldown.duration))
        .unwrap_or((0.0, 0.0))
}
