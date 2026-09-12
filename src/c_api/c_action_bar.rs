//! State-backed action cooldown queries; other action-bar APIs retain their existing owners.

use crate::lua_api::globals::action_bar_api::spell_cooldown_times;
use crate::lua_api::globals::lua_duration_object::push_timed_duration_object;
use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::stack_val;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(crate) fn read_action_cooldown(state: &LuaState) -> LuaResult<(f64, f64)> {
    let slot = match stack_val(state, 1) {
        Val::Num(number) if number >= 0.0 => Some(number as u32),
        _ => None,
    };
    let sim = borrow_state(state)?;
    let now = sim.start_time.elapsed().as_secs_f64();
    Ok(slot
        .and_then(|slot| sim.action_bars.get(&slot).copied())
        .map(|spell_id| spell_cooldown_times(&sim, spell_id, now))
        .unwrap_or((0.0, 0.0)))
}

pub(crate) fn get_action_cooldown_duration(state: &mut LuaState) -> LuaResult<u32> {
    let (start, seconds) = read_action_cooldown(state)?;
    push_timed_duration_object(state, start, seconds)
}
